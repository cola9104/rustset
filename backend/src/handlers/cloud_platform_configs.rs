use axum::{
    extract::{Path, State},
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::middleware::{ApiError, AuthUser};
use crate::state::AppState;
use crate::utils::log_action_auth;
use shared::Role;

/// 云平台配置数据模型
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CloudPlatformConfig {
    pub id: i32,
    pub platform_name: String,
    pub provider_id: i32,
    pub cloud_type: String,
    pub foundation: String,
    pub region_id: String,
    pub machine_room_id: i32,
    pub access_key_id: String,
    pub access_key_secret: String,
    pub remarks: Option<String>,
    pub status: String,
    pub last_test_time: Option<String>,
    pub last_test_result: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
}

/// 创建云平台配置请求
#[derive(Clone, Debug, Deserialize)]
pub struct CreateCloudPlatformConfigRequest {
    pub platform_name: String,
    pub provider_id: i32,
    pub cloud_type: String,
    pub foundation: String,
    pub region_id: String,
    pub machine_room_id: i32,
    pub access_key_id: String,
    pub access_key_secret: String,
    pub remarks: Option<String>,
    pub status: Option<String>,
}

/// 更新云平台配置请求
#[derive(Clone, Debug, Deserialize)]
pub struct UpdateCloudPlatformConfigRequest {
    pub platform_name: Option<String>,
    pub provider_id: Option<i32>,
    pub cloud_type: Option<String>,
    pub foundation: Option<String>,
    pub region_id: Option<String>,
    pub machine_room_id: Option<i32>,
    pub access_key_id: Option<String>,
    pub access_key_secret: Option<String>,
    pub remarks: Option<String>,
    pub status: Option<String>,
    pub last_test_time: Option<String>,
    pub last_test_result: Option<String>,
}

/// 获取云平台配置列表
pub async fn get_cloud_platform_configs(
    State(_state): State<AppState>,
    _user: AuthUser,
) -> Result<impl IntoResponse, ApiError> {
    match crate::database::get_db() {
        Some(conn) => match crate::database::get_all_cloud_platform_configs(&conn).await {
            Ok(configs) => Ok(Json(configs).into_response()),
            Err(e) => {
                eprintln!("Error loading cloud platform configs from database: {}", e);
                Err(ApiError::internal("Database error"))
            }
        },
        None => Err(ApiError::internal("Database not available")),
    }
}

/// 获取单个云平台配置
pub async fn get_cloud_platform_config(
    State(_state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApiError> {
    match crate::database::get_db() {
        Some(conn) => match crate::database::get_cloud_platform_config_by_id(&conn, id).await {
            Ok(Some(config)) => Ok(Json(config).into_response()),
            Ok(None) => Err(ApiError::not_found("Cloud platform config not found")),
            Err(e) => {
                eprintln!("Error loading cloud platform config from database: {}", e);
                Err(ApiError::internal("Database error"))
            }
        },
        None => Err(ApiError::internal("Database not available")),
    }
}

/// 创建云平台配置
pub async fn create_cloud_platform_config(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<CreateCloudPlatformConfigRequest>,
) -> Result<impl IntoResponse, ApiError> {
    if user.role != Role::SysAdmin && user.role != Role::SecAdmin {
        return Err(ApiError::forbidden("Access denied"));
    }

    match crate::database::get_db() {
        Some(conn) => {
            let now = chrono::Utc::now().to_rfc3339();
            let status = req.status.clone().unwrap_or_else(|| "active".to_string());

            match crate::database::insert_cloud_platform_config(
                &conn,
                &req.platform_name,
                req.provider_id,
                &req.cloud_type,
                &req.foundation,
                &req.region_id,
                req.machine_room_id,
                &req.access_key_id,
                &req.access_key_secret,
                req.remarks.as_deref(),
                &status,
                &now,
            )
            .await
            {
                Ok(id) => {
                    let config = CloudPlatformConfig {
                        id,
                        platform_name: req.platform_name.clone(),
                        provider_id: req.provider_id,
                        cloud_type: req.cloud_type.clone(),
                        foundation: req.foundation.clone(),
                        region_id: req.region_id.clone(),
                        machine_room_id: req.machine_room_id,
                        access_key_id: req.access_key_id.clone(),
                        access_key_secret: req.access_key_secret.clone(),
                        remarks: req.remarks.clone(),
                        status,
                        last_test_time: None,
                        last_test_result: None,
                        created_at: now,
                        updated_at: None,
                    };

                    log_action_auth(
                        &state.audit_logs,
                        &user,
                        "CREATE_CLOUD_PLATFORM_CONFIG",
                        &req.platform_name,
                        &format!("Created cloud platform config: {}", req.platform_name),
                    );

                    Ok(Json(json!({
                        "message": "云平台配置创建成功",
                        "data": config
                    }))
                    .into_response())
                }
                Err(e) => {
                    eprintln!("Error inserting cloud platform config: {}", e);
                    Err(ApiError::internal("Failed to create cloud platform config"))
                }
            }
        }
        None => Err(ApiError::internal("Database not available")),
    }
}

/// 更新云平台配置
pub async fn update_cloud_platform_config(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i32>,
    Json(req): Json<UpdateCloudPlatformConfigRequest>,
) -> Result<impl IntoResponse, ApiError> {
    if user.role != Role::SysAdmin && user.role != Role::SecAdmin {
        return Err(ApiError::forbidden("Access denied"));
    }

    match crate::database::get_db() {
        Some(conn) => match crate::database::get_cloud_platform_config_by_id(&conn, id).await {
            Ok(Some(_)) => {
                let now = chrono::Utc::now().to_rfc3339();
                match crate::database::update_cloud_platform_config(
                    &conn,
                    id,
                    req.platform_name.as_deref(),
                    req.provider_id,
                    req.cloud_type.as_deref(),
                    req.foundation.as_deref(),
                    req.region_id.as_deref(),
                    req.machine_room_id,
                    req.access_key_id.as_deref(),
                    req.access_key_secret.as_deref(),
                    req.remarks.as_deref(),
                    req.status.as_deref(),
                    req.last_test_time.as_deref(),
                    req.last_test_result.as_deref(),
                    Some(&now),
                )
                .await
                {
                    Ok(_) => {
                        log_action_auth(
                            &state.audit_logs,
                            &user,
                            "UPDATE_CLOUD_PLATFORM_CONFIG",
                            &format!("{}", id),
                            &format!("Updated cloud platform config: {}", id),
                        );

                        match crate::database::get_cloud_platform_config_by_id(&conn, id).await {
                            Ok(Some(config)) => Ok(Json(json!({
                                "message": "云平台配置更新成功",
                                "data": config
                            }))
                            .into_response()),
                            _ => Ok(
                                Json(json!({ "message": "云平台配置更新成功" })).into_response()
                            ),
                        }
                    }
                    Err(e) => {
                        eprintln!("Error updating cloud platform config: {}", e);
                        Err(ApiError::internal("Failed to update cloud platform config"))
                    }
                }
            }
            Ok(None) => Err(ApiError::not_found("Cloud platform config not found")),
            Err(e) => {
                eprintln!("Error checking cloud platform config existence: {}", e);
                Err(ApiError::internal("Database error"))
            }
        },
        None => Err(ApiError::internal("Database not available")),
    }
}

/// 删除云平台配置
pub async fn delete_cloud_platform_config(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApiError> {
    if user.role != Role::SysAdmin && user.role != Role::SecAdmin {
        return Err(ApiError::forbidden("Access denied"));
    }

    match crate::database::get_db() {
        Some(conn) => match crate::database::get_cloud_platform_config_by_id(&conn, id).await {
            Ok(Some(_)) => match crate::database::delete_cloud_platform_config(&conn, id).await {
                Ok(_) => {
                    log_action_auth(
                        &state.audit_logs,
                        &user,
                        "DELETE_CLOUD_PLATFORM_CONFIG",
                        &format!("{}", id),
                        &format!("Deleted cloud platform config: {}", id),
                    );

                    Ok(Json(json!({ "message": "云平台配置删除成功" })).into_response())
                }
                Err(e) => {
                    eprintln!("Error deleting cloud platform config: {}", e);
                    Err(ApiError::internal("Failed to delete cloud platform config"))
                }
            },
            Ok(None) => Err(ApiError::not_found("Cloud platform config not found")),
            Err(e) => {
                eprintln!("Error checking cloud platform config existence: {}", e);
                Err(ApiError::internal("Database error"))
            }
        },
        None => Err(ApiError::internal("Database not available")),
    }
}
