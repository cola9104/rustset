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

const CLOUD_ZONE: TableSpec = TableSpec {
    table: "infra_cloud_zone",
    seq: "infra_cloud_zone_seq",
};
const CLOUD_PLATFORM: TableSpec = TableSpec {
    table: "infra_cloud_platform",
    seq: "infra_cloud_platform_seq",
};
const CLOUD_PROVIDER_CONFIG: TableSpec = TableSpec {
    table: "infra_cloud_provider_config",
    seq: "infra_cloud_provider_config_seq",
};

pub fn routes() -> Router<InfraState> {
    Router::new()
        .route("/infra/cloud-zone/page", get(zone_page))
        .route("/infra/cloud-zone/list", get(zone_list))
        .route("/infra/cloud-zone/get", get(zone_get))
        .route("/infra/cloud-zone/create", post(zone_create))
        .route("/infra/cloud-zone/update", put(zone_update))
        .route("/infra/cloud-zone/delete", delete(zone_delete))
        .route("/infra/cloud-zone/delete-list", delete(zone_delete_list))
        .route("/infra/cloud-platform/page", get(platform_page))
        .route("/infra/cloud-platform/list", get(platform_list))
        .route(
            "/infra/cloud-platform/list-by-zone",
            get(platform_list_by_zone),
        )
        .route("/infra/cloud-platform/get", get(platform_get))
        .route("/infra/cloud-platform/create", post(platform_create))
        .route("/infra/cloud-platform/update", put(platform_update))
        .route("/infra/cloud-platform/delete", delete(platform_delete))
        .route(
            "/infra/cloud-platform/delete-list",
            delete(platform_delete_list),
        )
        .route("/infra/cloud-provider-config/page", get(config_page))
        .route("/infra/cloud-provider-config/list", get(config_list))
        .route("/infra/cloud-provider-config/get", get(config_get))
        .route("/infra/cloud-provider-config/create", post(config_create))
        .route("/infra/cloud-provider-config/update", put(config_update))
        .route("/infra/cloud-provider-config/delete", delete(config_delete))
        .route(
            "/infra/cloud-provider-config/delete-list",
            delete(config_delete_list),
        )
        .route(
            "/infra/cloud-provider-config/test-connection",
            post(config_test_connection),
        )
}

async fn zone_page(
    State(state): State<InfraState>,
    Query(params): Query<QueryParams>,
) -> Result<Json<ApiResponse<crate::Page<Value>>>, AppError> {
    table_page(&state.pool, CLOUD_ZONE, params).await
}
async fn zone_list(
    State(state): State<InfraState>,
) -> Result<Json<ApiResponse<Vec<Value>>>, AppError> {
    table_list(&state.pool, CLOUD_ZONE).await
}
async fn zone_get(
    State(state): State<InfraState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<Value>>, AppError> {
    table_get(&state.pool, CLOUD_ZONE, id_param(&params)?).await
}
async fn zone_create(
    State(state): State<InfraState>,
    Json(payload): Json<Value>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    table_create(&state.pool, CLOUD_ZONE, payload).await
}
async fn zone_update(
    State(state): State<InfraState>,
    Json(payload): Json<Value>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    table_update(&state.pool, CLOUD_ZONE, payload).await
}
async fn zone_delete(
    State(state): State<InfraState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    soft_delete(&state.pool, CLOUD_ZONE.table, id_param(&params)?).await
}
async fn zone_delete_list(
    State(state): State<InfraState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    soft_delete_list(&state.pool, CLOUD_ZONE.table, ids_param(&params)).await
}

async fn platform_page(
    State(state): State<InfraState>,
    Query(params): Query<QueryParams>,
) -> Result<Json<ApiResponse<crate::Page<Value>>>, AppError> {
    table_page(&state.pool, CLOUD_PLATFORM, params).await
}
async fn platform_list(
    State(state): State<InfraState>,
) -> Result<Json<ApiResponse<Vec<Value>>>, AppError> {
    table_list(&state.pool, CLOUD_PLATFORM).await
}
async fn platform_list_by_zone(
    State(state): State<InfraState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<Vec<Value>>>, AppError> {
    crate::table_list_by_i64(
        &state.pool,
        CLOUD_PLATFORM,
        "zone_id",
        crate::id_named_param(&params, "zoneId")?,
    )
    .await
}
async fn platform_get(
    State(state): State<InfraState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<Value>>, AppError> {
    table_get(&state.pool, CLOUD_PLATFORM, id_param(&params)?).await
}
async fn platform_create(
    State(state): State<InfraState>,
    Json(payload): Json<Value>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    table_create(&state.pool, CLOUD_PLATFORM, payload).await
}
async fn platform_update(
    State(state): State<InfraState>,
    Json(payload): Json<Value>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    table_update(&state.pool, CLOUD_PLATFORM, payload).await
}
async fn platform_delete(
    State(state): State<InfraState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    soft_delete(&state.pool, CLOUD_PLATFORM.table, id_param(&params)?).await
}
async fn platform_delete_list(
    State(state): State<InfraState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    soft_delete_list(&state.pool, CLOUD_PLATFORM.table, ids_param(&params)).await
}

async fn config_page(
    State(state): State<InfraState>,
    Query(params): Query<QueryParams>,
) -> Result<Json<ApiResponse<crate::Page<Value>>>, AppError> {
    table_page(&state.pool, CLOUD_PROVIDER_CONFIG, params).await
}
async fn config_list(
    State(state): State<InfraState>,
) -> Result<Json<ApiResponse<Vec<Value>>>, AppError> {
    table_list(&state.pool, CLOUD_PROVIDER_CONFIG).await
}
async fn config_get(
    State(state): State<InfraState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<Value>>, AppError> {
    table_get(&state.pool, CLOUD_PROVIDER_CONFIG, id_param(&params)?).await
}
async fn config_create(
    State(state): State<InfraState>,
    Json(payload): Json<Value>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    table_create(&state.pool, CLOUD_PROVIDER_CONFIG, payload).await
}
async fn config_update(
    State(state): State<InfraState>,
    Json(payload): Json<Value>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    table_update(&state.pool, CLOUD_PROVIDER_CONFIG, payload).await
}
async fn config_delete(
    State(state): State<InfraState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    soft_delete(&state.pool, CLOUD_PROVIDER_CONFIG.table, id_param(&params)?).await
}
async fn config_delete_list(
    State(state): State<InfraState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    soft_delete_list(&state.pool, CLOUD_PROVIDER_CONFIG.table, ids_param(&params)).await
}

async fn config_test_connection(
    State(state): State<InfraState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<Value>>, AppError> {
    let id = id_param(&params)?;
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let _ = sqlx::query("UPDATE infra_cloud_provider_config SET last_test_time=$2::timestamp, last_test_result=$3, update_time=now() WHERE id=$1 AND deleted=0")
        .bind(id).bind(&now).bind("Connection test not implemented").execute(&state.pool).await;
    Ok(Json(ApiResponse::new(
        json!({"success": false, "message": "Connection test not implemented", "testedAt": now}),
    )))
}
