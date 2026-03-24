use crate::config::dashboard_summary_url;
use crate::utils::storage::authorization_header;
use gloo_net::http::Request;
use serde::Deserialize;
use web_sys::RequestCredentials;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct DashboardSummary {
    pub asset_count: u64,
    pub scanned_asset_count: u64,
    pub task_count: u64,
    pub running_task_count: u64,
    pub risk_count: u64,
    pub open_risk_count: u64,
    pub user_count: u64,
    pub active_user_count: u64,
    pub database_connected: bool,
    pub version: String,
}

pub async fn fetch_dashboard_summary() -> Result<DashboardSummary, String> {
    let mut request =
        Request::get(&dashboard_summary_url()).credentials(RequestCredentials::Include);

    if let Some(header) = authorization_header() {
        request = request.header("Authorization", &header);
    }

    let response = request
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    if !response.ok() {
        return Err(format!("服务器错误: {}", response.status()));
    }

    response
        .json::<DashboardSummary>()
        .await
        .map_err(|e| format!("解析失败: {}", e))
}
