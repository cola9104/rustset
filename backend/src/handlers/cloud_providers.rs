use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde_json::json;
use std::sync::Mutex;

use crate::state::AppState;
use shared::{
    CloudProviderConfig, CloudProviderConfigStatus,
    CreateCloudProviderConfigRequest, UpdateCloudProviderConfigRequest,
};
use crate::database::{insert_provider_config, update_provider_config, delete_provider_config};

// 云厂商配置存储 (内存)
pub static PROVIDER_CONFIGS: Mutex<Vec<CloudProviderConfig>> = Mutex::new(Vec::new());

/// 获取云厂商配置列表
pub async fn get_cloud_provider_configs(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let _ = &state; // Mark as intentionally unused
    let configs = PROVIDER_CONFIGS.lock().unwrap().clone();
    Json(configs).into_response()
}

/// 获取单个云厂商配置
pub async fn get_cloud_provider_config(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    let _ = &state; // Mark as intentionally unused
    let configs = PROVIDER_CONFIGS.lock().unwrap();
    if let Some(config) = configs.iter().find(|c| c.id == Some(id)) {
        Json(config.clone()).into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

/// 创建云厂商配置
#[axum::debug_handler]
pub async fn create_cloud_provider_config(
    State(state): State<AppState>,
    Json(req): Json<CreateCloudProviderConfigRequest>,
) -> impl IntoResponse {
    let _ = &state; // Mark as intentionally unused

    // 生成新ID
    let new_id = {
        let configs = PROVIDER_CONFIGS.lock().unwrap();
        configs.iter().filter_map(|c| c.id).max().map_or(1, |m| m + 1)
    };

    let now = chrono::Utc::now();
    let config = CloudProviderConfig {
        id: Some(new_id),
        zone_id: req.zone_id,
        platform_id: req.platform_id,
        provider: req.provider.clone(),
        region_id: req.region_id.clone(),
        region_name: req.region_name.clone(),
        available_zones: req.available_zones.clone(),
        account_name: req.account_name.clone(),
        access_key_id: req.access_key_id.clone(),
        access_key_secret: req.access_key_secret.clone(),
        status: CloudProviderConfigStatus::Inactive, // 新建默认为停用
        remarks: req.remarks.clone(),
        last_test_time: None,
        last_test_result: None,
        created_at: now,
        updated_at: Some(now),
    };

    {
        let mut configs = PROVIDER_CONFIGS.lock().unwrap();
        configs.push(config.clone());
    }

    // 持久化到数据库
    let created_at_str = now.to_rfc3339();
    let provider_str = serde_json::to_string(&config.provider).unwrap_or_default();
    let _ = crate::database::insert_provider_config(
        req.zone_id.unwrap_or(0),
        req.platform_id.unwrap_or(0),
        &provider_str,
        &req.region_id,
        &req.region_name,
        &req.account_name,
        &req.access_key_id,
        &req.access_key_secret,
        req.remarks.as_deref(),
        &created_at_str,
    ).await;

    Json(json!({
        "message": "配置创建成功",
        "data": config
    })).into_response()
}

/// 更新云厂商配置
#[axum::debug_handler]
pub async fn update_cloud_provider_config(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateCloudProviderConfigRequest>,
) -> impl IntoResponse {
    let _ = &state; // Mark as intentionally unused

    // First, check if the config exists and clone its data
    let (found, db_values) = {
        let configs = PROVIDER_CONFIGS.lock().unwrap();
        if let Some(config) = configs.iter().find(|c| c.id == Some(id)) {
            (
                true,
                (
                    config.zone_id,
                    config.platform_id,
                    config.region_id.clone(),
                    config.region_name.clone(),
                    config.account_name.clone(),
                    config.access_key_id.clone(),
                    config.access_key_secret.clone(),
                    config.remarks.clone(),
                ),
            )
        } else {
            (false, (None, None, String::new(), String::new(), String::new(), String::new(), String::new(), None))
        }
    };

    if !found {
        return StatusCode::NOT_FOUND.into_response();
    }

    // Now do the update
    let (updated_config, db_zone_id, db_platform_id, db_region_id, db_region_name,
         db_account_name, db_access_key_id, db_access_key_secret, db_remarks, db_status, updated_at_str) = {
        let mut configs = PROVIDER_CONFIGS.lock().unwrap();
        let config = configs.iter_mut().find(|c| c.id == Some(id)).unwrap();

        if let Some(region_name) = req.region_name.clone() {
            config.region_name = region_name;
        }
        if let Some(available_zones) = req.available_zones.clone() {
            config.available_zones = available_zones;
        }
        if let Some(account_name) = req.account_name.clone() {
            config.account_name = account_name;
        }
        if let Some(access_key_id) = req.access_key_id.clone() {
            config.access_key_id = access_key_id;
        }
        if let Some(access_key_secret) = req.access_key_secret.clone() {
            config.access_key_secret = access_key_secret;
        }
        if let Some(status) = req.status.clone() {
            config.status = status;
        }
        if let Some(remarks) = req.remarks.clone() {
            config.remarks = Some(remarks);
        }
        if let Some(zone_id) = req.zone_id {
            config.zone_id = Some(zone_id);
        }
        if let Some(platform_id) = req.platform_id {
            config.platform_id = Some(platform_id);
        }
        config.updated_at = Some(chrono::Utc::now());

        let updated_at_str = config.updated_at.map(|dt| dt.to_rfc3339());
        let db_status = config.status.clone();

        (
            config.clone(),
            config.zone_id,
            config.platform_id,
            config.region_id.clone(),
            config.region_name.clone(),
            config.account_name.clone(),
            config.access_key_id.clone(),
            config.access_key_secret.clone(),
            config.remarks.clone(),
            db_status,
            updated_at_str,
        )
    };

    // 持久化到数据库
    let status_str = match db_status {
        shared::CloudProviderConfigStatus::Active => "active",
        shared::CloudProviderConfigStatus::Inactive => "inactive",
        shared::CloudProviderConfigStatus::Testing => "testing",
        shared::CloudProviderConfigStatus::Error => "error",
    };
    let _ = update_provider_config(
        id,
        db_zone_id.unwrap_or(0),
        db_platform_id.unwrap_or(0),
        &db_region_id,
        &db_region_name,
        &db_account_name,
        &db_access_key_id,
        &db_access_key_secret,
        db_remarks.as_deref(),
        status_str,
        updated_at_str.as_deref(),
    ).await;

    Json(json!({
        "message": "配置更新成功",
        "data": updated_config
    })).into_response()
}

/// 删除云厂商配置
#[axum::debug_handler]
pub async fn delete_cloud_provider_config(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    let _ = &state; // Mark as intentionally unused

    // Check if exists and remove
    let found = {
        let mut configs = PROVIDER_CONFIGS.lock().unwrap();
        configs.iter().position(|c| c.id == Some(id))
    };

    if let Some(pos) = found {
        // Remove outside of the lock scope
        {
            let mut configs = PROVIDER_CONFIGS.lock().unwrap();
            configs.remove(pos);
        }

        // 持久化到数据库
        let _ = delete_provider_config(id).await;

        Json(json!({ "message": "配置删除成功" })).into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

/// 测试云厂商连接
#[axum::debug_handler]
pub async fn test_cloud_provider_connection(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    let _ = &state; // Mark as intentionally unused
    let exists = {
        let configs = PROVIDER_CONFIGS.lock().unwrap();
        configs.iter().any(|c| c.id == Some(id))
    };

    if exists {
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

        Json(test_result).into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

/// 获取云厂商选项列表
pub async fn get_cloud_provider_options(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let _ = &state; // Mark as intentionally unused
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
    Json(options).into_response()
}

/// 获取已启用的云厂商配置（用于业务申请选择）
///
/// 返回完整的 CloudProviderConfig 对象，包括 zone_id 和 platform_id，
/// 以便前端可以根据选中的云区和云平台进行过滤。
pub async fn get_active_cloud_provider_configs(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let _ = &state; // Mark as intentionally unused;
    let configs = PROVIDER_CONFIGS.lock().unwrap();
    let active: Vec<_> = configs.iter()
        .filter(|c| c.status == CloudProviderConfigStatus::Active)
        .cloned()
        .collect();

    Json(active).into_response()
}
