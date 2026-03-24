use crate::config::api_base;
use crate::state::organization::OrganizationRecord;
use crate::utils::storage::authorization_header;
use gloo_net::http::{Request, RequestBuilder};
use serde::Serialize;
use web_sys::RequestCredentials;

#[derive(Clone, Debug, Serialize)]
pub struct OrganizationPayload {
    pub name: String,
    pub code: String,
    pub status: String,
    pub remarks: Option<String>,
}

fn organizations_url() -> String {
    format!("{}/organizations", api_base())
}

fn with_auth(mut request: RequestBuilder) -> RequestBuilder {
    if let Some(header) = authorization_header() {
        request = request.header("Authorization", &header);
    }
    request
}

pub async fn fetch_organizations() -> Result<Vec<OrganizationRecord>, String> {
    let response =
        with_auth(Request::get(&organizations_url()).credentials(RequestCredentials::Include))
            .send()
            .await
            .map_err(|e| format!("请求失败: {}", e))?;

    if !response.ok() {
        return Err(format!("服务器错误: {}", response.status()));
    }

    response
        .json::<Vec<OrganizationRecord>>()
        .await
        .map_err(|e| format!("解析失败: {}", e))
}

pub async fn create_organization(
    payload: &OrganizationPayload,
) -> Result<OrganizationRecord, String> {
    let response =
        with_auth(Request::post(&organizations_url()).credentials(RequestCredentials::Include))
            .json(payload)
            .map_err(|e| format!("构建请求失败: {}", e))?
            .send()
            .await
            .map_err(|e| format!("请求失败: {}", e))?;

    if !response.ok() {
        return Err(format!("服务器错误: {}", response.status()));
    }

    let value: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("解析失败: {}", e))?;
    serde_json::from_value(
        value
            .get("data")
            .cloned()
            .ok_or_else(|| "响应格式错误".to_string())?,
    )
    .map_err(|e| format!("解析响应失败: {}", e))
}

pub async fn update_organization(
    id: i32,
    payload: &OrganizationPayload,
) -> Result<OrganizationRecord, String> {
    let response = with_auth(
        Request::put(&format!("{}/{}", organizations_url(), id))
            .credentials(RequestCredentials::Include),
    )
    .json(payload)
    .map_err(|e| format!("构建请求失败: {}", e))?
    .send()
    .await
    .map_err(|e| format!("请求失败: {}", e))?;

    if !response.ok() {
        return Err(format!("服务器错误: {}", response.status()));
    }

    let value: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("解析失败: {}", e))?;
    serde_json::from_value(
        value
            .get("data")
            .cloned()
            .ok_or_else(|| "响应格式错误".to_string())?,
    )
    .map_err(|e| format!("解析响应失败: {}", e))
}

pub async fn delete_organization(id: i32) -> Result<(), String> {
    let response = with_auth(
        Request::delete(&format!("{}/{}", organizations_url(), id))
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
