use crate::config::api_base;
use crate::utils::storage::authorization_header;
use gloo_net::http::{Request, RequestBuilder};
use serde::Deserialize;
use web_sys::RequestCredentials;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct BusinessResourceRecord {
    pub id: Option<i32>,
    pub resource_type: String,
    pub ecs_name: String,
    pub ecs_status: String,
    pub resource_id: String,
    pub cloud_region: String,
    pub cloud_category: String,
    pub cloud_provider_config_id: Option<i32>,
    pub zone_name: Option<String>,
    pub platform_name: Option<String>,
    pub county_city: Option<String>,
    pub vdc_name: Option<String>,
    pub customer_name: String,
    pub application_name: Option<String>,
    pub contract_name: Option<String>,
    pub instance_id: String,
    pub ecs_type: String,
    pub ecs_os: String,
    pub cpu_cores: u32,
    pub memory_gb: u32,
    pub system_disk: String,
    pub system_disk_size_gb: u32,
    pub data_disk: Option<String>,
    pub has_security_product: bool,
    pub ip_address: String,
    pub remarks: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub created_by: Option<String>,
    pub updated_by: Option<String>,
    pub application_status: Option<String>,
    pub delivery_status: Option<String>,
}

fn with_auth(mut request: RequestBuilder) -> RequestBuilder {
    if let Some(header) = authorization_header() {
        request = request.header("Authorization", &header);
    }
    request
}

pub async fn fetch_business_resources() -> Result<Vec<BusinessResourceRecord>, String> {
    let response = with_auth(
        Request::get(&format!("{}/business-resources", api_base()))
            .credentials(RequestCredentials::Include),
    )
    .send()
    .await
    .map_err(|e| format!("请求失败: {}", e))?;

    if !response.ok() {
        return Err(format!("服务器错误: {}", response.status()));
    }

    response
        .json::<Vec<BusinessResourceRecord>>()
        .await
        .map_err(|e| format!("解析失败: {}", e))
}
