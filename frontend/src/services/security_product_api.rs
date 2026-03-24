use crate::config::api_base;
use crate::state::security_product::{
    SecurityProduct, SecurityProductCategory, SecurityProductStatus,
};
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use web_sys::RequestCredentials;

/// 后端返回的安全产品结构
#[derive(Clone, Debug, Serialize, Deserialize)]
struct BackendSecurityProduct {
    id: i32,
    name: String,
    category: String,
    vendor: String,
    model: String,
    version: String,
    serial_number: Option<String>,
    license_type: String,
    license_expiry: Option<String>,
    management_ip: Option<String>,
    deployment_mode: String,
    cloud_platform_id: Option<i32>,
    machine_room_id: Option<i32>,
    provider_id: Option<i32>,
    status: String,
    features: Option<String>,
    throughput: Option<String>,
    contact_person: String,
    contact_phone: String,
    remarks: Option<String>,
    created_at: String,
}

impl BackendSecurityProduct {
    fn to_frontend(&self) -> SecurityProduct {
        let features: Vec<String> = self
            .features
            .as_ref()
            .and_then(|f| serde_json::from_str(f).ok())
            .unwrap_or_default();

        SecurityProduct {
            id: self.id,
            name: self.name.clone(),
            category: SecurityProductCategory::from_str(&self.category),
            vendor: self.vendor.clone(),
            model: self.model.clone(),
            version: self.version.clone(),
            serial_number: self.serial_number.clone(),
            license_type: self.license_type.clone(),
            license_expiry: self.license_expiry.clone(),
            management_ip: self.management_ip.clone(),
            deployment_mode: self.deployment_mode.clone(),
            cloud_platform_id: self.cloud_platform_id,
            machine_room_id: self.machine_room_id,
            provider_id: self.provider_id,
            status: parse_status(&self.status),
            features,
            throughput: self.throughput.clone(),
            contact_person: self.contact_person.clone(),
            contact_phone: self.contact_phone.clone(),
            remarks: self.remarks.clone(),
            created_at: self.created_at.clone(),
        }
    }
}

fn parse_status(s: &str) -> SecurityProductStatus {
    match s {
        "active" | "Active" => SecurityProductStatus::Active,
        "inactive" | "Inactive" => SecurityProductStatus::Inactive,
        "maintenance" | "Maintenance" => SecurityProductStatus::Maintenance,
        "decommissioned" | "Decommissioned" => SecurityProductStatus::Decommissioned,
        _ => SecurityProductStatus::Active,
    }
}

/// 获取安全产品列表
pub async fn fetch_security_products() -> Result<Vec<SecurityProduct>, String> {
    let response = Request::get(&format!("{}/security-products", api_base()))
        .credentials(RequestCredentials::Include)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    if !response.ok() {
        return Err(format!("服务器错误: {}", response.status()));
    }

    let backend_products: Vec<BackendSecurityProduct> = response
        .json()
        .await
        .map_err(|e| format!("解析失败: {}", e))?;

    Ok(backend_products.iter().map(|p| p.to_frontend()).collect())
}

/// 创建安全产品请求体
#[derive(Clone, Debug, Serialize)]
pub struct CreateSecurityProductRequest {
    pub name: String,
    pub category: String,
    pub vendor: String,
    pub model: String,
    pub version: String,
    pub serial_number: Option<String>,
    pub license_type: String,
    pub license_expiry: Option<String>,
    pub management_ip: Option<String>,
    pub deployment_mode: String,
    pub cloud_platform_id: Option<i32>,
    pub machine_room_id: Option<i32>,
    pub provider_id: Option<i32>,
    pub status: Option<String>,
    pub features: Option<Vec<String>>,
    pub throughput: Option<String>,
    pub contact_person: String,
    pub contact_phone: String,
    pub remarks: Option<String>,
}

impl From<&SecurityProduct> for CreateSecurityProductRequest {
    fn from(product: &SecurityProduct) -> Self {
        Self {
            name: product.name.clone(),
            category: product.category.display_name().to_string(),
            vendor: product.vendor.clone(),
            model: product.model.clone(),
            version: product.version.clone(),
            serial_number: product.serial_number.clone(),
            license_type: product.license_type.clone(),
            license_expiry: product.license_expiry.clone(),
            management_ip: product.management_ip.clone(),
            deployment_mode: product.deployment_mode.clone(),
            cloud_platform_id: product.cloud_platform_id,
            machine_room_id: product.machine_room_id,
            provider_id: product.provider_id,
            status: Some(product.status.display_name().to_string()),
            features: Some(product.features.clone()),
            throughput: product.throughput.clone(),
            contact_person: product.contact_person.clone(),
            contact_phone: product.contact_phone.clone(),
            remarks: product.remarks.clone(),
        }
    }
}

/// 创建安全产品
pub async fn create_security_product(product: &SecurityProduct) -> Result<SecurityProduct, String> {
    let request_body = CreateSecurityProductRequest::from(product);

    let response = Request::post(&format!("{}/security-products", api_base()))
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
        let created: BackendSecurityProduct =
            serde_json::from_value(data.clone()).map_err(|e| format!("解析响应失败: {}", e))?;
        Ok(created.to_frontend())
    } else {
        Err("响应格式错误".to_string())
    }
}

/// 更新安全产品
pub async fn update_security_product(
    id: i32,
    product: &SecurityProduct,
) -> Result<SecurityProduct, String> {
    let request_body = CreateSecurityProductRequest::from(product);

    let response = Request::put(&format!("{}/security-products/{}", api_base(), id))
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
        let updated: BackendSecurityProduct =
            serde_json::from_value(data.clone()).map_err(|e| format!("解析响应失败: {}", e))?;
        Ok(updated.to_frontend())
    } else {
        Err("响应格式错误".to_string())
    }
}

/// 删除安全产品
pub async fn delete_security_product(id: i32) -> Result<(), String> {
    let response = Request::delete(&format!("{}/security-products/{}", api_base(), id))
        .credentials(RequestCredentials::Include)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    if !response.ok() {
        return Err(format!("服务器错误: {}", response.status()));
    }

    Ok(())
}
