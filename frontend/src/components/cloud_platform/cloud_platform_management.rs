use super::platform_form::{FormMode, PlatformForm};
use crate::app::{MACHINE_ROOMS_STATE, PROVIDERS_STATE};
use crate::services::{
    create_cloud_platform_config, delete_cloud_platform_config, fetch_cloud_platform_configs,
    fetch_machine_rooms, fetch_service_providers, update_cloud_platform_config,
};
use crate::state::cloud_platform::CloudPlatformConfig;
use crate::state::user_role::use_auth;
use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaCheck, FaCloud, FaEye, FaMagnifyingGlass, FaPenToSquare, FaPlus, FaPowerOff, FaTrash,
};
use dioxus_free_icons::Icon;

/// 云平台管理页面
#[allow(non_snake_case)]
pub fn CloudPlatformManagement() -> Element {
    let auth = use_auth();
    let current_auth = auth.read().clone();
    let mut search_query = use_signal(String::new);
    let mut status_filter = use_signal(|| "全部".to_string());
    let mut provider_filter = use_signal(|| "全部".to_string());
    let mut cloud_type_filter = use_signal(|| "全部".to_string());
    let mut show_add_modal = use_signal(|| false);
    let mut editing_config = use_signal(|| None::<CloudPlatformConfig>);
    let mut viewing_config = use_signal(|| None::<CloudPlatformConfig>);

    // 数据和加载状态
    let platforms = use_signal(Vec::<CloudPlatformConfig>::new);
    let is_loading = use_signal(|| true);

    // 加载数据 - 包括云平台、服务商和机房
    {
        let mut platforms_clone = platforms;
        let mut is_loading_clone = is_loading;
        use_effect(move || {
            spawn(async move {
                // 加载服务商数据
                match fetch_service_providers().await {
                    Ok(providers) => {
                        *PROVIDERS_STATE.write() = providers;
                    }
                    Err(e) => {
                        tracing::error!("加载服务商数据失败: {}", e);
                    }
                }
                // 加载机房数据
                match fetch_machine_rooms().await {
                    Ok(rooms) => {
                        *MACHINE_ROOMS_STATE.write() = rooms;
                    }
                    Err(e) => {
                        tracing::error!("加载机房数据失败: {}", e);
                    }
                }
                // 加载云平台数据
                match fetch_cloud_platform_configs().await {
                    Ok(data) => {
                        platforms_clone.set(data);
                        is_loading_clone.set(false);
                    }
                    Err(e) => {
                        tracing::error!("加载云平台数据失败: {}", e);
                        is_loading_clone.set(false);
                    }
                }
            });
        });
    }

    // 刷新数据的函数
    let refresh_data = move || {
        let mut platforms = platforms;
        spawn(async move {
            match fetch_cloud_platform_configs().await {
                Ok(data) => {
                    platforms.set(data);
                }
                Err(e) => {
                    tracing::error!("刷新云平台数据失败: {}", e);
                }
            }
        });
    };

    // 获取服务商名称的辅助函数
    let get_provider_name = |provider_id: i32| -> String {
        PROVIDERS_STATE
            .read()
            .iter()
            .find(|p| p.id == provider_id)
            .map(|p| p.short_name.clone())
            .unwrap_or_else(|| "未知服务商".to_string())
    };

    // 获取机房名称的辅助函数
    let get_room_name = |room_id: i32| -> String {
        MACHINE_ROOMS_STATE
            .read()
            .iter()
            .find(|r| r.id == room_id)
            .map(|r| r.room_name.clone())
            .unwrap_or_else(|| "未知机房".to_string())
    };

    // 过滤配置
    let filtered_configs: Vec<CloudPlatformConfig> = platforms
        .read()
        .iter()
        .filter(|config| {
            let query = search_query.read().to_lowercase();
            let status_match =
                status_filter.read().as_str() == "全部" || *status_filter.read() == config.status;
            let provider_match = provider_filter.read().as_str() == "全部"
                || provider_filter.read().parse::<i32>().ok() == Some(config.provider_id);
            let cloud_type_match = cloud_type_filter.read().as_str() == "全部"
                || cloud_type_filter.read().as_str() == config.cloud_type;

            let provider_name = get_provider_name(config.provider_id);
            let room_name = get_room_name(config.machine_room_id);

            status_match
                && provider_match
                && cloud_type_match
                && (query.is_empty()
                    || config.platform_name.to_lowercase().contains(&query)
                    || room_name.to_lowercase().contains(&query)
                    || provider_name.to_lowercase().contains(&query)
                    || config.foundation.to_lowercase().contains(&query))
        })
        .cloned()
        .collect();

    // 计算统计数据
    let total_count = platforms.read().len() as i32;
    let active_count = platforms
        .read()
        .iter()
        .filter(|c| c.status == "active")
        .count() as i32;
    let public_count = platforms
        .read()
        .iter()
        .filter(|c| c.cloud_type == "公有云")
        .count() as i32;
    let gov_count = platforms
        .read()
        .iter()
        .filter(|c| c.cloud_type == "政务云")
        .count() as i32;

    // 动态计算每个服务商的云平台数量
    let provider_stats: Vec<(i32, String, String, i32)> = PROVIDERS_STATE
        .read()
        .iter()
        .map(|provider| {
            let count = platforms
                .read()
                .iter()
                .filter(|c| c.provider_id == provider.id)
                .count() as i32;
            (
                provider.id,
                provider.short_name.clone(),
                provider.provider_name.clone(),
                count,
            )
        })
        .collect();

    // 服务商颜色映射
    let get_provider_color = |id: i32| -> &'static str {
        match id {
            1 => "bg-blue-600",   // 电信 - 蓝色
            2 => "bg-orange-600", // 联通 - 橙色
            3 => "bg-green-600",  // 移动 - 绿色
            4 => "bg-red-600",    // 广电 - 红色
            _ => "bg-gray-600",   // 其他 - 灰色
        }
    };

    let is_empty = filtered_configs.is_empty();

    // 预计算服务商名称和机房名称
    let configs_with_names: Vec<(CloudPlatformConfig, String, String)> = filtered_configs
        .iter()
        .map(|config| {
            let provider_name = get_provider_name(config.provider_id);
            let room_name = get_room_name(config.machine_room_id);
            (config.clone(), provider_name, room_name)
        })
        .collect();

    // 预计算查看配置的数据
    let viewing_config_clone = viewing_config.read().clone();
    let toggle_config_id = viewing_config_clone.as_ref().map(|c| c.id);
    let toggle_status = viewing_config_clone.as_ref().map(|c| c.status.clone());
    let can_manage = current_auth.can_manage_cloud_providers();

    rsx! {
        div { class: "space-y-6",
            // 页面标题和操作栏
            div { class: "flex justify-between items-center",
                div {
                    h1 { class: "text-2xl font-bold text-gray-800", "云平台管理" }
                    p { class: "text-sm text-gray-500 mt-1", "管理市级政务云平台配置" }
                }
                if can_manage {
                    button {
                        class: "flex items-center px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors",
                        onclick: move |_| show_add_modal.set(true),
                        Icon { icon: FaPlus, width: 16, height: 16 }
                        span { class: "ml-2", "添加平台" }
                    }
                }
            }

            if !can_manage {
                div { class: "rounded-lg border border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-800",
                    "当前账号仅可查看云平台配置，新增、编辑、删除和状态切换操作已禁用。"
                }
            }

            // 统计卡片
            div { class: "grid grid-cols-1 md:grid-cols-6 gap-4",
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-blue-500",
                            Icon { icon: FaCloud, width: 20, height: 20, class: "text-white" }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "平台总数" }
                            p { class: "text-xl font-bold text-gray-800", {total_count.to_string()} }
                        }
                    }
                }
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-green-500",
                            Icon { icon: FaCheck, width: 20, height: 20, class: "text-white" }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "运行中" }
                            p { class: "text-xl font-bold text-gray-800", {active_count.to_string()} }
                        }
                    }
                }
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-cyan-500",
                            span { class: "text-white text-xs font-bold", "公有" }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "公有云" }
                            p { class: "text-xl font-bold text-gray-800", {public_count.to_string()} }
                        }
                    }
                }
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-purple-500",
                            span { class: "text-white text-xs font-bold", "政务" }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "政务云" }
                            p { class: "text-xl font-bold text-gray-800", {gov_count.to_string()} }
                        }
                    }
                }
                // 动态生成服务商统计卡片
                for (id, short_name, _provider_name, count) in provider_stats.iter() {
                    div { class: "bg-white rounded-lg shadow p-4",
                        div { class: "flex items-center",
                            div { class: "p-2 rounded-full {get_provider_color(*id)}",
                                span { class: "text-white text-xs font-bold", "{short_name}" }
                            }
                            div { class: "ml-3",
                                p { class: "text-sm text-gray-500", "{short_name}云" }
                                p { class: "text-xl font-bold text-gray-800", "{count.to_string()}" }
                            }
                        }
                    }
                }
            }

            // 搜索和筛选栏
            div { class: "flex gap-4 bg-white rounded-lg shadow p-4",
                div { class: "flex items-center flex-1",
                    Icon { icon: FaMagnifyingGlass, width: 20, height: 20, class: "text-gray-400" }
                    input {
                        r#type: "text",
                        class: "ml-2 flex-1 border-0 focus:outline-none",
                        placeholder: "搜索平台名称、服务商或机房...",
                        value: search_query,
                        oninput: move |e| search_query.set(e.value()),
                    }
                }
                select {
                    class: "px-4 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                    value: status_filter,
                    onchange: move |e| status_filter.set(e.value()),
                    option { value: "全部", "全部状态" }
                    option { value: "active", "运行中" }
                    option { value: "inactive", "已停用" }
                }
                select {
                    class: "px-4 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                    value: provider_filter,
                    onchange: move |e| provider_filter.set(e.value()),
                    option { value: "全部", "全部服务商" }
                    for provider in PROVIDERS_STATE.read().iter() {
                        option { value: "{provider.id}", "{provider.short_name}" }
                    }
                }
                select {
                    class: "px-4 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                    value: cloud_type_filter,
                    onchange: move |e| cloud_type_filter.set(e.value()),
                    option { value: "全部", "全部类型" }
                    option { value: "公有云", "公有云" }
                    option { value: "政务云", "政务云" }
                }
            }

            // 加载状态
            if *is_loading.read() {
                div { class: "bg-white rounded-lg shadow p-12 text-center",
                    div { class: "text-gray-500", "加载数据中..." }
                }
            } else {
                // 配置列表
                div { class: "bg-white rounded-lg shadow overflow-hidden",
                    table { class: "min-w-full divide-y divide-gray-200",
                        thead { class: "bg-gray-50",
                            tr {
                                th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "平台名称" }
                                th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "服务商" }
                                th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "云类型" }
                                th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "云底座" }
                                th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "机房" }
                                th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "AccessKey ID" }
                                th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "状态" }
                                th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "操作" }
                            }
                        }
                        tbody { class: "bg-white divide-y divide-gray-200",
                            if is_empty {
                                tr {
                                    td { class: "px-6 py-12 text-center text-gray-500", colspan: "8",
                                        "暂无数据"
                                    }
                                }
                            } else {
                                for (_idx, (config, provider_name, room_name)) in configs_with_names.iter().enumerate() {
                                    tr {
                                        td { class: "px-6 py-4 whitespace-nowrap",
                                            div { class: "flex items-center",
                                                div { class: "flex-shrink-0 h-10 w-10 bg-blue-100 rounded-full flex items-center justify-center",
                                                    Icon { icon: FaCloud, width: 20, height: 20, class: "text-blue-600" }
                                                }
                                                div { class: "ml-4",
                                                    div { class: "text-sm font-medium text-gray-900", {config.platform_name.clone()} }
                                                }
                                            }
                                        }
                                        td { class: "px-6 py-4 whitespace-nowrap",
                                            span {
                                                class: match config.provider_id {
                                                    1 => "px-2 py-1 text-xs font-medium rounded-full bg-blue-100 text-blue-800",
                                                    2 => "px-2 py-1 text-xs font-medium rounded-full bg-orange-100 text-orange-800",
                                                    3 => "px-2 py-1 text-xs font-medium rounded-full bg-green-100 text-green-800",
                                                    _ => "px-2 py-1 text-xs font-medium rounded-full bg-gray-100 text-gray-800",
                                                },
                                                "{provider_name}"
                                            }
                                        }
                                        td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-900",
                                            span {
                                                class: if config.cloud_type == "公有云" {
                                                    "px-2 py-1 text-xs font-medium rounded-full bg-cyan-100 text-cyan-800"
                                                } else {
                                                    "px-2 py-1 text-xs font-medium rounded-full bg-purple-100 text-purple-800"
                                                },
                                                {config.cloud_type.clone()}
                                            }
                                        }
                                        td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-900",
                                            span {
                                                class: if config.foundation == "阿里云" {
                                                    "px-2 py-1 text-xs font-medium rounded-full bg-orange-100 text-orange-800"
                                                } else {
                                                    "px-2 py-1 text-xs font-medium rounded-full bg-red-100 text-red-800"
                                                },
                                                {config.foundation.clone()}
                                            }
                                        }
                                        td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-600", "{room_name}" }
                                        td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-600 font-mono text-xs",
                                            {config.access_key_id.clone()}
                                        }
                                        td { class: "px-6 py-4 whitespace-nowrap",
                                            span {
                                                class: if config.status == "active" {
                                                    "px-2 py-1 text-xs font-medium rounded-full bg-green-100 text-green-800"
                                                } else {
                                                    "px-2 py-1 text-xs font-medium rounded-full bg-gray-100 text-gray-800"
                                                },
                                                if config.status == "active" { "运行中" } else { "已停用" }
                                            }
                                        }
                                        td { class: "px-6 py-4 whitespace-nowrap text-sm font-medium",
                                            div { class: "flex space-x-2",
                                                button {
                                                    class: "text-blue-600 hover:text-blue-900",
                                                    onclick: {
                                                        let config = config.clone();
                                                        move |_| viewing_config.set(Some(config.clone()))
                                                    },
                                                    Icon { icon: FaEye, width: 16, height: 16 }
                                                }
                                                if can_manage {
                                                    button {
                                                        class: "text-yellow-600 hover:text-yellow-900",
                                                        onclick: {
                                                            let config = config.clone();
                                                            move |_| editing_config.set(Some(config.clone()))
                                                        },
                                                        Icon { icon: FaPenToSquare, width: 16, height: 16 }
                                                    }
                                                    button {
                                                        class: "text-red-600 hover:text-red-900",
                                                        onclick: {
                                                            let config_id = config.id;
                                                            // refresh_data
                                                            move |_| {
                                                                // refresh_data
                                                                spawn(async move {
                                                                    match delete_cloud_platform_config(config_id).await {
                                                                        Ok(()) => {
                                                                            refresh_data();
                                                                        }
                                                                        Err(e) => {
                                                                            tracing::error!("删除云平台失败: {}", e);
                                                                        }
                                                                    }
                                                                });
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
        }

        // 添加平台模态框
        if can_manage && *show_add_modal.read() {
            PlatformForm {
                mode: FormMode::New,
                config: None,
                on_save: move |config: CloudPlatformConfig| {
                    // refresh_data
                    spawn(async move {
                        match create_cloud_platform_config(&config).await {
                            Ok(_) => {
                                refresh_data();
                            }
                            Err(e) => {
                                tracing::error!("创建云平台失败: {}", e);
                            }
                        }
                    });
                    show_add_modal.set(false);
                },
                on_close: move |_| show_add_modal.set(false)
            }
        }

        // 编辑平台模态框
        if can_manage {
            if let Some(config) = editing_config.read().as_ref() {
                PlatformForm {
                    mode: FormMode::Edit,
                    config: Some(config.clone()),
                    on_save: move |updated: CloudPlatformConfig| {
                        // refresh_data
                        spawn(async move {
                            match update_cloud_platform_config(updated.id, &updated).await {
                                Ok(_) => {
                                    refresh_data();
                                }
                                Err(e) => {
                                    tracing::error!("更新云平台失败: {}", e);
                                }
                            }
                        });
                        editing_config.set(None);
                    },
                    on_close: move |_| editing_config.set(None)
                }
            }
        }

        // 查看详情模态框
        if let Some(config) = viewing_config_clone.clone() {
            ConfigDetailModal {
                config: config.clone(),
                can_manage: can_manage,
                on_close: move |_| viewing_config.set(None),
                on_toggle_status: {
                    // refresh_data
                    move |_| {
                        if !can_manage {
                            viewing_config.set(None);
                            return;
                        }
                        if let Some(id) = toggle_config_id {
                            let status = toggle_status.as_deref().unwrap_or("inactive");
                            let new_status = if status == "active" { "inactive" } else { "active" };
                            // refresh_data
                            // Find the config and update it
                            if let Some(config) = platforms.read().iter().find(|c| c.id == id).cloned() {
                                let mut updated = config.clone();
                                updated.status = new_status.to_string();
                                spawn(async move {
                                    match update_cloud_platform_config(id, &updated).await {
                                        Ok(_) => {
                                            refresh_data();
                                        }
                                        Err(e) => {
                                            tracing::error!("更新云平台状态失败: {}", e);
                                        }
                                    }
                                });
                            }
                        }
                        viewing_config.set(None);
                    }
                }
            }
        }
    }
}

/// 配置详情模态框
#[component]
fn ConfigDetailModal(
    config: CloudPlatformConfig,
    can_manage: bool,
    on_close: EventHandler<()>,
    on_toggle_status: EventHandler<()>,
) -> Element {
    // 获取服务商名称
    let provider_name = PROVIDERS_STATE
        .read()
        .iter()
        .find(|p| p.id == config.provider_id)
        .map(|p| p.short_name.clone())
        .unwrap_or_else(|| "未知服务商".to_string());

    // 获取机房名称
    let room_name = MACHINE_ROOMS_STATE
        .read()
        .iter()
        .find(|r| r.id == config.machine_room_id)
        .map(|r| r.room_name.clone())
        .unwrap_or_else(|| "未知机房".to_string());

    rsx! {
        div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            div { class: "bg-white rounded-lg shadow-xl w-full max-w-2xl mx-4",
                div { class: "flex justify-between items-center p-4 border-b",
                    h3 { class: "text-lg font-semibold", "云平台详情" }
                    button {
                        class: "text-gray-400 hover:text-gray-600",
                        onclick: move |_| on_close.call(()),
                        "×"
                    }
                }

                div { class: "p-6 space-y-6",
                    // 平台信息
                    div { class: "flex items-start",
                        div { class: "flex-shrink-0 h-16 w-16 bg-blue-100 rounded-full flex items-center justify-center",
                            Icon { icon: FaCloud, width: 32, height: 32, class: "text-blue-600" }
                        }
                        div { class: "ml-6 flex-1",
                            h2 { class: "text-2xl font-bold text-gray-900", {config.platform_name.clone()} }
                            div { class: "flex items-center mt-2 space-x-2",
                                span {
                                    class: match config.provider_id {
                                        1 => "px-2 py-1 text-xs font-medium rounded-full bg-blue-100 text-blue-800",
                                        2 => "px-2 py-1 text-xs font-medium rounded-full bg-orange-100 text-orange-800",
                                        3 => "px-2 py-1 text-xs font-medium rounded-full bg-green-100 text-green-800",
                                        _ => "px-2 py-1 text-xs font-medium rounded-full bg-gray-100 text-gray-800",
                                    },
                                    {provider_name.clone()}
                                }
                                span { class: "text-gray-500", "·" }
                                span {
                                    class: if config.cloud_type == "公有云" {
                                        "px-2 py-1 text-xs font-medium rounded-full bg-cyan-100 text-cyan-800"
                                    } else {
                                        "px-2 py-1 text-xs font-medium rounded-full bg-purple-100 text-purple-800"
                                    },
                                    {config.cloud_type.clone()}
                                }
                            }
                        }
                    }

                    // 详细信息网格
                    div { class: "grid grid-cols-2 gap-4",
                        div {
                            label { class: "block text-sm text-gray-500", "云底座" }
                            p { class: "text-gray-900", {config.foundation.clone()} }
                        }
                        div {
                            label { class: "block text-sm text-gray-500", "部署机房" }
                            p { class: "text-gray-900", "{room_name}" }
                        }
                        div {
                            label { class: "block text-sm text-gray-500", "Region ID" }
                            p { class: "text-gray-900 font-mono", {config.region_id.clone()} }
                        }
                        div {
                            label { class: "block text-sm text-gray-500", "状态" }
                            p {
                                span {
                                    class: if config.status == "active" {
                                        "px-2 py-1 text-xs font-medium rounded-full bg-green-100 text-green-800"
                                    } else {
                                        "px-2 py-1 text-xs font-medium rounded-full bg-gray-100 text-gray-800"
                                    },
                                    if config.status == "active" { "运行中" } else { "已停用" }
                                }
                            }
                        }
                        div { class: "col-span-2",
                            label { class: "block text-sm text-gray-500", "AccessKey ID" }
                            p { class: "text-gray-900 font-mono text-sm", {config.access_key_id.clone()} }
                        }
                        div { class: "col-span-2",
                            label { class: "block text-sm text-gray-500", "AccessKey Secret" }
                            p { class: "text-gray-900 font-mono text-sm", "••••••••••••••••" }
                        }
                        if let Some(remarks) = &config.remarks {
                            div { class: "col-span-2",
                                label { class: "block text-sm text-gray-500", "备注" }
                                p { class: "text-gray-900", "{remarks}" }
                            }
                        }
                        div {
                            label { class: "block text-sm text-gray-500", "创建时间" }
                            p { class: "text-gray-900", {config.created_at.clone()} }
                        }
                        if let Some(updated) = &config.updated_at {
                            div {
                                label { class: "block text-sm text-gray-500", "更新时间" }
                                p { class: "text-gray-900", "{updated}" }
                            }
                        }
                        div {
                            label { class: "block text-sm text-gray-500", "最近测试结果" }
                            p { class: "text-gray-900",
                                {config.last_test_result.clone().unwrap_or_else(|| "暂无后端测试记录".to_string())}
                            }
                        }
                        div {
                            label { class: "block text-sm text-gray-500", "最近测试时间" }
                            p { class: "text-gray-900",
                                {config.last_test_time.clone().unwrap_or_else(|| "未测试".to_string())}
                            }
                        }
                    }
                }

                div { class: "flex justify-end space-x-3 p-4 border-t bg-gray-50",
                    if can_manage {
                        button {
                            class: if config.status == "active" {
                                "px-4 py-2 text-red-600 border border-red-300 rounded-md hover:bg-red-50"
                            } else {
                                "px-4 py-2 text-green-600 border border-green-300 rounded-md hover:bg-green-50"
                            },
                            onclick: move |_| on_toggle_status.call(()),
                            Icon { icon: FaPowerOff, width: 16, height: 16, class: "mr-2 inline" }
                            if config.status == "active" {
                                "停用"
                            } else {
                                "启用"
                            }
                        }
                    }
                    button {
                        class: "px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700",
                        onclick: move |_| on_close.call(()),
                        "关闭"
                    }
                }
            }
        }
    }
}
