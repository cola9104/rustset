use crate::{
    InfraState, QueryParams, TableSpec, id_param, ids_param, soft_delete, soft_delete_list,
    table_create, table_get, table_list, table_page, table_update,
};
use axum::{
    Json, Router,
    extract::{Query, State},
    routing::{delete, get, post, put},
};
use rustset_framework_common::ApiResponse;
use rustset_framework_web::AppError;
use serde_json::{Value, json};
use std::collections::HashMap;

const ASSET: TableSpec = TableSpec {
    table: "infra_asset",
    seq: "infra_asset_seq",
};
const CLOUD_ASSET: TableSpec = TableSpec {
    table: "infra_cloud_asset",
    seq: "infra_cloud_asset_seq",
};
const BUSINESS_RESOURCE: TableSpec = TableSpec {
    table: "infra_business_resource",
    seq: "infra_business_resource_seq",
};

pub fn routes() -> Router<InfraState> {
    Router::new()
        .route("/infra/asset/page", get(asset_page))
        .route("/infra/asset/list", get(asset_list))
        .route("/infra/asset/get", get(asset_get))
        .route("/infra/asset/create", post(asset_create))
        .route("/infra/asset/update", put(asset_update))
        .route("/infra/asset/delete", delete(asset_delete))
        .route("/infra/asset/delete-list", delete(asset_delete_list))
        .route("/infra/asset/{id}/port/add", post(asset_add_port))
        .route("/infra/asset/{id}/port/{port}", put(asset_update_port))
        .route("/infra/asset/{id}/port/{port}", delete(asset_delete_port))
        .route("/infra/cloud-asset/page", get(cloud_asset_page))
        .route("/infra/cloud-asset/list", get(cloud_asset_list))
        .route("/infra/cloud-asset/get", get(cloud_asset_get))
        .route("/infra/cloud-asset/create", post(cloud_asset_create))
        .route("/infra/cloud-asset/update", put(cloud_asset_update))
        .route("/infra/cloud-asset/delete", delete(cloud_asset_delete))
        .route("/infra/business-resource/page", get(biz_page))
        .route("/infra/business-resource/list", get(biz_list))
        .route("/infra/business-resource/get", get(biz_get))
        .route("/infra/business-resource/create", post(biz_create))
        .route("/infra/business-resource/update", put(biz_update))
        .route("/infra/business-resource/delete", delete(biz_delete))
        .route(
            "/infra/business-resource/delete-list",
            delete(biz_delete_list),
        )
}

async fn asset_page(
    State(state): State<InfraState>,
    Query(p): Query<QueryParams>,
) -> Result<Json<ApiResponse<crate::Page<Value>>>, AppError> {
    table_page(&state.pool, ASSET, p).await
}
async fn asset_list(
    State(state): State<InfraState>,
) -> Result<Json<ApiResponse<Vec<Value>>>, AppError> {
    table_list(&state.pool, ASSET).await
}
async fn asset_get(
    State(state): State<InfraState>,
    Query(p): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<Value>>, AppError> {
    table_get(&state.pool, ASSET, id_param(&p)?).await
}
async fn asset_create(
    State(state): State<InfraState>,
    Json(p): Json<Value>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    table_create(&state.pool, ASSET, p).await
}
async fn asset_update(
    State(state): State<InfraState>,
    Json(p): Json<Value>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    table_update(&state.pool, ASSET, p).await
}
async fn asset_delete(
    State(state): State<InfraState>,
    Query(p): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    soft_delete(&state.pool, ASSET.table, id_param(&p)?).await
}
async fn asset_delete_list(
    State(state): State<InfraState>,
    Query(p): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    soft_delete_list(&state.pool, ASSET.table, ids_param(&p)).await
}

async fn asset_add_port(
    State(state): State<InfraState>,
    axum::extract::Path(id): axum::extract::Path<i64>,
    Json(payload): Json<Value>,
) -> Result<Json<ApiResponse<Value>>, AppError> {
    let port_num = crate::i32_field(&payload, "port", 0);
    let existing = crate::table_get_value(&state.pool, ASSET, id).await?;
    let mut ports: Vec<Value> = existing
        .get("ports")
        .and_then(|v| v.as_str())
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default();
    if ports
        .iter()
        .any(|p| p.get("port").and_then(|v| v.as_i64()) == Some(port_num as i64))
    {
        return Err(AppError::bad_request("Port already exists"));
    }
    ports.push(json!({"port": port_num, "isOpen": true, "service": crate::opt_str_field(&payload, "service"), "banner": crate::opt_str_field(&payload, "banner"), "isBound": crate::bool_field(&payload, "isBound", false), "systemName": crate::opt_str_field(&payload, "systemName"), "middleware": crate::opt_str_field(&payload, "middleware")}));
    let ports_json = serde_json::to_string(&ports).unwrap_or_default();
    sqlx::query("UPDATE infra_asset SET ports=$2, update_time=now() WHERE id=$1 AND deleted=0")
        .bind(id)
        .bind(&ports_json)
        .execute(&state.pool)
        .await
        .map_err(|_| AppError::internal("failed to add port"))?;
    Ok(Json(ApiResponse::new(json!({"id": id, "ports": ports}))))
}

async fn asset_update_port(
    State(state): State<InfraState>,
    axum::extract::Path((id, port_num)): axum::extract::Path<(i64, i32)>,
    Json(payload): Json<Value>,
) -> Result<Json<ApiResponse<Value>>, AppError> {
    let existing = crate::table_get_value(&state.pool, ASSET, id).await?;
    let mut ports: Vec<Value> = existing
        .get("ports")
        .and_then(|v| v.as_str())
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default();
    if let Some(p) = ports
        .iter_mut()
        .find(|p| p.get("port").and_then(|v| v.as_i64()) == Some(port_num as i64))
    {
        *p = json!({"port": port_num, "isOpen": true, "service": crate::opt_str_field(&payload, "service"), "banner": crate::opt_str_field(&payload, "banner"), "isBound": crate::bool_field(&payload, "isBound", p.get("isBound").and_then(|v| v.as_bool()).unwrap_or(false)), "systemName": crate::opt_str_field(&payload, "systemName"), "middleware": crate::opt_str_field(&payload, "middleware")});
    }
    let ports_json = serde_json::to_string(&ports).unwrap_or_default();
    sqlx::query("UPDATE infra_asset SET ports=$2, update_time=now() WHERE id=$1 AND deleted=0")
        .bind(id)
        .bind(&ports_json)
        .execute(&state.pool)
        .await
        .map_err(|_| AppError::internal("failed"))?;
    Ok(Json(ApiResponse::new(json!({"id": id, "ports": ports}))))
}

async fn asset_delete_port(
    State(state): State<InfraState>,
    axum::extract::Path((id, port_num)): axum::extract::Path<(i64, i32)>,
) -> Result<Json<ApiResponse<Value>>, AppError> {
    let existing = crate::table_get_value(&state.pool, ASSET, id).await?;
    let mut ports: Vec<Value> = existing
        .get("ports")
        .and_then(|v| v.as_str())
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default();
    ports.retain(|p| p.get("port").and_then(|v| v.as_i64()) != Some(port_num as i64));
    let ports_json = serde_json::to_string(&ports).unwrap_or_default();
    sqlx::query("UPDATE infra_asset SET ports=$2, update_time=now() WHERE id=$1 AND deleted=0")
        .bind(id)
        .bind(&ports_json)
        .execute(&state.pool)
        .await
        .map_err(|_| AppError::internal("failed"))?;
    Ok(Json(ApiResponse::new(json!({"id": id, "ports": ports}))))
}

async fn cloud_asset_page(
    State(state): State<InfraState>,
    Query(p): Query<QueryParams>,
) -> Result<Json<ApiResponse<crate::Page<Value>>>, AppError> {
    table_page(&state.pool, CLOUD_ASSET, p).await
}
async fn cloud_asset_list(
    State(state): State<InfraState>,
) -> Result<Json<ApiResponse<Vec<Value>>>, AppError> {
    table_list(&state.pool, CLOUD_ASSET).await
}
async fn cloud_asset_get(
    State(state): State<InfraState>,
    Query(p): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<Value>>, AppError> {
    table_get(&state.pool, CLOUD_ASSET, id_param(&p)?).await
}
async fn cloud_asset_create(
    State(state): State<InfraState>,
    Json(p): Json<Value>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    table_create(&state.pool, CLOUD_ASSET, p).await
}
async fn cloud_asset_update(
    State(state): State<InfraState>,
    Json(p): Json<Value>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    table_update(&state.pool, CLOUD_ASSET, p).await
}
async fn cloud_asset_delete(
    State(state): State<InfraState>,
    Query(p): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    soft_delete(&state.pool, CLOUD_ASSET.table, id_param(&p)?).await
}

async fn biz_page(
    State(state): State<InfraState>,
    Query(p): Query<QueryParams>,
) -> Result<Json<ApiResponse<crate::Page<Value>>>, AppError> {
    table_page(&state.pool, BUSINESS_RESOURCE, p).await
}
async fn biz_list(
    State(state): State<InfraState>,
) -> Result<Json<ApiResponse<Vec<Value>>>, AppError> {
    table_list(&state.pool, BUSINESS_RESOURCE).await
}
async fn biz_get(
    State(state): State<InfraState>,
    Query(p): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<Value>>, AppError> {
    table_get(&state.pool, BUSINESS_RESOURCE, id_param(&p)?).await
}
async fn biz_create(
    State(state): State<InfraState>,
    Json(p): Json<Value>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    table_create(&state.pool, BUSINESS_RESOURCE, p).await
}
async fn biz_update(
    State(state): State<InfraState>,
    Json(p): Json<Value>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    table_update(&state.pool, BUSINESS_RESOURCE, p).await
}
async fn biz_delete(
    State(state): State<InfraState>,
    Query(p): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    soft_delete(&state.pool, BUSINESS_RESOURCE.table, id_param(&p)?).await
}
async fn biz_delete_list(
    State(state): State<InfraState>,
    Query(p): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    soft_delete_list(&state.pool, BUSINESS_RESOURCE.table, ids_param(&p)).await
}
