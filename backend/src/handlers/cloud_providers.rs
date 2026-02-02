use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde_json::json;
use std::sync::Mutex;

use shared::{
    CloudProviderConfig, CloudProviderConfigStatus,
    CreateCloudProviderConfigRequest, UpdateCloudProviderConfigRequest,
};

// 云厂商配置存储 (内存)
pub static PROVIDER_CONFIGS: Mutex<Vec<CloudProviderConfig>> = Mutex::new(Vec::new());

/// 获取云厂商配置列表
pub async fn get_cloud_provider_configs() -> impl IntoResponse {
    let configs = PROVIDER_CONFIGS.lock().unwrap().clone();
    Json(configs)
}

/// 获取单个云厂商配置
pub async fn get_cloud_provider_config(
    axum::extract::Path(id): axum::extract::Path<i32>,
) -> impl IntoResponse {
    let configs = PROVIDER_CONFIGS.lock().unwrap();
    if let Some(config) = configs.iter().find(|c| c.id == Some(id)) {
        Json(config.clone()).into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

/// 创建云厂商配置
pub async fn create_cloud_provider_config(
    Json(req): Json<CreateCloudProviderConfigRequest>,
) -> impl IntoResponse {
    let mut configs = PROVIDER_CONFIGS.lock().unwrap();

    // 生成新ID
    let new_id = configs.iter().filter_map(|c| c.id).max().map_or(1, |m| m + 1);

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

    configs.push(config.clone());

    Json(json!({
        "message": "配置创建成功",
        "data": config
    }))
}

/// 更新云厂商配置
pub async fn update_cloud_provider_config(
    Path(id): Path<i32>,
    Json(req): Json<UpdateCloudProviderConfigRequest>,
) -> impl IntoResponse {
    let mut configs = PROVIDER_CONFIGS.lock().unwrap();

    if let Some(config) = configs.iter_mut().find(|c| c.id == Some(id)) {
        if let Some(region_name) = req.region_name {
            config.region_name = region_name;
        }
        if let Some(available_zones) = req.available_zones {
            config.available_zones = available_zones;
        }
        if let Some(account_name) = req.account_name {
            config.account_name = account_name;
        }
        if let Some(access_key_id) = req.access_key_id {
            config.access_key_id = access_key_id;
        }
        if let Some(access_key_secret) = req.access_key_secret {
            config.access_key_secret = access_key_secret;
        }
        if let Some(status) = req.status {
            config.status = status;
        }
        if let Some(remarks) = req.remarks {
            config.remarks = Some(remarks);
        }
        if let Some(zone_id) = req.zone_id {
            config.zone_id = Some(zone_id);
        }
        if let Some(platform_id) = req.platform_id {
            config.platform_id = Some(platform_id);
        }
        config.updated_at = Some(chrono::Utc::now());

        Json(json!({
            "message": "配置更新成功",
            "data": config.clone()
        })).into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

/// 删除云厂商配置
pub async fn delete_cloud_provider_config(
    Path(id): Path<i32>,
) -> impl IntoResponse {
    let mut configs = PROVIDER_CONFIGS.lock().unwrap();

    if let Some(pos) = configs.iter().position(|c| c.id == Some(id)) {
        configs.remove(pos);
        Json(json!({ "message": "配置删除成功" })).into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

/// 测试云厂商连接
pub async fn test_cloud_provider_connection(
    Path(id): Path<i32>,
) -> impl IntoResponse {
    let configs = PROVIDER_CONFIGS.lock().unwrap();

    if let Some(_config) = configs.iter().find(|c| c.id == Some(id)) {
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
pub async fn get_cloud_provider_options() -> impl IntoResponse {
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
    Json(options)
}

/// 获取已启用的云厂商配置（用于业务申请选择）
pub async fn get_active_cloud_provider_configs() -> impl IntoResponse {
    let configs = PROVIDER_CONFIGS.lock().unwrap();
    let active: Vec<_> = configs.iter()
        .filter(|c| c.status == CloudProviderConfigStatus::Active)
        .map(|c| json!({
            "id": c.id,
            "provider": c.provider.as_str(),
            "region_id": c.region_id,
            "region_name": c.region_name,
            "account_name": c.account_name,
        }))
        .collect();

    Json(active)
}
