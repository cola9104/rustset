use super::zone_form::{FormMode, ZoneForm};
use crate::app::NETWORK_ZONES_STATE;
use crate::state::network_zone::NetworkZone;
use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaCheck, FaCloud, FaMagnifyingGlass, FaNetworkWired, FaPenToSquare, FaPlus, FaPowerOff,
    FaServer, FaTrash,
};
use dioxus_free_icons::Icon;

/// 网络区域管理页面
#[allow(non_snake_case)]
pub fn NetworkZoneManagement() -> Element {
    let mut search_query = use_signal(String::new);
    let mut show_add_modal = use_signal(|| false);
    let mut editing_zone = use_signal(|| None::<NetworkZone>);

    // 过滤区域
    let filtered_zones: Vec<NetworkZone> = NETWORK_ZONES_STATE
        .read()
        .iter()
        .filter(|zone| {
            let search_query = search_query.read().to_lowercase();

            search_query.is_empty()
                || zone.name.to_lowercase().contains(&search_query)
                || zone.description.to_lowercase().contains(&search_query)
        })
        .cloned()
        .collect();

    // 统计 - 使用 is_cloud_zone() 和 is_physical_zone() 方法
    let cloud_count = NETWORK_ZONES_STATE
        .read()
        .iter()
        .filter(|z| z.is_cloud_zone())
        .count();
    let physical_count = NETWORK_ZONES_STATE
        .read()
        .iter()
        .filter(|z| z.is_physical_zone())
        .count();

    // 删除区域
    let delete_zone = move |id: i32| {
        let mut zones = NETWORK_ZONES_STATE.write();
        zones.retain(|z| z.id != id);
    };

    // 切换激活状态
    let toggle_active = move |id: i32| {
        let mut zones = NETWORK_ZONES_STATE.write();
        if let Some(zone) = zones.iter_mut().find(|z| z.id == id) {
            zone.is_active = !zone.is_active;
        }
    };

    rsx! {
        div { class: "p-6",
            // 页面标题
            div { class: "flex justify-between items-center mb-6",
                div {
                    h1 { class: "text-2xl font-bold text-gray-800", "网络区域管理" }
                    p { class: "text-sm text-gray-500 mt-1", "管理网络访问策略的来源和目标区域" }
                }
                button {
                    class: "bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-lg flex items-center gap-2 transition-colors",
                    onclick: move |_| {
                        editing_zone.set(None);
                        show_add_modal.set(true);
                    },
                    Icon { icon: FaPlus, width: 16, height: 16 }
                    "添加区域"
                }
            }

            // 说明卡片
            div { class: "bg-blue-50 border border-blue-200 rounded-lg p-4 mb-6",
                h3 { class: "text-sm font-semibold text-blue-800 mb-2", "网络区域说明" }
                div { class: "text-sm text-blue-700 space-y-1",
                    p { "• 网络区域用于标记网络访问策略的来源和目标" }
                    p { "• 云平台区域: 阿里云, 华为云等 (绑定云平台配置)" }
                    p { "• 物理机房区域: 市政务云机房A内网, 核心机房内网等 (绑定机房)" }
                    p { class: "text-xs mt-2 opacity-75", "物理机选机房后自动属于对应机房的内网区域; 云服务器自动属于云平台区域" }
                }
            }

            // 统计卡片
            div { class: "grid grid-cols-1 md:grid-cols-3 gap-4 mb-6",
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center gap-3",
                        div { class: "p-2 bg-blue-100 rounded-lg",
                            Icon { icon: FaNetworkWired, width: 24, height: 24, class: "text-blue-600" }
                        }
                        div {
                            p { class: "text-sm text-gray-500", "总区域数" }
                            p { class: "text-2xl font-bold text-gray-800", "{NETWORK_ZONES_STATE.read().len()}" }
                        }
                    }
                }
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center gap-3",
                        div { class: "p-2 bg-green-100 rounded-lg",
                            Icon { icon: FaCloud, width: 24, height: 24, class: "text-green-600" }
                        }
                        div {
                            p { class: "text-sm text-gray-500", "云平台区域" }
                            p { class: "text-2xl font-bold text-gray-800", "{cloud_count}" }
                        }
                    }
                }
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center gap-3",
                        div { class: "p-2 bg-orange-100 rounded-lg",
                            Icon { icon: FaServer, width: 24, height: 24, class: "text-orange-600" }
                        }
                        div {
                            p { class: "text-sm text-gray-500", "物理机房区域" }
                            p { class: "text-2xl font-bold text-gray-800", "{physical_count}" }
                        }
                    }
                }
            }

            // 搜索框
            div { class: "bg-white rounded-lg shadow p-4 mb-6",
                div { class: "relative",
                    Icon { icon: FaMagnifyingGlass,
                        width: 16,
                        height: 16,
                        class: "absolute left-3 top-1/2 -translate-y-1/2 text-gray-400"
                    }
                    input {
                        r#type: "text",
                        class: "w-full pl-10 pr-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                        placeholder: "搜索区域名称...",
                        value: "{search_query}",
                        oninput: move |e| search_query.set(e.value())
                    }
                }
            }

            // 区域列表
            div { class: "bg-white rounded-lg shadow overflow-hidden",
                if filtered_zones.is_empty() {
                    div { class: "p-12 text-center text-gray-500",
                        Icon { icon: FaNetworkWired, width: 48, height: 48, class: "mx-auto mb-4 text-gray-300" }
                        p { class: "text-lg", "暂无网络区域" }
                    }
                } else {
                    div { class: "overflow-x-auto",
                        table { class: "w-full",
                            thead {
                                tr { class: "bg-gray-50 border-b border-gray-200",
                                    th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "ID" }
                                    th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "区域名称" }
                                    th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "类型" }
                                    th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "网段配置" }
                                    th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "描述" }
                                    th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "状态" }
                                    th { class: "px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider", "操作" }
                                }
                            }
                            tbody {
                                for zone in filtered_zones.clone() {
                                    tr { class: "hover:bg-gray-50 border-b border-gray-100 transition-colors",
                                        td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-500", "{zone.id}" }
                                        td { class: "px-6 py-4 whitespace-nowrap",
                                            div { class: "text-sm font-medium text-gray-900", "{zone.name}" }
                                        }
                                        td { class: "px-6 py-4 whitespace-nowrap",
                                            if zone.is_cloud_zone() {
                                                span { class: "inline-flex items-center gap-1 px-2 py-1 text-xs font-semibold rounded-full bg-green-100 text-green-800",
                                                    Icon { icon: FaCloud, width: 12, height: 12 }
                                                    "云平台"
                                                }
                                            } else if zone.is_physical_zone() {
                                                span { class: "inline-flex items-center gap-1 px-2 py-1 text-xs font-semibold rounded-full bg-orange-100 text-orange-800",
                                                    Icon { icon: FaServer, width: 12, height: 12 }
                                                    "物理机房"
                                                }
                                            }
                                        }
                                        td { class: "px-6 py-4 text-sm text-gray-600 max-w-xs",
                                            if zone.has_network_config() {
                                                div { class: "space-y-1",
                                                    if !zone.cidr_blocks.is_empty() {
                                                        div { class: "text-xs",
                                                            span { class: "font-medium text-gray-700", "CIDR: " }
                                                            span { class: "font-mono text-gray-600", "{zone.cidr_blocks.join(\", \")}" }
                                                        }
                                                    }
                                                    if !zone.ip_ranges.is_empty() {
                                                        div { class: "text-xs",
                                                            span { class: "font-medium text-gray-700", "IP: " }
                                                            span { class: "font-mono text-gray-600", "{zone.ip_ranges.join(\", \")}" }
                                                        }
                                                    }
                                                }
                                            } else {
                                                span { class: "text-xs text-gray-400 italic", "未配置" }
                                            }
                                        }
                                        td { class: "px-6 py-4 text-sm text-gray-600", "{zone.description}" }
                                        td { class: "px-6 py-4 whitespace-nowrap",
                                            if zone.is_active {
                                                span { class: "inline-flex items-center gap-1 px-2 py-1 text-xs font-semibold rounded-full bg-green-100 text-green-800",
                                                    Icon { icon: FaCheck, width: 12, height: 12 }
                                                    "启用"
                                                }
                                            } else {
                                                span { class: "inline-flex items-center gap-1 px-2 py-1 text-xs font-semibold rounded-full bg-gray-100 text-gray-600",
                                                    Icon { icon: FaPowerOff, width: 12, height: 12 }
                                                    "停用"
                                                }
                                            }
                                        }
                                        td { class: "px-6 py-4 whitespace-nowrap text-right text-sm font-medium",
                                            div { class: "flex items-center justify-end gap-2",
                                                // 编辑按钮
                                                button {
                                                    class: "text-blue-600 hover:text-blue-800 p-1 hover:bg-blue-50 rounded transition-colors",
                                                    title: "编辑",
                                                    onclick: {
                                                        let zone_clone = zone.clone();
                                                        move |_| {
                                                            editing_zone.set(Some(zone_clone.clone()));
                                                            show_add_modal.set(true);
                                                        }
                                                    },
                                                    Icon { icon: FaPenToSquare, width: 16, height: 16 }
                                                }
                                                // 切换状态按钮
                                                if zone.is_active {
                                                    button {
                                                        class: "text-yellow-600 hover:text-yellow-800 hover:bg-yellow-50 p-1 rounded transition-colors",
                                                        title: "停用",
                                                        onclick: {
                                                            let zone_id = zone.id;
                                                            move |_| toggle_active(zone_id)
                                                        },
                                                        Icon { icon: FaPowerOff, width: 16, height: 16 }
                                                    }
                                                } else {
                                                    button {
                                                        class: "text-green-600 hover:text-green-800 hover:bg-green-50 p-1 rounded transition-colors",
                                                        title: "启用",
                                                        onclick: {
                                                            let zone_id = zone.id;
                                                            move |_| toggle_active(zone_id)
                                                        },
                                                        Icon { icon: FaPowerOff, width: 16, height: 16 }
                                                    }
                                                }
                                                // 删除按钮
                                                button {
                                                    class: "text-red-600 hover:text-red-800 p-1 hover:bg-red-50 rounded transition-colors",
                                                    title: "删除",
                                                    onclick: {
                                                        let zone_id = zone.id;
                                                        let zone_name = zone.name.clone();
                                                        move |_| {
                                                            if web_sys::window()
                                                                .and_then(|w| w.confirm_with_message(&format!("确定要删除网络区域\"{}\"吗？", zone_name)).ok())
                                                                .unwrap_or(false)
                                                            {
                                                                delete_zone(zone_id);
                                                            }
                                                        }
                                                    },
                                                    Icon { icon: FaTrash, width: 16, height: 16 }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // 添加/编辑弹窗
        if *show_add_modal.read() {
            ZoneForm {
                mode: if editing_zone.read().is_some() { FormMode::Edit } else { FormMode::New },
                zone: editing_zone.read().clone(),
                on_save: move |zone: NetworkZone| {
                    let mut zones = NETWORK_ZONES_STATE.write();
                    if let Some(idx) = zones.iter().position(|z| z.id == zone.id) {
                        zones[idx] = zone;
                    } else {
                        zones.push(zone);
                    }
                    show_add_modal.set(false);
                    editing_zone.set(None);
                },
                on_close: move |_| {
                    show_add_modal.set(false);
                    editing_zone.set(None);
                },
            }
        }
    }
}
