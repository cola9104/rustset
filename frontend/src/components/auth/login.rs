use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::{FaLock, FaUser};
use dioxus_free_icons::Icon;
use dioxus_router::navigator;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use web_sys::RequestCredentials;

use crate::app::AuthUser;
use crate::app::{AUTH_READY, AUTH_STATE};
use crate::config::login_url;
use crate::router::{first_accessible_route, Route};
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
    role: Value,
    #[serde(default)]
    permissions: Option<Value>,
}

impl LoginUser {
    /// 将后端用户转换为前端 AuthUser
    fn to_auth_user(&self) -> AuthUser {
        let permissions = permission_strings_from_value(self.permissions.as_ref());

        AuthUser {
            id: self.id.clone(),
            username: self.username.clone(),
            real_name: String::new(),
            display_name: self.username.clone(),
            email: String::new(),
            phone: String::new(),
            role: parse_role_value(&self.role),
            permissions,
            organization_id: None,
            organization_name: String::new(),
            department_id: None,
            department_name: String::new(),
        }
    }
}

fn parse_role_value(role: &Value) -> String {
    match role {
        Value::String(value) => value.clone(),
        Value::Object(map) => map
            .get("Custom")
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or_else(|| "Custom".to_string()),
        _ => "Unknown".to_string(),
    }
}

fn permission_strings_from_value(permissions: Option<&Value>) -> Vec<String> {
    permissions
        .and_then(Value::as_object)
        .map(|permissions| {
            let mut items = Vec::new();
            for (key, value) in permissions {
                if value.as_bool().is_some_and(|enabled| enabled) {
                    items.push(key.clone());
                } else if let Some(scope) = value.as_str() {
                    items.push(format!("{key}:{scope}"));
                }
            }
            items
        })
        .unwrap_or_default()
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
                                        let workflow_auth = WorkflowAuthState::from(&auth_user);
                                        auth_state.set(workflow_auth.clone());
                                        *AUTH_STATE.write() = Some(auth_user);
                                        *AUTH_READY.write() = true;

                                        nav.push(
                                            first_accessible_route(&workflow_auth)
                                                .unwrap_or(Route::Dashboard {}),
                                        );
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
