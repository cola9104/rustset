use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json},
};
use serde_json::json;
use std::sync::Mutex;
use chrono::Utc;

use crate::state::AppState;
use crate::utils::{get_current_user, log_action};
use super::cloud_zones::CLOUD_ZONES;
use crate::database::insert_cloud_platform_wrapper as db_insert_cloud_platform;
use crate::database::update_cloud_platform as db_update_cloud_platform;
use crate::database::delete_cloud_platform as db_delete_cloud_platform;
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
) -> impl IntoResponse {
    let _user = match get_current_user(&headers, &state.users) {
        Some(u) => u,
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()).into_response(),
    };

    let platforms = CLOUD_PLATFORMS.lock().unwrap();
    Json(platforms.clone()).into_response()
}

/// 获取单个云平台
pub async fn get_cloud_platform(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    let _user = match get_current_user(&headers, &state.users) {
        Some(u) => u,
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()).into_response(),
    };

    let platforms = CLOUD_PLATFORMS.lock().unwrap();
    if let Some(platform) = platforms.iter().find(|p| p.id == Some(id)) {
        Json(platform.clone()).into_response()
    } else {
        (StatusCode::NOT_FOUND, "Cloud platform not found".to_string()).into_response()
    }
}

/// 根据云区ID获取云平台列表
pub async fn get_platforms_by_zone(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(zone_id): Path<i32>,
) -> impl IntoResponse {
    let _user = match get_current_user(&headers, &state.users) {
        Some(u) => u,
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()).into_response(),
    };

    let platforms = CLOUD_PLATFORMS.lock().unwrap();
    let filtered: Vec<_> = platforms.iter()
        .filter(|p| p.zone_id == zone_id)
        .cloned()
        .collect();

    Json(filtered).into_response()
}

/// 创建云平台
#[axum::debug_handler]
pub async fn create_cloud_platform(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateCloudPlatformRequest>,
) -> impl IntoResponse {
    let user = match get_current_user(&headers, &state.users) {
        Some(u) => u,
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()).into_response(),
    };

    // 检查权限
    if user.role != Role::SysAdmin && user.role != Role::SecAdmin {
        return (StatusCode::FORBIDDEN, "Access denied".to_string()).into_response();
    }

    // 验证zone_id是否存在
    {
        let zones = CLOUD_ZONES.lock().unwrap();
        if !zones.iter().any(|z| z.id == Some(req.zone_id)) {
            return (StatusCode::BAD_REQUEST, "Zone not found".to_string()).into_response();
        }
    }

    // 检查同一zone下platform_code是否重复并生成新ID
    let (new_id, code_exists) = {
        let platforms = CLOUD_PLATFORMS.lock().unwrap();
        let exists = platforms.iter().any(|p| p.zone_id == req.zone_id && p.platform_code == req.platform_code);
        let max_id = platforms.iter().filter_map(|p| p.id).max().map_or(1, |m| m + 1);
        (max_id, exists)
    };

    if code_exists {
        return (StatusCode::BAD_REQUEST, "Platform code already exists in this zone".to_string()).into_response();
    }

    let now = Utc::now();
    let platform = CloudPlatform {
        id: Some(new_id),
        zone_id: req.zone_id,
        platform_name: req.platform_name.clone(),
        platform_code: req.platform_code.clone(),
        description: req.description.clone(),
        created_at: now,
    };

    {
        let mut platforms = CLOUD_PLATFORMS.lock().unwrap();
        platforms.push(platform.clone());
    }

    // 持久化到数据库（在锁释放后）
    let created_at_str = now.to_rfc3339();
    let _ = db_insert_cloud_platform(
        req.zone_id,
        &req.platform_name,
        &req.platform_code,
        req.description.as_deref(),
        &created_at_str,
    ).await;

    // 记录日志
    log_action(
        &state.audit_logs,
        &user,
        "CREATE_CLOUD_PLATFORM",
        &req.platform_name,
        &format!("Created cloud platform: {} in zone: {}", req.platform_name, req.zone_id),
    );

    Json(json!({
        "message": "云平台创建成功",
        "data": platform
    })).into_response()
}

/// 更新云平台
#[axum::debug_handler]
pub async fn update_cloud_platform(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i32>,
    Json(req): Json<UpdateCloudPlatformRequest>,
) -> impl IntoResponse {
    let user = match get_current_user(&headers, &state.users) {
        Some(u) => u,
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()).into_response(),
    };

    if user.role != Role::SysAdmin && user.role != Role::SecAdmin {
        return (StatusCode::FORBIDDEN, "Access denied".to_string()).into_response();
    }

    // 先进行所有验证检查（只读操作）
    {
        let platforms = CLOUD_PLATFORMS.lock().unwrap();
        if platforms.iter().find(|p| p.id == Some(id)).is_none() {
            return (StatusCode::NOT_FOUND, "Cloud platform not found".to_string()).into_response();
        }

        // 验证zone_id
        if let Some(target_zone_id) = req.zone_id {
            let zones = CLOUD_ZONES.lock().unwrap();
            if !zones.iter().any(|z| z.id == Some(target_zone_id)) {
                return (StatusCode::BAD_REQUEST, "Zone not found".to_string()).into_response();
            }
        }

        // 验证platform_code唯一性
        if let Some(ref code) = req.platform_code {
            let zone_id_to_check = req.zone_id.unwrap_or_else(|| {
                platforms.iter().find(|p| p.id == Some(id)).map(|p| p.zone_id).unwrap_or(0)
            });
            if platforms.iter().any(|p| p.id != Some(id) && p.zone_id == zone_id_to_check && p.platform_code == *code) {
                return (StatusCode::BAD_REQUEST, "Platform code already exists in this zone".to_string()).into_response();
            }
        }
    }

    // 验证通过后，进行更新
    let (platform_data, db_zone_id, db_platform_name, db_platform_code, db_description) = {
        let mut platforms = CLOUD_PLATFORMS.lock().unwrap();
        if let Some(platform) = platforms.iter_mut().find(|p| p.id == Some(id)) {
            // Clone values for database before moving
            let db_zone_id = req.zone_id;
            let db_platform_name = req.platform_name.as_ref().map(|s| s.clone());
            let db_platform_code = req.platform_code.as_ref().map(|s| s.clone());
            let db_description = req.description.as_ref().map(|s| s.clone());

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

            (platform.clone(), db_zone_id, db_platform_name, db_platform_code, db_description)
        } else {
            return (StatusCode::NOT_FOUND, "Cloud platform not found".to_string()).into_response();
        }
    };

    // 持久化到数据库
    let _ = db_update_cloud_platform(
        id,
        db_zone_id,
        db_platform_name.as_deref(),
        db_platform_code.as_deref(),
        db_description.as_deref(),
    ).await;

    log_action(
        &state.audit_logs,
        &user,
        "UPDATE_CLOUD_PLATFORM",
        &format!("{}", id),
        &format!("Updated cloud platform: {}", platform_data.platform_name),
    );

    Json(json!({
        "message": "云平台更新成功",
        "data": platform_data
    })).into_response()
}

/// 删除云平台
#[axum::debug_handler]
pub async fn delete_cloud_platform(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    let user = match get_current_user(&headers, &state.users) {
        Some(u) => u,
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()).into_response(),
    };

    if user.role != Role::SysAdmin && user.role != Role::SecAdmin {
        return (StatusCode::FORBIDDEN, "Access denied".to_string()).into_response();
    }

    let found_pos = {
        let mut platforms = CLOUD_PLATFORMS.lock().unwrap();
        platforms.iter().position(|p| p.id == Some(id))
    };

    if let Some(pos) = found_pos {
        // Remove the platform
        {
            let mut platforms = CLOUD_PLATFORMS.lock().unwrap();
            platforms.remove(pos);
        }

        // 持久化到数据库
        let _ = db_delete_cloud_platform(id).await;

        // 记录日志
        log_action(
            &state.audit_logs,
            &user,
            "DELETE_CLOUD_PLATFORM",
            &format!("{}", id),
            &format!("Deleted cloud platform: {}", id),
        );

        return Json(json!({ "message": "云平台删除成功" })).into_response();
    }

    (StatusCode::NOT_FOUND, "Cloud platform not found".to_string()).into_response()
}
