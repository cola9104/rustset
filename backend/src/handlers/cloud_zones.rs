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
    get_all_cloud_zones as db_get_all_cloud_zones,
    get_cloud_zone_by_id as db_get_cloud_zone_by_id,
    insert_cloud_zone_wrapper as db_insert_cloud_zone,
    update_cloud_zone as db_update_cloud_zone,
    delete_cloud_zone as db_delete_cloud_zone,
};
use shared::{
    CloudZone, CreateCloudZoneRequest, UpdateCloudZoneRequest,
    Role,
};

/// 获取云区列表 (直接从数据库读取)
pub async fn get_cloud_zones(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let _user = match get_current_user(&headers, &state.users) {
        Some(u) => u,
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()).into_response(),
    };

    match crate::database::get_db() {
        Some(conn) => {
            match db_get_all_cloud_zones(&conn).await {
                Ok(db_zones) => {
                    let zones: Vec<CloudZone> = db_zones.into_iter().map(|db| CloudZone {
                        id: Some(db.id),
                        zone_name: db.zone_name.clone(),
                        zone_code: db.zone_code.clone(),
                        description: db.description.clone(),
                        created_at: chrono::DateTime::parse_from_rfc3339(&db.created_at)
                            .map(|dt| dt.with_timezone(&chrono::Utc))
                            .unwrap_or_else(|_| Utc::now()),
                    }).collect();
                    Json(zones).into_response()
                }
                Err(e) => {
                    eprintln!("Error loading cloud zones from database: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()).into_response()
                }
            }
        }
        None => {
            (StatusCode::INTERNAL_SERVER_ERROR, "Database not available".to_string()).into_response()
        }
    }
}

/// 获取单个云区 (直接从数据库读取)
pub async fn get_cloud_zone(
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
            match db_get_cloud_zone_by_id(&conn, id).await {
                Ok(Some(db)) => {
                    let zone = CloudZone {
                        id: Some(db.id),
                        zone_name: db.zone_name.clone(),
                        zone_code: db.zone_code.clone(),
                        description: db.description.clone(),
                        created_at: chrono::DateTime::parse_from_rfc3339(&db.created_at)
                            .map(|dt| dt.with_timezone(&chrono::Utc))
                            .unwrap_or_else(|_| Utc::now()),
                    };
                    Json(zone).into_response()
                }
                Ok(None) => {
                    (StatusCode::NOT_FOUND, "Operator/Manufacturer not found".to_string()).into_response()
                }
                Err(e) => {
                    eprintln!("Error loading cloud zone from database: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()).into_response()
                }
            }
        }
        None => {
            (StatusCode::INTERNAL_SERVER_ERROR, "Database not available".to_string()).into_response()
        }
    }
}

/// 创建云区 (直接写入数据库)
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

    match crate::database::get_db() {
        Some(conn) => {
            // 检查zone_code是否已存在
            match db_get_all_cloud_zones(&conn).await {
                Ok(existing_zones) => {
                    if existing_zones.iter().any(|z| z.zone_code == req.zone_code) {
                        return (StatusCode::BAD_REQUEST, "Zone code already exists".to_string()).into_response();
                    }
                }
                Err(e) => {
                    eprintln!("Error checking zone code: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()).into_response();
                }
            }

            // 持久化到数据库
            let now = Utc::now();
            let created_at_str = now.to_rfc3339();
            match db_insert_cloud_zone(
                &req.zone_name,
                &req.zone_code,
                req.description.as_deref(),
                &created_at_str,
            ).await {
                Ok(id) => {
                    let zone = CloudZone {
                        id: Some(id),
                        zone_name: req.zone_name.clone(),
                        zone_code: req.zone_code.clone(),
                        description: req.description.clone(),
                        created_at: now,
                    };

                    // 记录日志
                    log_action(
                        &state.audit_logs,
                        &user,
                        "CREATE_CLOUD_ZONE",
                        &req.zone_name,
                        &format!("Created cloud zone: {}", req.zone_name),
                    );

                    Json(json!({
                        "message": "运营商/厂家创建成功",
                        "data": zone
                    })).into_response()
                }
                Err(e) => {
                    eprintln!("Error inserting cloud zone: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create operator/manufacturer".to_string()).into_response()
                }
            }
        }
        None => {
            (StatusCode::INTERNAL_SERVER_ERROR, "Database not available".to_string()).into_response()
        }
    }
}

/// 更新云区 (直接更新数据库)
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

    match crate::database::get_db() {
        Some(conn) => {
            // 检查zone是否存在
            match db_get_cloud_zone_by_id(&conn, id).await {
                Ok(Some(_)) => {
                    // 验证zone_code唯一性
                    if let Some(ref new_code) = req.zone_code {
                        match db_get_all_cloud_zones(&conn).await {
                            Ok(existing_zones) => {
                                if existing_zones.iter().any(|z| z.id != id && z.zone_code == *new_code) {
                                    return (StatusCode::BAD_REQUEST, "Zone code already exists".to_string()).into_response();
                                }
                            }
                            Err(e) => {
                                eprintln!("Error checking zone code: {}", e);
                            }
                        }
                    }
                }
                Ok(None) => {
                    return (StatusCode::NOT_FOUND, "Operator/Manufacturer not found".to_string()).into_response();
                }
                Err(e) => {
                    eprintln!("Error checking zone existence: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()).into_response();
                }
            }

            // 持久化到数据库
            match db_update_cloud_zone(
                id,
                req.zone_name.as_deref(),
                req.zone_code.as_deref(),
                req.description.as_deref(),
            ).await {
                Ok(_) => {
                    // 记录日志
                    log_action(
                        &state.audit_logs,
                        &user,
                        "UPDATE_CLOUD_ZONE",
                        &format!("{}", id),
                        &format!("Updated cloud zone: {}", id),
                    );

                    // 返回更新后的数据
                    match db_get_cloud_zone_by_id(&conn, id).await {
                        Ok(Some(db)) => {
                            let zone = CloudZone {
                                id: Some(db.id),
                                zone_name: db.zone_name.clone(),
                                zone_code: db.zone_code.clone(),
                                description: db.description.clone(),
                                created_at: chrono::DateTime::parse_from_rfc3339(&db.created_at)
                                    .map(|dt| dt.with_timezone(&chrono::Utc))
                                    .unwrap_or_else(|_| Utc::now()),
                            };
                            Json(json!({
                                "message": "运营商/厂家更新成功",
                                "data": zone
                            })).into_response()
                        }
                        _ => {
                            Json(json!({ "message": "运营商/厂家更新成功" })).into_response()
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error updating cloud zone: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update operator/manufacturer".to_string()).into_response()
                }
            }
        }
        None => {
            (StatusCode::INTERNAL_SERVER_ERROR, "Database not available".to_string()).into_response()
        }
    }
}

/// 删除云区 (直接从数据库删除)
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

    match crate::database::get_db() {
        Some(conn) => {
            // 检查是否存在
            match db_get_cloud_zone_by_id(&conn, id).await {
                Ok(Some(_)) => {
                    // 持久化到数据库
                    match db_delete_cloud_zone(id).await {
                        Ok(_) => {
                            // 记录日志
                            log_action(
                                &state.audit_logs,
                                &user,
                                "DELETE_CLOUD_ZONE",
                                &format!("{}", id),
                                &format!("Deleted cloud zone: {}", id),
                            );

                            Json(json!({ "message": "运营商/厂家删除成功" })).into_response()
                        }
                        Err(e) => {
                            eprintln!("Error deleting cloud zone: {}", e);
                            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete operator/manufacturer".to_string()).into_response()
                        }
                    }
                }
                Ok(None) => {
                    (StatusCode::NOT_FOUND, "Operator/Manufacturer not found".to_string()).into_response()
                }
                Err(e) => {
                    eprintln!("Error checking zone existence: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()).into_response()
                }
            }
        }
        None => {
            (StatusCode::INTERNAL_SERVER_ERROR, "Database not available".to_string()).into_response()
        }
    }
}
