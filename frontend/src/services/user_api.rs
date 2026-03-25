use crate::app::AuthUser;
use crate::config::{current_user_url, users_url};
use crate::utils::storage::authorization_header;
use gloo_net::http::{Request, RequestBuilder};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use web_sys::RequestCredentials;

#[derive(Clone, Debug, Deserialize)]
struct BackendUser {
    id: String,
    username: String,
    real_name: Option<String>,
    role: Value,
    created_at: String,
    last_login_at: Option<String>,
    email: Option<String>,
    phone: Option<String>,
    status: Option<String>,
    organization_id: Option<i32>,
    department_id: Option<i32>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UserRecord {
    pub id: String,
    pub username: String,
    pub real_name: String,
    pub role_label: String,
    pub role_value: String,
    pub email: String,
    pub phone: String,
    pub status: String,
    pub organization_id: Option<i32>,
    pub department_id: Option<i32>,
    pub last_login: String,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct CreateUserPayload {
    pub username: String,
    pub real_name: Option<String>,
    pub password: String,
    pub role: Value,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub status: Option<String>,
    pub organization_id: Option<i32>,
    pub department_id: Option<i32>,
}

#[derive(Clone, Debug, Serialize)]
pub struct UpdateUserPayload {
    pub real_name: Option<String>,
    pub password: Option<String>,
    pub role: Option<Value>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub status: Option<String>,
    pub organization_id: Option<i32>,
    pub department_id: Option<i32>,
}

#[derive(Clone, Debug, Serialize)]
pub struct UpdateCurrentUserProfilePayload {
    pub real_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct CurrentUserProfileResponse {
    id: String,
    username: String,
    #[serde(default)]
    real_name: Option<String>,
    #[serde(default)]
    display_name: Option<String>,
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    phone: Option<String>,
    role: Value,
    #[serde(default)]
    permissions: Value,
    #[serde(default)]
    organization_id: Option<i32>,
    #[serde(default)]
    organization_name: Option<String>,
    #[serde(default)]
    department_id: Option<i32>,
    #[serde(default)]
    department_name: Option<String>,
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

fn parse_backend_role(value: &Value) -> String {
    match value {
        Value::String(raw) => raw.clone(),
        Value::Object(map) => map
            .get("Custom")
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or_else(|| "Custom".to_string()),
        _ => "Unknown".to_string(),
    }
}

fn permission_strings_from_value(value: &Value) -> Vec<String> {
    value
        .as_object()
        .map(|permissions| {
            let mut items = Vec::new();
            for (key, raw_value) in permissions {
                if raw_value.as_bool().is_some_and(|enabled| enabled) {
                    items.push(key.clone());
                } else if let Some(scope) = raw_value.as_str() {
                    items.push(format!("{key}:{scope}"));
                }
            }
            items
        })
        .unwrap_or_default()
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

impl From<BackendUser> for UserRecord {
    fn from(user: BackendUser) -> Self {
        let (role_label, role_value) = parse_role(&user.role);

        Self {
            id: user.id,
            username: user.username,
            real_name: user.real_name.unwrap_or_else(|| "-".to_string()),
            role_label,
            role_value,
            email: user.email.unwrap_or_else(|| "-".to_string()),
            phone: user.phone.unwrap_or_else(|| "-".to_string()),
            status: user.status.unwrap_or_else(|| "active".to_string()),
            organization_id: user.organization_id,
            department_id: user.department_id,
            last_login: user
                .last_login_at
                .as_deref()
                .map(format_timestamp)
                .unwrap_or_else(|| "从未登录".to_string()),
            created_at: format_timestamp(&user.created_at),
        }
    }
}

impl From<CurrentUserProfileResponse> for AuthUser {
    fn from(user: CurrentUserProfileResponse) -> Self {
        Self {
            id: user.id,
            username: user.username.clone(),
            real_name: user.real_name.clone().unwrap_or_default(),
            display_name: user
                .display_name
                .filter(|value| !value.trim().is_empty())
                .or_else(|| user.real_name.clone())
                .unwrap_or_else(|| user.username.clone()),
            email: user.email.unwrap_or_default(),
            phone: user.phone.unwrap_or_default(),
            role: parse_backend_role(&user.role),
            permissions: permission_strings_from_value(&user.permissions),
            organization_id: user.organization_id,
            organization_name: user.organization_name.unwrap_or_default(),
            department_id: user.department_id,
            department_name: user.department_name.unwrap_or_default(),
        }
    }
}

pub async fn fetch_users() -> Result<Vec<UserRecord>, String> {
    let response = with_auth(Request::get(&users_url()).credentials(RequestCredentials::Include))
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    if !response.ok() {
        return Err(read_error_message(response).await);
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
        return Err(read_error_message(response).await);
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
        return Err(read_error_message(response).await);
    }

    Ok(())
}

pub async fn update_user(id: &str, payload: &UpdateUserPayload) -> Result<UserRecord, String> {
    let response = with_auth(
        Request::put(&format!("{}/{}", users_url(), id)).credentials(RequestCredentials::Include),
    )
    .json(payload)
    .map_err(|e| format!("构建请求失败: {}", e))?
    .send()
    .await
    .map_err(|e| format!("请求失败: {}", e))?;

    if !response.ok() {
        return Err(read_error_message(response).await);
    }

    let user = response
        .json::<BackendUser>()
        .await
        .map_err(|e| format!("解析失败: {}", e))?;

    Ok(UserRecord::from(user))
}

pub async fn update_current_user_profile(
    payload: &UpdateCurrentUserProfilePayload,
) -> Result<AuthUser, String> {
    let response =
        with_auth(Request::put(&current_user_url()).credentials(RequestCredentials::Include))
            .json(payload)
            .map_err(|e| format!("构建请求失败: {}", e))?
            .send()
            .await
            .map_err(|e| format!("请求失败: {}", e))?;

    if !response.ok() {
        return Err(read_error_message(response).await);
    }

    let user = response
        .json::<CurrentUserProfileResponse>()
        .await
        .map_err(|e| format!("解析失败: {}", e))?;

    Ok(AuthUser::from(user))
}
