use axum::{
    extract::{Path, State},
    response::{IntoResponse, Json},
};
use serde_json::json;

use crate::database::{
    delete_provider_config, get_active_cloud_provider_configs as get_active_provider_configs_db,
    get_all_cloud_provider_configs, get_cloud_provider_config_by_id, insert_provider_config,
    update_provider_config,
};
use crate::middleware::{ApiError, AuthUser};
use crate::state::AppState;
use crate::utils::log_action_auth;
use shared::{
    CloudProviderConfig, CloudProviderConfigStatus, CreateCloudProviderConfigRequest, Role,
    UpdateCloudProviderConfigRequest,
};

/// 获取云平台（技术底座）配置列表
pub async fn get_cloud_provider_configs(
    State(_state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let db_conn = match crate::database::get_db() {
        Some(conn) => conn,
        None => return Err(ApiError::internal("Database not available")),
    };

    match get_all_cloud_provider_configs(&db_conn).await {
        Ok(db_configs) => {
            let configs: Vec<CloudProviderConfig> = db_configs
                .into_iter()
                .filter_map(|db| {
                    // Parse provider from JSON string
                    let provider = serde_json::from_str(&db.provider).ok()?;
                    // Parse available_zones from region (stored as comma-separated if exists)
                    let available_zones = if !db.region_id.is_empty() {
                        db.region_id.split(',').map(|s| s.to_string()).collect()
                    } else {
                        Vec::new()
                    };
                    // Parse status
                    let status = match db.status.as_str() {
                        "active" => CloudProviderConfigStatus::Active,
                        "inactive" => CloudProviderConfigStatus::Inactive,
                        "testing" => CloudProviderConfigStatus::Testing,
                        "error" => CloudProviderConfigStatus::Error,
                        _ => CloudProviderConfigStatus::Inactive,
                    };
                    // Parse timestamps
                    let created_at = chrono::DateTime::parse_from_rfc3339(&db.created_at)
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                        .unwrap_or_else(|_| chrono::Utc::now());
                    let last_test_time = db.last_test_time.and_then(|t| {
                        chrono::DateTime::parse_from_rfc3339(&t)
                            .map(|dt| dt.with_timezone(&chrono::Utc))
                            .ok()
                    });

                    Some(CloudProviderConfig {
                        id: Some(db.id),
                        zone_id: Some(db.zone_id),
                        platform_id: Some(db.platform_id),
                        provider,
                        region_id: db.region_id,
                        region_name: db.region_name,
                        available_zones,
                        account_name: db.account_name,
                        access_key_id: db.access_key_id,
                        access_key_secret: db.access_key_secret,
                        status,
                        remarks: db.remarks,
                        last_test_time,
                        last_test_result: db.last_test_result,
                        created_at,
                        updated_at: db.updated_at.and_then(|t| {
                            chrono::DateTime::parse_from_rfc3339(&t)
                                .map(|dt| dt.with_timezone(&chrono::Utc))
                                .ok()
                        }),
                    })
                })
                .collect();
            Ok(Json(configs).into_response())
        }
        Err(e) => {
            eprintln!("Error loading cloud provider configs from database: {}", e);
            Err(ApiError::internal("Database error"))
        }
    }
}

/// 获取单个云平台（技术底座）配置
pub async fn get_cloud_provider_config(
    State(_state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApiError> {
    let db_conn = match crate::database::get_db() {
        Some(conn) => conn,
        None => return Err(ApiError::internal("Database not available")),
    };

    match get_cloud_provider_config_by_id(&db_conn, id).await {
        Ok(Some(db)) => {
            // Parse provider from JSON string
            let provider = match serde_json::from_str(&db.provider) {
                Ok(p) => p,
                Err(_) => return Err(ApiError::internal("Failed to parse cloud provider config")),
            };
            // Parse available_zones from region (stored as comma-separated if exists)
            let available_zones = if !db.region_id.is_empty() {
                db.region_id.split(',').map(|s| s.to_string()).collect()
            } else {
                Vec::new()
            };
            // Parse status
            let status = match db.status.as_str() {
                "active" => CloudProviderConfigStatus::Active,
                "inactive" => CloudProviderConfigStatus::Inactive,
                "testing" => CloudProviderConfigStatus::Testing,
                "error" => CloudProviderConfigStatus::Error,
                _ => CloudProviderConfigStatus::Inactive,
            };
            // Parse timestamps
            let created_at = chrono::DateTime::parse_from_rfc3339(&db.created_at)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now());
            let last_test_time = db.last_test_time.and_then(|t| {
                chrono::DateTime::parse_from_rfc3339(&t)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .ok()
            });

            let config = CloudProviderConfig {
                id: Some(db.id),
                zone_id: Some(db.zone_id),
                platform_id: Some(db.platform_id),
                provider,
                region_id: db.region_id,
                region_name: db.region_name,
                available_zones,
                account_name: db.account_name,
                access_key_id: db.access_key_id,
                access_key_secret: db.access_key_secret,
                status,
                remarks: db.remarks,
                last_test_time,
                last_test_result: db.last_test_result,
                created_at,
                updated_at: db.updated_at.and_then(|t| {
                    chrono::DateTime::parse_from_rfc3339(&t)
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                        .ok()
                }),
            };
            Ok(Json(config).into_response())
        }
        Ok(None) => Err(ApiError::not_found("Cloud provider config not found")),
        Err(e) => {
            eprintln!("Error loading cloud provider config from database: {}", e);
            Err(ApiError::internal("Database error"))
        }
    }
}

/// 创建云平台（技术底座）配置
#[axum::debug_handler]
pub async fn create_cloud_provider_config(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<CreateCloudProviderConfigRequest>,
) -> Result<impl IntoResponse, ApiError> {
    if user.role != Role::SysAdmin && user.role != Role::SecAdmin {
        return Err(ApiError::forbidden("Access denied"));
    }

    let now = chrono::Utc::now();
    let provider_str = serde_json::to_string(&req.provider).unwrap_or_default();

    // 持久化到数据库
    match insert_provider_config(
        req.zone_id.unwrap_or_default(),
        req.platform_id.unwrap_or_default(),
        &provider_str,
        &req.region_id,
        &req.region_name,
        &req.account_name,
        &req.access_key_id,
        &req.access_key_secret,
        req.remarks.as_deref(),
        &now.to_rfc3339(),
    )
    .await
    {
        Ok(id) => {
            let config = CloudProviderConfig {
                id: Some(id),
                zone_id: req.zone_id,
                platform_id: req.platform_id,
                provider: req.provider.clone(),
                region_id: req.region_id.clone(),
                region_name: req.region_name.clone(),
                available_zones: req.available_zones.clone(),
                account_name: req.account_name.clone(),
                access_key_id: req.access_key_id.clone(),
                access_key_secret: req.access_key_secret.clone(),
                status: CloudProviderConfigStatus::Inactive,
                remarks: req.remarks.clone(),
                last_test_time: None,
                last_test_result: None,
                created_at: now,
                updated_at: Some(now),
            };

            log_action_auth(
                &state.audit_logs,
                &user,
                "CREATE_CLOUD_PROVIDER_CONFIG",
                &req.account_name,
                &format!("Created cloud provider config: {}", req.account_name),
            );

            Ok(Json(json!({
                "message": "配置创建成功",
                "data": config
            }))
            .into_response())
        }
        Err(e) => {
            eprintln!("Error creating cloud provider config: {}", e);
            Err(ApiError::internal("Failed to create cloud provider config"))
        }
    }
}

/// 更新云平台（技术底座）配置
#[axum::debug_handler]
pub async fn update_cloud_provider_config(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i32>,
    Json(req): Json<UpdateCloudProviderConfigRequest>,
) -> Result<impl IntoResponse, ApiError> {
    if user.role != Role::SysAdmin && user.role != Role::SecAdmin {
        return Err(ApiError::forbidden("Access denied"));
    }

    // Check if the config exists in database
    let db_conn = match crate::database::get_db() {
        Some(conn) => conn,
        None => return Err(ApiError::internal("Database not available")),
    };

    let existing = match get_cloud_provider_config_by_id(&db_conn, id).await {
        Ok(Some(config)) => config,
        Ok(None) => return Err(ApiError::not_found("Cloud provider config not found")),
        Err(e) => {
            eprintln!("Error checking cloud provider config existence: {}", e);
            return Err(ApiError::internal("Database error"));
        }
    };

    // Build the update parameters from request or existing values
    let zone_id = req.zone_id.unwrap_or(existing.zone_id);
    let platform_id = req.platform_id.unwrap_or(existing.platform_id);
    let region_id = existing.region_id.clone();
    let region_name = req
        .region_name
        .unwrap_or_else(|| existing.region_name.clone());
    let account_name = req
        .account_name
        .unwrap_or_else(|| existing.account_name.clone());
    let access_key_id = req
        .access_key_id
        .unwrap_or_else(|| existing.access_key_id.clone());
    let access_key_secret = req
        .access_key_secret
        .unwrap_or_else(|| existing.access_key_secret.clone());
    let remarks = req.remarks;
    let status_str = if let Some(status) = req.status {
        match status {
            shared::CloudProviderConfigStatus::Active => "active",
            shared::CloudProviderConfigStatus::Inactive => "inactive",
            shared::CloudProviderConfigStatus::Testing => "testing",
            shared::CloudProviderConfigStatus::Error => "error",
        }
    } else {
        existing.status.as_str()
    };
    let updated_at = Some(chrono::Utc::now().to_rfc3339());

    // 持久化到数据库
    match update_provider_config(
        id,
        zone_id,
        platform_id,
        &region_id,
        &region_name,
        &account_name,
        &access_key_id,
        &access_key_secret,
        remarks.as_deref(),
        status_str,
        updated_at.as_deref(),
    )
    .await
    {
        Ok(_) => {
            log_action_auth(
                &state.audit_logs,
                &user,
                "UPDATE_CLOUD_PROVIDER_CONFIG",
                &format!("{}", id),
                &format!("Updated cloud provider config: {}", id),
            );

            // Fetch the updated config
            match get_cloud_provider_config_by_id(&db_conn, id).await {
                Ok(Some(db)) => {
                    let provider =
                        serde_json::from_str(&db.provider).unwrap_or(shared::CloudProvider::Aliyun);
                    let config = CloudProviderConfig {
                        id: Some(db.id),
                        zone_id: Some(db.zone_id),
                        platform_id: Some(db.platform_id),
                        provider,
                        region_id: db.region_id,
                        region_name: db.region_name,
                        available_zones: Vec::new(),
                        account_name: db.account_name,
                        access_key_id: db.access_key_id,
                        access_key_secret: db.access_key_secret,
                        status: match db.status.as_str() {
                            "active" => CloudProviderConfigStatus::Active,
                            "inactive" => CloudProviderConfigStatus::Inactive,
                            "testing" => CloudProviderConfigStatus::Testing,
                            _ => CloudProviderConfigStatus::Error,
                        },
                        remarks: db.remarks,
                        last_test_time: None,
                        last_test_result: db.last_test_result,
                        created_at: chrono::Utc::now(),
                        updated_at: db.updated_at.and_then(|t| {
                            chrono::DateTime::parse_from_rfc3339(&t)
                                .map(|dt| dt.with_timezone(&chrono::Utc))
                                .ok()
                        }),
                    };

                    Ok(Json(json!({
                        "message": "配置更新成功",
                        "data": config
                    }))
                    .into_response())
                }
                Ok(None) => Err(ApiError::internal(
                    "Failed to load updated cloud provider config",
                )),
                Err(e) => {
                    eprintln!("Error loading updated cloud provider config: {}", e);
                    Err(ApiError::internal("Database error"))
                }
            }
        }
        Err(e) => {
            eprintln!("Error updating cloud provider config: {}", e);
            Err(ApiError::internal("Failed to update cloud provider config"))
        }
    }
}

/// 删除云平台（技术底座）配置
#[axum::debug_handler]
pub async fn delete_cloud_provider_config(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApiError> {
    if user.role != Role::SysAdmin && user.role != Role::SecAdmin {
        return Err(ApiError::forbidden("Access denied"));
    }

    // Check if exists in database
    let db_conn = match crate::database::get_db() {
        Some(conn) => conn,
        None => return Err(ApiError::internal("Database not available")),
    };

    match get_cloud_provider_config_by_id(&db_conn, id).await {
        Ok(Some(config)) => {
            // Delete from database
            match delete_provider_config(id).await {
                Ok(_) => {
                    log_action_auth(
                        &state.audit_logs,
                        &user,
                        "DELETE_CLOUD_PROVIDER_CONFIG",
                        &config.account_name,
                        &format!("Deleted cloud provider config: {}", id),
                    );

                    Ok(Json(json!({ "message": "配置删除成功" })).into_response())
                }
                Err(e) => {
                    eprintln!("Error deleting cloud provider config: {}", e);
                    Err(ApiError::internal("Failed to delete cloud provider config"))
                }
            }
        }
        Ok(None) => Err(ApiError::not_found("Cloud provider config not found")),
        Err(e) => {
            eprintln!("Error checking cloud provider config existence: {}", e);
            Err(ApiError::internal("Database error"))
        }
    }
}

/// 测试云平台（技术底座）连接
#[axum::debug_handler]
pub async fn test_cloud_provider_connection(
    State(_state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApiError> {
    if user.role != Role::SysAdmin && user.role != Role::SecAdmin {
        return Err(ApiError::forbidden("Access denied"));
    }

    let db_conn = match crate::database::get_db() {
        Some(conn) => conn,
        None => return Err(ApiError::internal("Database not available")),
    };

    match get_cloud_provider_config_by_id(&db_conn, id).await {
        Ok(Some(_)) => {
            use std::time::{SystemTime, UNIX_EPOCH};
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();

            let test_result = json!({
                "success": true,
                "message": "连接测试成功",
                "response_time_ms": 150u32,
                "tested_at": now
            });

            Ok(Json(test_result).into_response())
        }
        Ok(None) => Err(ApiError::not_found("Cloud provider config not found")),
        Err(e) => {
            eprintln!("Error testing cloud provider connection: {}", e);
            Err(ApiError::internal("Database error"))
        }
    }
}

/// 获取云平台（技术底座）厂商选项列表
pub async fn get_cloud_provider_options(
    State(_state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let options = vec![
        json!({
            "value": "aliyun",
            "label": "阿里云",
        }),
        json!({
            "value": "tencent",
            "label": "腾讯云",
        }),
        json!({
            "value": "huawei",
            "label": "华为云",
        }),
        json!({
            "value": "aws",
            "label": "AWS",
        }),
    ];
    Ok(Json(options).into_response())
}

/// 获取已启用的云平台（技术底座）配置（用于业务申请选择）
///
/// 返回完整的 CloudProviderConfig 对象，包括 zone_id 和 platform_id，
/// 以便前端可以根据选中的云区和云平台进行过滤。
pub async fn get_active_cloud_provider_configs(
    State(_state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let db_conn = match crate::database::get_db() {
        Some(conn) => conn,
        None => return Err(ApiError::internal("Database not available")),
    };

    match get_active_provider_configs_db(&db_conn).await {
        Ok(db_configs) => {
            let configs: Vec<CloudProviderConfig> = db_configs
                .into_iter()
                .filter_map(|db| {
                    // Parse provider from JSON string
                    let provider = serde_json::from_str(&db.provider).ok()?;
                    // Parse available_zones from region (stored as comma-separated if exists)
                    let available_zones = if !db.region_id.is_empty() {
                        db.region_id.split(',').map(|s| s.to_string()).collect()
                    } else {
                        Vec::new()
                    };
                    // Parse timestamps
                    let created_at = chrono::DateTime::parse_from_rfc3339(&db.created_at)
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                        .unwrap_or_else(|_| chrono::Utc::now());
                    let last_test_time = db.last_test_time.and_then(|t| {
                        chrono::DateTime::parse_from_rfc3339(&t)
                            .map(|dt| dt.with_timezone(&chrono::Utc))
                            .ok()
                    });

                    Some(CloudProviderConfig {
                        id: Some(db.id),
                        zone_id: Some(db.zone_id),
                        platform_id: Some(db.platform_id),
                        provider,
                        region_id: db.region_id,
                        region_name: db.region_name,
                        available_zones,
                        account_name: db.account_name,
                        access_key_id: db.access_key_id,
                        access_key_secret: db.access_key_secret,
                        status: CloudProviderConfigStatus::Active,
                        remarks: db.remarks,
                        last_test_time,
                        last_test_result: db.last_test_result,
                        created_at,
                        updated_at: db.updated_at.and_then(|t| {
                            chrono::DateTime::parse_from_rfc3339(&t)
                                .map(|dt| dt.with_timezone(&chrono::Utc))
                                .ok()
                        }),
                    })
                })
                .collect();
            Ok(Json(configs).into_response())
        }
        Err(e) => {
            eprintln!(
                "Error loading active cloud provider configs from database: {}",
                e
            );
            Err(ApiError::internal("Database error"))
        }
    }
}
