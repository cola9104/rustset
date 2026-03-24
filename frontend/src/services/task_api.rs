use crate::config::tasks_url;
use crate::utils::storage::authorization_header;
use gloo_net::http::{Request, RequestBuilder};
use serde::{Deserialize, Serialize};
use web_sys::RequestCredentials;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct TaskRecord {
    pub id: String,
    pub name: String,
    pub target: String,
    pub status: String,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub found_assets: usize,
    pub found_risks: usize,
    pub port_policy: String,
    pub domain_brute: bool,
    pub service_detection: bool,
    pub os_detection: bool,
    pub site_identify: bool,
    pub created_by: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CreateTaskPayload {
    pub name: String,
    pub target: String,
    pub port_policy: String,
    pub domain_brute: bool,
    pub service_detection: bool,
    pub os_detection: bool,
    pub site_identify: bool,
}

fn with_auth(mut request: RequestBuilder) -> RequestBuilder {
    if let Some(header) = authorization_header() {
        request = request.header("Authorization", &header);
    }
    request
}

pub async fn fetch_tasks() -> Result<Vec<TaskRecord>, String> {
    let response = with_auth(Request::get(&tasks_url()).credentials(RequestCredentials::Include))
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    if !response.ok() {
        return Err(format!("服务器错误: {}", response.status()));
    }

    response
        .json::<Vec<TaskRecord>>()
        .await
        .map_err(|e| format!("解析失败: {}", e))
}

pub async fn create_task(payload: &CreateTaskPayload) -> Result<TaskRecord, String> {
    let response = with_auth(Request::post(&tasks_url()).credentials(RequestCredentials::Include))
        .json(payload)
        .map_err(|e| format!("构建请求失败: {}", e))?
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    if !response.ok() {
        return Err(format!("服务器错误: {}", response.status()));
    }

    response
        .json::<TaskRecord>()
        .await
        .map_err(|e| format!("解析失败: {}", e))
}

pub async fn delete_task(id: &str) -> Result<(), String> {
    let response = with_auth(
        Request::delete(&format!("{}/{}", tasks_url(), id))
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
