//! CMDB configuration-item instances: dynamic JSONB payloads validated
//! against the owning model's attribute definitions, with unique-key
//! enforcement (veops-style 唯一键) and keyword search.

use std::collections::BTreeMap;
use schemars::JsonSchema;

use aide::axum::routing::{delete, get, post, put};
use aide::axum::ApiRouter;
use axum::{
    Json,
    extract::{Query, State},
};
use rustset_cmdb_api::AttrType;
use rustset_framework_common::ApiResponse;
use rustset_framework_security::CurrentUser;
use rustset_framework_web::AppError;
use serde::Deserialize;
use serde_json::{Map, Value, json};
use sqlx::{PgPool, Row};

use crate::{CmdbState, require};

pub fn routes() -> ApiRouter<CmdbState> {
    ApiRouter::new()
        .api_route(
"/cmdb/instance/page", get(instance_page))
        .api_route(
"/cmdb/instance/get", get(instance_get))
        .api_route(
"/cmdb/instance/create", post(instance_create))
        .api_route(
"/cmdb/instance/update", put(instance_update))
        .api_route(
"/cmdb/instance/delete", delete(instance_delete))
        .api_route(
"/cmdb/instance/delete-list", delete(instance_delete_list))
        .api_route(
"/cmdb/instance/export", get(instance_export))
        .api_route(
"/cmdb/instance/import", post(instance_import))
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct InstancePageParams {
    model_id: i64,
    #[serde(default)]
    page_no: Option<i64>,
    #[serde(default)]
    page_size: Option<i64>,
    #[serde(default)]
    keyword: Option<String>,
}

struct AttributeDef {
    code: String,
    attr_type: AttrType,
    required: bool,
    choices: Option<Value>,
    default_value: Option<Value>,
}

struct ModelHeader {
    id: i64,
    unique_key: Option<String>,
}

async fn load_model(pool: &PgPool, model_id: i64) -> Result<ModelHeader, AppError> {
    let row = sqlx::query("SELECT id, unique_key FROM cmdb_model WHERE id = $1 AND deleted = 0")
        .bind(model_id)
        .fetch_optional(pool)
        .await
        .map_err(|_| AppError::internal("failed to read model"))?
        .ok_or_else(|| AppError::not_found("model not found"))?;
    Ok(ModelHeader {
        id: row.get("id"),
        unique_key: row.get("unique_key"),
    })
}

async fn load_attributes(pool: &PgPool, model_id: i64) -> Result<Vec<AttributeDef>, AppError> {
    let rows = sqlx::query(
        "SELECT code, attr_type, required, choices, default_value
         FROM cmdb_attribute WHERE model_id = $1 AND deleted = 0",
    )
    .bind(model_id)
    .fetch_all(pool)
    .await
    .map_err(|_| AppError::internal("failed to read model attributes"))?;
    let mut defs = Vec::new();
    for row in rows {
        let attr_type: String = row.get("attr_type");
        let Some(attr_type) = AttrType::from_code(&attr_type) else {
            return Err(AppError::internal(format!(
                "model attribute {attr_type:?} has an unknown type"
            )));
        };
        defs.push(AttributeDef {
            code: row.get("code"),
            attr_type,
            required: row.get("required"),
            choices: row.get("choices"),
            default_value: row.get("default_value"),
        });
    }
    Ok(defs)
}

fn is_empty_value(value: &Value) -> bool {
    match value {
        Value::Null => true,
        Value::String(text) => text.trim().is_empty(),
        Value::Array(items) => items.is_empty(),
        _ => false,
    }
}

/// Validate a payload against the model definition. On create the full
/// required-check applies and defaults are filled; on update only the
/// provided keys are checked (merge semantics). Unknown keys are rejected
/// so instances cannot drift away from their model.
async fn validate_payload(
    pool: &PgPool,
    model: &ModelHeader,
    payload: &Value,
    is_create: bool,
) -> Result<Map<String, Value>, AppError> {
    let object = payload
        .as_object()
        .ok_or_else(|| AppError::bad_request("payload must be a JSON object"))?;
    let attributes = load_attributes(pool, model.id).await?;
    if attributes.is_empty() {
        return Err(AppError::bad_request(
            "model has no attributes; define them first",
        ));
    }
    let by_code: BTreeMap<&str, &AttributeDef> = attributes
        .iter()
        .map(|def| (def.code.as_str(), def))
        .collect();

    let mut unknown = Vec::new();
    for key in object.keys() {
        if key != "id" && !by_code.contains_key(key.as_str()) {
            unknown.push(key.clone());
        }
    }
    if !unknown.is_empty() {
        return Err(AppError::bad_request(format!(
            "unknown attributes for this model: {}",
            unknown.join(", ")
        )));
    }

    let mut missing = Vec::new();
    let mut invalid = Vec::new();
    let mut data = Map::new();
    for def in &attributes {
        match object.get(&def.code) {
            None | Some(Value::Null) if is_create => {
                if let Some(default) = &def.default_value {
                    data.insert(def.code.clone(), default.clone());
                } else if def.required {
                    missing.push(def.code.clone());
                }
            }
            provided => {
                if let Some(value) = provided {
                    if is_empty_value(value) {
                        if def.required && is_create {
                            missing.push(def.code.clone());
                        }
                    } else if let Err(reason) = def.attr_type.validate(value, def.choices.as_ref())
                    {
                        invalid.push(format!("{}: {reason}", def.code));
                    } else {
                        data.insert(def.code.clone(), value.clone());
                    }
                }
            }
        }
    }
    if !missing.is_empty() {
        return Err(AppError::bad_request(format!(
            "missing required attributes: {}",
            missing.join(", ")
        )));
    }
    if !invalid.is_empty() {
        return Err(AppError::bad_request(format!(
            "invalid attribute values: {}",
            invalid.join("; ")
        )));
    }
    if data.is_empty() {
        return Err(AppError::bad_request("no attribute values provided"));
    }
    Ok(data)
}

async fn enforce_unique_key(
    pool: &PgPool,
    model: &ModelHeader,
    data: &Map<String, Value>,
    instance_id: Option<i64>,
) -> Result<(), AppError> {
    let Some(key) = model.unique_key.as_deref().filter(|key| !key.is_empty()) else {
        return Ok(());
    };
    let Some(value) = data
        .get(key)
        .and_then(Value::as_str)
        .filter(|v| !v.trim().is_empty())
    else {
        return Ok(());
    };
    let duplicate: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM cmdb_instance
         WHERE model_id = $1 AND deleted = 0 AND id <> $2 AND attributes->>$3 = $4",
    )
    .bind(model.id)
    .bind(instance_id.unwrap_or(0))
    .bind(key)
    .bind(value)
    .fetch_one(pool)
    .await
    .map_err(|_| AppError::internal("failed to check unique key"))?;
    if duplicate > 0 {
        return Err(AppError::bad_request(format!(
            "unique key {key:?} already has an instance with value {value:?}"
        )));
    }
    Ok(())
}

fn instance_row(row: &sqlx::postgres::PgRow) -> Value {
    let attributes: Value = row.get("attributes");
    json!({
        "id": row.get::<i64, _>("id"),
        "modelId": row.get::<i64, _>("model_id"),
        "attributes": attributes,
        "createTime": row.get::<chrono::NaiveDateTime, _>("create_time").to_string(),
        "updateTime": row.get::<chrono::NaiveDateTime, _>("update_time").to_string(),
    })
}

fn page_bounds(page_no: Option<i64>, page_size: Option<i64>) -> (i64, i64) {
    let size = page_size.unwrap_or(20).clamp(1, 200);
    let no = page_no.unwrap_or(1).max(1);
    (size, (no - 1) * size)
}

async fn instance_page(
    State(state): State<CmdbState>,
    user: CurrentUser,
    Query(params): Query<InstancePageParams>,
) -> Result<Json<ApiResponse<Value>>, AppError> {
    require(&user, "cmdb:instance:query")?;
    load_model(&state.pool, params.model_id).await?;
    let (size, offset) = page_bounds(params.page_no, params.page_size);
    let keyword = params
        .keyword
        .as_deref()
        .map(str::trim)
        .filter(|k| !k.is_empty())
        .map(|k| format!("%{k}%"));
    let total: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM cmdb_instance
         WHERE model_id = $1 AND deleted = 0
           AND ($2::text IS NULL OR attributes::text ILIKE $2)",
    )
    .bind(params.model_id)
    .bind(&keyword)
    .fetch_one(&state.pool)
    .await
    .map_err(|_| AppError::internal("failed to count instances"))?;
    let rows = sqlx::query(
        "SELECT id, model_id, attributes, create_time, update_time
         FROM cmdb_instance
         WHERE model_id = $1 AND deleted = 0
           AND ($2::text IS NULL OR attributes::text ILIKE $2)
         ORDER BY id DESC LIMIT $3 OFFSET $4",
    )
    .bind(params.model_id)
    .bind(&keyword)
    .bind(size)
    .bind(offset)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| AppError::internal("failed to read instances"))?;
    Ok(Json(ApiResponse::new(json!({
        "list": rows.iter().map(instance_row).collect::<Vec<_>>(),
        "total": total,
    }))))
}

#[derive(Debug, Deserialize, JsonSchema)]
struct InstanceIdParams {
    id: i64,
}

async fn instance_get(
    State(state): State<CmdbState>,
    user: CurrentUser,
    Query(params): Query<InstanceIdParams>,
) -> Result<Json<ApiResponse<Value>>, AppError> {
    require(&user, "cmdb:instance:query")?;
    let row = sqlx::query(
        "SELECT id, model_id, attributes, create_time, update_time
         FROM cmdb_instance WHERE id = $1 AND deleted = 0",
    )
    .bind(params.id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| AppError::internal("failed to read instance"))?
    .ok_or_else(|| AppError::not_found("instance not found"))?;
    Ok(Json(ApiResponse::new(instance_row(&row))))
}

async fn instance_create(
    State(state): State<CmdbState>,
    user: CurrentUser,
    Json(payload): Json<Value>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    require(&user, "cmdb:instance:create")?;
    let model_id = payload
        .get("modelId")
        .and_then(Value::as_i64)
        .filter(|id| *id > 0)
        .ok_or_else(|| AppError::bad_request("modelId is required"))?;
    let model = load_model(&state.pool, model_id).await?;
    let attributes = payload
        .get("attributes")
        .ok_or_else(|| AppError::bad_request("attributes object is required"))?;
    let data = validate_payload(&state.pool, &model, attributes, true).await?;
    enforce_unique_key(&state.pool, &model, &data, None).await?;
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO cmdb_instance (model_id, attributes, creator, updater)
         VALUES ($1, $2, $3, $3) RETURNING id",
    )
    .bind(model_id)
    .bind(Value::Object(data))
    .bind(&user.username)
    .fetch_one(&state.pool)
    .await
    .map_err(|_| AppError::internal("failed to create instance"))?;
    Ok(Json(ApiResponse::new(id.to_string())))
}

async fn instance_update(
    State(state): State<CmdbState>,
    user: CurrentUser,
    Json(payload): Json<Value>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    require(&user, "cmdb:instance:update")?;
    let id = payload
        .get("id")
        .and_then(Value::as_i64)
        .filter(|id| *id > 0)
        .ok_or_else(|| AppError::bad_request("id is required"))?;
    let existing = sqlx::query(
        "SELECT id, model_id, attributes FROM cmdb_instance WHERE id = $1 AND deleted = 0",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| AppError::internal("failed to read instance"))?
    .ok_or_else(|| AppError::not_found("instance not found"))?;
    let model = load_model(&state.pool, existing.get::<i64, _>("model_id")).await?;
    let attributes = payload
        .get("attributes")
        .ok_or_else(|| AppError::bad_request("attributes object is required"))?;
    let patch = validate_payload(&state.pool, &model, attributes, false).await?;

    // Merge patch onto the stored attributes; the unique key may live in the
    // stored document when the patch does not repeat it.
    let mut merged = existing
        .get::<Value, _>("attributes")
        .as_object()
        .cloned()
        .unwrap_or_default();
    merged.extend(patch);
    enforce_unique_key(&state.pool, &model, &merged, Some(id)).await?;

    sqlx::query(
        "UPDATE cmdb_instance SET attributes = $2, updater = $3, update_time = now()
         WHERE id = $1 AND deleted = 0",
    )
    .bind(id)
    .bind(Value::Object(merged))
    .bind(&user.username)
    .execute(&state.pool)
    .await
    .map_err(|_| AppError::internal("failed to update instance"))?;
    Ok(Json(ApiResponse::new(())))
}

async fn instance_delete(
    State(state): State<CmdbState>,
    user: CurrentUser,
    Query(params): Query<InstanceIdParams>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    require(&user, "cmdb:instance:delete")?;
    let result = sqlx::query(
        "UPDATE cmdb_instance SET deleted = 1, updater = $2, update_time = now()
         WHERE id = $1 AND deleted = 0",
    )
    .bind(params.id)
    .bind(&user.username)
    .execute(&state.pool)
    .await
    .map_err(|_| AppError::internal("failed to delete instance"))?;
    if result.rows_affected() == 0 {
        return Err(AppError::not_found("instance not found"));
    }
    sqlx::query(
        "UPDATE cmdb_relation SET deleted = 1, updater = $2
         WHERE deleted = 0 AND (source_id = $1 OR target_id = $1)",
    )
    .bind(params.id)
    .bind(&user.username)
    .execute(&state.pool)
    .await
    .map_err(|_| AppError::internal("failed to detach relations"))?;
    Ok(Json(ApiResponse::new(())))
}

#[derive(Debug, Deserialize, JsonSchema)]
struct DeleteListParams {
    ids: String,
}

async fn instance_delete_list(
    State(state): State<CmdbState>,
    user: CurrentUser,
    Query(params): Query<DeleteListParams>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    require(&user, "cmdb:instance:delete")?;
    let ids: Vec<i64> = params
        .ids
        .split(',')
        .filter_map(|item| item.trim().parse().ok())
        .filter(|id| *id > 0)
        .collect();
    if ids.is_empty() {
        return Err(AppError::bad_request("ids is required"));
    }
    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|_| AppError::internal("failed to start delete"))?;
    for id in &ids {
        sqlx::query(
            "UPDATE cmdb_instance SET deleted = 1, updater = $2, update_time = now()
             WHERE id = $1 AND deleted = 0",
        )
        .bind(id)
        .bind(&user.username)
        .execute(&mut *tx)
        .await
        .map_err(|_| AppError::internal("failed to delete instance"))?;
        sqlx::query(
            "UPDATE cmdb_relation SET deleted = 1, updater = $2
             WHERE deleted = 0 AND (source_id = $1 OR target_id = $1)",
        )
        .bind(id)
        .bind(&user.username)
        .execute(&mut *tx)
        .await
        .map_err(|_| AppError::internal("failed to detach relations"))?;
    }
    tx.commit()
        .await
        .map_err(|_| AppError::internal("failed to commit delete"))?;
    Ok(Json(ApiResponse::new(())))
}

// ---------- Excel export / import ----------

#[derive(Debug, Deserialize, JsonSchema)]
struct ExportParams {
    model_id: i64,
}

async fn instance_export(
    State(state): State<CmdbState>,
    user: CurrentUser,
    Query(params): Query<ExportParams>,
) -> Result<axum::response::Response, AppError> {
    use rust_xlsxwriter::Workbook;

    require(&user, "cmdb:instance:query")?;
    let model = load_model(&state.pool, params.model_id).await?;
    let attributes = load_attributes(&state.pool, params.model_id).await?;
    let rows = sqlx::query(
        "SELECT attributes FROM cmdb_instance WHERE model_id = $1 AND deleted = 0 ORDER BY id LIMIT 10000",
    )
    .bind(params.model_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| AppError::internal("failed to read instances"))?;

    let mut workbook = Workbook::new();
    let worksheet = workbook
        .add_worksheet()
        .set_name("instances")
        .map_err(|_| AppError::internal("failed to create sheet"))?;
    for (col, attr) in attributes.iter().enumerate() {
        worksheet
            .write_string(0, col as u16, &attr.code)
            .map_err(|_| AppError::internal("failed to write header"))?;
    }
    for (row_index, row) in rows.iter().enumerate() {
        let payload: Value = row.get("attributes");
        for (col, attr) in attributes.iter().enumerate() {
            let cell = payload.get(&attr.code).map(cell_text).unwrap_or_default();
            worksheet
                .write_string((row_index + 1) as u32, col as u16, cell)
                .map_err(|_| AppError::internal("failed to write row"))?;
        }
    }
    let bytes = workbook
        .save_to_buffer()
        .map_err(|_| AppError::internal("failed to build excel"))?;
    axum::response::Response::builder()
        .header(
            "content-type",
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        )
        .header(
            "content-disposition",
            format!(
                "attachment; filename=\"cmdb_instances_model_{}.xlsx\"",
                model.id
            ),
        )
        .body(axum::body::Body::from(bytes))
        .map_err(|_| AppError::internal("failed to build download"))
}

fn cell_text(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Bool(flag) => flag.to_string(),
        Value::Number(_) => value.to_string(),
        Value::Array(items) => items.iter().map(cell_text).collect::<Vec<_>>().join(","),
        other => serde_json::to_string(other).unwrap_or_default(),
    }
}

async fn instance_import(
    State(state): State<CmdbState>,
    user: CurrentUser,
    mut multipart: axum::extract::Multipart,
) -> Result<Json<ApiResponse<Value>>, AppError> {
    use calamine::{Data, DataType as _, Reader, Xlsx};

    require(&user, "cmdb:instance:create")?;
    let model_id = std::env::var("CMDB_IMPORT_MODEL")
        .ok()
        .and_then(|_| None::<i64>);
    let mut model_id = model_id.unwrap_or(0);
    let mut file_bytes: Option<Vec<u8>> = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| AppError::bad_request("invalid multipart payload"))?
    {
        match field.name() {
            Some("modelId") => {
                let text = field
                    .text()
                    .await
                    .map_err(|_| AppError::bad_request("invalid modelId"))?;
                model_id = text.trim().parse().unwrap_or(0);
            }
            Some("file") => {
                file_bytes = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|_| AppError::bad_request("invalid file"))?
                        .to_vec(),
                );
            }
            _ => {}
        }
    }
    if model_id <= 0 {
        return Err(AppError::bad_request("modelId is required"));
    }
    let Some(file_bytes) = file_bytes else {
        return Err(AppError::bad_request("file is required"));
    };
    let model = load_model(&state.pool, model_id).await?;
    let attributes = load_attributes(&state.pool, model_id).await?;
    let by_code: std::collections::BTreeMap<&str, &AttributeDef> = attributes
        .iter()
        .map(|def| (def.code.as_str(), def))
        .collect();

    let mut reader = Xlsx::new(std::io::Cursor::new(file_bytes))
        .map_err(|_| AppError::bad_request("无法读取 xlsx 文件"))?;
    let sheet = reader.sheet_names().first().cloned().unwrap_or_default();
    let range = reader
        .worksheet_range(&sheet)
        .map_err(|_| AppError::bad_request("无法读取工作表"))?;

    let mut created = 0i64;
    let mut errors: Vec<Value> = Vec::new();
    for (row_index, row) in range.rows().enumerate() {
        if row_index == 0 {
            let unknown: Vec<String> = row
                .iter()
                .filter_map(|cell| cell.get_string().map(str::to_string))
                .filter(|code| !code.trim().is_empty() && !by_code.contains_key(code.trim()))
                .collect();
            if !unknown.is_empty() {
                return Err(AppError::bad_request(format!(
                    "表头包含模型未定义的属性: {}",
                    unknown.join(", ")
                )));
            }
            continue;
        }
        if row.iter().all(|cell| matches!(cell, Data::Empty)) {
            continue;
        }
        let mut payload = Map::new();
        for (col, cell) in row.iter().enumerate() {
            let Some(code) = range
                .rows()
                .next()
                .and_then(|header| header.get(col))
                .and_then(|cell| cell.get_string().map(str::to_string))
                .map(|code| code.trim().to_string())
                .filter(|code| !code.is_empty())
            else {
                continue;
            };
            let Some(def) = by_code.get(code.as_str()) else {
                continue;
            };
            let raw = match cell {
                Data::Empty => continue,
                Data::String(text) => Value::String(text.clone()),
                Data::Float(number) => def
                    .attr_type
                    .code()
                    .contains("float")
                    .then(|| json!(number))
                    .unwrap_or_else(|| json!(*number as i64)),
                Data::Int(number) => json!(number),
                Data::Bool(flag) => json!(flag),
                Data::DateTime(excel) => json!(excel.to_string()),
                other => Value::String(other.to_string()),
            };
            payload.insert(code, raw);
        }
        if payload.is_empty() {
            continue;
        }
        let row_no = row_index + 1;
        match validate_payload(&state.pool, &model, &Value::Object(payload.clone()), true).await {
            Ok(data) => {
                if let Err(reason) = enforce_unique_key(&state.pool, &model, &data, None).await {
                    errors.push(json!({"row": row_no, "error": reason}));
                    continue;
                }
                let insert = sqlx::query(
                    "INSERT INTO cmdb_instance (model_id, attributes, creator, updater)
                     VALUES ($1, $2, $3, $3)",
                )
                .bind(model_id)
                .bind(Value::Object(data))
                .bind(&user.username)
                .execute(&state.pool)
                .await;
                match insert {
                    Ok(_) => created += 1,
                    Err(_) => errors.push(json!({"row": row_no, "error": "写入数据库失败"})),
                }
            }
            Err(error) => errors.push(json!({"row": row_no, "error": error.message()})),
        }
    }

    Ok(Json(ApiResponse::new(json!({
        "created": created,
        "failed": errors.len(),
        "errors": errors.into_iter().take(50).collect::<Vec<_>>(),
    }))))
}
