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

    // 获取 zones 配置
    let zones = state
        .zones
        .read()
        .map_err(|e| ApiError::internal(format!("Failed to read zones: {}", e)))?;

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
        Some(z) => {
            let zone_type = match z.name.as_str() {
                "Internet" => NetworkZone::Internet,
                "DMZ" => NetworkZone::DMZ,
                "Intranet" => NetworkZone::Intranet,
                other => NetworkZone::Custom(other.to_string()),
            };
            (z.name.clone(), zone_type, Some(z.cidr.clone()))
        }
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
    let zones = state
        .zones
        .read()
        .map_err(|e| ApiError::internal(format!("Failed to read zones: {}", e)))?;
    Ok(Json(zones.clone()))
}

/// Zone 创建请求
#[derive(Debug, Deserialize)]
pub struct CreateZoneRequest {
    pub name: String,
    pub cidr: String,
    pub priority: i32,
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

    let new_zone = ZoneConfig {
        id: uuid::Uuid::new_v4().to_string(),
        name: req.name.clone(),
        cidr: req.cidr.clone(),
        priority: req.priority,
    };

    let mut zones = state
        .zones
        .write()
        .map_err(|e| ApiError::internal(format!("Failed to write zones: {}", e)))?;

    // 检查 CIDR 冲突
    for z in zones.iter() {
        if z.cidr == new_zone.cidr {
            return Err(ApiError::conflict("CIDR already exists"));
        }
    }

    zones.push(new_zone.clone());

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

    // Get zone name for audit log before deletion
    let zone_name = {
        let zones = state
            .zones
            .read()
            .map_err(|e| ApiError::internal(format!("Failed to read zones: {}", e)))?;
        zones.iter().find(|z| z.id == id).map(|z| z.name.clone())
    };

    let mut zones = state
        .zones
        .write()
        .map_err(|e| ApiError::internal(format!("Failed to write zones: {}", e)))?;

    let initial_len = zones.len();
    zones.retain(|z| z.id != id);

    if zones.len() == initial_len {
        return Err(ApiError::not_found("Zone not found"));
    }

    // Audit log
    let target = zone_name.unwrap_or_else(|| id.clone());
    log_action_auth(
        &state.audit_logs,
        &user,
        "IP_ZONE_DELETED",
        &target,
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

    let mut zones = state
        .zones
        .write()
        .map_err(|e| ApiError::internal(format!("Failed to write zones: {}", e)))?;

    // Find the zone
    let zone = zones
        .iter_mut()
        .find(|z| z.id == id)
        .ok_or_else(|| ApiError::not_found("Zone not found"))?;

    // Update fields if provided
    if let Some(name) = req.name {
        zone.name = name.clone();
    }
    if let Some(cidr) = req.cidr {
        // Validate CIDR format
        let _network: IpNetwork = cidr
            .parse()
            .map_err(|_| ApiError::bad_request("Invalid CIDR format"))?;
        zone.cidr = cidr;
    }
    if let Some(priority) = req.priority {
        zone.priority = priority;
    }

    let updated_zone = zone.clone();

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
