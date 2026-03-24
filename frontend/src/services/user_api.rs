use crate::config::users_url;
use crate::utils::storage::authorization_header;
use gloo_net::http::{Request, RequestBuilder};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use web_sys::RequestCredentials;

#[derive(Clone, Debug, Deserialize)]
struct BackendUser {
    id: String,
    username: String,
    role: Value,
    created_at: String,
    last_login_at: Option<String>,
    email: Option<String>,
    status: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UserRecord {
    pub id: String,
    pub username: String,
    pub role_label: String,
    pub role_value: String,
    pub email: String,
    pub status: String,
    pub last_login: String,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct CreateUserPayload {
    pub username: String,
    pub password: String,
    pub role: String,
}

fn with_auth(mut request: RequestBuilder) -> RequestBuilder {
    if let Some(header) = authorization_header() {
        request = request.header("Authorization", &header);
    }
    request
}

fn format_timestamp(value: &str) -> String {
    value
        .split('.')
        .next()
        .unwrap_or(value)
        .replace('T', " ")
        .replace("+00:00", " UTC")
}

fn parse_role(value: &Value) -> (String, String) {
    match value {
        Value::String(raw) => match raw.as_str() {
            "SysAdmin" => ("系统管理员".to_string(), "SysAdmin".to_string()),
            "SecAdmin" => ("安全管理员".to_string(), "SecAdmin".to_string()),
            "Auditor" => ("审计员".to_string(), "Auditor".to_string()),
            other => (other.to_string(), other.to_string()),
        },
        Value::Object(map) => map
            .get("Custom")
            .and_then(Value::as_str)
            .map(|name| (format!("自定义角色: {}", name), name.to_string()))
            .unwrap_or_else(|| ("未知角色".to_string(), "Unknown".to_string())),
        _ => ("未知角色".to_string(), "Unknown".to_string()),
    }
}

impl From<BackendUser> for UserRecord {
    fn from(user: BackendUser) -> Self {
        let (role_label, role_value) = parse_role(&user.role);

        Self {
            id: user.id,
            username: user.username,
            role_label,
            role_value,
            email: user.email.unwrap_or_else(|| "-".to_string()),
            status: user.status.unwrap_or_else(|| "active".to_string()),
            last_login: user
                .last_login_at
                .as_deref()
                .map(format_timestamp)
                .unwrap_or_else(|| "从未登录".to_string()),
            created_at: format_timestamp(&user.created_at),
        }
    }
}

pub async fn fetch_users() -> Result<Vec<UserRecord>, String> {
    let response = with_auth(Request::get(&users_url()).credentials(RequestCredentials::Include))
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    if !response.ok() {
        return Err(format!("服务器错误: {}", response.status()));
    }

    let users = response
        .json::<Vec<BackendUser>>()
        .await
        .map_err(|e| format!("解析失败: {}", e))?;

    Ok(users.into_iter().map(UserRecord::from).collect())
}

pub async fn create_user(payload: &CreateUserPayload) -> Result<UserRecord, String> {
    let response = with_auth(Request::post(&users_url()).credentials(RequestCredentials::Include))
        .json(payload)
        .map_err(|e| format!("构建请求失败: {}", e))?
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    if !response.ok() {
        return Err(format!("服务器错误: {}", response.status()));
    }

    let user = response
        .json::<BackendUser>()
        .await
        .map_err(|e| format!("解析失败: {}", e))?;

    Ok(UserRecord::from(user))
}

pub async fn delete_user(id: &str) -> Result<(), String> {
    let response = with_auth(
        Request::delete(&format!("{}/{}", users_url(), id))
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
