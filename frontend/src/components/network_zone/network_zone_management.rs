use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaPlus, FaPenToSquare, FaTrash, FaMagnifyingGlass, FaNetworkWired,
    FaCheck, FaPowerOff, FaServer, FaCloud,
};
use crate::state::network_zone::NetworkZone;
use crate::app::NETWORK_ZONES_STATE;
use crate::app::CLOUD_PLATFORMS_STATE;
use crate::app::MACHINE_ROOMS_STATE;

/// 网络区域管理页面
#[allow(non_snake_case)]
pub fn NetworkZoneManagement() -> Element {
    let mut search_query = use_signal(String::new);
    let mut show_add_modal = use_signal(|| false);
    let mut editing_zone = use_signal(|| None::<NetworkZone>);

    // 过滤区域
    let filtered_zones: Vec<NetworkZone> = NETWORK_ZONES_STATE.read()
        .iter()
        .filter(|zone| {
            let search_query = search_query.read().to_lowercase();
            let matches_search = search_query.is_empty()
                || zone.name.to_lowercase().contains(&search_query)
                || zone.description.to_lowercase().contains(&search_query);
            matches_search
        })
        .cloned()
        .collect();

    // 统计 - 使用 is_cloud_zone() 和 is_physical_zone() 方法
    let cloud_count = NETWORK_ZONES_STATE.read().iter().filter(|z| z.is_cloud_zone()).count();
    let physical_count = NETWORK_ZONES_STATE.read().iter().filter(|z| z.is_physical_zone()).count();

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
        NetworkZoneModal {
            show: show_add_modal,
            editing_zone: editing_zone,
            on_close: move |_| {
                show_add_modal.set(false);
                editing_zone.set(None);
            }
        }
    }
}

/// 添加/编辑网络区域弹窗
#[component]
fn NetworkZoneModal(
    show: Signal<bool>,
    editing_zone: Signal<Option<NetworkZone>>,
    on_close: EventHandler<()>,
) -> Element {
    use dioxus_free_icons::icons::fa_solid_icons::{FaPlus, FaXmark};
    use dioxus_free_icons::Icon;

    let mut name = use_signal(String::new);
    let mut description = use_signal(String::new);
    let mut zone_type = use_signal(|| "cloud".to_string());  // "cloud" 或 "physical"
    let mut selected_cloud_platform = use_signal(|| Option::<i32>::None);
    let mut selected_machine_room = use_signal(|| Option::<i32>::None);
    let mut cidr_blocks = use_signal(|| Vec::<String>::new());
    let mut ip_ranges = use_signal(|| Vec::<String>::new());
    let mut new_cidr = use_signal(String::new);
    let mut is_active = use_signal(|| true);
    let mut error_message = use_signal(|| None::<String>);

    // 当编辑项变化时，更新表单
    use_effect(move || {
        if let Some(zone) = editing_zone.read().as_ref() {
            name.set(zone.name.clone());
            description.set(zone.description.clone());
            cidr_blocks.set(zone.cidr_blocks.clone());
            ip_ranges.set(zone.ip_ranges.clone());
            if zone.is_cloud_zone() {
                zone_type.set("cloud".to_string());
                selected_cloud_platform.set(zone.cloud_platform_id);
                selected_machine_room.set(None);
            } else if zone.is_physical_zone() {
                zone_type.set("physical".to_string());
                selected_machine_room.set(zone.machine_room_id);
                selected_cloud_platform.set(None);
            }
            is_active.set(zone.is_active);
        } else {
            name.set(String::new());
            description.set(String::new());
            cidr_blocks.set(Vec::new());
            ip_ranges.set(Vec::new());
            zone_type.set("cloud".to_string());
            selected_cloud_platform.set(None);
            selected_machine_room.set(None);
            is_active.set(true);
        }
    });

    let is_editing = editing_zone.read().is_some();

    // 删除 CIDR 网段
    let mut remove_cidr = move |index: usize| {
        let mut blocks = cidr_blocks.write();
        blocks.remove(index);
    };

    // 删除 IP 范围
    let mut remove_ip_range = move |index: usize| {
        let mut ranges = ip_ranges.write();
        ranges.remove(index);
    };

    // 保存
    let mut handle_save = move || {
        error_message.set(None);

        // 验证
        if name.read().is_empty() {
            error_message.set(Some("区域名称不能为空".to_string()));
            return;
        }

        let (cloud_platform_id, machine_room_id) = if *zone_type.read() == "cloud" {
            (*selected_cloud_platform.read(), None)
        } else {
            (None, *selected_machine_room.read())
        };

        // 验证选择了对应的资源
        if *zone_type.read() == "cloud" && cloud_platform_id.is_none() {
            error_message.set(Some("请选择云平台".to_string()));
            return;
        }
        if *zone_type.read() == "physical" && machine_room_id.is_none() {
            error_message.set(Some("请选择机房".to_string()));
            return;
        }

        let mut zones = NETWORK_ZONES_STATE.write();

        if is_editing {
            if let Some(original) = editing_zone.read().as_ref() {
                // 检查名称是否与其他区域重复
                if zones.iter().any(|z| z.id != original.id && z.name == *name.read()) {
                    error_message.set(Some("网络区域名称已存在".to_string()));
                    return;
                }

                // 更新
                if let Some(zone) = zones.iter_mut().find(|z| z.id == original.id) {
                    zone.name = name.read().clone();
                    zone.description = description.read().clone();
                    zone.cloud_platform_id = cloud_platform_id;
                    zone.machine_room_id = machine_room_id;
                    zone.cidr_blocks = cidr_blocks.read().clone();
                    zone.ip_ranges = ip_ranges.read().clone();
                    zone.is_active = *is_active.read();
                }
            }
        } else {
            // 检查名称是否重复
            if zones.iter().any(|z| z.name == *name.read()) {
                error_message.set(Some("网络区域名称已存在".to_string()));
                return;
            }

            // 新增
            let new_id = zones.iter().map(|z| z.id).max().unwrap_or(0) + 1;
            zones.push(NetworkZone {
                id: new_id,
                name: name.read().clone(),
                cloud_platform_id,
                machine_room_id,
                cidr_blocks: cidr_blocks.read().clone(),
                ip_ranges: ip_ranges.read().clone(),
                description: description.read().clone(),
                is_active: *is_active.read(),
            });
        }

        on_close(());
    };

    if !*show.read() {
        return rsx! { "" };
    }

    let modal_title = if is_editing { "编辑网络区域" } else { "添加网络区域" };
    let button_text = if is_editing { "保存" } else { "添加" };

    // 获取云平台列表和机房列表
    let cloud_platforms = CLOUD_PLATFORMS_STATE.read().clone();
    let machine_rooms = MACHINE_ROOMS_STATE.read().clone();

    rsx! {
        div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            div { class: "bg-white rounded-lg shadow-xl max-w-2xl w-full mx-4 max-h-[90vh] overflow-y-auto",
                // 弹窗头部
                div { class: "flex items-center justify-between p-6 border-b border-gray-200",
                    h2 { class: "text-xl font-bold text-gray-800",
                        "{modal_title}"
                    }
                    button {
                        class: "text-gray-400 hover:text-gray-600 transition-colors",
                        onclick: move |_| on_close(()),
                        "×"
                    }
                }

                // 弹窗内容
                div { class: "p-6",
                    // 错误提示
                    if let Some(error) = error_message.read().as_ref() {
                        div { class: "mb-4 p-3 bg-red-100 border border-red-400 text-red-700 rounded-lg",
                            "{error}"
                        }
                    }

                    div { class: "space-y-4",
                        // 区域类型选择
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "区域类型 *" }
                            div { class: "flex gap-4",
                                label { class: "flex items-center gap-2 cursor-pointer",
                                    input {
                                        r#type: "radio",
                                        class: "w-4 h-4 text-blue-600 focus:ring-blue-500",
                                        name: "zone_type",
                                        checked: "{*zone_type.read() == \"cloud\"}",
                                        oninput: move |_| zone_type.set("cloud".to_string())
                                    }
                                    span { class: "text-sm text-gray-700", "云平台区域" }
                                }
                                label { class: "flex items-center gap-2 cursor-pointer",
                                    input {
                                        r#type: "radio",
                                        class: "w-4 h-4 text-blue-600 focus:ring-blue-500",
                                        name: "zone_type",
                                        checked: "{*zone_type.read() == \"physical\"}",
                                        oninput: move |_| zone_type.set("physical".to_string())
                                    }
                                    span { class: "text-sm text-gray-700", "物理机房区域" }
                                }
                            }
                        }

                        // 关联云平台（仅当选择云平台区域时显示）
                        if *zone_type.read() == "cloud" {
                            div {
                                label { class: "block text-sm font-medium text-gray-700 mb-1", "关联云平台 *" }
                                select {
                                    class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                                    value: "{selected_cloud_platform.read().unwrap_or(-1)}",
                                    onchange: move |e| {
                                        let val: i32 = e.value().parse().unwrap_or(-1);
                                        selected_cloud_platform.set(if val > 0 { Some(val) } else { None });
                                    },
                                    option { value: "-1", "请选择云平台" }
                                    for platform in cloud_platforms.iter() {
                                        option {
                                            value: "{platform.id}",
                                            selected: *selected_cloud_platform.read() == Some(platform.id),
                                            "{platform.foundation} ({platform.platform_name})"
                                        }
                                    }
                                }
                                p { class: "mt-1 text-xs text-gray-500", "云网络区域绑定到云平台配置" }
                            }
                        }

                        // 关联机房（仅当选择物理机房区域时显示）
                        if *zone_type.read() == "physical" {
                            div {
                                label { class: "block text-sm font-medium text-gray-700 mb-1", "关联机房 *" }
                                select {
                                    class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                                    value: "{selected_machine_room.read().unwrap_or(-1)}",
                                    onchange: move |e| {
                                        let val: i32 = e.value().parse().unwrap_or(-1);
                                        selected_machine_room.set(if val > 0 { Some(val) } else { None });
                                    },
                                    option { value: "-1", "请选择机房" }
                                    for room in machine_rooms.iter() {
                                        option {
                                            value: "{room.id}",
                                            selected: *selected_machine_room.read() == Some(room.id),
                                            "{room.room_name} - {room.facility_type}"
                                        }
                                    }
                                }
                                p { class: "mt-1 text-xs text-gray-500", "物理网络区域绑定到机房配置" }
                            }
                        }

                        // 区域名称（自动填充，可编辑）
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "区域名称 *" }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                                placeholder: "例如: 阿里云, 市政务云机房A内网",
                                value: "{name}",
                                oninput: move |e| name.set(e.value())
                            }
                        }

                        // 网段配置部分
                        div { class: "border-t border-gray-200 pt-4 mt-4",
                            h4 { class: "text-sm font-semibold text-gray-800 mb-3", "网段配置" }
                            p { class: "text-xs text-gray-500 mb-3", "支持 CIDR (192.168.1.0/24)、IP 范围 (192.168.1.1-192.168.1.100) 或单个 IP (10.0.0.1)" }

                            div { class: "flex gap-2 mb-3",
                                input {
                                    r#type: "text",
                                    class: "flex-1 px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                                    placeholder: "输入网段/IP，按回车快速添加",
                                    value: "{new_cidr}",
                                    oninput: move |e| new_cidr.set(e.value()),
                                    onkeydown: move |e| {
                                        if e.key().to_string() == "Enter" {
                                            let cidr = new_cidr.read().trim().to_string();
                                            if !cidr.is_empty() {
                                                let is_cidr = cidr.contains('/');
                                                if is_cidr {
                                                    let mut blocks = cidr_blocks.write();
                                                    if !blocks.contains(&cidr) {
                                                        blocks.push(cidr);
                                                    }
                                                } else {
                                                    let mut ranges = ip_ranges.write();
                                                    if !ranges.contains(&cidr) {
                                                        ranges.push(cidr);
                                                    }
                                                }
                                                new_cidr.set(String::new());
                                            }
                                        }
                                    }
                                }
                                button {
                                    class: "px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors flex items-center gap-2",
                                    onclick: move |_| {
                                        let cidr = new_cidr.read().trim().to_string();
                                        if !cidr.is_empty() {
                                            let is_cidr = cidr.contains('/');
                                            if is_cidr {
                                                let mut blocks = cidr_blocks.write();
                                                if !blocks.contains(&cidr) {
                                                    blocks.push(cidr);
                                                }
                                            } else {
                                                let mut ranges = ip_ranges.write();
                                                if !ranges.contains(&cidr) {
                                                    ranges.push(cidr);
                                                }
                                            }
                                            new_cidr.set(String::new());
                                        }
                                    },
                                    Icon { icon: FaPlus, width: 16, height: 16 }
                                    "添加"
                                }
                            }

                            // 合并显示所有网段/IP
                            div { class: "space-y-1",
                                // CIDR 网段
                                for (index, cidr) in cidr_blocks.read().iter().enumerate() {
                                    div { class: "flex items-center justify-between p-2 bg-blue-50 rounded-lg border border-blue-100",
                                        div { class: "flex items-center gap-2",
                                            span { class: "inline-flex items-center px-2 py-0.5 text-xs font-medium rounded bg-blue-100 text-blue-800", "CIDR" }
                                            span { class: "text-sm text-gray-700 font-mono", "{cidr}" }
                                        }
                                        button {
                                            class: "text-red-600 hover:text-red-800 p-1 hover:bg-red-50 rounded",
                                            onclick: move |_| remove_cidr(index),
                                            Icon { icon: FaXmark, width: 16, height: 16 }
                                        }
                                    }
                                }
                                // IP 范围
                                for (index, range) in ip_ranges.read().iter().enumerate() {
                                    div { class: "flex items-center justify-between p-2 bg-green-50 rounded-lg border border-green-100",
                                        div { class: "flex items-center gap-2",
                                            span { class: "inline-flex items-center px-2 py-0.5 text-xs font-medium rounded bg-green-100 text-green-800", "IP" }
                                            span { class: "text-sm text-gray-700 font-mono", "{range}" }
                                        }
                                        button {
                                            class: "text-red-600 hover:text-red-800 p-1 hover:bg-red-50 rounded",
                                            onclick: move |_| remove_ip_range(index),
                                            Icon { icon: FaXmark, width: 16, height: 16 }
                                        }
                                    }
                                }
                                // 空状态提示
                                if cidr_blocks.read().is_empty() && ip_ranges.read().is_empty() {
                                    div { class: "text-center py-4 text-sm text-gray-400 italic", "暂未配置网段" }
                                }
                            }
                        }

                        // 描述
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "描述" }
                            textarea {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                                rows: 2,
                                placeholder: "描述该区域的用途",
                                value: "{description}",
                                oninput: move |e| description.set(e.value())
                            }
                        }

                        // 激活状态
                        div {
                            label { class: "flex items-center gap-2 cursor-pointer",
                                input {
                                    r#type: "checkbox",
                                    class: "w-4 h-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500",
                                    checked: *is_active.read(),
                                    oninput: move |e| is_active.set(e.checked())
                                }
                                span { class: "text-sm text-gray-700", "启用该区域" }
                            }
                        }
                    }
                }

                // 弹窗底部
                div { class: "flex justify-end gap-3 p-6 border-t border-gray-200 bg-gray-50",
                    button {
                        class: "px-4 py-2 border border-gray-300 rounded-lg text-gray-700 hover:bg-gray-100 transition-colors",
                        onclick: move |_| on_close(()),
                        "取消"
                    }
                    button {
                        class: "px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors",
                        onclick: move |_| handle_save(),
                        "{button_text}"
                    }
                }
            }
        }
    }
}
