use crate::config::api_base;
use crate::utils::storage::authorization_header;
use gloo_net::http::{Request, RequestBuilder};
use serde::{Deserialize, Serialize};
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

#[derive(Clone, Debug, Serialize)]
pub struct RolePayload {
    pub name: String,
    pub description: Option<String>,
    pub permissions: Value,
}

fn with_auth(mut request: RequestBuilder) -> RequestBuilder {
    if let Some(header) = authorization_header() {
        request = request.header("Authorization", &header);
    }
    request
}

async fn read_error_message(response: gloo_net::http::Response) -> String {
    let status = response.status();
    let body = response.text().await.unwrap_or_default();

    serde_json::from_str::<Value>(&body)
        .ok()
        .and_then(|json| {
            json.get("message")
                .and_then(Value::as_str)
                .or_else(|| json.get("error").and_then(Value::as_str))
                .map(|msg| msg.to_string())
        })
        .filter(|msg| !msg.trim().is_empty())
        .unwrap_or_else(|| {
            if body.trim().is_empty() {
                format!("服务器错误: {}", status)
            } else {
                body
            }
        })
}

pub async fn fetch_roles() -> Result<Vec<RoleRecord>, String> {
    let response = with_auth(
        Request::get(&format!("{}/roles", api_base())).credentials(RequestCredentials::Include),
    )
    .send()
    .await
    .map_err(|e| format!("请求失败: {}", e))?;

    if !response.ok() {
        return Err(read_error_message(response).await);
    }

    response
        .json::<Vec<RoleRecord>>()
        .await
        .map_err(|e| format!("解析失败: {}", e))
}

pub async fn create_role(payload: &RolePayload) -> Result<(), String> {
    let response = with_auth(
        Request::post(&format!("{}/roles", api_base())).credentials(RequestCredentials::Include),
    )
    .json(payload)
    .map_err(|e| format!("构建请求失败: {}", e))?
    .send()
    .await
    .map_err(|e| format!("请求失败: {}", e))?;

    if !response.ok() {
        return Err(read_error_message(response).await);
    }

    Ok(())
}

pub async fn update_role(id: &str, payload: &RolePayload) -> Result<(), String> {
    let response = with_auth(
        Request::put(&format!("{}/roles/{}", api_base(), id))
            .credentials(RequestCredentials::Include),
    )
    .json(payload)
    .map_err(|e| format!("构建请求失败: {}", e))?
    .send()
    .await
    .map_err(|e| format!("请求失败: {}", e))?;

    if !response.ok() {
        return Err(read_error_message(response).await);
    }

    Ok(())
}

pub async fn delete_role(id: &str) -> Result<(), String> {
    let response = with_auth(
        Request::delete(&format!("{}/roles/{}", api_base(), id))
            .credentials(RequestCredentials::Include),
    )
    .send()
    .await
    .map_err(|e| format!("请求失败: {}", e))?;

    if !response.ok() {
        return Err(read_error_message(response).await);
    }

    Ok(())
}
