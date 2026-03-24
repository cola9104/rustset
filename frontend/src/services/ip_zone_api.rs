use crate::config::ip_zones_url;
use crate::utils::storage::authorization_header;
use gloo_net::http::Request;
use serde::Serialize;
use shared::ZoneConfig;
use web_sys::RequestCredentials;

/// 获取 IP Zone 列表
pub async fn fetch_ip_zones() -> Result<Vec<ZoneConfig>, String> {
    let mut request = Request::get(&ip_zones_url()).credentials(RequestCredentials::Include);
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
        .json()
        .await
        .map_err(|e| format!("解析失败: {}", e))
}

#[derive(Clone, Debug, Serialize)]
pub struct CreateIpZoneRequest {
    pub name: String,
    pub cidr: String,
    pub priority: i32,
    pub cloud_platform_id: Option<i32>,
    pub machine_room_id: Option<i32>,
}

pub async fn create_ip_zone(req: &CreateIpZoneRequest) -> Result<ZoneConfig, String> {
    let mut request = Request::post(&ip_zones_url()).credentials(RequestCredentials::Include);
    if let Some(header) = authorization_header() {
        request = request.header("Authorization", &header);
    }

    let response = request
        .json(req)
        .map_err(|e| format!("构建请求失败: {}", e))?
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    if !response.ok() {
        let status = response.status();
        let message = response
            .json::<serde_json::Value>()
            .await
            .ok()
            .and_then(|json| {
                json.get("message")
                    .and_then(|v| v.as_str())
                    .or_else(|| json.get("error").and_then(|v| v.as_str()))
                    .map(ToOwned::to_owned)
            })
            .unwrap_or_else(|| format!("服务器错误: {}", status));
        return Err(message);
    }

    response
        .json()
        .await
        .map_err(|e| format!("解析失败: {}", e))
}

pub async fn delete_ip_zone(id: &str) -> Result<(), String> {
    let mut request = Request::delete(&format!("{}/{}", ip_zones_url(), id))
        .credentials(RequestCredentials::Include);
    if let Some(header) = authorization_header() {
        request = request.header("Authorization", &header);
    }

    let response = request
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    if !response.ok() {
        let status = response.status();
        let message = response
            .json::<serde_json::Value>()
            .await
            .ok()
            .and_then(|json| {
                json.get("message")
                    .and_then(|v| v.as_str())
                    .or_else(|| json.get("error").and_then(|v| v.as_str()))
                    .map(ToOwned::to_owned)
            })
            .unwrap_or_else(|| format!("服务器错误: {}", status));
        return Err(message);
    }

    Ok(())
}
