//! IP Zones Management Page

use crate::config::ip_find_zone_url;
use crate::services::cloud_platform_api::fetch_cloud_platform_configs;
use crate::services::ip_zone_api::{
    create_ip_zone, delete_ip_zone, fetch_ip_zones, CreateIpZoneRequest,
};
use crate::services::machine_room_api::fetch_machine_rooms;
use crate::state::cloud_platform::CloudPlatformConfig;
use crate::state::machine_room::MachineRoomConfig;
use crate::state::user_role::use_auth;
use crate::utils::storage::authorization_header;
use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaArrowsRotate, FaCircleNodes, FaCloud, FaMagnifyingGlass, FaNetworkWired, FaPlus, FaServer,
    FaTrash, FaTriangleExclamation,
};
use dioxus_free_icons::Icon;
use shared::ZoneConfig;
use web_sys::RequestCredentials;

fn zone_binding_label(zone: &ZoneConfig) -> (&'static str, String) {
    if let Some(name) = &zone.cloud_platform_name {
        ("云平台", name.clone())
    } else if let Some(name) = &zone.machine_room_name {
        ("物理机房", name.clone())
    } else {
        ("未绑定", "-".to_string())
    }
}

#[component]
pub fn IpZonesPage() -> Element {
    let auth = use_auth();
    let current_auth = auth.read().clone();
    let mut zones = use_signal(Vec::<ZoneConfig>::new);
    let mut cloud_platforms = use_signal(Vec::<CloudPlatformConfig>::new);
    let mut machine_rooms = use_signal(Vec::<MachineRoomConfig>::new);
    let mut loading = use_signal(|| true);
    let mut error_message = use_signal(String::new);
    let mut search_ip = use_signal(String::new);
    let mut search_loading = use_signal(|| false);
    let mut search_error = use_signal(String::new);
    let mut found_zone = use_signal(|| Option::<(String, String)>::None);
    let mut form_name = use_signal(String::new);
    let mut form_cidr = use_signal(String::new);
    let mut form_priority = use_signal(|| "100".to_string());
    let mut binding_kind = use_signal(|| "cloud".to_string());
    let mut selected_cloud_platform_id = use_signal(|| None::<i32>);
    let mut selected_machine_room_id = use_signal(|| None::<i32>);
    let mut form_error = use_signal(String::new);
    let mut creating = use_signal(|| false);
    let mut deleting_zone_id = use_signal(String::new);

    let load_all = move || async move {
        loading.set(true);
        error_message.set(String::new());

        let zones_result = fetch_ip_zones().await;
        let cloud_platforms_result = fetch_cloud_platform_configs().await;
        let machine_rooms_result = fetch_machine_rooms().await;

        match zones_result {
            Ok(data) => zones.set(data),
            Err(err) => error_message.set(err),
        }

        match cloud_platforms_result {
            Ok(data) => cloud_platforms.set(data),
            Err(err) => {
                if error_message.read().is_empty() {
                    error_message.set(err);
                }
            }
        }

        match machine_rooms_result {
            Ok(data) => machine_rooms.set(data),
            Err(err) => {
                if error_message.read().is_empty() {
                    error_message.set(err);
                }
            }
        }

        loading.set(false);
    };

    use_effect(move || {
        spawn(load_all());
    });

    let refresh_all = move |_| {
        spawn(load_all());
    };

    let find_zone = move |_| {
        let ip = search_ip();
        async move {
            if ip.trim().is_empty() {
                search_error.set("请输入要查询的 IP 地址".to_string());
                found_zone.set(None);
                return;
            }

            search_loading.set(true);
            search_error.set(String::new());
            found_zone.set(None);

            let mut request = gloo_net::http::Request::get(&ip_find_zone_url(&ip))
                .credentials(RequestCredentials::Include);
            if let Some(header) = authorization_header() {
                request = request.header("Authorization", &header);
            }

            match request.send().await {
                Ok(response) if response.ok() => match response.json::<serde_json::Value>().await {
                    Ok(data) => {
                        found_zone.set(Some((
                            data["zone"].as_str().unwrap_or("Internet").to_string(),
                            data["matched_cidr"].as_str().unwrap_or("N/A").to_string(),
                        )));
                    }
                    Err(err) => search_error.set(format!("解析查询结果失败: {}", err)),
                },
                Ok(response) => search_error.set(format!("查询失败: HTTP {}", response.status())),
                Err(err) => search_error.set(format!("查询失败: {}", err)),
            }

            search_loading.set(false);
        }
    };

    let create_zone_action = move |_| {
        let name = form_name();
        let cidr = form_cidr();
        let priority_raw = form_priority();
        let kind = binding_kind();
        let cloud_platform_id = selected_cloud_platform_id();
        let machine_room_id = selected_machine_room_id();

        async move {
            form_error.set(String::new());

            if name.trim().is_empty() {
                form_error.set("区域名称不能为空".to_string());
                return;
            }
            if cidr.trim().is_empty() {
                form_error.set("CIDR 不能为空".to_string());
                return;
            }
            let priority = match priority_raw.trim().parse::<i32>() {
                Ok(value) => value,
                Err(_) => {
                    form_error.set("优先级必须是整数".to_string());
                    return;
                }
            };

            let req = match kind.as_str() {
                "cloud" => {
                    let cloud_platform_id = match cloud_platform_id {
                        Some(id) => id,
                        None => {
                            form_error.set("请选择要绑定的云平台".to_string());
                            return;
                        }
                    };
                    CreateIpZoneRequest {
                        name,
                        cidr,
                        priority,
                        cloud_platform_id: Some(cloud_platform_id),
                        machine_room_id: None,
                    }
                }
                "machine" => {
                    let machine_room_id = match machine_room_id {
                        Some(id) => id,
                        None => {
                            form_error.set("请选择要绑定的物理机房".to_string());
                            return;
                        }
                    };
                    CreateIpZoneRequest {
                        name,
                        cidr,
                        priority,
                        cloud_platform_id: None,
                        machine_room_id: Some(machine_room_id),
                    }
                }
                _ => {
                    form_error.set("请选择绑定类型".to_string());
                    return;
                }
            };

            creating.set(true);
            match create_ip_zone(&req).await {
                Ok(_) => {
                    form_name.set(String::new());
                    form_cidr.set(String::new());
                    form_priority.set("100".to_string());
                    selected_cloud_platform_id.set(None);
                    selected_machine_room_id.set(None);
                    spawn(load_all());
                }
                Err(err) => form_error.set(err),
            }
            creating.set(false);
        }
    };

    let delete_zone_action = move |zone_id: String, zone_name: String| async move {
        let confirmed = web_sys::window()
            .and_then(|window| {
                window
                    .confirm_with_message(&format!("确定要删除网络区域“{}”吗？", zone_name))
                    .ok()
            })
            .unwrap_or(false);
        if !confirmed {
            return;
        }

        deleting_zone_id.set(zone_id.clone());
        error_message.set(String::new());

        match delete_ip_zone(&zone_id).await {
            Ok(()) => {
                zones.write().retain(|zone| zone.id != zone_id);
            }
            Err(err) => error_message.set(err),
        }

        deleting_zone_id.set(String::new());
    };

    let eligible_machine_rooms: Vec<MachineRoomConfig> =
        machine_rooms.read().iter().cloned().collect();
    let selected_cloud_platform_value = selected_cloud_platform_id()
        .map(|value| value.to_string())
        .unwrap_or_default();
    let selected_machine_room_value = selected_machine_room_id()
        .map(|value| value.to_string())
        .unwrap_or_default();

    let zone_count = zones.read().len();
    let cloud_bound_count = zones
        .read()
        .iter()
        .filter(|zone| zone.cloud_platform_id.is_some())
        .count();
    let machine_bound_count = zones
        .read()
        .iter()
        .filter(|zone| zone.machine_room_id.is_some())
        .count();
    let can_manage = current_auth.can_manage_operations();

    rsx! {
        div { class: "flex flex-col gap-6",
            div { class: "rounded-2xl bg-gradient-to-r from-slate-900 via-slate-800 to-cyan-900 p-6 text-white shadow-lg",
                div { class: "flex flex-col gap-5 lg:flex-row lg:items-end lg:justify-between",
                    div {
                        div { class: "mb-3 inline-flex h-12 w-12 items-center justify-center rounded-xl bg-white/10 ring-1 ring-white/15",
                            Icon { icon: FaNetworkWired, width: 22, height: 22, class: "text-cyan-200" }
                        }
                        h1 { class: "text-2xl font-bold tracking-tight", "网络区域管理" }
                        p { class: "mt-2 max-w-3xl text-sm text-slate-200",
                            "网络区域按真实业务语义绑定到云平台或物理机房，具体归属取决于这段 IP 所属的网络。"
                        }
                    }

                    button {
                        class: "inline-flex items-center gap-2 rounded-xl bg-white/10 px-4 py-2.5 text-sm font-medium text-white ring-1 ring-white/15 transition hover:bg-white/15",
                        onclick: refresh_all,
                        Icon { icon: FaArrowsRotate, width: 14, height: 14 }
                        "刷新数据"
                    }
                }
            }

            div { class: "grid grid-cols-1 gap-4 md:grid-cols-3",
                div { class: "rounded-2xl border border-slate-200 bg-white p-5 shadow-sm",
                    div { class: "flex items-center gap-3",
                        div { class: "flex h-11 w-11 items-center justify-center rounded-xl bg-cyan-50 text-cyan-700",
                            Icon { icon: FaCircleNodes, width: 18, height: 18 }
                        }
                        div {
                            p { class: "text-sm text-slate-500", "已配置区域" }
                            p { class: "mt-1 text-2xl font-semibold text-slate-900", "{zone_count}" }
                        }
                    }
                }
                div { class: "rounded-2xl border border-slate-200 bg-white p-5 shadow-sm",
                    div { class: "flex items-center gap-3",
                        div { class: "flex h-11 w-11 items-center justify-center rounded-xl bg-sky-50 text-sky-700",
                            Icon { icon: FaCloud, width: 18, height: 18 }
                        }
                        div {
                            p { class: "text-sm text-slate-500", "绑定云平台" }
                            p { class: "mt-1 text-2xl font-semibold text-slate-900", "{cloud_bound_count}" }
                        }
                    }
                }
                div { class: "rounded-2xl border border-slate-200 bg-white p-5 shadow-sm",
                    div { class: "flex items-center gap-3",
                        div { class: "flex h-11 w-11 items-center justify-center rounded-xl bg-emerald-50 text-emerald-700",
                            Icon { icon: FaServer, width: 18, height: 18 }
                        }
                        div {
                            p { class: "text-sm text-slate-500", "绑定物理机房" }
                            p { class: "mt-1 text-2xl font-semibold text-slate-900", "{machine_bound_count}" }
                        }
                    }
                }
            }

            div { class: "grid grid-cols-1 gap-6 xl:grid-cols-[420px_minmax(0,1fr)]",
                div { class: "flex flex-col gap-6",
                    if can_manage {
                        div { class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm",
                        div { class: "flex items-center gap-3",
                            div { class: "flex h-10 w-10 items-center justify-center rounded-xl bg-slate-100 text-slate-700",
                                Icon { icon: FaPlus, width: 16, height: 16 }
                            }
                            div {
                                h2 { class: "text-lg font-semibold text-slate-900", "新增网络区域" }
                                p { class: "text-sm text-slate-500", "区域必须绑定一个云平台或一个物理机房。" }
                            }
                        }

                        div { class: "mt-5 flex flex-col gap-4",
                            div {
                                label { class: "mb-1 block text-sm font-medium text-slate-700", "区域名称" }
                                input {
                                    r#type: "text",
                                    class: "w-full rounded-xl border border-slate-300 px-4 py-3 text-sm outline-none transition focus:border-cyan-500 focus:ring-4 focus:ring-cyan-100",
                                    value: "{form_name}",
                                    placeholder: "例如 核心云生产网 / 主数据中心办公网",
                                    oninput: move |e| form_name.set(e.value()),
                                }
                            }

                            div {
                                label { class: "mb-1 block text-sm font-medium text-slate-700", "CIDR" }
                                input {
                                    r#type: "text",
                                    class: "w-full rounded-xl border border-slate-300 px-4 py-3 text-sm outline-none transition focus:border-cyan-500 focus:ring-4 focus:ring-cyan-100",
                                    value: "{form_cidr}",
                                    placeholder: "例如 10.10.0.0/16",
                                    oninput: move |e| form_cidr.set(e.value()),
                                }
                            }

                            div {
                                label { class: "mb-1 block text-sm font-medium text-slate-700", "优先级" }
                                input {
                                    r#type: "number",
                                    class: "w-full rounded-xl border border-slate-300 px-4 py-3 text-sm outline-none transition focus:border-cyan-500 focus:ring-4 focus:ring-cyan-100",
                                    value: "{form_priority}",
                                    oninput: move |e| form_priority.set(e.value()),
                                }
                            }

                            div {
                                label { class: "mb-1 block text-sm font-medium text-slate-700", "绑定类型" }
                                select {
                                    class: "w-full rounded-xl border border-slate-300 px-4 py-3 text-sm outline-none transition focus:border-cyan-500 focus:ring-4 focus:ring-cyan-100",
                                    value: "{binding_kind}",
                                    onchange: move |e| {
                                        binding_kind.set(e.value());
                                        form_error.set(String::new());
                                    },
                                    option { value: "cloud", "云平台" }
                                    option { value: "machine", "物理机房" }
                                }
                            }

                            if binding_kind.read().as_str() == "cloud" {
                                div {
                                    label { class: "mb-1 block text-sm font-medium text-slate-700", "云平台" }
                                    select {
                                        class: "w-full rounded-xl border border-slate-300 px-4 py-3 text-sm outline-none transition focus:border-cyan-500 focus:ring-4 focus:ring-cyan-100",
                                        value: "{selected_cloud_platform_value}",
                                        onchange: move |e| {
                                            let value = e.value();
                                            selected_cloud_platform_id.set(value.parse::<i32>().ok());
                                        },
                                        option { value: "", "请选择云平台" }
                                        for platform in cloud_platforms.read().iter() {
                                            option { value: "{platform.id}", "{platform.platform_name}" }
                                        }
                                    }
                                }
                            } else {
                                div {
                                    label { class: "mb-1 block text-sm font-medium text-slate-700", "物理机房" }
                                    select {
                                        class: "w-full rounded-xl border border-slate-300 px-4 py-3 text-sm outline-none transition focus:border-cyan-500 focus:ring-4 focus:ring-cyan-100",
                                        value: "{selected_machine_room_value}",
                                        onchange: move |e| {
                                            let value = e.value();
                                            selected_machine_room_id.set(value.parse::<i32>().ok());
                                        },
                                        option { value: "", "请选择物理机房" }
                                        for room in eligible_machine_rooms.iter() {
                                            option { value: "{room.id}", "{room.room_name}（{room.facility_type}）" }
                                        }
                                    }
                                    p { class: "mt-2 text-xs text-slate-500", "这里展示所有可选物理机房，请按这段 IP 实际归属的网络来选择。" }
                                }
                            }

                            if !form_error.read().is_empty() {
                                div { class: "rounded-xl border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700",
                                    "{form_error}"
                                }
                            }

                            button {
                                class: "inline-flex items-center justify-center gap-2 rounded-xl bg-slate-900 px-4 py-3 text-sm font-medium text-white transition hover:bg-slate-800 disabled:cursor-not-allowed disabled:opacity-60",
                                disabled: *creating.read(),
                                onclick: create_zone_action,
                                Icon { icon: FaPlus, width: 14, height: 14 }
                                if *creating.read() { "创建中..." } else { "创建网络区域" }
                            }
                        }
                    }
                    } else {
                        div { class: "rounded-2xl border border-amber-200 bg-amber-50 p-6 shadow-sm",
                            div { class: "flex items-center gap-3",
                                div { class: "flex h-10 w-10 items-center justify-center rounded-xl bg-amber-100 text-amber-700",
                                    Icon { icon: FaTriangleExclamation, width: 16, height: 16 }
                                }
                                div {
                                    h2 { class: "text-lg font-semibold text-amber-900", "当前为只读模式" }
                                    p { class: "text-sm text-amber-800", "你可以查询和查看网络区域，但创建、删除操作已按权限关闭。" }
                                }
                            }
                        }
                    }

                    div { class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm",
                        div { class: "flex items-center gap-3",
                            div { class: "flex h-10 w-10 items-center justify-center rounded-xl bg-slate-100 text-slate-700",
                                Icon { icon: FaMagnifyingGlass, width: 16, height: 16 }
                            }
                            div {
                                h2 { class: "text-lg font-semibold text-slate-900", "IP 归属查询" }
                                p { class: "text-sm text-slate-500", "输入单个 IP，查看命中的网络区域和对应网段。" }
                            }
                        }

                        div { class: "mt-5 flex flex-col gap-3",
                            label { class: "text-sm font-medium text-slate-700", "IP 地址" }
                            input {
                                r#type: "text",
                                class: "w-full rounded-xl border border-slate-300 bg-white px-4 py-3 text-sm text-slate-900 outline-none transition focus:border-cyan-500 focus:ring-4 focus:ring-cyan-100",
                                placeholder: "例如 192.168.1.100",
                                value: "{search_ip}",
                                oninput: move |e| search_ip.set(e.value()),
                            }
                            button {
                                class: "inline-flex items-center justify-center gap-2 rounded-xl bg-slate-900 px-4 py-3 text-sm font-medium text-white transition hover:bg-slate-800 disabled:cursor-not-allowed disabled:opacity-60",
                                disabled: *search_loading.read(),
                                onclick: find_zone,
                                Icon { icon: FaMagnifyingGlass, width: 14, height: 14 }
                                if *search_loading.read() { "查询中..." } else { "查找区域" }
                            }
                        }

                        if !search_error.read().is_empty() {
                            div { class: "mt-4 rounded-xl border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700",
                                "{search_error}"
                            }
                        }

                        if let Some((zone, cidr)) = found_zone() {
                            div { class: "mt-4 rounded-2xl border border-cyan-200 bg-cyan-50 p-4",
                                p { class: "text-xs font-semibold uppercase tracking-wide text-cyan-700", "查询结果" }
                                div { class: "mt-3 flex flex-col gap-3",
                                    div {
                                        p { class: "text-sm text-slate-500", "所属区域" }
                                        p { class: "text-lg font-semibold text-slate-900", "{zone}" }
                                    }
                                    div {
                                        p { class: "text-sm text-slate-500", "命中 CIDR" }
                                        p { class: "text-base font-medium text-slate-900", "{cidr}" }
                                    }
                                }
                            }
                        }
                    }
                }

                div { class: "rounded-2xl border border-slate-200 bg-white shadow-sm",
                    div { class: "flex items-center justify-between border-b border-slate-200 px-6 py-5",
                        div {
                            h2 { class: "text-lg font-semibold text-slate-900", "区域列表" }
                            p { class: "mt-1 text-sm text-slate-500", "按真实后端当前配置展示区域绑定对象、CIDR 和优先级。" }
                        }
                        span { class: "rounded-full bg-slate-100 px-3 py-1 text-xs font-medium text-slate-600",
                            "{zone_count} 条"
                        }
                    }

                    if *loading.read() {
                        div { class: "px-6 py-16 text-center text-sm text-slate-500", "Loading..." }
                    } else if !error_message.read().is_empty() {
                        div { class: "m-6 rounded-2xl border border-red-200 bg-red-50 p-5 text-sm text-red-700",
                            div { class: "mb-2 flex items-center gap-2 font-medium",
                                Icon { icon: FaTriangleExclamation, width: 14, height: 14 }
                                "加载失败"
                            }
                            p { "{error_message}" }
                        }
                    } else if zones.read().is_empty() {
                        div { class: "px-6 py-16 text-center",
                            div { class: "mx-auto flex h-14 w-14 items-center justify-center rounded-2xl bg-slate-100 text-slate-500",
                                Icon { icon: FaNetworkWired, width: 22, height: 22 }
                            }
                            h3 { class: "mt-4 text-base font-semibold text-slate-900", "暂无网络区域配置" }
                            p { class: "mt-2 text-sm text-slate-500", "你可以在左侧表单中直接创建并绑定到云平台或物理机房。" }
                        }
                    } else {
                        div { class: "overflow-x-auto",
                            table { class: "min-w-full divide-y divide-slate-200",
                                thead { class: "bg-slate-50",
                                    tr {
                                        th { class: "px-6 py-3 text-left text-xs font-semibold uppercase tracking-wide text-slate-500", "区域名称" }
                                        th { class: "px-6 py-3 text-left text-xs font-semibold uppercase tracking-wide text-slate-500", "绑定类型" }
                                        th { class: "px-6 py-3 text-left text-xs font-semibold uppercase tracking-wide text-slate-500", "绑定对象" }
                                        th { class: "px-6 py-3 text-left text-xs font-semibold uppercase tracking-wide text-slate-500", "CIDR" }
                                        th { class: "px-6 py-3 text-left text-xs font-semibold uppercase tracking-wide text-slate-500", "优先级" }
                                        if can_manage {
                                            th { class: "px-6 py-3 text-right text-xs font-semibold uppercase tracking-wide text-slate-500", "操作" }
                                        }
                                    }
                                }
                                tbody { class: "divide-y divide-slate-100 bg-white",
                                    for zone in zones.read().iter() {
                                        {
                                            let (binding_type, binding_name) = zone_binding_label(zone);
                                            let zone_id = zone.id.clone();
                                            let zone_name = zone.name.clone();
                                            let is_deleting = deleting_zone_id.read().as_str() == zone.id.as_str();
                                            rsx! {
                                                tr { class: "hover:bg-slate-50",
                                                    td { class: "px-6 py-4",
                                                        div { class: "font-medium text-slate-900", "{zone.name}" }
                                                    }
                                                    td { class: "px-6 py-4 text-sm text-slate-600", "{binding_type}" }
                                                    td { class: "px-6 py-4 text-sm text-slate-900", "{binding_name}" }
                                                    td { class: "px-6 py-4",
                                                        code { class: "rounded-md bg-slate-100 px-2 py-1 text-sm text-slate-700", "{zone.cidr}" }
                                                    }
                                                    td { class: "px-6 py-4",
                                                        span { class: "rounded-full bg-cyan-50 px-2.5 py-1 text-xs font-semibold text-cyan-700",
                                                            "{zone.priority}"
                                                        }
                                                    }
                                                    if can_manage {
                                                        td { class: "px-6 py-4 text-right",
                                                            button {
                                                                class: "inline-flex items-center gap-2 rounded-lg border border-red-200 px-3 py-2 text-sm font-medium text-red-700 transition hover:bg-red-50 disabled:cursor-not-allowed disabled:opacity-60",
                                                                disabled: is_deleting,
                                                                onclick: move |_| {
                                                                    spawn(delete_zone_action(zone_id.clone(), zone_name.clone()));
                                                                },
                                                                Icon { icon: FaTrash, width: 14, height: 14 }
                                                                if is_deleting { "删除中..." } else { "删除" }
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
                }
            }
        }
    }
}
