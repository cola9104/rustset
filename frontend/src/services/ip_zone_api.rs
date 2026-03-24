use crate::config::ip_zones_url;
use gloo_net::http::Request;
use shared::ZoneConfig;
use web_sys::RequestCredentials;

/// 获取 IP Zone 列表
pub async fn fetch_ip_zones() -> Result<Vec<ZoneConfig>, String> {
    let response = Request::get(&ip_zones_url())
        .credentials(RequestCredentials::Include)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    if !response.ok() {
        return Err(format!("服务器错误: {}", response.status()));
    }

    response
        .json()
        .await
        .map_err(|e| format!("解析失败: {}", e))
}
