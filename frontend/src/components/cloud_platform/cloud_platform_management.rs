use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaPlus, FaPenToSquare, FaTrash, FaMagnifyingGlass, FaCloud,
    FaCheck, FaKey, FaEye, FaVial, FaPowerOff, FaBuilding,
};
use crate::state::cloud_platform::CloudPlatformConfig;
use crate::state::service_provider::ServiceProviderConfig;
use crate::state::machine_room::MachineRoomConfig;
use crate::app::CLOUD_PLATFORMS_STATE;
use crate::app::PROVIDERS_STATE;
use crate::app::MACHINE_ROOMS_STATE;

/// 云平台管理页面
#[allow(non_snake_case)]
pub fn CloudPlatformManagement() -> Element {
    let mut search_query = use_signal(String::new);
    let mut status_filter = use_signal(|| "全部".to_string());
    let mut provider_filter = use_signal(|| "全部".to_string());
    let mut cloud_type_filter = use_signal(|| "全部".to_string());
    let mut show_add_modal = use_signal(|| false);
    let mut editing_config = use_signal(|| None::<CloudPlatformConfig>);
    let mut viewing_config = use_signal(|| None::<CloudPlatformConfig>);

    // 获取服务商名称的辅助函数
    let get_provider_name = |provider_id: i32| -> String {
        PROVIDERS_STATE.read()
            .iter()
            .find(|p| p.id == provider_id)
            .map(|p| p.short_name.clone())
            .unwrap_or_else(|| "未知服务商".to_string())
    };

    // 获取机房名称的辅助函数
    let get_room_name = |room_id: i32| -> String {
        MACHINE_ROOMS_STATE.read()
            .iter()
            .find(|r| r.id == room_id)
            .map(|r| r.room_name.clone())
            .unwrap_or_else(|| "未知机房".to_string())
    };

    // 过滤配置
    let filtered_configs: Vec<CloudPlatformConfig> = CLOUD_PLATFORMS_STATE.read()
        .iter()
        .filter(|config| {
            let query = search_query.read().to_lowercase();
            let status_match = status_filter.read().as_str() == "全部"
                || *status_filter.read() == config.status;
            let provider_match = provider_filter.read().as_str() == "全部"
                || provider_filter.read().parse::<i32>().ok() == Some(config.provider_id);
            let cloud_type_match = cloud_type_filter.read().as_str() == "全部"
                || cloud_type_filter.read().as_str() == config.cloud_type;

            let provider_name = get_provider_name(config.provider_id);
            let room_name = get_room_name(config.machine_room_id);

            status_match && provider_match && cloud_type_match && (query.is_empty()
                || config.platform_name.to_lowercase().contains(&query)
                || room_name.to_lowercase().contains(&query)
                || provider_name.to_lowercase().contains(&query)
                || config.foundation.to_lowercase().contains(&query))
        })
        .cloned()
        .collect();

    // 计算统计数据
    let total_count = CLOUD_PLATFORMS_STATE.read().len() as i32;
    let active_count = CLOUD_PLATFORMS_STATE.read().iter().filter(|c| c.status == "active").count() as i32;
    let public_count = CLOUD_PLATFORMS_STATE.read().iter().filter(|c| c.cloud_type == "公有云").count() as i32;
    let gov_count = CLOUD_PLATFORMS_STATE.read().iter().filter(|c| c.cloud_type == "政务云").count() as i32;

    // 动态计算每个服务商的云平台数量
    let provider_stats: Vec<(i32, String, String, i32)> = PROVIDERS_STATE.read()
        .iter()
        .map(|provider| {
            let count = CLOUD_PLATFORMS_STATE.read().iter().filter(|c| c.provider_id == provider.id).count() as i32;
            (provider.id, provider.short_name.clone(), provider.provider_name.clone(), count)
        })
        .collect();

    // 服务商颜色映射
    let get_provider_color = |id: i32| -> &'static str {
        match id {
            1 => "bg-blue-600",      // 电信 - 蓝色
            2 => "bg-orange-600",    // 联通 - 橙色
            3 => "bg-green-600",     // 移动 - 绿色
            4 => "bg-red-600",       // 广电 - 红色
            _ => "bg-gray-600",      // 其他 - 灰色
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

    // 计算网格列数：基础4个卡片 + 服务商数量，最多显示6个服务商
    let total_providers = provider_stats.len();
    let grid_cols = if total_providers <= 2 { 6 } else { 4 + total_providers };

    rsx! {
        div { class: "space-y-6",
            // 页面标题和操作栏
            div { class: "flex justify-between items-center",
                div {
                    h1 { class: "text-2xl font-bold text-gray-800", "云平台管理" }
                    p { class: "text-sm text-gray-500 mt-1", "管理市级政务云平台配置" }
                }
                button {
                    class: "flex items-center px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors",
                    onclick: move |_| show_add_modal.set(true),
                    Icon { icon: FaPlus, width: 16, height: 16 }
                    span { class: "ml-2", "添加平台" }
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
                                                    move |_| {
                                                        CLOUD_PLATFORMS_STATE.write().retain(|c| c.id != config_id);
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

        // 添加平台模态框
        if *show_add_modal.read() {
            AddPlatformModal {
                on_close: move |_| show_add_modal.set(false),
                on_save: move |config| {
                    CLOUD_PLATFORMS_STATE.write().push(config);
                    show_add_modal.set(false);
                }
            }
        }

        // 编辑平台模态框
        if let Some(config) = &*editing_config.read() {
            EditPlatformModal {
                config: config.clone(),
                on_close: move |_| editing_config.set(None),
                on_save: move |config: CloudPlatformConfig| {
                    let idx = CLOUD_PLATFORMS_STATE.read().iter().position(|c| c.id == config.id);
                    if let Some(idx) = idx {
                        CLOUD_PLATFORMS_STATE.write()[idx] = config;
                    }
                    editing_config.set(None);
                }
            }
        }

        // 查看详情模态框
        if let Some(ref config) = viewing_config_clone {
            ConfigDetailModal {
                config: (*config).clone(),
                on_close: move |_| viewing_config.set(None),
                on_test: move |_| {
                    // TODO: 实现测试连接逻辑
                },
                on_toggle_status: move |_| {
                    if let Some(id) = toggle_config_id {
                        let status = toggle_status.as_ref().map(|s| s.as_str()).unwrap_or("inactive");
                        CLOUD_PLATFORMS_STATE.write().iter_mut().find(|c| c.id == id).map(|c| {
                            c.status = if status == "active" { "inactive".to_string() } else { "active".to_string() };
                        });
                    }
                    viewing_config.set(None);
                }
            }
        }
    }
}

/// 添加平台模态框
#[component]
fn AddPlatformModal(
    on_close: EventHandler<()>,
    on_save: EventHandler<CloudPlatformConfig>
) -> Element {
    let mut platform_name = use_signal(String::new);
    let mut provider_id = use_signal(|| 1);
    let mut cloud_type = use_signal(|| String::from("公有云"));
    let mut foundation = use_signal(|| String::from("阿里云"));
    let mut region_id = use_signal(String::new);
    let mut machine_room_id = use_signal(|| 1);
    let mut access_key_id = use_signal(String::new);
    let mut access_key_secret = use_signal(String::new);
    let mut remarks = use_signal(String::new);
    let mut status = use_signal(|| String::from("active"));

    // 根据选中的服务商筛选机房
    let filtered_rooms = MACHINE_ROOMS_STATE.read()
        .iter()
        .filter(|r| r.provider_id == *provider_id.read())
        .cloned()
        .collect::<Vec<_>>();

    rsx! {
        div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            div { class: "bg-white rounded-lg shadow-xl w-full max-w-2xl mx-4",
                div { class: "flex justify-between items-center p-4 border-b",
                    h3 { class: "text-lg font-semibold", "添加云平台" }
                    button {
                        class: "text-gray-400 hover:text-gray-600",
                        onclick: move |_| on_close.call(()),
                        "×"
                    }
                }
                div { class: "p-6",
                    div { class: "grid grid-cols-2 gap-4",
                        div { class: "col-span-2",
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "平台名称 *" }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                placeholder: "例如: 电信-公有云（留空自动生成）",
                                value: platform_name,
                                oninput: move |e| platform_name.set(e.value()),
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "服务商 *" }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: "{provider_id}",
                                onchange: move |e| {
                                    if let Ok(id) = e.value().parse::<i32>() {
                                        provider_id.set(id);
                                        // 重置机房选择为该服务商的第一个机房
                                        if let Some(first_room) = MACHINE_ROOMS_STATE.read().iter().find(|r| r.provider_id == id) {
                                            machine_room_id.set(first_room.id);
                                        }
                                        // 如果平台名称为空，自动生成默认名称
                                        if platform_name.read().is_empty() {
                                            if let Some(provider) = PROVIDERS_STATE.read().iter().find(|p| p.id == id) {
                                                let default_name = format!("{}-{}", provider.short_name, cloud_type.read().clone());
                                                platform_name.set(default_name);
                                            }
                                        }
                                    }
                                },
                                for provider in PROVIDERS_STATE.read().iter() {
                                    option { value: "{provider.id}", "{provider.short_name}" }
                                }
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "云类型 *" }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: cloud_type,
                                onchange: move |e| {
                                    let new_cloud_type = e.value();
                                    cloud_type.set(new_cloud_type.clone());
                                    // 如果平台名称为空，自动生成默认名称
                                    if platform_name.read().is_empty() {
                                        if let Some(provider) = PROVIDERS_STATE.read().iter().find(|p| p.id == *provider_id.read()) {
                                            let default_name = format!("{}-{}", provider.short_name, new_cloud_type);
                                            platform_name.set(default_name);
                                        }
                                    }
                                },
                                option { value: "公有云", "公有云" }
                                option { value: "政务云", "政务云" }
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "云底座 *" }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: foundation,
                                onchange: move |e| foundation.set(e.value()),
                                option { value: "阿里云", "阿里云" }
                                option { value: "华为云", "华为云" }
                                option { value: "腾讯云", "腾讯云" }
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "Region ID" }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                placeholder: "例如: cn-hangzhou",
                                value: region_id,
                                oninput: move |e| region_id.set(e.value()),
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "部署机房 *" }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: "{machine_room_id}",
                                onchange: move |e| {
                                    if let Ok(id) = e.value().parse::<i32>() {
                                        machine_room_id.set(id);
                                    }
                                },
                                if filtered_rooms.is_empty() {
                                    option { value: "0", "该服务商暂无机房" }
                                } else {
                                    for room in filtered_rooms.iter() {
                                        option { value: "{room.id}", "{room.room_name}" }
                                    }
                                }
                            }
                        }
                        div { class: "col-span-2",
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "AccessKey ID *" }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 font-mono text-sm",
                                placeholder: "LTAI5t...",
                                value: access_key_id,
                                oninput: move |e| access_key_id.set(e.value()),
                            }
                        }
                        div { class: "col-span-2",
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "AccessKey Secret *" }
                            input {
                                r#type: "password",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 font-mono text-sm",
                                placeholder: "••••••••••••••••",
                                value: access_key_secret,
                                oninput: move |e| access_key_secret.set(e.value()),
                            }
                        }
                        div { class: "col-span-2",
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "备注" }
                            textarea {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                rows: "3",
                                placeholder: "填写备注信息...",
                                value: remarks,
                                oninput: move |e| remarks.set(e.value()),
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "状态" }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: status,
                                onchange: move |e| status.set(e.value()),
                                option { value: "active", "运行中" }
                                option { value: "inactive", "已停用" }
                            }
                        }
                    }
                }
                div { class: "flex justify-end space-x-3 p-4 border-t",
                    button {
                        class: "px-4 py-2 text-gray-600 border border-gray-300 rounded-md hover:bg-gray-50",
                        onclick: move |_| on_close.call(()),
                        "取消"
                    }
                    button {
                        class: "px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700",
                        onclick: move |_| {
                            let new_id = CLOUD_PLATFORMS_STATE.read().iter().map(|c| c.id).max().unwrap_or(0) + 1;
                            let config = CloudPlatformConfig {
                                id: new_id,
                                platform_name: platform_name.read().clone(),
                                provider_id: *provider_id.read(),
                                cloud_type: cloud_type.read().clone(),
                                foundation: foundation.read().clone(),
                                region_id: region_id.read().clone(),
                                machine_room_id: *machine_room_id.read(),
                                access_key_id: access_key_id.read().clone(),
                                access_key_secret: access_key_secret.read().clone(),
                                remarks: if remarks.read().is_empty() { None } else { Some(remarks.read().clone()) },
                                status: status.read().clone(),
                                last_test_time: None,
                                last_test_result: None,
                                created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M").to_string(),
                                updated_at: None,
                            };
                            on_save.call(config);
                        },
                        "保存"
                    }
                }
            }
        }
    }
}

/// 编辑平台模态框
#[component]
fn EditPlatformModal(
    config: CloudPlatformConfig,
    on_close: EventHandler<()>,
    on_save: EventHandler<CloudPlatformConfig>
) -> Element {
    let mut platform_name = use_signal(|| config.platform_name.clone());
    let mut provider_id = use_signal(|| config.provider_id);
    let mut cloud_type = use_signal(|| config.cloud_type.clone());
    let mut foundation = use_signal(|| config.foundation.clone());
    let mut region_id = use_signal(|| config.region_id.clone());
    let mut machine_room_id = use_signal(|| config.machine_room_id);
    let mut access_key_id = use_signal(|| config.access_key_id.clone());
    let mut access_key_secret = use_signal(|| config.access_key_secret.clone());
    let mut remarks = use_signal(|| config.remarks.clone().unwrap_or_default());
    let mut status = use_signal(|| config.status.clone());

    // 当 config prop 变化时更新所有信号
    use_effect(move || {
        platform_name.set(config.platform_name.clone());
        provider_id.set(config.provider_id);
        cloud_type.set(config.cloud_type.clone());
        foundation.set(config.foundation.clone());
        region_id.set(config.region_id.clone());
        machine_room_id.set(config.machine_room_id);
        access_key_id.set(config.access_key_id.clone());
        access_key_secret.set(config.access_key_secret.clone());
        remarks.set(config.remarks.clone().unwrap_or_default());
        status.set(config.status.clone());
    });

    // 根据选中的服务商筛选机房
    let filtered_rooms = MACHINE_ROOMS_STATE.read()
        .iter()
        .filter(|r| r.provider_id == *provider_id.read())
        .cloned()
        .collect::<Vec<_>>();

    rsx! {
        div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            div { class: "bg-white rounded-lg shadow-xl w-full max-w-2xl mx-4",
                div { class: "flex justify-between items-center p-4 border-b",
                    h3 { class: "text-lg font-semibold", "编辑云平台" }
                    button {
                        class: "text-gray-400 hover:text-gray-600",
                        onclick: move |_| on_close.call(()),
                        "×"
                    }
                }
                div { class: "p-6",
                    div { class: "grid grid-cols-2 gap-4",
                        div { class: "col-span-2",
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "平台名称 *" }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: platform_name,
                                oninput: move |e| platform_name.set(e.value()),
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "服务商 *" }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: "{provider_id}",
                                onchange: move |e| {
                                    if let Ok(id) = e.value().parse::<i32>() {
                                        provider_id.set(id);
                                        // 重置机房选择为该服务商的第一个机房
                                        if let Some(first_room) = MACHINE_ROOMS_STATE.read().iter().find(|r| r.provider_id == id) {
                                            machine_room_id.set(first_room.id);
                                        }
                                    }
                                },
                                for provider in PROVIDERS_STATE.read().iter() {
                                    option { value: "{provider.id}", "{provider.short_name}" }
                                }
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "云类型 *" }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: cloud_type,
                                onchange: move |e| cloud_type.set(e.value()),
                                option { value: "公有云", "公有云" }
                                option { value: "政务云", "政务云" }
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "云底座 *" }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: foundation,
                                onchange: move |e| foundation.set(e.value()),
                                option { value: "阿里云", "阿里云" }
                                option { value: "华为云", "华为云" }
                                option { value: "腾讯云", "腾讯云" }
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "Region ID" }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: region_id,
                                oninput: move |e| region_id.set(e.value()),
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "部署机房 *" }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: "{machine_room_id}",
                                onchange: move |e| {
                                    if let Ok(id) = e.value().parse::<i32>() {
                                        machine_room_id.set(id);
                                    }
                                },
                                if filtered_rooms.is_empty() {
                                    option { value: "0", "该服务商暂无机房" }
                                } else {
                                    for room in filtered_rooms.iter() {
                                        option { value: "{room.id}", "{room.room_name}" }
                                    }
                                }
                            }
                        }
                        div { class: "col-span-2",
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "AccessKey ID *" }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 font-mono text-sm",
                                value: access_key_id,
                                oninput: move |e| access_key_id.set(e.value()),
                            }
                        }
                        div { class: "col-span-2",
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "AccessKey Secret *" }
                            input {
                                r#type: "password",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 font-mono text-sm",
                                value: access_key_secret,
                                oninput: move |e| access_key_secret.set(e.value()),
                            }
                        }
                        div { class: "col-span-2",
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "备注" }
                            textarea {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                rows: "3",
                                value: remarks,
                                oninput: move |e| remarks.set(e.value()),
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "状态" }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: status,
                                onchange: move |e| status.set(e.value()),
                                option { value: "active", "运行中" }
                                option { value: "inactive", "已停用" }
                            }
                        }
                    }
                }
                div { class: "flex justify-end space-x-3 p-4 border-t",
                    button {
                        class: "px-4 py-2 text-gray-600 border border-gray-300 rounded-md hover:bg-gray-50",
                        onclick: move |_| on_close.call(()),
                        "取消"
                    }
                    button {
                        class: "px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700",
                        onclick: move |_| {
                            let updated = CloudPlatformConfig {
                                id: config.id,
                                platform_name: platform_name.read().clone(),
                                provider_id: *provider_id.read(),
                                cloud_type: cloud_type.read().clone(),
                                foundation: foundation.read().clone(),
                                region_id: region_id.read().clone(),
                                machine_room_id: *machine_room_id.read(),
                                access_key_id: access_key_id.read().clone(),
                                access_key_secret: access_key_secret.read().clone(),
                                remarks: if remarks.read().is_empty() { None } else { Some(remarks.read().clone()) },
                                status: status.read().clone(),
                                last_test_time: config.last_test_time.clone(),
                                last_test_result: config.last_test_result.clone(),
                                created_at: config.created_at.clone(),
                                updated_at: Some(chrono::Utc::now().format("%Y-%m-%d %H:%M").to_string()),
                            };
                            on_save.call(updated);
                        },
                        "保存"
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
    on_close: EventHandler<()>,
    on_test: EventHandler<()>,
    on_toggle_status: EventHandler<()>
) -> Element {
    // 获取服务商名称
    let provider_name = PROVIDERS_STATE.read()
        .iter()
        .find(|p| p.id == config.provider_id)
        .map(|p| p.short_name.clone())
        .unwrap_or_else(|| "未知服务商".to_string());

    // 获取机房名称
    let room_name = MACHINE_ROOMS_STATE.read()
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
                                span { class: "text-gray-500", "·" }
                                span {
                                    class: if config.foundation == "阿里云" {
                                        "px-2 py-1 text-xs font-medium rounded-full bg-orange-100 text-orange-800"
                                    } else {
                                        "px-2 py-1 text-xs font-medium rounded-full bg-red-100 text-red-800"
                                    },
                                    {config.foundation.clone()}
                                }
                                span { class: "text-gray-500", "·" }
                                span { class: "text-gray-600", "{room_name.clone()}" }
                            }
                        }
                    }

                    div { class: "border-t border-gray-200" }

                    // 详细信息
                    div { class: "grid grid-cols-2 gap-6",
                        div { class: "flex items-center",
                            Icon { icon: FaBuilding, width: 18, height: 18, class: "text-gray-400 mr-3" }
                            div {
                                p { class: "text-sm text-gray-500", "服务商" }
                                p { class: "text-sm font-medium text-gray-900", {provider_name} }
                            }
                        }
                        div { class: "flex items-center",
                            div {
                                p { class: "text-sm text-gray-500", "运行状态" }
                                span {
                                    class: if config.status == "active" {
                                        "px-2 py-1 text-sm font-medium rounded-full bg-green-100 text-green-800"
                                    } else {
                                        "px-2 py-1 text-sm font-medium rounded-full bg-gray-100 text-gray-800"
                                    },
                                    if config.status == "active" { "运行中" } else { "已停用" }
                                }
                            }
                        }
                        div { class: "col-span-2 flex items-center",
                            Icon { icon: FaKey, width: 18, height: 18, class: "text-gray-400 mr-3" }
                            div {
                                p { class: "text-sm text-gray-500", "AccessKey ID" }
                                p { class: "text-sm font-mono text-gray-900", {config.access_key_id.clone()} }
                            }
                        }
                    }

                    // 密钥信息（部分隐藏）
                    div { class: "bg-yellow-50 border border-yellow-200 rounded-lg p-4",
                        div { class: "flex items-center mb-2",
                            Icon { icon: FaVial, width: 18, height: 18, class: "text-yellow-600 mr-2" }
                            h4 { class: "text-sm font-medium text-yellow-800", "AccessKey Secret" }
                        }
                        div { class: "font-mono text-sm text-yellow-900",
                            {if config.access_key_secret.len() > 8 {
                                format!("{}****{}", &config.access_key_secret[..8], "****")
                            } else {
                                "****".to_string()
                            }}
                        }
                    }

                    // 备注
                    if let Some(remarks) = &config.remarks {
                        if !remarks.is_empty() {
                            div {
                                h4 { class: "text-sm font-medium text-gray-700 mb-2", "备注说明" }
                                div { class: "bg-gray-50 rounded-lg p-4 text-sm text-gray-700",
                                    {remarks.clone()}
                                }
                            }
                        }
                    }

                    // 创建和更新时间
                    div { class: "grid grid-cols-2 gap-6 text-sm text-gray-500",
                        div { "创建时间: {config.created_at}" }
                        div {
                            "更新时间: "
                            {config.updated_at.clone().unwrap_or_else(|| "-".to_string())}
                        }
                    }
                }

                // 操作按钮
                div { class: "flex justify-end space-x-3 p-4 border-t",
                    button {
                        class: "px-4 py-2 text-gray-600 border border-gray-300 rounded-md hover:bg-gray-50",
                        onclick: move |_| on_close.call(()),
                        "关闭"
                    }
                    button {
                        class: "px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700 flex items-center",
                        onclick: move |_| on_test.call(()),
                        Icon { icon: FaVial, width: 16, height: 16, class: "mr-2" }
                        "测试连接"
                    }
                    button {
                        class: "px-4 py-2 bg-yellow-600 text-white rounded-md hover:bg-yellow-700 flex items-center",
                        onclick: move |_| on_toggle_status.call(()),
                        Icon { icon: FaPowerOff, width: 16, height: 16, class: "mr-2" }
                        if config.status == "active" { "停用" } else { "启用" }
                    }
                }
            }
        }
    }
}
