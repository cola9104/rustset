use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::{FaBars, FaRightFromBracket};
use dioxus_free_icons::Icon;
use dioxus_router::{navigator, use_route, Outlet};
use gloo_net::http::Request;
use web_sys::RequestCredentials;

use super::Sidebar;
use crate::app::{is_authenticated, logout, AUTH_READY};
use crate::config::logout_url;
use crate::router::{can_access_route, first_accessible_route, Route};
use crate::state::user_role::{use_auth, AuthState as WorkflowAuthState};

/// 主布局组件
#[allow(non_snake_case)]
pub fn Layout() -> Element {
    let mut sidebar_collapsed = use_signal(|| false);
    let mut show_user_menu = use_signal(|| false);
    let mut auth_state = use_auth();
    let nav = navigator();
    let current_route: Route = use_route();
    let auth_ready = *AUTH_READY.read();

    use_effect(move || {
        if auth_ready && !is_authenticated() {
            nav.push(Route::Login {});
        }
    });

    if !auth_ready {
        return rsx! { div { class: "flex h-screen items-center justify-center bg-gray-100 text-sm text-gray-500", "正在恢复会话..." } };
    }

    if !is_authenticated() {
        return rsx! { div {} };
    }

    let current_auth = auth_state.read().clone();
    let fallback_route = first_accessible_route(&current_auth);
    let route_allowed = can_access_route(&current_route, &current_auth);

    // 获取用户名的首字母作为头像
    let user_initial = current_auth.username.chars().next().unwrap_or('U');
    let user_role_name = current_auth.display_role_label().to_string();

    rsx! {
        div { class: "flex h-screen bg-gray-100",
            // 侧边栏
            Sidebar { collapsed: sidebar_collapsed }

            // 主内容区
            div { class: "flex-1 flex flex-col overflow-hidden",
                // 顶部栏
                header { class: "bg-white shadow-sm border-b border-gray-200 h-14 flex items-center justify-between px-4",
                    // 左侧菜单按钮
                    button {
                        class: "p-2 rounded-md hover:bg-gray-100",
                        onclick: move |_| sidebar_collapsed.toggle(),
                        Icon { icon: FaBars, width: 20, height: 20 }
                    }

                    // 右侧用户信息
                    div { class: "relative",
                        button {
                            class: "flex items-center gap-3 px-3 py-2 rounded-lg hover:bg-gray-100 transition-colors",
                            onclick: move |_| show_user_menu.toggle(),
                            // 头像
                            div { class: "w-8 h-8 rounded-full bg-blue-600 flex items-center justify-center text-white font-semibold text-sm",
                                "{user_initial}"
                            }
                            // 用户信息
                            div { class: "flex flex-col items-start",
                                span { class: "text-sm font-medium text-gray-800", "{current_auth.username}" }
                                span { class: "text-xs text-gray-500", "{user_role_name}" }
                            }
                        }

                        // 下拉菜单
                        if *show_user_menu.read() {
                            div { class: "absolute right-0 top-full mt-2 w-48 bg-white rounded-lg shadow-lg border border-gray-200 py-2 z-50",
                                div { class: "px-4 py-2 border-b border-gray-100",
                                    p { class: "text-sm font-medium text-gray-800", "{current_auth.username}" }
                                    p { class: "text-xs text-gray-500", "{user_role_name}" }
                                }
                                button {
                                    class: "w-full flex items-center gap-3 px-4 py-2 text-left text-sm text-gray-700 hover:bg-gray-100 transition-colors",
                                    onclick: move |_| {
                                        spawn(async move {
                                            let _ = Request::post(&logout_url())
                                                .credentials(RequestCredentials::Include)
                                                .send()
                                                .await;
                                        });

                                        logout();
                                        auth_state.set(WorkflowAuthState::guest());
                                        show_user_menu.set(false);
                                        nav.push(Route::Login {});
                                    },
                                    Icon { icon: FaRightFromBracket, width: 16, height: 16, class: "text-gray-500" }
                                    "退出登录"
                                }
                            }
                        }

                        // 点击外部关闭菜单
                        if *show_user_menu.read() {
                            div {
                                class: "fixed inset-0 z-40",
                                onclick: move |_| show_user_menu.set(false),
                            }
                        }
                    }
                }

                // 页面内容
                main { class: "flex-1 overflow-auto p-6",
                    if route_allowed {
                        Outlet::<Route> {}
                    } else {
                        AccessDeniedPage {
                            current_route: current_route.clone(),
                            fallback_route: fallback_route.clone(),
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn AccessDeniedPage(current_route: Route, fallback_route: Option<Route>) -> Element {
    let nav = navigator();

    rsx! {
        div { class: "flex min-h-full items-center justify-center",
            div { class: "w-full max-w-xl rounded-2xl border border-amber-200 bg-white p-8 shadow-sm",
                div { class: "mb-4 inline-flex rounded-full bg-amber-100 px-3 py-1 text-sm font-medium text-amber-800",
                    "403 无权限访问"
                }
                h1 { class: "text-2xl font-bold text-slate-900", "当前页面不在你的权限范围内" }
                p { class: "mt-3 text-sm leading-6 text-slate-600",
                    "你可以通过侧边栏访问自己有权限的模块；如果这个页面本应可见，需要检查角色权限配置是否已同步。"
                }
                p { class: "mt-2 text-sm text-slate-500", "当前路由: {current_route}" }
                div { class: "mt-6 flex gap-3",
                    if let Some(route) = fallback_route {
                        button {
                            class: "rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700",
                            onclick: move |_| {
                                nav.push(route.clone());
                            },
                            "前往可访问页面"
                        }
                    }
                }
            }
        }
    }
}
