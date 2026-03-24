use crate::config::api_base;
use crate::utils::storage::authorization_header;
use gloo_net::http::{Request, RequestBuilder};
use serde::Deserialize;
use web_sys::RequestCredentials;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct PortRecord {
    pub port: u16,
    pub is_open: bool,
    pub service: Option<String>,
    pub version: Option<String>,
    pub banner: Option<String>,
    pub is_bound: bool,
    pub system_name: Option<String>,
    pub middleware: Option<String>,
    pub created_by: Option<String>,
    pub updated_by: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum NetworkZoneRecord {
    Named(String),
    Custom { Custom: String },
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct AssetRecord {
    pub id: Option<i32>,
    pub name: String,
    pub ip: String,
    pub zone: NetworkZoneRecord,
    pub ports: Vec<PortRecord>,
    pub last_scanned: Option<String>,
    pub contact_person: Option<String>,
    pub contact_phone: Option<String>,
    pub created_by: Option<String>,
    pub updated_by: Option<String>,
    pub owner: Option<String>,
    pub weight: i32,
    pub labels: Vec<String>,
    pub os: Option<String>,
    pub device_type: Option<String>,
}

fn with_auth(mut request: RequestBuilder) -> RequestBuilder {
    if let Some(header) = authorization_header() {
        request = request.header("Authorization", &header);
    }
    request
}

pub async fn fetch_assets() -> Result<Vec<AssetRecord>, String> {
    let response = with_auth(
        Request::get(&format!("{}/assets", api_base())).credentials(RequestCredentials::Include),
    )
    .send()
    .await
    .map_err(|e| format!("请求失败: {}", e))?;

    if !response.ok() {
        return Err(format!("服务器错误: {}", response.status()));
    }

    response
        .json::<Vec<AssetRecord>>()
        .await
        .map_err(|e| format!("解析失败: {}", e))
}
