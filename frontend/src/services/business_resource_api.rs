use crate::config::api_base;
use crate::utils::storage::authorization_header;
use gloo_net::http::{Request, RequestBuilder};
use serde::Deserialize;
use shared::{
    CreateApplicationEndpointRequest, CreateBusinessApplicationRequest,
    UpdateApplicationEndpointRequest, UpdateBusinessApplicationRequest,
};
use web_sys::RequestCredentials;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct ApplicationEndpointRecord {
    pub id: Option<i32>,
    pub business_application_id: i32,
    pub business_application_name: String,
    pub protocol: String,
    pub dest_ip: String,
    pub nat_ip: Option<String>,
    pub dest_port: String,
    pub domain: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub created_by: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct BusinessApplicationRecord {
    pub id: Option<i32>,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub created_by: Option<String>,
    pub endpoints: Vec<ApplicationEndpointRecord>,
}

fn with_auth(mut request: RequestBuilder) -> RequestBuilder {
    if let Some(header) = authorization_header() {
        request = request.header("Authorization", &header);
    }
    request
}

pub async fn fetch_business_applications() -> Result<Vec<BusinessApplicationRecord>, String> {
    let response = with_auth(
        Request::get(&format!("{}/business-applications", api_base()))
            .credentials(RequestCredentials::Include),
    )
    .send()
    .await
    .map_err(|e| format!("请求失败: {}", e))?;

    if !response.ok() {
        return Err(format!("服务器错误: {}", response.status()));
    }

    response
        .json::<Vec<BusinessApplicationRecord>>()
        .await
        .map_err(|e| format!("解析失败: {}", e))
}

pub async fn create_business_application(
    req: &CreateBusinessApplicationRequest,
) -> Result<BusinessApplicationRecord, String> {
    let response = with_auth(
        Request::post(&format!("{}/business-applications", api_base()))
            .credentials(RequestCredentials::Include),
    )
    .json(req)
    .map_err(|e| format!("请求构建失败: {}", e))?
    .send()
    .await
    .map_err(|e| format!("请求失败: {}", e))?;

    if !response.ok() {
        let message = response
            .text()
            .await
            .unwrap_or_else(|_| format!("服务器错误: {}", response.status()));
        return Err(message);
    }

    response
        .json::<BusinessApplicationRecord>()
        .await
        .map_err(|e| format!("解析失败: {}", e))
}

pub async fn create_application_endpoint(
    req: &CreateApplicationEndpointRequest,
) -> Result<ApplicationEndpointRecord, String> {
    let response = with_auth(
        Request::post(&format!("{}/business-applications/endpoints", api_base()))
            .credentials(RequestCredentials::Include),
    )
    .json(req)
    .map_err(|e| format!("请求构建失败: {}", e))?
    .send()
    .await
    .map_err(|e| format!("请求失败: {}", e))?;

    if !response.ok() {
        let message = response
            .text()
            .await
            .unwrap_or_else(|_| format!("服务器错误: {}", response.status()));
        return Err(message);
    }

    response
        .json::<ApplicationEndpointRecord>()
        .await
        .map_err(|e| format!("解析失败: {}", e))
}

pub async fn update_business_application(
    id: i32,
    req: &UpdateBusinessApplicationRequest,
) -> Result<BusinessApplicationRecord, String> {
    let response = with_auth(
        Request::put(&format!("{}/business-applications/{}", api_base(), id))
            .credentials(RequestCredentials::Include),
    )
    .json(req)
    .map_err(|e| format!("请求构建失败: {}", e))?
    .send()
    .await
    .map_err(|e| format!("请求失败: {}", e))?;

    if !response.ok() {
        let message = response
            .text()
            .await
            .unwrap_or_else(|_| format!("服务器错误: {}", response.status()));
        return Err(message);
    }

    response
        .json::<BusinessApplicationRecord>()
        .await
        .map_err(|e| format!("解析失败: {}", e))
}

pub async fn delete_business_application(id: i32) -> Result<(), String> {
    let response = with_auth(
        Request::delete(&format!("{}/business-applications/{}", api_base(), id))
            .credentials(RequestCredentials::Include),
    )
    .send()
    .await
    .map_err(|e| format!("请求失败: {}", e))?;

    if !response.ok() {
        let message = response
            .text()
            .await
            .unwrap_or_else(|_| format!("服务器错误: {}", response.status()));
        return Err(message);
    }

    Ok(())
}

pub async fn update_application_endpoint(
    id: i32,
    req: &UpdateApplicationEndpointRequest,
) -> Result<ApplicationEndpointRecord, String> {
    let response = with_auth(
        Request::put(&format!("{}/business-applications/endpoints/{}", api_base(), id))
            .credentials(RequestCredentials::Include),
    )
    .json(req)
    .map_err(|e| format!("请求构建失败: {}", e))?
    .send()
    .await
    .map_err(|e| format!("请求失败: {}", e))?;

    if !response.ok() {
        let message = response
            .text()
            .await
            .unwrap_or_else(|_| format!("服务器错误: {}", response.status()));
        return Err(message);
    }

    response
        .json::<ApplicationEndpointRecord>()
        .await
        .map_err(|e| format!("解析失败: {}", e))
}

pub async fn delete_application_endpoint(id: i32) -> Result<(), String> {
    let response = with_auth(
        Request::delete(&format!("{}/business-applications/endpoints/{}", api_base(), id))
            .credentials(RequestCredentials::Include),
    )
    .send()
    .await
    .map_err(|e| format!("请求失败: {}", e))?;

    if !response.ok() {
        let message = response
            .text()
            .await
            .unwrap_or_else(|_| format!("服务器错误: {}", response.status()));
        return Err(message);
    }

    Ok(())
}
