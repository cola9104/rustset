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
use crate::database::insert_cloud_zone_wrapper as db_insert_cloud_zone;
use crate::database::update_cloud_zone as db_update_cloud_zone;
use crate::database::delete_cloud_zone as db_delete_cloud_zone;
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
) -> impl IntoResponse {
    let _user = match get_current_user(&headers, &state.users) {
        Some(u) => u,
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()).into_response(),
    };

    let zones = CLOUD_ZONES.lock().unwrap();
    Json(zones.clone()).into_response()
}

/// 获取单个云区
pub async fn get_cloud_zone(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    let _user = match get_current_user(&headers, &state.users) {
        Some(u) => u,
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()).into_response(),
    };

    let zones = CLOUD_ZONES.lock().unwrap();
    if let Some(zone) = zones.iter().find(|z| z.id == Some(id)) {
        Json(zone.clone()).into_response()
    } else {
        (StatusCode::NOT_FOUND, "Cloud zone not found".to_string()).into_response()
    }
}

/// 创建云区
#[axum::debug_handler]
pub async fn create_cloud_zone(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateCloudZoneRequest>,
) -> impl IntoResponse {
    let user = match get_current_user(&headers, &state.users) {
        Some(u) => u,
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()).into_response(),
    };

    // 检查权限
    if user.role != Role::SysAdmin && user.role != Role::SecAdmin {
        return (StatusCode::FORBIDDEN, "Access denied".to_string()).into_response();
    }

    // 检查zone_code是否重复并生成新ID
    let (new_id, zone_code_exists) = {
        let zones = CLOUD_ZONES.lock().unwrap();
        let exists = zones.iter().any(|z| z.zone_code == req.zone_code);
        let max_id = zones.iter().filter_map(|z| z.id).max().map_or(1, |m| m + 1);
        (max_id, exists)
    };

    if zone_code_exists {
        return (StatusCode::BAD_REQUEST, "Zone code already exists".to_string()).into_response();
    }

    let now = Utc::now();
    let zone = CloudZone {
        id: Some(new_id),
        zone_name: req.zone_name.clone(),
        zone_code: req.zone_code.clone(),
        description: req.description.clone(),
        created_at: now,
    };

    {
        let mut zones = CLOUD_ZONES.lock().unwrap();
        zones.push(zone.clone());
    }

    // 持久化到数据库（在锁释放后）
    let created_at_str = now.to_rfc3339();
    let _ = db_insert_cloud_zone(
        &req.zone_name,
        &req.zone_code,
        req.description.as_deref(),
        &created_at_str,
    ).await;

    // 记录日志
    log_action(
        &state.audit_logs,
        &user,
        "CREATE_CLOUD_ZONE",
        &req.zone_name,
        &format!("Created cloud zone: {}", req.zone_name),
    );

    Json(json!({
        "message": "云区创建成功",
        "data": zone
    })).into_response()
}

/// 更新云区
#[axum::debug_handler]
pub async fn update_cloud_zone(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i32>,
    Json(req): Json<UpdateCloudZoneRequest>,
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
        let zones = CLOUD_ZONES.lock().unwrap();
        if zones.iter().find(|z| z.id == Some(id)).is_none() {
            return (StatusCode::NOT_FOUND, "Cloud zone not found".to_string()).into_response();
        }

        // 验证zone_code唯一性
        if let Some(ref new_code) = req.zone_code {
            if zones.iter().any(|z| z.id != Some(id) && z.zone_code == *new_code) {
                return (StatusCode::BAD_REQUEST, "Zone code already exists".to_string()).into_response();
            }
        }
    }

    // 验证通过后，进行更新
    let (zone_data, db_zone_name, db_zone_code, db_description) = {
        let mut zones = CLOUD_ZONES.lock().unwrap();
        if let Some(zone) = zones.iter_mut().find(|z| z.id == Some(id)) {
            // Clone values for database before moving
            let db_zone_name = req.zone_name.as_ref().map(|s| s.clone());
            let db_zone_code = req.zone_code.as_ref().map(|s| s.clone());
            let db_description = req.description.as_ref().map(|s| s.clone());

            if let Some(name) = req.zone_name {
                zone.zone_name = name;
            }
            if let Some(code) = req.zone_code {
                zone.zone_code = code;
            }
            if let Some(desc) = req.description {
                zone.description = Some(desc);
            }

            (zone.clone(), db_zone_name, db_zone_code, db_description)
        } else {
            return (StatusCode::NOT_FOUND, "Cloud zone not found".to_string()).into_response();
        }
    };

    // 持久化到数据库
    let _ = db_update_cloud_zone(
        id,
        db_zone_name.as_deref(),
        db_zone_code.as_deref(),
        db_description.as_deref(),
    ).await;

    // 记录日志
    log_action(
        &state.audit_logs,
        &user,
        "UPDATE_CLOUD_ZONE",
        &format!("{}", id),
        &format!("Updated cloud zone: {}", zone_data.zone_name),
    );

    Json(json!({
        "message": "云区更新成功",
        "data": zone_data
    })).into_response()
}

/// 删除云区
#[axum::debug_handler]
pub async fn delete_cloud_zone(
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
        let mut zones = CLOUD_ZONES.lock().unwrap();
        zones.iter().position(|z| z.id == Some(id))
    };

    if let Some(pos) = found_pos {
        // Remove the zone
        {
            let mut zones = CLOUD_ZONES.lock().unwrap();
            zones.remove(pos);
        }

        // 持久化到数据库
        let _ = db_delete_cloud_zone(id).await;

        // 记录日志
        log_action(
            &state.audit_logs,
            &user,
            "DELETE_CLOUD_ZONE",
            &format!("{}", id),
            &format!("Deleted cloud zone: {}", id),
        );

        return Json(json!({ "message": "云区删除成功" })).into_response();
    }

    (StatusCode::NOT_FOUND, "Cloud zone not found".to_string()).into_response()
}
