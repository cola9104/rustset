use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaPlus, FaPenToSquare, FaTrash, FaMagnifyingGlass, FaBuilding,
    FaServer, FaDatabase, FaCheck, FaCircleXmark, FaEye, FaCloud
};
use crate::app::CLOUD_PLATFORMS_STATE;
use crate::app::PROVIDERS_STATE;
use crate::app::MACHINE_ROOMS_STATE;

/// 业务应用状态
#[derive(Clone, Debug, PartialEq)]
pub enum AppState {
    Online,
    Offline,
    Maintenance,
}

impl AppState {
    fn as_str(&self) -> &'static str {
        match self {
            AppState::Online => "在线",
            AppState::Offline => "离线",
            AppState::Maintenance => "维护中",
        }
    }

    fn color_class(&self) -> &'static str {
        match self {
            AppState::Online => "bg-green-100 text-green-800",
            AppState::Offline => "bg-gray-100 text-gray-800",
            AppState::Maintenance => "bg-yellow-100 text-yellow-800",
        }
    }
}

/// 业务应用数据模型
#[derive(Clone, Debug, PartialEq)]
pub struct BusinessApplication {
    pub id: i32,
    pub name: String,
    pub app_type: String,
    pub version: String,
    pub status: AppState,
    pub owner: String,
    pub server: String,
    pub port: i32,
    pub url: String,
    pub cloud_platform_id: Option<i32>,   // 关联的云平台ID（云平台部署时有值）
    pub machine_room_id: Option<i32>,     // 关联的机房ID（机房部署时有值）
    pub provider_id: Option<i32>,         // 关联的服务商ID
    pub description: String,
    pub created_at: String,
}

impl BusinessApplication {
    /// 获取服务商名称
    pub fn provider_name(&self) -> String {
        if let Some(pid) = self.provider_id {
            PROVIDERS_STATE.read()
                .iter()
                .find(|p| p.id == pid)
                .map(|p| p.short_name.clone())
                .unwrap_or_else(|| format!("服务商{}", pid))
        } else {
            "未分配".to_string()
        }
    }

    /// 获取云平台名称
    pub fn cloud_platform_name(&self) -> String {
        if let Some(cid) = self.cloud_platform_id {
            CLOUD_PLATFORMS_STATE.read()
                .iter()
                .find(|c| c.id == cid)
                .map(|c| c.foundation.clone())
                .unwrap_or_else(|| format!("云平台{}", cid))
        } else {
            "未分配".to_string()
        }
    }

    /// 获取服务商颜色
    pub fn provider_color(&self) -> &'static str {
        if let Some(pid) = self.provider_id {
            match pid {
                1 => "bg-blue-100 text-blue-800",  // 电信
                2 => "bg-orange-100 text-orange-800", // 联通
                3 => "bg-green-100 text-green-800", // 移动
                4 => "bg-purple-100 text-purple-800", // 广电
                _ => "bg-gray-100 text-gray-800",
            }
        } else {
            "bg-gray-100 text-gray-600"
        }
    }

    /// 获取机房名称
    pub fn machine_room_name(&self) -> String {
        if let Some(mid) = self.machine_room_id {
            MACHINE_ROOMS_STATE.read()
                .iter()
                .find(|m| m.id == mid)
                .map(|m| m.room_name.clone())
                .unwrap_or_else(|| format!("机房{}", mid))
        } else {
            "未分配".to_string()
        }
    }

    /// 判断部署类型
    pub fn deployment_type(&self) -> &'static str {
        if self.cloud_platform_id.is_some() {
            "云平台"
        } else if self.machine_room_id.is_some() {
            "机房"
        } else {
            "未分配"
        }
    }

    /// 获取部署位置名称（云平台或机房）
    pub fn deployment_location_name(&self) -> String {
        if self.cloud_platform_id.is_some() {
            self.cloud_platform_name()
        } else if self.machine_room_id.is_some() {
            self.machine_room_name()
        } else {
            "未分配".to_string()
        }
    }

    /// 是否部署在云平台
    pub fn is_on_cloud(&self) -> bool {
        self.cloud_platform_id.is_some()
    }

    /// 是否部署在机房
    pub fn is_in_machine_room(&self) -> bool {
        self.machine_room_id.is_some()
    }
}

/// 业务应用管理页面
#[allow(non_snake_case)]
pub fn BusinessApplication() -> Element {
    let mut apps = use_signal(|| vec![
        BusinessApplication {
            id: 1,
            name: "OA办公系统".to_string(),
            app_type: "Web应用".to_string(),
            version: "v2.5.1".to_string(),
            status: AppState::Online,
            owner: "信息部".to_string(),
            server: "192.168.1.100".to_string(),
            port: 8080,
            url: "http://oa.company.com".to_string(),
            cloud_platform_id: Some(1),  // 电信-公有云(阿里云)
            machine_room_id: None,
            provider_id: Some(1),         // 电信
            description: "企业办公自动化系统".to_string(),
            created_at: "2024-01-10".to_string(),
        },
        BusinessApplication {
            id: 2,
            name: "ERP企业资源计划".to_string(),
            app_type: "企业应用".to_string(),
            version: "v3.2.0".to_string(),
            status: AppState::Online,
            owner: "信息部".to_string(),
            server: "192.168.1.101".to_string(),
            port: 8443,
            url: "https://erp.company.com".to_string(),
            cloud_platform_id: None,
            machine_room_id: Some(2),     // 核心机房(电信)
            provider_id: Some(1),         // 电信
            description: "企业资源管理系统".to_string(),
            created_at: "2024-01-12".to_string(),
        },
        BusinessApplication {
            id: 3,
            name: "CRM客户关系管理".to_string(),
            app_type: "Web应用".to_string(),
            version: "v1.8.5".to_string(),
            status: AppState::Offline,
            owner: "销售部".to_string(),
            server: "192.168.1.102".to_string(),
            port: 8000,
            url: "http://crm.company.com".to_string(),
            cloud_platform_id: Some(3),  // 联通-公有云(华为云)
            machine_room_id: None,
            provider_id: Some(2),         // 联通
            description: "客户关系管理系统".to_string(),
            created_at: "2024-01-15".to_string(),
        },
        BusinessApplication {
            id: 4,
            name: "数据库主库".to_string(),
            app_type: "数据库".to_string(),
            version: "MySQL 8.0".to_string(),
            status: AppState::Online,
            owner: "信息部".to_string(),
            server: "192.168.1.200".to_string(),
            port: 3306,
            url: "jdbc:mysql://192.168.1.200:3306".to_string(),
            cloud_platform_id: None,
            machine_room_id: Some(4),     // 联通核心机房
            provider_id: Some(2),         // 联通
            description: "核心业务数据库".to_string(),
            created_at: "2024-01-08".to_string(),
        },
        BusinessApplication {
            id: 5,
            name: "文件服务器".to_string(),
            app_type: "文件服务".to_string(),
            version: "v1.0.0".to_string(),
            status: AppState::Maintenance,
            owner: "信息部".to_string(),
            server: "192.168.1.150".to_string(),
            port: 21,
            url: "ftp://file.company.com".to_string(),
            cloud_platform_id: None,
            machine_room_id: Some(6),     // 移动核心机房
            provider_id: Some(3),         // 移动
            description: "企业文件存储服务".to_string(),
            created_at: "2024-01-05".to_string(),
        },
    ]);

    let mut search_query = use_signal(String::new);
    let mut status_filter = use_signal(|| "全部".to_string());
    let mut app_type_filter = use_signal(|| "全部".to_string());
    let mut show_add_modal = use_signal(|| false);
    let mut editing_app = use_signal(|| None::<BusinessApplication>);
    let mut viewing_app = use_signal(|| None::<BusinessApplication>);

    // 统计数据
    let total_count = apps.read().len() as i32;
    let online_count = apps.read().iter().filter(|a| a.status == AppState::Online).count() as i32;
    let offline_count = apps.read().iter().filter(|a| a.status == AppState::Offline).count() as i32;
    let maintenance_count = apps.read().iter().filter(|a| a.status == AppState::Maintenance).count() as i32;

    // 过滤应用
    let filtered_apps: Vec<BusinessApplication> = apps.read()
        .iter()
        .filter(|app| {
            let matches_search = search_query.read().is_empty()
                || app.name.to_lowercase().contains(&search_query.read().to_lowercase())
                || app.app_type.to_lowercase().contains(&search_query.read().to_lowercase())
                || app.server.contains(search_query.read().as_str());

            let matches_status = status_filter.read().as_str() == "全部"
                || match status_filter.read().as_str() {
                    "在线" => app.status == AppState::Online,
                    "离线" => app.status == AppState::Offline,
                    "维护中" => app.status == AppState::Maintenance,
                    _ => true,
                };

            let matches_type = app_type_filter.read().as_str() == "全部"
                || app_type_filter.read().as_str() == app.app_type;

            matches_search && matches_status && matches_type
        })
        .cloned()
        .collect();

    let is_empty = filtered_apps.is_empty();

    rsx! {
        div { class: "space-y-6",
            // 页面标题和操作栏
            div { class: "flex justify-between items-center",
                div {
                    h1 { class: "text-2xl font-bold text-gray-800", "业务应用管理" }
                    p { class: "text-sm text-gray-500 mt-1", "管理系统中的各类业务应用" }
                }
                button {
                    class: "flex items-center px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors",
                    onclick: move |_| show_add_modal.set(true),
                    Icon { icon: FaPlus, width: 16, height: 16 }
                    span { class: "ml-2", "添加应用" }
                }
            }

            // 统计卡片
            div { class: "grid grid-cols-1 md:grid-cols-4 gap-4",
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-blue-500",
                            Icon { icon: FaBuilding, width: 20, height: 20, class: "text-white" }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "总应用" }
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
                            p { class: "text-sm text-gray-500", "在线" }
                            p { class: "text-xl font-bold text-gray-800", {online_count.to_string()} }
                        }
                    }
                }
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-gray-500",
                            Icon { icon: FaCircleXmark, width: 20, height: 20, class: "text-white" }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "离线" }
                            p { class: "text-xl font-bold text-gray-800", {offline_count.to_string()} }
                        }
                    }
                }
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-yellow-500",
                            Icon { icon: FaServer, width: 20, height: 20, class: "text-white" }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "维护中" }
                            p { class: "text-xl font-bold text-gray-800", {maintenance_count.to_string()} }
                        }
                    }
                }
            }

            // 搜索和筛选
            div { class: "bg-white rounded-lg shadow p-4",
                div { class: "flex flex-wrap gap-4",
                    div { class: "relative flex-1 min-w-[200px]",
                        Icon { icon: FaMagnifyingGlass,
                            width: 16,
                            height: 16,
                            class: "absolute left-3 top-1/2 -translate-y-1/2 text-gray-400"
                        }
                        input {
                            r#type: "text",
                            class: "w-full pl-10 pr-4 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            placeholder: "搜索应用名称、类型或服务器...",
                            value: "{search_query}",
                            oninput: move |e| search_query.set(e.value())
                        }
                    }
                    select {
                        class: "px-4 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                        value: status_filter,
                        onchange: move |e| status_filter.set(e.value()),
                        option { value: "全部", "全部状态" }
                        option { value: "在线", "在线" }
                        option { value: "离线", "离线" }
                        option { value: "维护中", "维护中" }
                    }
                    select {
                        class: "px-4 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                        value: app_type_filter,
                        onchange: move |e| app_type_filter.set(e.value()),
                        option { value: "全部", "全部类型" }
                        option { value: "Web应用", "Web应用" }
                        option { value: "企业应用", "企业应用" }
                        option { value: "数据库", "数据库" }
                        option { value: "文件服务", "文件服务" }
                    }
                }
            }

            // 应用列表表格
            div { class: "bg-white rounded-lg shadow overflow-hidden",
                table { class: "min-w-full divide-y divide-gray-200",
                    thead { class: "bg-gray-50",
                        tr {
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "应用名称" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "类型" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "服务商" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "部署位置" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "版本" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "服务器" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "端口" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "状态" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "操作" }
                        }
                    }
                    tbody { class: "bg-white divide-y divide-gray-200",
                        if is_empty {
                            tr {
                                td { colspan: "9", class: "px-6 py-12 text-center text-gray-500",
                                    "没有找到匹配的应用"
                                }
                            }
                        } else {
                            for app in filtered_apps.iter() {
                                tr { class: "hover:bg-gray-50",
                                    td { class: "px-6 py-4 whitespace-nowrap",
                                        div { class: "flex items-center",
                                            div { class: "flex-shrink-0 h-10 w-10 bg-blue-100 rounded-full flex items-center justify-center",
                                                Icon { icon: FaBuilding, width: 20, height: 20, class: "text-blue-600" }
                                            }
                                            div { class: "ml-4",
                                                div { class: "text-sm font-medium text-gray-900", {app.name.clone()} }
                                                div { class: "text-sm text-gray-500", {app.url.clone()} }
                                            }
                                        }
                                    }
                                    td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-900",
                                        {app.app_type.clone()}
                                    }
                                    td { class: "px-6 py-4 whitespace-nowrap",
                                        span {
                                            class: "px-2 inline-flex text-xs leading-5 font-semibold rounded-full {app.provider_color()}",
                                            {app.provider_name()}
                                        }
                                    }
                                    td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-900",
                                        if app.is_on_cloud() {
                                            div { class: "flex items-center gap-1",
                                                Icon { icon: FaCloud, width: 14, height: 14, class: "text-blue-500" }
                                                span { class: "text-gray-600 text-xs mr-1", "云平台" }
                                                {app.cloud_platform_name()}
                                            }
                                        } else if app.is_in_machine_room() {
                                            div { class: "flex items-center gap-1",
                                                Icon { icon: FaServer, width: 14, height: 14, class: "text-orange-500" }
                                                span { class: "text-gray-600 text-xs mr-1", "机房" }
                                                {app.machine_room_name()}
                                            }
                                        } else {
                                            span { class: "text-gray-400", "未分配" }
                                        }
                                    }
                                    td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-500",
                                        {app.version.clone()}
                                    }
                                    td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-900",
                                        {app.server.clone()}
                                    }
                                    td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-500",
                                        {app.port.to_string()}
                                    }
                                    td { class: "px-6 py-4 whitespace-nowrap",
                                        span {
                                            class: "px-2 inline-flex text-xs leading-5 font-semibold rounded-full {app.status.color_class()}",
                                            {app.status.as_str()}
                                        }
                                    }
                                    td { class: "px-6 py-4 whitespace-nowrap text-sm font-medium",
                                        button {
                                            class: "text-green-600 hover:text-green-900 mr-3",
                                            onclick: {
                                                let app = app.clone();
                                                move |_| viewing_app.set(Some(app.clone()))
                                            },
                                            Icon { icon: FaEye, width: 16, height: 16 }
                                        }
                                        button {
                                            class: "text-blue-600 hover:text-blue-900 mr-3",
                                            onclick: {
                                                let app = app.clone();
                                                move |_| editing_app.set(Some(app.clone()))
                                            },
                                            Icon { icon: FaPenToSquare, width: 16, height: 16 }
                                        }
                                        button {
                                            class: "text-red-600 hover:text-red-900",
                                            onclick: {
                                                let app_id = app.id;
                                                move |_| {
                                                    let mut list = apps.write();
                                                    list.retain(|a| a.id != app_id);
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

        // 详情弹窗
        if let Some(app) = viewing_app.read().as_ref() {
            div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
                div { class: "bg-white rounded-lg shadow-xl max-w-2xl w-full mx-4 max-h-[90vh] overflow-y-auto",
                    div { class: "flex items-center justify-between p-6 border-b border-gray-200",
                        h3 { class: "text-lg font-bold text-gray-800", "应用详情" }
                        button {
                            class: "text-gray-400 hover:text-gray-600",
                            onclick: move |_| viewing_app.set(None),
                            "×"
                        }
                    }
                    div { class: "p-6",
                        div { class: "grid grid-cols-2 gap-4",
                            div {
                                p { class: "text-sm text-gray-500", "应用名称" }
                                p { class: "text-base font-semibold text-gray-900", {app.name.clone()} }
                            }
                            div {
                                p { class: "text-sm text-gray-500", "应用类型" }
                                p { class: "text-base text-gray-900", {app.app_type.clone()} }
                            }
                            div {
                                p { class: "text-sm text-gray-500", "版本" }
                                p { class: "text-base text-gray-900", {app.version.clone()} }
                            }
                            div {
                                p { class: "text-sm text-gray-500", "服务商" }
                                p { class: "text-base text-gray-900", {app.provider_name()} }
                            }
                            div {
                                p { class: "text-sm text-gray-500", "部署位置" }
                                p { class: "text-base text-gray-900",
                                    if app.cloud_platform_id.is_some() {
                                        span { class: "flex items-center gap-1",
                                            Icon { icon: FaCloud, width: 14, height: 14, class: "text-blue-500" }
                                            span { class: "text-gray-600 text-xs mr-1", "云平台" }
                                            {app.cloud_platform_name()}
                                        }
                                    } else if app.machine_room_id.is_some() {
                                        span { class: "flex items-center gap-1",
                                            Icon { icon: FaServer, width: 14, height: 14, class: "text-orange-500" }
                                            span { class: "text-gray-600 text-xs mr-1", "机房" }
                                            {app.machine_room_name()}
                                        }
                                    } else {
                                        span { class: "text-gray-400", "未分配" }
                                    }
                                }
                            }
                            div {
                                p { class: "text-sm text-gray-500", "负责人" }
                                p { class: "text-base text-gray-900", {app.owner.clone()} }
                            }
                            div {
                                p { class: "text-sm text-gray-500", "服务器" }
                                p { class: "text-base text-gray-900 font-mono", {app.server.clone()} }
                            }
                            div {
                                p { class: "text-sm text-gray-500", "端口" }
                                p { class: "text-base text-gray-900", {app.port.to_string()} }
                            }
                            div {
                                p { class: "text-sm text-gray-500", "状态" }
                                span {
                                    class: "px-2 inline-flex text-xs leading-5 font-semibold rounded-full {app.status.color_class()}",
                                    {app.status.as_str()}
                                }
                            }
                            div {
                                p { class: "text-sm text-gray-500", "创建时间" }
                                p { class: "text-base text-gray-900", {app.created_at.clone()} }
                            }
                        }
                        div { class: "mt-4",
                            p { class: "text-sm text-gray-500 mb-2", "访问地址" }
                            p { class: "text-base text-blue-600", {app.url.clone()} }
                        }
                        div { class: "mt-4",
                            p { class: "text-sm text-gray-500 mb-2", "描述" }
                            p { class: "text-base text-gray-900", {app.description.clone()} }
                        }
                    }
                }
            }
        }

        // 编辑弹窗
        if let Some(original_app) = editing_app.read().as_ref() {
            div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
                div { class: "bg-white rounded-lg shadow-xl max-w-2xl w-full mx-4 max-h-[90vh] overflow-y-auto",
                    div { class: "flex items-center justify-between p-6 border-b border-gray-200",
                        h3 { class: "text-lg font-bold text-gray-800", "编辑应用" }
                        button {
                            class: "text-gray-400 hover:text-gray-600",
                            onclick: move |_| editing_app.set(None),
                            "×"
                        }
                    }
                    div { class: "p-6",
                        AppForm {
                            app: Some(original_app.clone()),
                            apps: apps,
                            on_close: move |_| editing_app.set(None)
                        }
                    }
                }
            }
        }

        // 添加弹窗
        if *show_add_modal.read() {
            div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
                div { class: "bg-white rounded-lg shadow-xl max-w-2xl w-full mx-4 max-h-[90vh] overflow-y-auto",
                    div { class: "flex items-center justify-between p-6 border-b border-gray-200",
                        h3 { class: "text-lg font-bold text-gray-800", "添加应用" }
                        button {
                            class: "text-gray-400 hover:text-gray-600",
                            onclick: move |_| show_add_modal.set(false),
                            "×"
                        }
                    }
                    div { class: "p-6",
                        AppForm {
                            app: None,
                            apps: apps,
                            on_close: move |_| show_add_modal.set(false)
                        }
                    }
                }
            }
        }
    }
}

/// 应用表单组件（用于添加和编辑）
#[component]
fn AppForm(
    app: Option<BusinessApplication>,
    apps: Signal<Vec<BusinessApplication>>,
    on_close: EventHandler<()>,
) -> Element {
    let mut name = use_signal(|| app.as_ref().map(|a| a.name.clone()).unwrap_or_default());
    let mut app_type = use_signal(|| app.as_ref().map(|a| a.app_type.clone()).unwrap_or_else(|| "Web应用".to_string()));
    let mut version = use_signal(|| app.as_ref().map(|a| a.version.clone()).unwrap_or_default());
    let mut owner = use_signal(|| app.as_ref().map(|a| a.owner.clone()).unwrap_or_else(|| "信息部".to_string()));
    let mut server = use_signal(|| app.as_ref().map(|a| a.server.clone()).unwrap_or_default());
    let mut port = use_signal(|| app.as_ref().map(|a| a.port).unwrap_or(8080));
    let mut url = use_signal(|| app.as_ref().map(|a| a.url.clone()).unwrap_or_default());
    let mut selected_provider = use_signal(|| app.as_ref().and_then(|a| a.provider_id));
    let mut selected_cloud_platform = use_signal(|| app.as_ref().and_then(|a| a.cloud_platform_id));
    let mut selected_machine_room = use_signal(|| app.as_ref().and_then(|a| a.machine_room_id));
    // 部署类型: "cloud" = 云平台, "room" = 机房, "none" = 未分配
    let mut deployment_type = use_signal(|| {
        if app.as_ref().and_then(|a| a.cloud_platform_id).is_some() {
            "cloud".to_string()
        } else if app.as_ref().and_then(|a| a.machine_room_id).is_some() {
            "room".to_string()
        } else {
            "none".to_string()
        }
    });
    let mut status = use_signal(|| app.as_ref().map(|a| a.status.clone()).unwrap_or_else(|| AppState::Online));
    let mut description = use_signal(|| app.as_ref().map(|a| a.description.clone()).unwrap_or_default());
    let mut error_message = use_signal(|| None::<String>);

    let is_editing = app.is_some();
    let providers = PROVIDERS_STATE.read().clone();
    let cloud_platforms = CLOUD_PLATFORMS_STATE.read().clone();
    let machine_rooms = MACHINE_ROOMS_STATE.read().clone();

    // 保存
    let handle_save = move |_| {
        error_message.set(None);

        // 验证
        if name.read().is_empty() {
            error_message.set(Some("应用名称不能为空".to_string()));
            return;
        }

        let provider_id = *selected_provider.read();
        let (cloud_platform_id, machine_room_id) = match deployment_type.read().as_str() {
            "cloud" => (*selected_cloud_platform.read(), None),
            "room" => (None, *selected_machine_room.read()),
            _ => (None, None),
        };

        if is_editing {
            // 更新
            if let Some(original) = app.as_ref() {
                let mut list = apps.write();
                if let Some(app) = list.iter_mut().find(|a| a.id == original.id) {
                    app.name = name.read().clone();
                    app.app_type = app_type.read().clone();
                    app.version = version.read().clone();
                    app.owner = owner.read().clone();
                    app.server = server.read().clone();
                    app.port = *port.read();
                    app.url = url.read().clone();
                    app.provider_id = provider_id;
                    app.cloud_platform_id = cloud_platform_id;
                    app.machine_room_id = machine_room_id;
                    app.status = status.read().clone();
                    app.description = description.read().clone();
                }
            }
        } else {
            // 新增
            let new_id = apps.read().iter().map(|a| a.id).max().unwrap_or(0) + 1;
            let new_app = BusinessApplication {
                id: new_id,
                name: name.read().clone(),
                app_type: app_type.read().clone(),
                version: version.read().clone(),
                status: status.read().clone(),
                owner: owner.read().clone(),
                server: server.read().clone(),
                port: *port.read(),
                url: url.read().clone(),
                cloud_platform_id,
                machine_room_id,
                provider_id,
                description: description.read().clone(),
                created_at: "2024-01-01".to_string(),
            };
            apps.write().push(new_app);
        }

        on_close(());
    };

    let modal_title = if is_editing { "编辑应用" } else { "添加应用" };
    let button_text = if is_editing { "保存" } else { "添加" };

    rsx! {
        div { class: "space-y-4",
            // 错误提示
            if let Some(error) = error_message.read().as_ref() {
                div { class: "p-3 bg-red-100 border border-red-400 text-red-700 rounded-lg",
                    "{error}"
                }
            }

            // 基本信息
            div { class: "grid grid-cols-2 gap-4",
                div {
                    label { class: "block text-sm font-medium text-gray-700 mb-1", "应用名称 *" }
                    input {
                        r#type: "text",
                        class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                        placeholder: "例如: OA办公系统",
                        value: "{name}",
                        oninput: move |e| name.set(e.value())
                    }
                }
                div {
                    label { class: "block text-sm font-medium text-gray-700 mb-1", "应用类型 *" }
                    select {
                        class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                        value: "{app_type}",
                        onchange: move |e| app_type.set(e.value()),
                        option { value: "Web应用", "Web应用" }
                        option { value: "企业应用", "企业应用" }
                        option { value: "数据库", "数据库" }
                        option { value: "文件服务", "文件服务" }
                        option { value: "中间件", "中间件" }
                    }
                }
            }

            div { class: "grid grid-cols-2 gap-4",
                div {
                    label { class: "block text-sm font-medium text-gray-700 mb-1", "版本" }
                    input {
                        r#type: "text",
                        class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                        placeholder: "例如: v1.0.0",
                        value: "{version}",
                        oninput: move |e| version.set(e.value())
                    }
                }
                div {
                    label { class: "block text-sm font-medium text-gray-700 mb-1", "负责人 *" }
                    input {
                        r#type: "text",
                        class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                        placeholder: "例如: 信息部",
                        value: "{owner}",
                        oninput: move |e| owner.set(e.value())
                    }
                }
            }

            // 部署位置选择
            div {
                label { class: "block text-sm font-medium text-gray-700 mb-1", "部署类型" }
                div { class: "flex gap-4",
                    label { class: "flex items-center",
                        input {
                            r#type: "radio",
                            name: "deployment_type",
                            value: "cloud",
                            class: "mr-2",
                            checked: deployment_type.read().as_str() == "cloud",
                            onchange: move |_| deployment_type.set("cloud".to_string())
                        }
                        span { class: "text-sm text-gray-700", "云平台" }
                    }
                    label { class: "flex items-center",
                        input {
                            r#type: "radio",
                            name: "deployment_type",
                            value: "room",
                            class: "mr-2",
                            checked: deployment_type.read().as_str() == "room",
                            onchange: move |_| deployment_type.set("room".to_string())
                        }
                        span { class: "text-sm text-gray-700", "机房" }
                    }
                    label { class: "flex items-center",
                        input {
                            r#type: "radio",
                            name: "deployment_type",
                            value: "none",
                            class: "mr-2",
                            checked: deployment_type.read().as_str() == "none",
                            onchange: move |_| deployment_type.set("none".to_string())
                        }
                        span { class: "text-sm text-gray-700", "未分配" }
                    }
                }
            }

            // 服务商和部署位置
            div { class: "grid grid-cols-2 gap-4",
                div {
                    label { class: "block text-sm font-medium text-gray-700 mb-1", "服务商" }
                    select {
                        class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                        value: "{selected_provider.read().unwrap_or(-1)}",
                        onchange: move |e| {
                            let val: i32 = e.value().parse().unwrap_or(-1);
                            selected_provider.set(if val > 0 { Some(val) } else { None });
                        },
                        option { value: "-1", "未分配" }
                        for provider in providers.iter() {
                            option {
                                value: "{provider.id}",
                                selected: *selected_provider.read() == Some(provider.id),
                                "{provider.short_name}"
                            }
                        }
                    }
                }
                // 根据部署类型显示云平台或机房选择
                if deployment_type.read().as_str() == "cloud" {
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "云平台" }
                        select {
                            class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                            value: "{selected_cloud_platform.read().unwrap_or(-1)}",
                            onchange: move |e| {
                                let val: i32 = e.value().parse().unwrap_or(-1);
                                selected_cloud_platform.set(if val > 0 { Some(val) } else { None });
                            },
                            option { value: "-1", "未选择" }
                            for platform in cloud_platforms.iter() {
                                option {
                                    value: "{platform.id}",
                                    selected: *selected_cloud_platform.read() == Some(platform.id),
                                    "{platform.foundation} - {platform.platform_name}"
                                }
                            }
                        }
                    }
                } else if deployment_type.read().as_str() == "room" {
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "机房" }
                        select {
                            class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                            value: "{selected_machine_room.read().unwrap_or(-1)}",
                            onchange: move |e| {
                                let val: i32 = e.value().parse().unwrap_or(-1);
                                selected_machine_room.set(if val > 0 { Some(val) } else { None });
                            },
                            option { value: "-1", "未选择" }
                            for room in machine_rooms.iter() {
                                option {
                                    value: "{room.id}",
                                    selected: *selected_machine_room.read() == Some(room.id),
                                    "{room.room_name} - {room.room_type}"
                                }
                            }
                        }
                    }
                } else {
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "部署位置" }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-lg bg-gray-100",
                            value: "未分配",
                            disabled: true
                        }
                    }
                }
            }

            // 服务器和端口
            div { class: "grid grid-cols-2 gap-4",
                div {
                    label { class: "block text-sm font-medium text-gray-700 mb-1", "服务器 *" }
                    input {
                        r#type: "text",
                        class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                        placeholder: "例如: 192.168.1.100",
                        value: "{server}",
                        oninput: move |e| server.set(e.value())
                    }
                }
                div {
                    label { class: "block text-sm font-medium text-gray-700 mb-1", "端口 *" }
                    input {
                        r#type: "number",
                        class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                        placeholder: "例如: 8080",
                        value: "{port}",
                        oninput: move |e| {
                            if let Ok(p) = e.value().parse::<i32>() {
                                port.set(p);
                            }
                        }
                    }
                }
            }

            // URL
            div {
                label { class: "block text-sm font-medium text-gray-700 mb-1", "访问地址 *" }
                input {
                    r#type: "text",
                    class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                    placeholder: "例如: http://app.company.com",
                    value: "{url}",
                    oninput: move |e| url.set(e.value())
                }
            }

            // 状态
            div {
                label { class: "block text-sm font-medium text-gray-700 mb-1", "状态" }
                select {
                    class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                    value: "{status.read().as_str()}",
                    onchange: move |e| {
                        match e.value().as_str() {
                            "在线" => status.set(AppState::Online),
                            "离线" => status.set(AppState::Offline),
                            "维护中" => status.set(AppState::Maintenance),
                            _ => {}
                        }
                    },
                    option { value: "在线", "在线" }
                    option { value: "离线", "离线" }
                    option { value: "维护中", "维护中" }
                }
            }

            // 描述
            div {
                label { class: "block text-sm font-medium text-gray-700 mb-1", "描述" }
                textarea {
                    class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                    rows: 3,
                    placeholder: "描述该应用的用途和功能",
                    value: "{description}",
                    oninput: move |e| description.set(e.value())
                }
            }

            // 按钮
            div { class: "flex justify-end gap-3 pt-4 border-t border-gray-200",
                button {
                    class: "px-4 py-2 border border-gray-300 rounded-lg text-gray-700 hover:bg-gray-100",
                    onclick: move |_| on_close(()),
                    "取消"
                }
                button {
                    class: "px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700",
                    onclick: handle_save,
                    "{button_text}"
                }
            }
        }
    }
}

