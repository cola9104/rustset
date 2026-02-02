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
use super::cloud_zones::CLOUD_ZONES;
use shared::{
    CloudPlatform, CreateCloudPlatformRequest, UpdateCloudPlatformRequest,
    Role,
};

// 云平台存储 (内存)
pub static CLOUD_PLATFORMS: Mutex<Vec<CloudPlatform>> = Mutex::new(Vec::new());

/// 获取云平台列表
pub async fn get_cloud_platforms(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<CloudPlatform>>, (StatusCode, String)> {
    let _user = get_current_user(&headers, &state.users)
        .ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;

    let platforms = CLOUD_PLATFORMS.lock().unwrap();
    Ok(Json(platforms.clone()))
}

/// 获取单个云平台
pub async fn get_cloud_platform(
    Path(id): Path<i32>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<CloudPlatform>, (StatusCode, String)> {
    let _user = get_current_user(&headers, &state.users)
        .ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;

    let platforms = CLOUD_PLATFORMS.lock().unwrap();
    if let Some(platform) = platforms.iter().find(|p| p.id == Some(id)) {
        Ok(Json(platform.clone()))
    } else {
        Err((StatusCode::NOT_FOUND, "Cloud platform not found".to_string()))
    }
}

/// 根据云区ID获取云平台列表
pub async fn get_platforms_by_zone(
    Path(zone_id): Path<i32>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<CloudPlatform>>, (StatusCode, String)> {
    let _user = get_current_user(&headers, &state.users)
        .ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;

    let platforms = CLOUD_PLATFORMS.lock().unwrap();
    let filtered: Vec<_> = platforms.iter()
        .filter(|p| p.zone_id == zone_id)
        .cloned()
        .collect();

    Ok(Json(filtered))
}

/// 创建云平台
pub async fn create_cloud_platform(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateCloudPlatformRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user = get_current_user(&headers, &state.users)
        .ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;

    // 检查权限
    if user.role != Role::SysAdmin && user.role != Role::SecAdmin {
        return Err((StatusCode::FORBIDDEN, "Access denied".to_string()));
    }

    // 验证zone_id是否存在
    {
        let zones = CLOUD_ZONES.lock().unwrap();
        if !zones.iter().any(|z| z.id == Some(req.zone_id)) {
            return Err((StatusCode::BAD_REQUEST, "Zone not found".to_string()));
        }
    }

    let mut platforms = CLOUD_PLATFORMS.lock().unwrap();

    // 检查同一zone下platform_code是否重复
    if platforms.iter().any(|p| p.zone_id == req.zone_id && p.platform_code == req.platform_code) {
        return Err((StatusCode::BAD_REQUEST, "Platform code already exists in this zone".to_string()));
    }

    // 生成新ID
    let new_id = platforms.iter().filter_map(|p| p.id).max().map_or(1, |m| m + 1);

    let now = Utc::now();
    let platform = CloudPlatform {
        id: Some(new_id),
        zone_id: req.zone_id,
        platform_name: req.platform_name.clone(),
        platform_code: req.platform_code.clone(),
        description: req.description.clone(),
        created_at: now,
    };

    platforms.push(platform.clone());

    // 记录日志
    log_action(
        &state.audit_logs,
        &user,
        "CREATE_CLOUD_PLATFORM",
        &req.platform_name,
        &format!("Created cloud platform: {} in zone: {}", req.platform_name, req.zone_id),
    );

    Ok(Json(json!({
        "message": "云平台创建成功",
        "data": platform
    })))
}

/// 更新云平台
pub async fn update_cloud_platform(
    Path(id): Path<i32>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<UpdateCloudPlatformRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user = get_current_user(&headers, &state.users)
        .ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;

    if user.role != Role::SysAdmin && user.role != Role::SecAdmin {
        return Err((StatusCode::FORBIDDEN, "Access denied".to_string()));
    }

    // 先进行所有验证检查（只读操作）
    {
        let platforms = CLOUD_PLATFORMS.lock().unwrap();
        let platform = platforms.iter().find(|p| p.id == Some(id))
            .ok_or((StatusCode::NOT_FOUND, "Cloud platform not found".to_string()))?;

        // 验证zone_id
        if let Some(target_zone_id) = req.zone_id {
            let zones = CLOUD_ZONES.lock().unwrap();
            if !zones.iter().any(|z| z.id == Some(target_zone_id)) {
                return Err((StatusCode::BAD_REQUEST, "Zone not found".to_string()));
            }
        }

        // 验证platform_code唯一性
        if let Some(ref code) = req.platform_code {
            let zone_id_to_check = req.zone_id.unwrap_or(platform.zone_id);
            if platforms.iter().any(|p| p.id != Some(id) && p.zone_id == zone_id_to_check && p.platform_code == *code) {
                return Err((StatusCode::BAD_REQUEST, "Platform code already exists in this zone".to_string()));
            }
        }
    }

    // 验证通过后，进行更新
    let mut platforms = CLOUD_PLATFORMS.lock().unwrap();
    if let Some(platform) = platforms.iter_mut().find(|p| p.id == Some(id)) {
        if let Some(zone_id) = req.zone_id {
            platform.zone_id = zone_id;
        }
        if let Some(name) = req.platform_name {
            platform.platform_name = name;
        }
        if let Some(code) = req.platform_code {
            platform.platform_code = code;
        }
        if let Some(desc) = req.description {
            platform.description = Some(desc);
        }

        log_action(
            &state.audit_logs,
            &user,
            "UPDATE_CLOUD_PLATFORM",
            &format!("{}", id),
            &format!("Updated cloud platform: {}", platform.platform_name),
        );

        return Ok(Json(json!({
            "message": "云平台更新成功",
            "data": platform.clone()
        })));
    }

    Err((StatusCode::NOT_FOUND, "Cloud platform not found".to_string()))
}

/// 删除云平台
pub async fn delete_cloud_platform(
    Path(id): Path<i32>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user = get_current_user(&headers, &state.users)
        .ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;

    if user.role != Role::SysAdmin && user.role != Role::SecAdmin {
        return Err((StatusCode::FORBIDDEN, "Access denied".to_string()));
    }

    let mut platforms = CLOUD_PLATFORMS.lock().unwrap();

    if let Some(pos) = platforms.iter().position(|p| p.id == Some(id)) {
        platforms.remove(pos);

        // 记录日志
        log_action(
            &state.audit_logs,
            &user,
            "DELETE_CLOUD_PLATFORM",
            &format!("{}", id),
            &format!("Deleted cloud platform: {}", id),
        );

        return Ok(Json(json!({ "message": "云平台删除成功" })));
    }

    Err((StatusCode::NOT_FOUND, "Cloud platform not found".to_string()))
}
