use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::{FaLock, FaUser};
use dioxus_free_icons::Icon;
use dioxus_router::navigator;
use serde::{Deserialize, Serialize};
use web_sys::RequestCredentials;

use crate::app::AuthUser;
use crate::app::{AUTH_READY, AUTH_STATE};
use crate::config::login_url;
use crate::router::Route;
use crate::state::user_role::{use_auth, AuthState as WorkflowAuthState};
use crate::utils::storage::set_token;

/// 登录请求
#[derive(Debug, Serialize)]
struct LoginRequest {
    username: String,
    password: String,
}

/// 登录响应结构 (对应后端 LoginResponse)
#[derive(Debug, Deserialize)]
struct LoginResponse {
    token: String,
    user: LoginUser,
}

/// 后端用户结构 (对应后端 User)
#[derive(Debug, Deserialize)]
struct LoginUser {
    id: String,
    username: String,
    role: LoginRole,
    #[serde(default)]
    permissions: Option<LoginPermissions>,
}

/// 后端角色枚举 (对应后端 Role)
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
enum LoginRole {
    #[serde(rename = "SysAdmin", alias = "sysadmin")]
    SysAdmin,
    #[serde(rename = "SecAdmin", alias = "secadmin")]
    SecAdmin,
    #[serde(rename = "Auditor", alias = "auditor")]
    Auditor,
    #[serde(rename = "Operator", alias = "operator")]
    Operator,
    #[serde(rename = "Custom", alias = "custom")]
    Custom,
}

/// 后端权限结构 (对应后端 Permissions)
#[derive(Debug, Clone, Deserialize, Default)]
struct LoginPermissions {
    #[serde(default)]
    can_access_general: bool,
    #[serde(default)]
    can_view_dashboard: bool,
    #[serde(default)]
    can_view_tasks: bool,
    #[serde(default)]
    can_create_task: bool,
    #[serde(default)]
    can_delete_task: bool,
    #[serde(default)]
    can_update_task: bool,
    #[serde(default)]
    can_view_advanced_scan: bool,
    #[serde(default)]
    can_create_scan: bool,
    #[serde(default)]
    can_delete_scan: bool,
    #[serde(default)]
    can_export_scan: bool,
    #[serde(default)]
    can_access_assets_risks: bool,
    #[serde(default)]
    can_view_cloud_assets: bool,
    #[serde(default)]
    can_create_cloud_asset: bool,
    #[serde(default)]
    can_update_cloud_asset: bool,
    #[serde(default)]
    can_delete_cloud_asset: bool,
    #[serde(default)]
    can_view_users: bool,
    #[serde(default)]
    can_create_user: bool,
    #[serde(default)]
    can_update_user: bool,
    #[serde(default)]
    can_delete_user: bool,
    #[serde(default)]
    can_view_audit_logs: bool,
    #[serde(default)]
    can_view_roles: bool,
    #[serde(default)]
    can_create_role: bool,
    #[serde(default)]
    can_update_role: bool,
    #[serde(default)]
    can_delete_role: bool,
    #[serde(default)]
    can_view_service_providers: bool,
    #[serde(default)]
    can_view_machine_rooms: bool,
    #[serde(default)]
    can_view_cloud_platforms: bool,
    #[serde(default)]
    can_view_security_products: bool,
    #[serde(default)]
    can_view_network_zones: bool,
    #[serde(default)]
    can_view_resource_tickets: bool,
    #[serde(default)]
    can_create_resource_tickets: bool,
    #[serde(default)]
    can_approve_resource_tickets: bool,
    #[serde(default)]
    can_provision_resource_tickets: bool,
    #[serde(default)]
    can_deliver_resource_tickets: bool,
    #[serde(default)]
    can_delete_resource_tickets: bool,
    #[serde(default)]
    resource_ticket_scope: String,
    #[serde(default)]
    can_view_ip_zones: bool,
    #[serde(default)]
    can_view_scanners: bool,
    #[serde(default)]
    can_view_port_details: bool,
}

impl LoginUser {
    /// 将后端用户转换为前端 AuthUser
    fn to_auth_user(&self) -> AuthUser {
        let permissions = self
            .permissions
            .as_ref()
            .map(|p| p.to_permission_strings())
            .unwrap_or_default();

        AuthUser {
            id: self.id.clone(),
            username: self.username.clone(),
            real_name: String::new(),
            display_name: self.username.clone(),
            role: self.role.to_string(),
            permissions,
            organization_id: None,
            organization_name: String::new(),
            department_id: None,
            department_name: String::new(),
        }
    }
}

impl LoginRole {}

impl std::fmt::Display for LoginRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            LoginRole::SysAdmin => "SysAdmin",
            LoginRole::SecAdmin => "SecAdmin",
            LoginRole::Auditor => "Auditor",
            LoginRole::Operator => "Operator",
            LoginRole::Custom => "Custom",
        };
        f.write_str(value)
    }
}

impl LoginPermissions {
    /// 将权限转换为字符串列表
    fn to_permission_strings(&self) -> Vec<String> {
        let mut perms = Vec::new();
        if self.can_access_general {
            perms.push("can_access_general".to_string());
        }
        if self.can_view_dashboard {
            perms.push("can_view_dashboard".to_string());
        }
        if self.can_view_tasks {
            perms.push("can_view_tasks".to_string());
        }
        if self.can_create_task {
            perms.push("can_create_task".to_string());
        }
        if self.can_delete_task {
            perms.push("can_delete_task".to_string());
        }
        if self.can_update_task {
            perms.push("can_update_task".to_string());
        }
        if self.can_view_advanced_scan {
            perms.push("can_view_advanced_scan".to_string());
        }
        if self.can_create_scan {
            perms.push("can_create_scan".to_string());
        }
        if self.can_delete_scan {
            perms.push("can_delete_scan".to_string());
        }
        if self.can_export_scan {
            perms.push("can_export_scan".to_string());
        }
        if self.can_access_assets_risks {
            perms.push("can_access_assets_risks".to_string());
        }
        if self.can_view_cloud_assets {
            perms.push("can_view_cloud_assets".to_string());
        }
        if self.can_create_cloud_asset {
            perms.push("can_create_cloud_asset".to_string());
        }
        if self.can_update_cloud_asset {
            perms.push("can_update_cloud_asset".to_string());
        }
        if self.can_delete_cloud_asset {
            perms.push("can_delete_cloud_asset".to_string());
        }
        if self.can_view_audit_logs {
            perms.push("can_view_audit_logs".to_string());
        }
        if self.can_view_users {
            perms.push("can_view_users".to_string());
        }
        if self.can_create_user {
            perms.push("can_create_user".to_string());
        }
        if self.can_update_user {
            perms.push("can_update_user".to_string());
        }
        if self.can_delete_user {
            perms.push("can_delete_user".to_string());
        }
        if self.can_view_roles {
            perms.push("can_view_roles".to_string());
        }
        if self.can_create_role {
            perms.push("can_create_role".to_string());
        }
        if self.can_update_role {
            perms.push("can_update_role".to_string());
        }
        if self.can_delete_role {
            perms.push("can_delete_role".to_string());
        }
        if self.can_view_service_providers {
            perms.push("can_view_service_providers".to_string());
        }
        if self.can_view_machine_rooms {
            perms.push("can_view_machine_rooms".to_string());
        }
        if self.can_view_cloud_platforms {
            perms.push("can_view_cloud_platforms".to_string());
        }
        if self.can_view_security_products {
            perms.push("can_view_security_products".to_string());
        }
        if self.can_view_network_zones {
            perms.push("can_view_network_zones".to_string());
        }
        if self.can_view_resource_tickets {
            perms.push("can_view_resource_tickets".to_string());
        }
        if self.can_create_resource_tickets {
            perms.push("can_create_resource_tickets".to_string());
        }
        if self.can_approve_resource_tickets {
            perms.push("can_approve_resource_tickets".to_string());
        }
        if self.can_provision_resource_tickets {
            perms.push("can_provision_resource_tickets".to_string());
        }
        if self.can_deliver_resource_tickets {
            perms.push("can_deliver_resource_tickets".to_string());
        }
        if self.can_delete_resource_tickets {
            perms.push("can_delete_resource_tickets".to_string());
        }
        if !self.resource_ticket_scope.trim().is_empty() {
            perms.push(format!(
                "resource_ticket_scope:{}",
                self.resource_ticket_scope
            ));
        }
        if self.can_view_ip_zones {
            perms.push("can_view_ip_zones".to_string());
        }
        if self.can_view_scanners {
            perms.push("can_view_scanners".to_string());
        }
        if self.can_view_port_details {
            perms.push("can_view_port_details".to_string());
        }
        perms
    }
}

/// 登录页面
#[allow(non_snake_case)]
pub fn Login() -> Element {
    let mut username = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut error = use_signal(String::new);
    let mut loading = use_signal(|| false);
    let mut auth_state = use_auth();
    let nav = navigator();

    let handle_login = move |_| {
        let user = username.read().clone();
        let pass = password.read().clone();

        spawn(async move {
            loading.set(true);
            error.set(String::new());

            // 验证输入
            if user.is_empty() || pass.is_empty() {
                error.set("用户名和密码不能为空".to_string());
                loading.set(false);
                return;
            }

            // 调用后端登录 API
            let request_body = LoginRequest {
                username: user.clone(),
                password: pass.clone(),
            };

            match gloo_net::http::Request::post(&login_url())
                .credentials(RequestCredentials::Include)
                .header("Content-Type", "application/json")
                .body(serde_json::to_string(&request_body).unwrap_or_default())
            {
                Ok(request) => {
                    match request.send().await {
                        Ok(response) => {
                            // 检查响应状态码
                            if response.status() == 200 {
                                match response.json::<LoginResponse>().await {
                                    Ok(login_response) => {
                                        // 存储 token 到 localStorage
                                        set_token(&login_response.token);

                                        // 转换用户数据并更新认证状态
                                        let auth_user = login_response.user.to_auth_user();
                                        auth_state.set(WorkflowAuthState::from(&auth_user));
                                        *AUTH_STATE.write() = Some(auth_user);
                                        *AUTH_READY.write() = true;

                                        // 跳转到仪表板
                                        nav.push(Route::Dashboard {});
                                    }
                                    Err(e) => {
                                        error.set(format!("登录响应解析失败: {}", e));
                                    }
                                }
                            } else {
                                // 处理非 200 响应
                                let status = response.status();
                                match response.json::<serde_json::Value>().await {
                                    Ok(error_json) => {
                                        if let Some(error_msg) =
                                            error_json.get("error").and_then(|v| v.as_str())
                                        {
                                            error.set(error_msg.to_string());
                                        } else {
                                            error.set(format!("登录失败 (HTTP {})", status));
                                        }
                                    }
                                    Err(_) => {
                                        error.set(format!("登录失败 (HTTP {})", status));
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error.set(format!("网络请求失败: {}", e));
                        }
                    }
                }
                Err(e) => {
                    error.set(format!("创建请求失败: {}", e));
                }
            }

            loading.set(false);
        });
    };

    rsx! {
        div { class: "min-h-screen flex items-center justify-center bg-gradient-to-br from-slate-800 to-slate-900",
            div { class: "bg-white rounded-lg shadow-xl p-8 w-full max-w-md",
                // Logo 和标题
                div { class: "text-center mb-8",
                    h1 { class: "text-3xl font-bold text-slate-800", "RustSet" }
                    p { class: "text-slate-500 mt-2", "安全资产管理平台" }
                }

                // 登录表单
                div { class: "space-y-6",
                    // 用户名
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "用户名" }
                        div { class: "relative",
                            div { class: "absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none",
                                Icon { icon: FaUser, width: 20, height: 20 }
                            }
                            input {
                                r#type: "text",
                                class: "block w-full pl-10 pr-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                                placeholder: "请输入用户名",
                                value: username,
                                oninput: move |e| username.set(e.value()),
                            }
                        }
                    }

                    // 密码
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "密码" }
                        div { class: "relative",
                            div { class: "absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none",
                                Icon { icon: FaLock, width: 20, height: 20 }
                            }
                            input {
                                r#type: "password",
                                class: "block w-full pl-10 pr-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                                placeholder: "请输入密码",
                                value: password,
                                oninput: move |e| password.set(e.value()),
                            }
                        }
                    }

                    // 错误信息
                    if !error.read().is_empty() {
                        div { class: "bg-red-50 text-red-500 p-3 rounded-md text-sm",
                            {error.read().clone()}
                        }
                    }

                    // 登录按钮
                    button {
                        r#type: "button",
                        class: if *loading.read() {
                            "w-full py-2 px-4 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 opacity-50 cursor-not-allowed"
                        } else {
                            "w-full py-2 px-4 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2"
                        },
                        disabled: *loading.read(),
                        onclick: handle_login,
                        if *loading.read() {
                            "登录中..."
                        } else {
                            "登录"
                        }
                    }
                }

                // 底部信息
                div { class: "mt-6 text-center text-sm text-gray-500",
                    "默认账号: admin / admin123"
                }
            }
        }
    }
}
