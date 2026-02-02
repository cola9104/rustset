use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json, Response},
};
use serde_json::json;
use std::sync::Mutex;
use chrono::Utc;

use crate::state::AppState;
use crate::utils::{get_current_user, log_action};
use shared::{
    CloudZone, CreateCloudZoneRequest, UpdateCloudZoneRequest,
    Role,
};

// 云区存储 (内存)
pub static CLOUD_ZONES: Mutex<Vec<CloudZone>> = Mutex::new(Vec::new());

/// 获取云区列表
pub async fn get_cloud_zones(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<CloudZone>>, (StatusCode, String)> {
    let _user = get_current_user(&headers, &state.users)
        .ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;

    let zones = CLOUD_ZONES.lock().unwrap();
    Ok(Json(zones.clone()))
}

/// 获取单个云区
pub async fn get_cloud_zone(
    Path(id): Path<i32>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<CloudZone>, (StatusCode, String)> {
    let _user = get_current_user(&headers, &state.users)
        .ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;

    let zones = CLOUD_ZONES.lock().unwrap();
    if let Some(zone) = zones.iter().find(|z| z.id == Some(id)) {
        Ok(Json(zone.clone()))
    } else {
        Err((StatusCode::NOT_FOUND, "Cloud zone not found".to_string()))
    }
}

/// 创建云区
pub async fn create_cloud_zone(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateCloudZoneRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user = get_current_user(&headers, &state.users)
        .ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;

    // 检查权限
    if user.role != Role::SysAdmin && user.role != Role::SecAdmin {
        return Err((StatusCode::FORBIDDEN, "Access denied".to_string()));
    }

    let mut zones = CLOUD_ZONES.lock().unwrap();

    // 检查zone_code是否重复
    if zones.iter().any(|z| z.zone_code == req.zone_code) {
        return Err((StatusCode::BAD_REQUEST, "Zone code already exists".to_string()));
    }

    // 生成新ID
    let new_id = zones.iter().filter_map(|z| z.id).max().map_or(1, |m| m + 1);

    let now = Utc::now();
    let zone = CloudZone {
        id: Some(new_id),
        zone_name: req.zone_name.clone(),
        zone_code: req.zone_code.clone(),
        description: req.description.clone(),
        created_at: now,
    };

    zones.push(zone.clone());

    // 记录日志
    log_action(
        &state.audit_logs,
        &user,
        "CREATE_CLOUD_ZONE",
        &req.zone_name,
        &format!("Created cloud zone: {}", req.zone_name),
    );

    Ok(Json(json!({
        "message": "云区创建成功",
        "data": zone
    })))
}

/// 更新云区
pub async fn update_cloud_zone(
    Path(id): Path<i32>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<UpdateCloudZoneRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user = get_current_user(&headers, &state.users)
        .ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;

    if user.role != Role::SysAdmin && user.role != Role::SecAdmin {
        return Err((StatusCode::FORBIDDEN, "Access denied".to_string()));
    }

    // 先进行所有验证检查（只读操作）
    {
        let zones = CLOUD_ZONES.lock().unwrap();
        let _zone = zones.iter().find(|z| z.id == Some(id))
            .ok_or((StatusCode::NOT_FOUND, "Cloud zone not found".to_string()))?;

        // 验证zone_code唯一性
        if let Some(ref new_code) = req.zone_code {
            if zones.iter().any(|z| z.id != Some(id) && z.zone_code == *new_code) {
                return Err((StatusCode::BAD_REQUEST, "Zone code already exists".to_string()));
            }
        }
    }

    // 验证通过后，进行更新
    let mut zones = CLOUD_ZONES.lock().unwrap();
    if let Some(zone) = zones.iter_mut().find(|z| z.id == Some(id)) {
        if let Some(name) = req.zone_name {
            zone.zone_name = name;
        }
        if let Some(code) = req.zone_code {
            zone.zone_code = code;
        }
        if let Some(desc) = req.description {
            zone.description = Some(desc);
        }

        // 记录日志
        log_action(
            &state.audit_logs,
            &user,
            "UPDATE_CLOUD_ZONE",
            &format!("{}", id),
            &format!("Updated cloud zone: {}", zone.zone_name),
        );

        return Ok(Json(json!({
            "message": "云区更新成功",
            "data": zone.clone()
        })));
    }

    Err((StatusCode::NOT_FOUND, "Cloud zone not found".to_string()))
}

/// 删除云区
pub async fn delete_cloud_zone(
    Path(id): Path<i32>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user = get_current_user(&headers, &state.users)
        .ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;

    if user.role != Role::SysAdmin && user.role != Role::SecAdmin {
        return Err((StatusCode::FORBIDDEN, "Access denied".to_string()));
    }

    let mut zones = CLOUD_ZONES.lock().unwrap();

    if let Some(pos) = zones.iter().position(|z| z.id == Some(id)) {
        zones.remove(pos);

        // 记录日志
        log_action(
            &state.audit_logs,
            &user,
            "DELETE_CLOUD_ZONE",
            &format!("{}", id),
            &format!("Deleted cloud zone: {}", id),
        );

        return Ok(Json(json!({ "message": "云区删除成功" })));
    }

    Err((StatusCode::NOT_FOUND, "Cloud zone not found".to_string()))
}
