//! CMDB configuration-item instances: dynamic JSONB payloads validated
//! against the owning model's attribute definitions, with unique-key
//! enforcement (veops-style 唯一键) and keyword search.

use std::collections::BTreeMap;

use axum::{
    Json, Router,
    extract::{Query, State},
    routing::{delete, get, post, put},
};
use rustset_cmdb_api::AttrType;
use rustset_framework_common::ApiResponse;
use rustset_framework_security::CurrentUser;
use rustset_framework_web::AppError;
use serde::Deserialize;
use serde_json::{Map, Value, json};
use sqlx::{PgPool, Row};

use crate::{CmdbState, require};

pub fn routes() -> Router<CmdbState> {
    Router::new()
        .route("/cmdb/instance/page", get(instance_page))
        .route("/cmdb/instance/get", get(instance_get))
        .route("/cmdb/instance/create", post(instance_create))
        .route("/cmdb/instance/update", put(instance_update))
        .route("/cmdb/instance/delete", delete(instance_delete))
        .route("/cmdb/instance/delete-list", delete(instance_delete_list))
}

#[derive(Debug, Deserialize)]
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
                    } else if let Err(reason) =
                        def.attr_type.validate(value, def.choices.as_ref())
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
    let Some(value) = data.get(key).and_then(Value::as_str).filter(|v| !v.trim().is_empty()) else {
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

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
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
