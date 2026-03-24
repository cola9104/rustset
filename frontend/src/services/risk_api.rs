use crate::config::api_base;
use crate::utils::storage::authorization_header;
use gloo_net::http::{Request, RequestBuilder};
use serde::Deserialize;
use web_sys::RequestCredentials;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct RiskRecord {
    pub id: String,
    pub asset_ip: String,
    pub port: u16,
    pub severity: String,
    pub description: String,
    pub solution: Option<String>,
    pub status: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub assigned_to: Option<String>,
}

fn with_auth(mut request: RequestBuilder) -> RequestBuilder {
    if let Some(header) = authorization_header() {
        request = request.header("Authorization", &header);
    }
    request
}

pub async fn fetch_risks() -> Result<Vec<RiskRecord>, String> {
    let response = with_auth(
        Request::get(&format!("{}/risks", api_base())).credentials(RequestCredentials::Include),
    )
    .send()
    .await
    .map_err(|e| format!("请求失败: {}", e))?;

    if !response.ok() {
        return Err(format!("服务器错误: {}", response.status()));
    }

    response
        .json::<Vec<RiskRecord>>()
        .await
        .map_err(|e| format!("解析失败: {}", e))
}

pub async fn update_risk_status(id: &str, status: &str) -> Result<(), String> {
    let response = with_auth(
        Request::post(&format!("{}/risks/{}/status/{}", api_base(), id, status))
            .credentials(RequestCredentials::Include),
    )
    .send()
    .await
    .map_err(|e| format!("请求失败: {}", e))?;

    if !response.ok() {
        return Err(format!("服务器错误: {}", response.status()));
    }

    Ok(())
}
