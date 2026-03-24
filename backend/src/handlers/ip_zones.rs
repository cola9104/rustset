//! IP Zones Handler - IP 区域管理
//!
//! 功能：
//! - 自动判断 IP 所属区域
//! - CIDR 批量管理
//! - Zone 冲突检测

use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;

use crate::database::{
    db_zone_to_shared, delete_zone as db_delete_zone, get_cloud_platform_config_by_id, get_db,
    get_machine_room_by_id, get_zones as db_get_zones, insert_zone_wrapper as db_insert_zone,
    update_zone as db_update_zone,
};
use crate::middleware::{ApiError, AuthUser};
use crate::state::AppState;
use crate::utils::log_action_auth;
use shared::{NetworkZone, Role, ZoneConfig};

/// IP Zone 查询结果
#[derive(Debug, Serialize)]
pub struct IpZoneResult {
    pub ip: String,
    pub zone: String,
    pub zone_type: NetworkZone,
    pub matched_cidr: Option<String>,
}

/// 查询 IP 所属区域
#[derive(Debug, Deserialize)]
pub struct FindZoneQuery {
    pub ip: String,
}

fn sync_zone_cache(state: &AppState, zones: &[ZoneConfig]) -> Result<(), ApiError> {
    *state
        .zones
        .write()
        .map_err(|e| ApiError::internal(format!("Failed to write zones cache: {}", e)))? =
        zones.to_vec();
    Ok(())
}

fn load_zone_cache(state: &AppState) -> Result<Vec<ZoneConfig>, ApiError> {
    state
        .zones
        .read()
        .map(|zones| zones.clone())
        .map_err(|e| ApiError::internal(format!("Failed to read zones cache: {}", e)))
}

async fn load_zones(state: &AppState) -> Result<Vec<ZoneConfig>, ApiError> {
    match db_get_zones().await {
        Ok(db_zones) => {
            let zones: Vec<ZoneConfig> = db_zones.into_iter().map(db_zone_to_shared).collect();
            sync_zone_cache(state, &zones)?;
            Ok(zones)
        }
        Err(_) => load_zone_cache(state),
    }
}

fn zone_type_from_name(name: &str) -> NetworkZone {
    match name {
        "Internet" => NetworkZone::Internet,
        "DMZ" => NetworkZone::DMZ,
        "Intranet" => NetworkZone::Intranet,
        other => NetworkZone::Custom(other.to_string()),
    }
}

/// 查找 IP 所属区域
pub async fn find_zone_by_ip(
    _user: AuthUser,
    State(state): State<AppState>,
    Query(query): Query<FindZoneQuery>,
) -> Result<Json<IpZoneResult>, ApiError> {
    // 解析 IP
    let ip: IpAddr = query
        .ip
        .parse()
        .map_err(|_| ApiError::bad_request("Invalid IP address format"))?;

    let zones = load_zones(&state).await?;

    // 查找匹配的 zone
    let mut matching_zones: Vec<&ZoneConfig> = zones
        .iter()
        .filter(|z| {
            if let Ok(net) = z.cidr.parse::<IpNetwork>() {
                net.contains(ip)
            } else {
                false
            }
        })
        .collect();

    // 按优先级排序，取最高优先级
    matching_zones.sort_by(|a, b| b.priority.cmp(&a.priority));

    let matched = matching_zones.first();

    let (zone_name, zone_type, matched_cidr) = match matched {
        Some(z) => (
            z.name.clone(),
            zone_type_from_name(&z.name),
            Some(z.cidr.clone()),
        ),
        None => ("Internet".to_string(), NetworkZone::Internet, None),
    };

    Ok(Json(IpZoneResult {
        ip: query.ip,
        zone: zone_name,
        zone_type,
        matched_cidr,
    }))
}

/// 获取所有 Zone 配置
pub async fn get_ip_zones(
    _user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<ZoneConfig>>, ApiError> {
    Ok(Json(load_zones(&state).await?))
}

/// Zone 创建请求
#[derive(Debug, Deserialize)]
pub struct CreateZoneRequest {
    pub name: String,
    pub cidr: String,
    pub priority: i32,
    pub cloud_platform_id: Option<i32>,
    pub machine_room_id: Option<i32>,
}

async fn resolve_zone_binding(
    cloud_platform_id: Option<i32>,
    machine_room_id: Option<i32>,
) -> Result<(Option<i32>, Option<String>, Option<i32>, Option<String>), ApiError> {
    match (cloud_platform_id, machine_room_id) {
        (Some(_), Some(_)) => {
            return Err(ApiError::bad_request(
                "网络区域只能绑定一个云平台或一个物理机房",
            ));
        }
        (None, None) => {
            return Err(ApiError::bad_request(
                "网络区域必须绑定一个云平台或一个物理机房",
            ));
        }
        _ => {}
    }

    let conn = get_db().ok_or_else(|| ApiError::internal("Database not available"))?;

    let cloud_platform_name = match cloud_platform_id {
        Some(id) => Some(
            get_cloud_platform_config_by_id(&conn, id)
                .await
                .map_err(|e| ApiError::internal(format!("Failed to load cloud platform: {}", e)))?
                .ok_or_else(|| ApiError::bad_request("绑定的云平台不存在"))?
                .platform_name,
        ),
        None => None,
    };

    let machine_room_name = match machine_room_id {
        Some(id) => Some(
            get_machine_room_by_id(&conn, id)
                .await
                .map_err(|e| ApiError::internal(format!("Failed to load machine room: {}", e)))?
                .ok_or_else(|| ApiError::bad_request("绑定的物理机房不存在"))?
                .room_name,
        ),
        None => None,
    };

    Ok((
        cloud_platform_id,
        cloud_platform_name,
        machine_room_id,
        machine_room_name,
    ))
}

/// 创建 Zone
pub async fn create_ip_zone(
    user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<CreateZoneRequest>,
) -> Result<Json<ZoneConfig>, ApiError> {
    // 只有安全管理员和系统管理员可以创建 IP zone
    if user.role != Role::SysAdmin && user.role != Role::SecAdmin {
        return Err(ApiError::forbidden("只有管理员可以创建 IP zone"));
    }

    // 验证 CIDR 格式
    let _network: IpNetwork = req
        .cidr
        .parse()
        .map_err(|_| ApiError::bad_request("Invalid CIDR format"))?;

    let mut zones = load_zones(&state).await?;
    if zones.iter().any(|zone| zone.cidr == req.cidr) {
        return Err(ApiError::conflict("CIDR already exists"));
    }

    let (cloud_platform_id, cloud_platform_name, machine_room_id, machine_room_name) =
        resolve_zone_binding(req.cloud_platform_id, req.machine_room_id).await?;

    let new_zone = ZoneConfig {
        id: uuid::Uuid::new_v4().to_string(),
        name: req.name.clone(),
        cidr: req.cidr.clone(),
        priority: req.priority,
        cloud_platform_id,
        cloud_platform_name,
        machine_room_id,
        machine_room_name,
    };

    db_insert_zone(&new_zone)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to persist IP zone: {}", e)))?;
    zones.push(new_zone.clone());
    sync_zone_cache(&state, &zones)?;

    // Audit log
    log_action_auth(
        &state.audit_logs,
        &user,
        "IP_ZONE_CREATED",
        &req.name,
        &format!("Created IP zone with CIDR {}", req.cidr),
    );

    Ok(Json(new_zone))
}

/// 删除 Zone
pub async fn delete_ip_zone(
    user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<String>, ApiError> {
    // 只有安全管理员和系统管理员可以删除 IP zone
    if user.role != Role::SysAdmin && user.role != Role::SecAdmin {
        return Err(ApiError::forbidden("只有管理员可以删除 IP zone"));
    }

    let mut zones = load_zones(&state).await?;
    let deleted_zone = zones
        .iter()
        .find(|zone| zone.id == id)
        .cloned()
        .ok_or_else(|| ApiError::not_found("Zone not found"))?;

    db_delete_zone(&id)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to delete IP zone: {}", e)))?;
    zones.retain(|zone| zone.id != id);
    sync_zone_cache(&state, &zones)?;

    // Audit log
    log_action_auth(
        &state.audit_logs,
        &user,
        "IP_ZONE_DELETED",
        &deleted_zone.name,
        &format!("Deleted IP zone with ID {}", id),
    );

    Ok(Json("Deleted".to_string()))
}

/// Zone 更新请求
#[derive(Debug, Deserialize)]
pub struct UpdateZoneRequest {
    pub name: Option<String>,
    pub cidr: Option<String>,
    pub priority: Option<i32>,
    pub cloud_platform_id: Option<i32>,
    pub machine_room_id: Option<i32>,
}

/// 更新 Zone
pub async fn update_ip_zone(
    user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateZoneRequest>,
) -> Result<Json<ZoneConfig>, ApiError> {
    // 只有安全管理员和系统管理员可以修改 IP zone
    if user.role != Role::SysAdmin && user.role != Role::SecAdmin {
        return Err(ApiError::forbidden("只有管理员可以修改 IP zone"));
    }

    let mut zones = load_zones(&state).await?;
    let zone_index = zones
        .iter()
        .position(|zone| zone.id == id)
        .ok_or_else(|| ApiError::not_found("Zone not found"))?;
    let zone = zones
        .get(zone_index)
        .cloned()
        .ok_or_else(|| ApiError::not_found("Zone not found"))?;
    let mut updated_zone = zone;

    // Update fields if provided
    if let Some(name) = req.name {
        updated_zone.name = name;
    }
    if let Some(cidr) = req.cidr {
        // Validate CIDR format
        let _network: IpNetwork = cidr
            .parse()
            .map_err(|_| ApiError::bad_request("Invalid CIDR format"))?;
        if zones.iter().any(|zone| zone.id != id && zone.cidr == cidr) {
            return Err(ApiError::conflict("CIDR already exists"));
        }
        updated_zone.cidr = cidr;
    }
    if let Some(priority) = req.priority {
        updated_zone.priority = priority;
    }

    let requested_cloud_platform_id = req.cloud_platform_id.or(updated_zone.cloud_platform_id);
    let requested_machine_room_id = req.machine_room_id.or(updated_zone.machine_room_id);
    let (cloud_platform_id, cloud_platform_name, machine_room_id, machine_room_name) =
        resolve_zone_binding(requested_cloud_platform_id, requested_machine_room_id).await?;
    updated_zone.cloud_platform_id = cloud_platform_id;
    updated_zone.cloud_platform_name = cloud_platform_name;
    updated_zone.machine_room_id = machine_room_id;
    updated_zone.machine_room_name = machine_room_name;

    db_update_zone(&id, &updated_zone)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to update IP zone: {}", e)))?;
    zones[zone_index] = updated_zone.clone();
    sync_zone_cache(&state, &zones)?;

    // Audit log
    log_action_auth(
        &state.audit_logs,
        &user,
        "IP_ZONE_UPDATED",
        &updated_zone.name,
        &format!("Updated IP zone with ID {}", id),
    );

    Ok(Json(updated_zone))
}
