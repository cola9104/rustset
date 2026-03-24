use crate::config::api_base;
use crate::utils::storage::authorization_header;
use gloo_net::http::{Request, RequestBuilder};
use serde::Deserialize;
use serde_json::Value;
use web_sys::RequestCredentials;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct RoleRecord {
    pub id: Value,
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub is_system: bool,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub permissions: Option<Value>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

fn with_auth(mut request: RequestBuilder) -> RequestBuilder {
    if let Some(header) = authorization_header() {
        request = request.header("Authorization", &header);
    }
    request
}

pub async fn fetch_roles() -> Result<Vec<RoleRecord>, String> {
    let response = with_auth(
        Request::get(&format!("{}/roles", api_base())).credentials(RequestCredentials::Include),
    )
    .send()
    .await
    .map_err(|e| format!("请求失败: {}", e))?;

    if !response.ok() {
        return Err(format!("服务器错误: {}", response.status()));
    }

    response
        .json::<Vec<RoleRecord>>()
        .await
        .map_err(|e| format!("解析失败: {}", e))
}
