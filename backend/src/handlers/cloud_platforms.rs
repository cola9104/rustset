use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json},
};
use serde_json::json;
use chrono::Utc;

use crate::state::AppState;
use crate::utils::{get_current_user, log_action};
use crate::database::{
    get_all_cloud_platforms as db_get_all_cloud_platforms,
    get_cloud_platform_by_id as db_get_cloud_platform_by_id,
    get_platforms_by_zone_id as db_get_platforms_by_zone_id,
    get_all_cloud_zones as db_get_all_cloud_zones,
    insert_cloud_platform_wrapper as db_insert_cloud_platform,
    update_cloud_platform as db_update_cloud_platform,
    delete_cloud_platform as db_delete_cloud_platform,
};
use shared::{
    CloudPlatform, CreateCloudPlatformRequest, UpdateCloudPlatformRequest,
    Role,
};

/// 获取云平台列表 (直接从数据库读取)
pub async fn get_cloud_platforms(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let _user = match get_current_user(&headers, &state.users) {
        Some(u) => u,
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()).into_response(),
    };

    match crate::database::get_db() {
        Some(conn) => {
            match db_get_all_cloud_platforms(&conn).await {
                Ok(db_platforms) => {
                    let platforms: Vec<CloudPlatform> = db_platforms.into_iter().map(|db| CloudPlatform {
                        id: Some(db.id),
                        zone_id: db.zone_id,
                        platform_name: db.service_name.clone(),
                        platform_code: db.service_code.clone(),
                        description: db.description.clone(),
                        created_at: chrono::DateTime::parse_from_rfc3339(&db.created_at)
                            .map(|dt| dt.with_timezone(&chrono::Utc))
                            .unwrap_or_else(|_| Utc::now()),
                    }).collect();
                    Json(platforms).into_response()
                }
                Err(e) => {
                    eprintln!("Error loading cloud platforms from database: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()).into_response()
                }
            }
        }
        None => {
            (StatusCode::INTERNAL_SERVER_ERROR, "Database not available".to_string()).into_response()
        }
    }
}

/// 获取单个云平台 (直接从数据库读取)
pub async fn get_cloud_platform(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    let _user = match get_current_user(&headers, &state.users) {
        Some(u) => u,
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()).into_response(),
    };

    match crate::database::get_db() {
        Some(conn) => {
            match db_get_cloud_platform_by_id(&conn, id).await {
                Ok(Some(db)) => {
                    let platform = CloudPlatform {
                        id: Some(db.id),
                        zone_id: db.zone_id,
                        platform_name: db.service_name.clone(),
                        platform_code: db.service_code.clone(),
                        description: db.description.clone(),
                        created_at: chrono::DateTime::parse_from_rfc3339(&db.created_at)
                            .map(|dt| dt.with_timezone(&chrono::Utc))
                            .unwrap_or_else(|_| Utc::now()),
                    };
                    Json(platform).into_response()
                }
                Ok(None) => {
                    (StatusCode::NOT_FOUND, "Cloud service not found".to_string()).into_response()
                }
                Err(e) => {
                    eprintln!("Error loading cloud platform from database: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()).into_response()
                }
            }
        }
        None => {
            (StatusCode::INTERNAL_SERVER_ERROR, "Database not available".to_string()).into_response()
        }
    }
}

/// 根据云区ID获取云平台列表 (直接从数据库读取)
pub async fn get_platforms_by_zone(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(zone_id): Path<i32>,
) -> impl IntoResponse {
    let _user = match get_current_user(&headers, &state.users) {
        Some(u) => u,
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()).into_response(),
    };

    match crate::database::get_db() {
        Some(conn) => {
            match db_get_platforms_by_zone_id(&conn, zone_id).await {
                Ok(db_platforms) => {
                    let platforms: Vec<CloudPlatform> = db_platforms.into_iter().map(|db| CloudPlatform {
                        id: Some(db.id),
                        zone_id: db.zone_id,
                        platform_name: db.service_name.clone(),
                        platform_code: db.service_code.clone(),
                        description: db.description.clone(),
                        created_at: chrono::DateTime::parse_from_rfc3339(&db.created_at)
                            .map(|dt| dt.with_timezone(&chrono::Utc))
                            .unwrap_or_else(|_| Utc::now()),
                    }).collect();
                    Json(platforms).into_response()
                }
                Err(e) => {
                    eprintln!("Error loading cloud platforms by zone from database: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()).into_response()
                }
            }
        }
        None => {
            (StatusCode::INTERNAL_SERVER_ERROR, "Database not available".to_string()).into_response()
        }
    }
}

/// 创建云平台 (直接写入数据库)
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

    match crate::database::get_db() {
        Some(conn) => {
            // 验证zone_id是否存在
            match db_get_all_cloud_zones(&conn).await {
                Ok(zones) => {
                    if !zones.iter().any(|z| z.id == req.zone_id) {
                        return (StatusCode::BAD_REQUEST, "Operator/Manufacturer not found".to_string()).into_response();
                    }
                }
                Err(e) => {
                    eprintln!("Error checking zone existence: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()).into_response();
                }
            }

            // 检查同一zone下platform_code是否重复
            match db_get_all_cloud_platforms(&conn).await {
                Ok(existing_platforms) => {
                    if existing_platforms.iter().any(|p| p.zone_id == req.zone_id && p.service_code == req.platform_code) {
                        return (StatusCode::BAD_REQUEST, "Cloud service code already exists in this operator/manufacturer".to_string()).into_response();
                    }
                }
                Err(e) => {
                    eprintln!("Error checking platform code: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()).into_response();
                }
            }

            // 持久化到数据库
            let now = Utc::now();
            let created_at_str = now.to_rfc3339();
            match db_insert_cloud_platform(
                req.zone_id,
                &req.platform_name,
                &req.platform_code,
                req.description.as_deref(),
                &created_at_str,
            ).await {
                Ok(id) => {
                    let platform = CloudPlatform {
                        id: Some(id),
                        zone_id: req.zone_id,
                        platform_name: req.platform_name.clone(),
                        platform_code: req.platform_code.clone(),
                        description: req.description.clone(),
                        created_at: now,
                    };

                    // 记录日志
                    log_action(
                        &state.audit_logs,
                        &user,
                        "CREATE_CLOUD_PLATFORM",
                        &req.platform_name,
                        &format!("Created cloud platform: {} in zone: {}", req.platform_name, req.zone_id),
                    );

                    Json(json!({
                        "message": "云服务创建成功",
                        "data": platform
                    })).into_response()
                }
                Err(e) => {
                    eprintln!("Error inserting cloud platform: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create cloud service".to_string()).into_response()
                }
            }
        }
        None => {
            (StatusCode::INTERNAL_SERVER_ERROR, "Database not available".to_string()).into_response()
        }
    }
}

/// 更新云平台 (直接更新数据库)
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

    match crate::database::get_db() {
        Some(conn) => {
            // 检查platform是否存在
            let existing_platform = match db_get_cloud_platform_by_id(&conn, id).await {
                Ok(Some(p)) => p,
                Ok(None) => {
                    return (StatusCode::NOT_FOUND, "Cloud service not found".to_string()).into_response();
                }
                Err(e) => {
                    eprintln!("Error checking platform existence: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()).into_response();
                }
            };

            // 验证zone_id
            let zone_id_to_check = req.zone_id.unwrap_or(existing_platform.zone_id);
            if let Some(target_zone_id) = req.zone_id {
                match db_get_all_cloud_zones(&conn).await {
                    Ok(zones) => {
                        if !zones.iter().any(|z| z.id == target_zone_id) {
                            return (StatusCode::BAD_REQUEST, "Operator/Manufacturer not found".to_string()).into_response();
                        }
                    }
                    Err(e) => {
                        eprintln!("Error checking zone existence: {}", e);
                    }
                }
            }

            // 验证platform_code唯一性
            if let Some(ref code) = req.platform_code {
                match db_get_all_cloud_platforms(&conn).await {
                    Ok(existing_platforms) => {
                        if existing_platforms.iter().any(|p| p.id != id && p.zone_id == zone_id_to_check && p.service_code == *code) {
                            return (StatusCode::BAD_REQUEST, "Cloud service code already exists in this operator/manufacturer".to_string()).into_response();
                        }
                    }
                    Err(e) => {
                        eprintln!("Error checking platform code: {}", e);
                    }
                }
            }

            // 持久化到数据库
            match db_update_cloud_platform(
                id,
                req.zone_id,
                req.platform_name.as_deref(),
                req.platform_code.as_deref(),
                req.description.as_deref(),
            ).await {
                Ok(_) => {
                    // 记录日志
                    log_action(
                        &state.audit_logs,
                        &user,
                        "UPDATE_CLOUD_PLATFORM",
                        &format!("{}", id),
                        &format!("Updated cloud platform: {}", id),
                    );

                    // 返回更新后的数据
                    match db_get_cloud_platform_by_id(&conn, id).await {
                        Ok(Some(db)) => {
                            let platform = CloudPlatform {
                                id: Some(db.id),
                                zone_id: db.zone_id,
                                platform_name: db.service_name.clone(),
                                platform_code: db.service_code.clone(),
                                description: db.description.clone(),
                                created_at: chrono::DateTime::parse_from_rfc3339(&db.created_at)
                                    .map(|dt| dt.with_timezone(&chrono::Utc))
                                    .unwrap_or_else(|_| Utc::now()),
                            };
                            Json(json!({
                                "message": "云服务更新成功",
                                "data": platform
                            })).into_response()
                        }
                        _ => {
                            Json(json!({ "message": "云服务更新成功" })).into_response()
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error updating cloud platform: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update cloud service".to_string()).into_response()
                }
            }
        }
        None => {
            (StatusCode::INTERNAL_SERVER_ERROR, "Database not available".to_string()).into_response()
        }
    }
}

/// 删除云平台 (直接从数据库删除)
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

    match crate::database::get_db() {
        Some(conn) => {
            // 检查是否存在
            match db_get_cloud_platform_by_id(&conn, id).await {
                Ok(Some(_)) => {
                    // 持久化到数据库
                    match db_delete_cloud_platform(id).await {
                        Ok(_) => {
                            // 记录日志
                            log_action(
                                &state.audit_logs,
                                &user,
                                "DELETE_CLOUD_PLATFORM",
                                &format!("{}", id),
                                &format!("Deleted cloud platform: {}", id),
                            );

                            Json(json!({ "message": "云服务删除成功" })).into_response()
                        }
                        Err(e) => {
                            eprintln!("Error deleting cloud platform: {}", e);
                            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete cloud service".to_string()).into_response()
                        }
                    }
                }
                Ok(None) => {
                    (StatusCode::NOT_FOUND, "Cloud service not found".to_string()).into_response()
                }
                Err(e) => {
                    eprintln!("Error checking platform existence: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()).into_response()
                }
            }
        }
        None => {
            (StatusCode::INTERNAL_SERVER_ERROR, "Database not available".to_string()).into_response()
        }
    }
}
