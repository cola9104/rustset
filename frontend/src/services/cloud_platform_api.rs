use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use crate::state::cloud_platform::CloudPlatformConfig;
use crate::utils::storage::get_token;

const API_BASE: &str = "http://localhost:3003/api";

fn auth_header() -> Result<String, String> {
    get_token().ok_or_else(|| "未登录，请先登录".to_string())
}

/// 后端返回的云平台配置结构
#[derive(Clone, Debug, Serialize, Deserialize)]
struct BackendCloudPlatformConfig {
    id: i32,
    platform_name: String,
    provider_id: i32,
    cloud_type: String,
    foundation: String,
    region_id: String,
    machine_room_id: i32,
    access_key_id: String,
    access_key_secret: String,
    remarks: Option<String>,
    status: String,
    last_test_time: Option<String>,
    last_test_result: Option<String>,
    created_at: String,
    updated_at: Option<String>,
}

impl BackendCloudPlatformConfig {
    fn to_frontend(&self) -> CloudPlatformConfig {
        CloudPlatformConfig {
            id: self.id,
            platform_name: self.platform_name.clone(),
            provider_id: self.provider_id,
            cloud_type: self.cloud_type.clone(),
            foundation: self.foundation.clone(),
            region_id: self.region_id.clone(),
            machine_room_id: self.machine_room_id,
            access_key_id: self.access_key_id.clone(),
            access_key_secret: self.access_key_secret.clone(),
            remarks: self.remarks.clone(),
            status: self.status.clone(),
            last_test_time: self.last_test_time.clone(),
            last_test_result: self.last_test_result.clone(),
            created_at: self.created_at.clone(),
            updated_at: self.updated_at.clone(),
        }
    }
}

/// 获取云平台配置列表
pub async fn fetch_cloud_platform_configs() -> Result<Vec<CloudPlatformConfig>, String> {
    let token = auth_header()?;
    let response = Request::get(&format!("{}/cloud-platform-configs", API_BASE))
        .header("Authorization", &token)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    if !response.ok() {
        return Err(format!("服务器错误: {}", response.status()));
    }

    let backend_configs: Vec<BackendCloudPlatformConfig> = response
        .json()
        .await
        .map_err(|e| format!("解析失败: {}", e))?;

    Ok(backend_configs.iter().map(|c| c.to_frontend()).collect())
}

/// 创建云平台配置请求体
#[derive(Clone, Debug, Serialize)]
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

impl From<&CloudPlatformConfig> for CreateCloudPlatformConfigRequest {
    fn from(config: &CloudPlatformConfig) -> Self {
        Self {
            platform_name: config.platform_name.clone(),
            provider_id: config.provider_id,
            cloud_type: config.cloud_type.clone(),
            foundation: config.foundation.clone(),
            region_id: config.region_id.clone(),
            machine_room_id: config.machine_room_id,
            access_key_id: config.access_key_id.clone(),
            access_key_secret: config.access_key_secret.clone(),
            remarks: config.remarks.clone(),
            status: Some(config.status.clone()),
        }
    }
}

/// 创建云平台配置
pub async fn create_cloud_platform_config(config: &CloudPlatformConfig) -> Result<CloudPlatformConfig, String> {
    let token = auth_header()?;
    let request_body = CreateCloudPlatformConfigRequest::from(config);

    let response = Request::post(&format!("{}/cloud-platform-configs", API_BASE))
        .header("Authorization", &token)
        .json(&request_body)
        .map_err(|e| format!("构建请求失败: {}", e))?
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    if !response.ok() {
        return Err(format!("服务器错误: {}", response.status()));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("解析失败: {}", e))?;

    if let Some(data) = json.get("data") {
        let created: BackendCloudPlatformConfig = serde_json::from_value(data.clone())
            .map_err(|e| format!("解析响应失败: {}", e))?;
        Ok(created.to_frontend())
    } else {
        Err("响应格式错误".to_string())
    }
}

/// 更新云平台配置
pub async fn update_cloud_platform_config(id: i32, config: &CloudPlatformConfig) -> Result<CloudPlatformConfig, String> {
    let token = auth_header()?;
    let request_body = CreateCloudPlatformConfigRequest::from(config);

    let response = Request::put(&format!("{}/cloud-platform-configs/{}", API_BASE, id))
        .header("Authorization", &token)
        .json(&request_body)
        .map_err(|e| format!("构建请求失败: {}", e))?
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    if !response.ok() {
        return Err(format!("服务器错误: {}", response.status()));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("解析失败: {}", e))?;

    if let Some(data) = json.get("data") {
        let updated: BackendCloudPlatformConfig = serde_json::from_value(data.clone())
            .map_err(|e| format!("解析响应失败: {}", e))?;
        Ok(updated.to_frontend())
    } else {
        Err("响应格式错误".to_string())
    }
}

/// 删除云平台配置
pub async fn delete_cloud_platform_config(id: i32) -> Result<(), String> {
    let token = auth_header()?;
    let response = Request::delete(&format!("{}/cloud-platform-configs/{}", API_BASE, id))
        .header("Authorization", &token)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    if !response.ok() {
        return Err(format!("服务器错误: {}", response.status()));
    }

    Ok(())
}
