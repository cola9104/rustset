use crate::config::api_base;
use crate::state::department::DepartmentRecord;
use crate::utils::storage::authorization_header;
use gloo_net::http::{Request, RequestBuilder};
use serde::Serialize;
use web_sys::RequestCredentials;

#[derive(Clone, Debug, Serialize)]
pub struct DepartmentPayload {
    pub organization_id: i32,
    pub name: String,
    pub code: String,
    pub parent_id: Option<i32>,
    pub level: u32,
    pub status: String,
    pub remarks: Option<String>,
}

fn departments_url() -> String {
    format!("{}/departments", api_base())
}

fn with_auth(mut request: RequestBuilder) -> RequestBuilder {
    if let Some(header) = authorization_header() {
        request = request.header("Authorization", &header);
    }
    request
}

pub async fn fetch_departments() -> Result<Vec<DepartmentRecord>, String> {
    let response =
        with_auth(Request::get(&departments_url()).credentials(RequestCredentials::Include))
            .send()
            .await
            .map_err(|e| format!("请求失败: {}", e))?;

    if !response.ok() {
        return Err(format!("服务器错误: {}", response.status()));
    }

    response
        .json::<Vec<DepartmentRecord>>()
        .await
        .map_err(|e| format!("解析失败: {}", e))
}

pub async fn create_department(payload: &DepartmentPayload) -> Result<DepartmentRecord, String> {
    let response =
        with_auth(Request::post(&departments_url()).credentials(RequestCredentials::Include))
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

pub async fn update_department(
    id: i32,
    payload: &DepartmentPayload,
) -> Result<DepartmentRecord, String> {
    let response = with_auth(
        Request::put(&format!("{}/{}", departments_url(), id))
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

pub async fn delete_department(id: i32) -> Result<(), String> {
    let response = with_auth(
        Request::delete(&format!("{}/{}", departments_url(), id))
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
