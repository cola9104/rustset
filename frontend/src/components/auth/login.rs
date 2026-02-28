use dioxus::prelude::*;
use dioxus_router::navigator;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::{FaUser, FaLock};

use crate::app::AuthUser;
use crate::app::AUTH_STATE;
use crate::router::Route;

/// 登录页面
#[allow(non_snake_case)]
pub fn Login() -> Element {
    let mut username = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut error = use_signal(String::new);
    let mut loading = use_signal(|| false);
    let nav = navigator();

    let handle_login = move |_| {
        let user = username.read().clone();
        let pass = password.read().clone();

        spawn(async move {
            loading.set(true);
            error.set(String::new());

            // 模拟登录验证
            if user.is_empty() || pass.is_empty() {
                error.set("用户名和密码不能为空".to_string());
                loading.set(false);
                return;
            }

            // 简单的登录逻辑（实际项目中应该调用 API）
            if user == "admin" && pass == "admin" {
                let auth_user = AuthUser {
                    id: 1,
                    username: user,
                    role: "admin".to_string(),
                    permissions: vec!["all".to_string()],
                };
                *AUTH_STATE.write() = Some(auth_user);
                nav.push(Route::Dashboard {});
            } else {
                error.set("用户名或密码错误".to_string());
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
                    "默认账号: admin / admin"
                }
            }
        }
    }
}
