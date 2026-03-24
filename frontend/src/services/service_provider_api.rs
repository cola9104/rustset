use crate::config::api_base;
use crate::state::service_provider::ServiceProviderConfig;
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use web_sys::RequestCredentials;

/// 后端返回的服务商结构
#[derive(Clone, Debug, Serialize, Deserialize)]
struct BackendServiceProvider {
    id: i32,
    provider_name: String,
    provider_code: String,
    short_name: String,
    logo_url: Option<String>,
    contact_person: String,
    contact_phone: String,
    contact_email: String,
    headquarters: String,
    service_area: String,
    business_license: String,
    remarks: Option<String>,
    status: String,
    created_at: String,
    updated_at: Option<String>,
}

impl BackendServiceProvider {
    fn to_frontend(&self) -> ServiceProviderConfig {
        ServiceProviderConfig {
            id: self.id,
            provider_name: self.provider_name.clone(),
            provider_code: self.provider_code.clone(),
            short_name: self.short_name.clone(),
            logo_url: self.logo_url.clone(),
            contact_person: self.contact_person.clone(),
            contact_phone: self.contact_phone.clone(),
            contact_email: self.contact_email.clone(),
            headquarters: self.headquarters.clone(),
            service_area: self.service_area.clone(),
            business_license: self.business_license.clone(),
            remarks: self.remarks.clone(),
            status: self.status.clone(),
            created_at: self.created_at.clone(),
            updated_at: self.updated_at.clone(),
        }
    }
}

/// 获取服务商列表
pub async fn fetch_service_providers() -> Result<Vec<ServiceProviderConfig>, String> {
    let response = Request::get(&format!("{}/service-providers", api_base()))
        .credentials(RequestCredentials::Include)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    if !response.ok() {
        return Err(format!("服务器错误: {}", response.status()));
    }

    let backend_providers: Vec<BackendServiceProvider> = response
        .json()
        .await
        .map_err(|e| format!("解析失败: {}", e))?;

    Ok(backend_providers.iter().map(|p| p.to_frontend()).collect())
}

/// 创建服务商请求体
#[derive(Clone, Debug, Serialize)]
pub struct CreateServiceProviderRequest {
    pub provider_name: String,
    pub provider_code: String,
    pub short_name: String,
    pub logo_url: Option<String>,
    pub contact_person: String,
    pub contact_phone: String,
    pub contact_email: String,
    pub headquarters: String,
    pub service_area: String,
    pub business_license: String,
    pub remarks: Option<String>,
    pub status: Option<String>,
}

impl From<&ServiceProviderConfig> for CreateServiceProviderRequest {
    fn from(config: &ServiceProviderConfig) -> Self {
        Self {
            provider_name: config.provider_name.clone(),
            provider_code: config.provider_code.clone(),
            short_name: config.short_name.clone(),
            logo_url: config.logo_url.clone(),
            contact_person: config.contact_person.clone(),
            contact_phone: config.contact_phone.clone(),
            contact_email: config.contact_email.clone(),
            headquarters: config.headquarters.clone(),
            service_area: config.service_area.clone(),
            business_license: config.business_license.clone(),
            remarks: config.remarks.clone(),
            status: Some(config.status.clone()),
        }
    }
}

/// 创建服务商
pub async fn create_service_provider(
    config: &ServiceProviderConfig,
) -> Result<ServiceProviderConfig, String> {
    let request_body = CreateServiceProviderRequest::from(config);

    let response = Request::post(&format!("{}/service-providers", api_base()))
        .credentials(RequestCredentials::Include)
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
        let created: BackendServiceProvider =
            serde_json::from_value(data.clone()).map_err(|e| format!("解析响应失败: {}", e))?;
        Ok(created.to_frontend())
    } else {
        Err("响应格式错误".to_string())
    }
}

/// 更新服务商
pub async fn update_service_provider(
    id: i32,
    config: &ServiceProviderConfig,
) -> Result<ServiceProviderConfig, String> {
    let request_body = CreateServiceProviderRequest::from(config);

    let response = Request::put(&format!("{}/service-providers/{}", api_base(), id))
        .credentials(RequestCredentials::Include)
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
        let updated: BackendServiceProvider =
            serde_json::from_value(data.clone()).map_err(|e| format!("解析响应失败: {}", e))?;
        Ok(updated.to_frontend())
    } else {
        Err("响应格式错误".to_string())
    }
}

/// 删除服务商
pub async fn delete_service_provider(id: i32) -> Result<(), String> {
    let response = Request::delete(&format!("{}/service-providers/{}", api_base(), id))
        .credentials(RequestCredentials::Include)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    if !response.ok() {
        return Err(format!("服务器错误: {}", response.status()));
    }

    Ok(())
}
