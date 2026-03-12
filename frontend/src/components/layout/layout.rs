use dioxus::prelude::*;
use dioxus_router::{Outlet, navigator};
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::{FaBars, FaRightFromBracket};

use super::Sidebar;
use crate::router::Route;
use crate::state::user_role::use_auth;
use crate::app::logout;

/// 主布局组件
#[allow(non_snake_case)]
pub fn Layout() -> Element {
    let mut sidebar_collapsed = use_signal(|| false);
    let mut show_user_menu = use_signal(|| false);
    let auth_state = use_auth();
    let nav = navigator();

    // 获取用户名的首字母作为头像
    let user_initial = auth_state.read().username.chars().next().unwrap_or('U');
    let user_role_name = auth_state.read().role.display_name();

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
                                span { class: "text-sm font-medium text-gray-800", "{auth_state.read().username}" }
                                span { class: "text-xs text-gray-500", "{user_role_name}" }
                            }
                        }

                        // 下拉菜单
                        if *show_user_menu.read() {
                            div { class: "absolute right-0 top-full mt-2 w-48 bg-white rounded-lg shadow-lg border border-gray-200 py-2 z-50",
                                div { class: "px-4 py-2 border-b border-gray-100",
                                    p { class: "text-sm font-medium text-gray-800", "{auth_state.read().username}" }
                                    p { class: "text-xs text-gray-500", "{user_role_name}" }
                                }
                                button {
                                    class: "w-full flex items-center gap-3 px-4 py-2 text-left text-sm text-gray-700 hover:bg-gray-100 transition-colors",
                                    onclick: move |_| {
                                        // 调用登出函数清除认证状态
                                        logout();
                                        show_user_menu.set(false);
                                        // 导航到登录页面
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
                    Outlet::<Route> {}
                }
            }
        }
    }
}
