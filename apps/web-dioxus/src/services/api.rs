use gloo_net::http::Method;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

fn base_url() -> String {
    // Backend API is always on port 8080 from the same origin
    let Some(window) = web_sys::window() else {
        return "http://127.0.0.1:8080".into();
    };
    let location = window.location();
    let protocol = location.protocol().unwrap_or_else(|_| "http:".into());
    let hostname = location.hostname().unwrap_or_else(|_| "127.0.0.1".into());
    // Always use port 8080 for the backend API, regardless of frontend port
    format!("{protocol}//{hostname}:8080")
}

// ── Core types ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub code: i32,
    pub msg: String,
    pub data: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageData<T> {
    pub list: Vec<T>,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tenant {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    #[serde(alias = "accessToken")]
    pub access_token: String,
    #[serde(alias = "refreshToken")]
    pub refresh_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentUser {
    #[serde(alias = "userId")]
    pub user_id: String,
    pub username: String,
    #[serde(alias = "tenantId")]
    pub tenant_id: Option<String>,
    #[serde(alias = "roleCodes")]
    pub role_codes: Vec<String>,
    pub permissions: Vec<String>,
}

// ── Token management ──

pub fn get_token() -> Option<String> {
    web_sys::window()?
        .local_storage()
        .ok()??
        .get_item("token")
        .ok()?
}

pub fn set_token(token: &str) {
    if let Some(s) = web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
    {
        let _ = s.set_item("token", token);
    }
}

pub fn clear_token() {
    if let Some(s) = web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
    {
        let _ = s.remove_item("token");
    }
}

// ── Request builder ──

fn build_request(method: Method, path: &str, body: Option<String>) -> gloo_net::http::Request {
    let url = format!("{}{}", base_url(), path);
    let mut request = gloo_net::http::RequestBuilder::new(&url)
        .method(method)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json");
    if let Some(token) = get_token() {
        request = request.header("Authorization", &format!("Bearer {}", token));
    }
    match body {
        Some(body) => request.body(body),
        None => request.build(),
    }
    .expect("valid request")
}

async fn do_request(method: Method, path: &str, body: Option<Value>) -> Result<Value, String> {
    let body_str = body.map(|b| serde_json::to_string(&b).unwrap_or_default());
    let resp = build_request(method, path, body_str)
        .send()
        .await
        .map_err(|e| format!("网络错误: {}", e))?;
    let status = resp.status();
    let text = resp
        .text()
        .await
        .map_err(|e| format!("读取响应失败: {}", e))?;
    if status == 401 {
        clear_token();
        redirect_to_login();
        return Err("会话已过期，请重新登录".into());
    }
    if status >= 400 {
        let msg = serde_json::from_str::<Value>(&text)
            .ok()
            .and_then(|v| v["msg"].as_str().map(String::from))
            .unwrap_or_else(|| format!("HTTP {}", status));
        return Err(msg);
    }
    serde_json::from_str(&text).map_err(|e| format!("解析失败: {}", e))
}

fn redirect_to_login() {
    if let Some(window) = web_sys::window() {
        let location = window.location();
        let origin = location.origin().unwrap_or_default();
        let _ = location.set_href(&format!("{}/auth/login", origin));
    }
}

pub async fn get_path(path: &str) -> Result<Value, String> { do_request(Method::GET, path, None).await }
fn get(path: &str) -> impl std::future::Future<Output = Result<Value, String>> {
    do_request(Method::GET, path, None)
}
fn post(path: &str, body: Value) -> impl std::future::Future<Output = Result<Value, String>> {
    do_request(Method::POST, path, Some(body))
}
fn put(path: &str, body: Value) -> impl std::future::Future<Output = Result<Value, String>> {
    do_request(Method::PUT, path, Some(body))
}
fn del(path: &str) -> impl std::future::Future<Output = Result<Value, String>> {
    do_request(Method::DELETE, path, None)
}

// ── Auth API ──

pub async fn login(req: &LoginRequest) -> Result<TokenResponse, String> {
    let v = post("/system/auth/login", serde_json::to_value(req).unwrap()).await?;
    let token: TokenResponse =
        serde_json::from_value(v["data"].clone()).map_err(|e| format!("解析: {}", e))?;
    set_token(&token.access_token);
    Ok(token)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MenuItem {
    pub id: i64,
    #[serde(default, alias = "parentId")]
    pub parent_id: i64,
    pub name: String,
    #[serde(default, deserialize_with = "null_to_empty")]
    pub path: String,
    #[serde(default, deserialize_with = "null_to_empty")]
    pub component: String,
    #[serde(default, alias = "componentName", deserialize_with = "null_to_empty")]
    pub component_name: String,
    #[serde(default, deserialize_with = "null_to_empty")]
    pub icon: String,
    #[serde(default = "default_true")]
    pub visible: bool,
    #[serde(default, alias = "keepAlive")]
    pub keep_alive: bool,
    #[serde(default)]
    pub children: Vec<MenuItem>,
}
fn default_true() -> bool { true }
fn null_to_empty<'de, D: serde::Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    use serde::Deserialize;
    Option::<String>::deserialize(d).map(|o| o.unwrap_or_default())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: Option<i64>,
    pub nickname: String,
    #[serde(default)]
    pub avatar: String,
    #[serde(default, alias = "deptId")]
    pub dept_id: Option<i64>,
    pub username: String,
    #[serde(default)]
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionInfo {
    pub user: UserInfo,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub menus: Vec<MenuItem>,
}

pub async fn get_permission_info() -> Result<PermissionInfo, String> {
    let v = get("/system/auth/get-permission-info").await?;
    serde_json::from_value(v["data"].clone()).map_err(|e| format!("解析权限信息失败: {e}"))
}

pub async fn tenant_list() -> Result<Vec<Tenant>, String> {
    let value = get("/system/tenant/simple-list").await?;
    serde_json::from_value(value["data"].clone())
        .map_err(|error| format!("解析公司列表失败: {error}"))
}

pub async fn tenant_by_website(website: &str) -> Result<Option<Tenant>, String> {
    let value = get(&format!("/system/tenant/get-by-website?website={website}")).await?;
    let tenant = value["data"].clone();
    if tenant.get("id").is_none() {
        return Ok(None);
    }
    serde_json::from_value(tenant)
        .map(Some)
        .map_err(|error| format!("解析公司信息失败: {error}"))
}

pub async fn get_current_user() -> Result<CurrentUser, String> {
    let v = get("/system/auth/me").await?;
    serde_json::from_value(v["data"].clone()).map_err(|e| format!("解析: {}", e))
}

// ── Generic CRUD (infra prefix) ──

pub async fn page(module: &str, page_no: i64, page_size: i64) -> Result<(Vec<Value>, i64), String> {
    crud_page("infra", module, page_no, page_size).await
}

pub async fn list(module: &str) -> Result<Vec<Value>, String> {
    let v = get(&format!("/infra/{}/list", module)).await?;
    Ok(v["data"].as_array().cloned().unwrap_or_default())
}

pub async fn get_one(module: &str, id: &str) -> Result<Value, String> {
    let v = get(&format!("/infra/{}/get?id={}", module, id)).await?;
    Ok(v["data"].clone())
}

pub async fn create(module: &str, data: Value) -> Result<String, String> {
    let v = post(&format!("/infra/{}/create", module), data).await?;
    Ok(v["data"].as_str().unwrap_or("ok").to_string())
}

pub async fn update(module: &str, data: Value) -> Result<(), String> {
    put(&format!("/infra/{}/update", module), data).await?;
    Ok(())
}

pub async fn remove(module: &str, id: &str) -> Result<(), String> {
    del(&format!("/infra/{}/delete?id={}", module, id)).await?;
    Ok(())
}

// ── Generic CRUD (system prefix) ──

pub async fn sys_page(module: &str, page_no: i64, page_size: i64) -> Result<(Vec<Value>, i64), String> {
    crud_page("system", module, page_no, page_size).await
}

pub async fn sys_list(module: &str) -> Result<Vec<Value>, String> {
    let v = get(&format!("/system/{}/list", module)).await?;
    Ok(v["data"].as_array().cloned().unwrap_or_default())
}

pub async fn sys_get_one(module: &str, id: &str) -> Result<Value, String> {
    let v = get(&format!("/system/{}/get?id={}", module, id)).await?;
    Ok(v["data"].clone())
}

pub async fn sys_create(module: &str, data: Value) -> Result<String, String> {
    let v = post(&format!("/system/{}/create", module), data).await?;
    Ok(v["data"].as_str().unwrap_or("ok").to_string())
}

pub async fn sys_update(module: &str, data: Value) -> Result<(), String> {
    put(&format!("/system/{}/update", module), data).await?;
    Ok(())
}

pub async fn sys_remove(module: &str, id: &str) -> Result<(), String> {
    del(&format!("/system/{}/delete?id={}", module, id)).await?;
    Ok(())
}

// ── Shared CRUD helpers ──

async fn crud_page(prefix: &str, module: &str, page_no: i64, page_size: i64) -> Result<(Vec<Value>, i64), String> {
    let v = get(&format!(
        "/{}/{}/page?pageNo={}&pageSize={}",
        prefix, module, page_no, page_size
    ))
    .await?;
    let list = v["data"]["list"].as_array().cloned().unwrap_or_default();
    let total = v["data"]["total"].as_i64().unwrap_or(0);
    Ok((list, total))
}

pub async fn action(path: &str, data: Value) -> Result<Value, String> {
    post(path, data).await
}

// ── Custom API helpers ──

pub async fn list_by(module: &str, param: &str, value: &str) -> Result<Vec<Value>, String> {
    let v = get(&format!(
        "/infra/{}/list-by-{}?{}= {} ",
        module, param, param, value
    ))
    .await?;
    Ok(v["data"].as_array().cloned().unwrap_or_default())
}

pub async fn sys_list_by(module: &str, param: &str, value: &str) -> Result<Vec<Value>, String> {
    let v = get(&format!(
        "/system/{}/list-by-{}?{}= {} ",
        module, param, param, value
    ))
    .await?;
    Ok(v["data"].as_array().cloned().unwrap_or_default())
}

pub async fn get_page(path: &str, params: &HashMap<String, String>) -> Result<Value, String> {
    let qs: String = params
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("&");
    get(&format!("{}?{}", path, qs)).await
}
