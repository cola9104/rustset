use axum::{
    extract::{Path, State},
    response::{IntoResponse, Json},
};
use chrono::Utc;
use serde_json::json;

use crate::database::{
    delete_cloud_zone as db_delete_cloud_zone, get_all_cloud_zones as db_get_all_cloud_zones,
    get_cloud_zone_by_id as db_get_cloud_zone_by_id,
    insert_cloud_zone_wrapper as db_insert_cloud_zone, update_cloud_zone as db_update_cloud_zone,
};
use crate::middleware::{ApiError, AuthUser};
use crate::state::AppState;
use crate::utils::log_action_auth;
use shared::{CloudZone, CreateCloudZoneRequest, Role, UpdateCloudZoneRequest};

/// 获取云区列表 (直接从数据库读取)
pub async fn get_cloud_zones(
    State(_state): State<AppState>,
    _user: AuthUser,
) -> Result<impl IntoResponse, ApiError> {
    match crate::database::get_db() {
        Some(conn) => match db_get_all_cloud_zones(&conn).await {
            Ok(db_zones) => {
                let zones: Vec<CloudZone> = db_zones
                    .into_iter()
                    .map(|db| CloudZone {
                        id: Some(db.id),
                        zone_name: db.zone_name.clone(),
                        zone_code: db.zone_code.clone(),
                        description: db.description.clone(),
                        created_at: chrono::DateTime::parse_from_rfc3339(&db.created_at)
                            .map(|dt| dt.with_timezone(&chrono::Utc))
                            .unwrap_or_else(|_| Utc::now()),
                    })
                    .collect();
                Ok(Json(zones).into_response())
            }
            Err(e) => {
                tracing::error!("Error loading cloud zones from database: {}", e);
                Err(ApiError::internal("Database error"))
            }
        },
        None => Err(ApiError::internal("Database not available")),
    }
}

/// 获取单个云区 (直接从数据库读取)
pub async fn get_cloud_zone(
    State(_state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApiError> {
    match crate::database::get_db() {
        Some(conn) => match db_get_cloud_zone_by_id(&conn, id).await {
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
                Ok(Json(zone).into_response())
            }
            Ok(None) => Err(ApiError::not_found("Operator/Manufacturer not found")),
            Err(e) => {
                tracing::error!("Error loading cloud zone from database: {}", e);
                Err(ApiError::internal("Database error"))
            }
        },
        None => Err(ApiError::internal("Database not available")),
    }
}

/// 创建云区 (直接写入数据库)
#[axum::debug_handler]
pub async fn create_cloud_zone(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<CreateCloudZoneRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // 检查权限
    if user.role != Role::SysAdmin && user.role != Role::SecAdmin {
        return Err(ApiError::forbidden("Access denied"));
    }

    match crate::database::get_db() {
        Some(conn) => {
            // 检查zone_code是否已存在
            match db_get_all_cloud_zones(&conn).await {
                Ok(existing_zones) => {
                    if existing_zones.iter().any(|z| z.zone_code == req.zone_code) {
                        return Err(ApiError::bad_request("Zone code already exists"));
                    }
                }
                Err(e) => {
                    tracing::error!("Error checking zone code: {}", e);
                    return Err(ApiError::internal("Database error"));
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
            )
            .await
            {
                Ok(id) => {
                    let zone = CloudZone {
                        id: Some(id),
                        zone_name: req.zone_name.clone(),
                        zone_code: req.zone_code.clone(),
                        description: req.description.clone(),
                        created_at: now,
                    };

                    // 记录日志
                    log_action_auth(
                        &state.audit_logs,
                        &user,
                        "CREATE_CLOUD_ZONE",
                        &req.zone_name,
                        &format!("Created cloud zone: {}", req.zone_name),
                    );

                    Ok(Json(json!({
                        "message": "运营商/厂家创建成功",
                        "data": zone
                    }))
                    .into_response())
                }
                Err(e) => {
                    tracing::error!("Error inserting cloud zone: {}", e);
                    Err(ApiError::internal("Failed to create operator/manufacturer"))
                }
            }
        }
        None => Err(ApiError::internal("Database not available")),
    }
}

/// 更新云区 (直接更新数据库)
#[axum::debug_handler]
pub async fn update_cloud_zone(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i32>,
    Json(req): Json<UpdateCloudZoneRequest>,
) -> Result<impl IntoResponse, ApiError> {
    if user.role != Role::SysAdmin && user.role != Role::SecAdmin {
        return Err(ApiError::forbidden("Access denied"));
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
                                if existing_zones
                                    .iter()
                                    .any(|z| z.id != id && z.zone_code == *new_code)
                                {
                                    return Err(ApiError::bad_request("Zone code already exists"));
                                }
                            }
                            Err(e) => {
                                tracing::error!("Error checking zone code: {}", e);
                                return Err(ApiError::internal("Database error"));
                            }
                        }
                    }
                }
                Ok(None) => {
                    return Err(ApiError::not_found("Operator/Manufacturer not found"));
                }
                Err(e) => {
                    tracing::error!("Error checking zone existence: {}", e);
                    return Err(ApiError::internal("Database error"));
                }
            }

            // 持久化到数据库
            match db_update_cloud_zone(
                id,
                req.zone_name.as_deref(),
                req.zone_code.as_deref(),
                req.description.as_deref(),
            )
            .await
            {
                Ok(_) => {
                    // 记录日志
                    log_action_auth(
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
                            Ok(Json(json!({
                                "message": "运营商/厂家更新成功",
                                "data": zone
                            }))
                            .into_response())
                        }
                        _ => Ok(Json(json!({ "message": "运营商/厂家更新成功" })).into_response()),
                    }
                }
                Err(e) => {
                    tracing::error!("Error updating cloud zone: {}", e);
                    Err(ApiError::internal("Failed to update operator/manufacturer"))
                }
            }
        }
        None => Err(ApiError::internal("Database not available")),
    }
}

/// 删除云区 (直接从数据库删除)
#[axum::debug_handler]
pub async fn delete_cloud_zone(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApiError> {
    if user.role != Role::SysAdmin && user.role != Role::SecAdmin {
        return Err(ApiError::forbidden("Access denied"));
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
                            log_action_auth(
                                &state.audit_logs,
                                &user,
                                "DELETE_CLOUD_ZONE",
                                &format!("{}", id),
                                &format!("Deleted cloud zone: {}", id),
                            );

                            Ok(Json(json!({ "message": "运营商/厂家删除成功" })).into_response())
                        }
                        Err(e) => {
                            tracing::error!("Error deleting cloud zone: {}", e);
                            Err(ApiError::internal("Failed to delete operator/manufacturer"))
                        }
                    }
                }
                Ok(None) => Err(ApiError::not_found("Operator/Manufacturer not found")),
                Err(e) => {
                    tracing::error!("Error checking zone existence: {}", e);
                    Err(ApiError::internal("Database error"))
                }
            }
        }
        None => Err(ApiError::internal("Database not available")),
    }
}
