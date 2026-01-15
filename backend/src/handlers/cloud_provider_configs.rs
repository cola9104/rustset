//! Cloud Provider Config / 云区对接管理 API handlers
//!
//! 职责说明：
//! - 本模块负责管理已对接的云平台账户和区域配置
//! - 业务申请时只能选择已配置且启用的云区
//! - 提供云平台连接测试功能

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::time::Instant;
use shared::{
    CloudProviderConfig, CloudProviderConfigStatus,
    CreateCloudProviderConfigRequest, UpdateCloudProviderConfigRequest,
    CloudProviderConfigQuery, ConnectionTestResult,
};
use crate::state::AppState;
use chrono::Utc;
use uuid::Uuid;

/// 获取云区对接配置列表
pub async fn get_cloud_provider_configs(
    State(state): State<AppState>,
    Query(query): Query<CloudProviderConfigQuery>,
) -> impl IntoResponse {
    let configs = state.cloud_provider_configs.lock().unwrap().clone();

    // 过滤
    let filtered: Vec<CloudProviderConfig> = configs
        .into_iter()
        .filter(|c| {
            query.provider.as_ref().map_or(true, |p| &c.provider == p)
        })
        .filter(|c| {
            query.region_id.as_ref().map_or(true, |r| &c.region_id == r)
        })
        .filter(|c| {
            query.status.as_ref().map_or(true, |s| &c.status == s)
        })
        .filter(|c| {
            query.account_name.as_ref().map_or(true, |a| &c.account_name == a)
        })
        .collect();

    // 返回时隐藏密钥
    let sanitized: Vec<serde_json::Value> = filtered
        .into_iter()
        .map(|c| {
            json!({
                "id": c.id,
                "provider": c.provider,
                "region_id": c.region_id,
                "region_name": c.region_name,
                "available_zones": c.available_zones,
                "account_name": c.account_name,
                "access_key_id": if c.access_key_id.len() > 8 {
                    format!("{}****", &c.access_key_id[..8])
                } else {
                    "****".to_string()
                },
                "access_key_secret": "****",
                "status": c.status,
                "remarks": c.remarks,
                "last_test_time": c.last_test_time,
                "last_test_result": c.last_test_result,
                "created_at": c.created_at,
                "updated_at": c.updated_at,
            })
        })
        .collect();

    Json(sanitized)
}

/// 获取单个云区对接配置（包含完整密钥）
pub async fn get_cloud_provider_config(
    Path(config_id): Path<i32>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let configs = state.cloud_provider_configs.lock().unwrap();

    if let Some(config) = configs.iter().find(|c| c.id == Some(config_id)) {
        Json(config.clone()).into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Cloud provider config not found"}))
        ).into_response()
    }
}

/// 创建云区对接配置
pub async fn create_cloud_provider_config(
    State(state): State<AppState>,
    Json(req): Json<CreateCloudProviderConfigRequest>,
) -> impl IntoResponse {
    let mut configs = state.cloud_provider_configs.lock().unwrap();
    let new_id = configs.iter().filter_map(|c| c.id).max().map(|m| m + 1).unwrap_or(1);

    let new_config = CloudProviderConfig {
        id: Some(new_id),
        provider: req.provider.clone(),
        region_id: req.region_id.clone(),
        region_name: req.region_name.clone(),
        available_zones: req.available_zones.clone(),
        account_name: req.account_name.clone(),
        access_key_id: req.access_key_id.clone(),
        access_key_secret: req.access_key_secret.clone(),
        status: CloudProviderConfigStatus::Testing, // 新建默认为测试中
        remarks: req.remarks.clone(),
        last_test_time: None,
        last_test_result: None,
        created_at: Utc::now(),
        updated_at: None,
    };

    configs.push(new_config.clone());

    // 记录审计日志
    let mut logs = state.audit_logs.lock().unwrap();
    logs.push(shared::AuditLog {
        id: Uuid::new_v4().to_string(),
        user_id: "system".to_string(),
        username: "system".to_string(),
        action: "CREATE_CLOUD_PROVIDER_CONFIG".to_string(),
        target: format!("CloudProviderConfig: {}", new_id),
        details: format!(
            "创建云区对接配置: {} - {} ({})",
            new_config.provider.as_str(),
            new_config.region_name,
            new_config.account_name
        ),
        timestamp: Utc::now(),
    });

    (StatusCode::CREATED, Json(new_config)).into_response()
}

/// 更新云区对接配置
pub async fn update_cloud_provider_config(
    Path(config_id): Path<i32>,
    State(state): State<AppState>,
    Json(req): Json<UpdateCloudProviderConfigRequest>,
) -> impl IntoResponse {
    let mut configs = state.cloud_provider_configs.lock().unwrap();

    if let Some(config) = configs.iter_mut().find(|c| c.id == Some(config_id)) {
        if let Some(region_name) = req.region_name {
            config.region_name = region_name;
        }
        if let Some(zones) = req.available_zones {
            config.available_zones = zones;
        }
        if let Some(account_name) = req.account_name {
            config.account_name = account_name;
        }
        if let Some(ak_id) = req.access_key_id {
            config.access_key_id = ak_id;
        }
        if let Some(ak_secret) = req.access_key_secret {
            config.access_key_secret = ak_secret;
        }
        if let Some(status) = req.status {
            config.status = status;
        }
        if let Some(remarks) = req.remarks {
            config.remarks = Some(remarks);
        }
        config.updated_at = Some(Utc::now());

        // 记录审计日志
        let mut logs = state.audit_logs.lock().unwrap();
        logs.push(shared::AuditLog {
            id: Uuid::new_v4().to_string(),
            user_id: "system".to_string(),
            username: "system".to_string(),
            action: "UPDATE_CLOUD_PROVIDER_CONFIG".to_string(),
            target: format!("CloudProviderConfig: {}", config_id),
            details: format!("更新云区对接配置: {}", config_id),
            timestamp: Utc::now(),
        });

        Json(config.clone()).into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Cloud provider config not found"}))
        ).into_response()
    }
}

/// 删除云区对接配置
pub async fn delete_cloud_provider_config(
    Path(config_id): Path<i32>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let mut configs = state.cloud_provider_configs.lock().unwrap();

    let original_len = configs.len();
    configs.retain(|c| c.id != Some(config_id));

    if configs.len() < original_len {
        // 记录审计日志
        let mut logs = state.audit_logs.lock().unwrap();
        logs.push(shared::AuditLog {
            id: Uuid::new_v4().to_string(),
            user_id: "system".to_string(),
            username: "system".to_string(),
            action: "DELETE_CLOUD_PROVIDER_CONFIG".to_string(),
            target: format!("CloudProviderConfig: {}", config_id),
            details: format!("删除云区对接配置: {}", config_id),
            timestamp: Utc::now(),
        });

        Json(json!({"message": "Cloud provider config deleted successfully"})).into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Cloud provider config not found"}))
        ).into_response()
    }
}

/// 测试云平台连接
///
/// 测试指定配置的云平台连接是否正常
/// 实际生产环境需要对接各云厂商的 SDK
pub async fn test_cloud_provider_connection(
    Path(config_id): Path<i32>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let mut configs = state.cloud_provider_configs.lock().unwrap();

    if let Some(config) = configs.iter_mut().find(|c| c.id == Some(config_id)) {
        let start = Instant::now();
        let tested_at = Utc::now();

        // TODO: 实际对接云厂商 API
        // 阿里云: 使用 ecs_sdk::Client::describe_regions
        // 腾讯云: 使用 tencent_cloud_sdk::describe_regions
        // 华为云: 使用 huawei_cloud_sdk::list_regions

        // 模拟连接测试
        let (success, message) = simulate_connection_test(&config.provider, &config.region_id);

        let response_time_ms = Some(start.elapsed().as_millis() as u64);

        // 更新配置状态
        config.last_test_time = Some(tested_at);
        config.last_test_result = Some(message.clone());
        config.status = if success {
            CloudProviderConfigStatus::Active
        } else {
            CloudProviderConfigStatus::Error
        };
        config.updated_at = Some(tested_at);

        let result = ConnectionTestResult {
            success,
            message,
            response_time_ms,
            tested_at,
        };

        // 记录审计日志
        let mut logs = state.audit_logs.lock().unwrap();
        logs.push(shared::AuditLog {
            id: Uuid::new_v4().to_string(),
            user_id: "system".to_string(),
            username: "system".to_string(),
            action: "TEST_CLOUD_PROVIDER_CONNECTION".to_string(),
            target: format!("CloudProviderConfig: {}", config_id),
            details: format!(
                "测试云平台连接: {} - {} - 结果: {}",
                config.provider.as_str(),
                config.region_name,
                if result.success { "成功" } else { "失败" }
            ),
            timestamp: tested_at,
        });

        Json(result).into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Cloud provider config not found"}))
        ).into_response()
    }
}

/// 获取可用的云区配置（用于业务申请下拉选择）
///
/// 只返回状态为 Active 的配置
pub async fn get_active_cloud_provider_configs(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let configs = state.cloud_provider_configs.lock().unwrap();

    let active_configs: Vec<serde_json::Value> = configs
        .iter()
        .filter(|c| c.status == CloudProviderConfigStatus::Active)
        .map(|c| {
            json!({
                "id": c.id,
                "provider": c.provider,
                "region_id": c.region_id,
                "region_name": c.region_name,
                "available_zones": c.available_zones,
                "account_name": c.account_name,
                "display_name": format!("{} - {}", c.provider.as_str(), c.region_name),
            })
        })
        .collect();

    Json(active_configs)
}

/// 模拟云平台连接测试
///
/// 实际生产环境需要替换为真实的云厂商 SDK 调用
fn simulate_connection_test(provider: &shared::CloudProvider, region_id: &str) -> (bool, String) {
    // 模拟不同的响应结果
    match provider {
        shared::CloudProvider::Aliyun => {
            if region_id.starts_with("cn-") {
                (true, "连接成功: 阿里云区域可用".to_string())
            } else {
                (false, "连接失败: 区域不存在".to_string())
            }
        }
        shared::CloudProvider::Tencent => {
            if region_id.starts_with("ap-") {
                (true, "连接成功: 腾讯云区域可用".to_string())
            } else {
                (false, "连接失败: 区域不存在".to_string())
            }
        }
        shared::CloudProvider::Huawei => {
            if region_id.contains("cn-") {
                (true, "连接成功: 华为云区域可用".to_string())
            } else {
                (false, "连接失败: 区域不存在".to_string())
            }
        }
        shared::CloudProvider::Aws => {
            if region_id.contains("-") {
                (true, "连接成功: AWS区域可用".to_string())
            } else {
                (false, "连接失败: 区域不存在".to_string())
            }
        }
        _ => (true, "连接成功: 云平台响应正常".to_string())
    }
}
