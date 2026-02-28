use dioxus::prelude::*;
use dioxus_router::Outlet;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::FaBars;

use super::Sidebar;
use crate::router::Route;

/// 主布局组件
#[allow(non_snake_case)]
pub fn Layout() -> Element {
    let mut sidebar_collapsed = use_signal(|| false);

    rsx! {
        div { class: "flex h-screen bg-gray-100",
            // 侧边栏
            Sidebar { collapsed: sidebar_collapsed }

            // 主内容区
            div { class: "flex-1 flex flex-col overflow-hidden",
                // 顶部栏
                header { class: "bg-white shadow-sm border-b border-gray-200 h-14 flex items-center justify-between px-4",
                    button {
                        class: "p-2 rounded-md hover:bg-gray-100",
                        onclick: move |_| sidebar_collapsed.toggle(),
                        Icon { icon: FaBars, width: 20, height: 20 }
                    }
                    div { class: "text-lg font-semibold text-gray-800", "RustSet" }
                }

                // 页面内容
                main { class: "flex-1 overflow-auto p-6",
                    Outlet::<Route> {}
                }
            }
        }
    }
}
