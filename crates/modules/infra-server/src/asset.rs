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
use rustset_framework_security::CurrentUser;
use rustset_framework_web::AppError;
use serde_json::{Value, json};
use sqlx::Row;
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
const NETWORK_POLICY: TableSpec = TableSpec {
    table: "infra_network_policy",
    seq: "infra_network_policy_seq",
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
        .route("/infra/network-policy/page", get(network_policy_page))
        .route("/infra/network-policy/list", get(network_policy_list))
        .route("/infra/network-policy/get", get(network_policy_get))
        .route("/infra/network-policy/create", post(network_policy_create))
        .route("/infra/network-policy/update", put(network_policy_update))
        .route(
            "/infra/network-policy/delete",
            delete(network_policy_delete),
        )
        .route(
            "/infra/network-policy/delete-list",
            delete(network_policy_delete_list),
        )
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

async fn network_policy_page(
    State(state): State<InfraState>,
    Query(p): Query<QueryParams>,
) -> Result<Json<ApiResponse<crate::Page<Value>>>, AppError> {
    table_page(&state.pool, NETWORK_POLICY, p).await
}
async fn network_policy_list(
    State(state): State<InfraState>,
) -> Result<Json<ApiResponse<Vec<Value>>>, AppError> {
    table_list(&state.pool, NETWORK_POLICY).await
}
async fn network_policy_get(
    State(state): State<InfraState>,
    Query(p): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<Value>>, AppError> {
    table_get(&state.pool, NETWORK_POLICY, id_param(&p)?).await
}
async fn network_policy_create(
    State(state): State<InfraState>,
    Json(p): Json<Value>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    table_create(&state.pool, NETWORK_POLICY, p).await
}
async fn network_policy_update(
    State(state): State<InfraState>,
    Json(p): Json<Value>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    table_update(&state.pool, NETWORK_POLICY, p).await
}
async fn network_policy_delete(
    State(state): State<InfraState>,
    Query(p): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    soft_delete(&state.pool, NETWORK_POLICY.table, id_param(&p)?).await
}
async fn network_policy_delete_list(
    State(state): State<InfraState>,
    Query(p): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    soft_delete_list(&state.pool, NETWORK_POLICY.table, ids_param(&p)).await
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
    user: CurrentUser,
    Json(p): Json<Value>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    let Json(created) = table_create(&state.pool, ASSET, p).await?;
    if let Ok(asset_id) = created.data.parse::<i64>() {
        auto_attribute_ownership(&state.pool, asset_id).await;
    }
    let _ = user;
    Ok(Json(created))
}

/// Longest-prefix segment match over cmdb_net_zone; fills net_zone_id and
/// organization_name when the IP falls inside a managed segment and the
/// ownership is not manually pinned.
pub(crate) async fn auto_attribute_ownership(pool: &sqlx::PgPool, asset_id: i64) {
    let Ok((asset_id, ip)) = sqlx::query_as::<_, (i64, String)>(
        // Skip only assets whose organization a human chose; the default
        // 'manual' marker with an empty organization still gets attributed.
        "SELECT id, ip FROM infra_asset WHERE id = $1 AND deleted = 0
           AND NOT (ownership_source = 'manual'
                    AND organization_name IS NOT NULL AND organization_name <> '')",
    )
    .bind(asset_id)
    .fetch_one(pool)
    .await
    else {
        return;
    };
    let segments = sqlx::query(
        "SELECT id, cidr FROM cmdb_net_zone WHERE deleted = 0 AND cidr IS NOT NULL AND cidr <> ''",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    let mut best: Option<(u8, i64)> = None;
    for row in &segments {
        let cidr: String = row.get("cidr");
        let Some((address, prefix)) = cidr.split_once('/') else {
            continue;
        };
        let (Ok(network), Ok(prefix)) = (
            address.trim().parse::<std::net::Ipv4Addr>(),
            prefix.trim().parse::<u8>(),
        ) else {
            continue;
        };
        if prefix > 32 {
            continue;
        }
        let Ok(ip_addr) = ip.trim().parse::<std::net::Ipv4Addr>() else {
            return;
        };
        let mask = if prefix == 0 {
            0
        } else {
            u32::MAX << (32 - prefix as u32)
        };
        if (u32::from(network) & mask) == (u32::from(ip_addr) & mask)
            && best
                .as_ref()
                .map(|(best_prefix, _)| prefix > *best_prefix)
                .unwrap_or(true)
        {
            best = Some((prefix, row.get("id")));
        }
    }
    let Some((_, zone_id)) = best else { return };
    // Attribute the nearest company/subsidiary ancestor's name, falling
    // back to the matched segment itself.
    let _ = sqlx::query(
        "UPDATE infra_asset a
         SET net_zone_id = $2,
             organization_name = COALESCE((
                 WITH RECURSIVE ancestors AS (
                     SELECT id, name, zone_type, parent_id FROM cmdb_net_zone WHERE id = $2
                     UNION ALL
                     SELECT z.id, z.name, z.zone_type, z.parent_id
                     FROM cmdb_net_zone z JOIN ancestors an ON z.id = an.parent_id
                 )
                 SELECT name FROM ancestors
                 WHERE zone_type IN ('company','subsidiary')
                 ORDER BY id LIMIT 1
             ), (SELECT name FROM cmdb_net_zone WHERE id = $2), a.organization_name),
             ownership_source = 'segment',
             update_time = now()
         WHERE a.id = $1 AND a.deleted = 0",
    )
    .bind(asset_id)
    .bind(zone_id)
    .execute(pool)
    .await;
}
async fn asset_update(
    State(state): State<InfraState>,
    Json(p): Json<Value>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    let id = p
        .get("id")
        .and_then(Value::as_i64)
        .filter(|value| *value > 0)
        .ok_or_else(|| AppError::bad_request("id is required"))?;
    let _ = table_update(&state.pool, ASSET, p).await?;
    auto_attribute_ownership(&state.pool, id).await;
    Ok(Json(ApiResponse::new(())))
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
    Json(mut p): Json<Value>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    if let Some(object) = p.as_object_mut() {
        if let Some(Value::Bool(enabled)) = object.get("hasSecurityProduct").cloned() {
            object.insert(
                "hasSecurityProduct".to_string(),
                Value::Number((enabled as i32).into()),
            );
        }
        let resource_id = uuid::Uuid::new_v4().to_string();
        object
            .entry("resourceId".to_string())
            .or_insert(Value::String(resource_id.clone()));
        object
            .entry("instanceId".to_string())
            .or_insert(Value::String(resource_id));
    }
    table_create(&state.pool, BUSINESS_RESOURCE, p).await
}
async fn biz_update(
    State(state): State<InfraState>,
    Json(mut p): Json<Value>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    if let Some(object) = p.as_object_mut()
        && let Some(Value::Bool(enabled)) = object.get("hasSecurityProduct").cloned()
    {
        object.insert(
            "hasSecurityProduct".to_string(),
            Value::Number((enabled as i32).into()),
        );
    }
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
