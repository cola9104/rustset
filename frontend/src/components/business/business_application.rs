use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaPlus, FaPenToSquare, FaTrash, FaMagnifyingGlass, FaBuilding,
    FaServer, FaDatabase, FaCheck, FaCircleXmark, FaEye
};

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
    pub description: String,
    pub created_at: String,
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

            // 搜索和筛选栏
            div { class: "bg-white rounded-lg shadow p-4",
                div { class: "flex items-center gap-4",
                    div { class: "flex items-center flex-1",
                        Icon { icon: FaMagnifyingGlass, width: 20, height: 20, class: "text-gray-400" }
                        input {
                            r#type: "text",
                            class: "ml-2 flex-1 border-0 focus:outline-none",
                            placeholder: "搜索应用名称、类型或服务器...",
                            value: search_query,
                            oninput: move |e| search_query.set(e.value()),
                        }
                    }
                    select {
                        class: "px-4 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                        value: "{status_filter}",
                        onchange: move |e| status_filter.set(e.value()),
                        option { value: "全部", "全部状态" }
                        option { value: "在线", "在线" }
                        option { value: "离线", "离线" }
                        option { value: "维护中", "维护中" }
                    }
                    select {
                        class: "px-4 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                        value: "{app_type_filter}",
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
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "版本" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "服务器" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "端口" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "负责人" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "状态" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "操作" }
                        }
                    }
                    tbody { class: "bg-white divide-y divide-gray-200",
                        if is_empty {
                            tr {
                                td { colspan: "8", class: "px-6 py-12 text-center text-gray-500",
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
                                    td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-500",
                                        {app.version.clone()}
                                    }
                                    td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-900",
                                        {app.server.clone()}
                                    }
                                    td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-500",
                                        {app.port.to_string()}
                                    }
                                    td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-900",
                                        {app.owner.clone()}
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

        // 添加应用模态框
        if *show_add_modal.read() {
            AddAppModal {
                on_close: move |_| show_add_modal.set(false),
                on_save: move |new_app: BusinessApplication| {
                    apps.write().push(new_app);
                    show_add_modal.set(false);
                }
            }
        }

        // 编辑应用模态框
        if let Some(app) = editing_app.read().clone() {
            EditAppModal {
                app: app.clone(),
                on_close: move |_| editing_app.set(None),
                on_save: move |updated: BusinessApplication| {
                    let mut list = apps.write();
                    if let Some(a) = list.iter_mut().find(|a| a.id == updated.id) {
                        *a = updated;
                    }
                    editing_app.set(None);
                }
            }
        }

        // 查看应用模态框
        if let Some(app) = viewing_app.read().clone() {
            ViewAppModal {
                app: app.clone(),
                on_close: move |_| viewing_app.set(None)
            }
        }
    }
}

/// 添加应用模态框
#[component]
fn AddAppModal(on_close: EventHandler<()>, on_save: EventHandler<BusinessApplication>) -> Element {
    let mut name = use_signal(String::new);
    let mut app_type = use_signal(|| "Web应用".to_string());
    let mut version = use_signal(String::new);
    let mut owner = use_signal(String::new);
    let mut server = use_signal(String::new);
    let mut port = use_signal(|| 8080);
    let mut url = use_signal(String::new);
    let mut description = use_signal(String::new);
    let mut status = use_signal(|| AppState::Online);

    rsx! {
        div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            div { class: "bg-white rounded-lg shadow-xl w-full max-w-md mx-4",
                div { class: "flex justify-between items-center p-4 border-b",
                    h3 { class: "text-lg font-semibold", "添加应用" }
                    button {
                        class: "text-gray-400 hover:text-gray-600",
                        onclick: move |_| on_close.call(()),
                        "×"
                    }
                }

                div { class: "p-4 space-y-4",
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "应用名称" }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            placeholder: "OA办公系统",
                            value: name,
                            oninput: move |e| name.set(e.value()),
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "应用类型" }
                        select {
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: "{app_type}",
                            onchange: move |e| app_type.set(e.value()),
                            option { value: "Web应用", "Web应用" }
                            option { value: "企业应用", "企业应用" }
                            option { value: "数据库", "数据库" }
                            option { value: "文件服务", "文件服务" }
                        }
                    }

                    div { class: "grid grid-cols-2 gap-4",
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "版本" }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                placeholder: "v1.0.0",
                                value: version,
                                oninput: move |e| version.set(e.value()),
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "负责人" }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                placeholder: "信息部",
                                value: owner,
                                oninput: move |e| owner.set(e.value()),
                            }
                        }
                    }

                    div { class: "grid grid-cols-2 gap-4",
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "服务器" }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                placeholder: "192.168.1.100",
                                value: server,
                                oninput: move |e| server.set(e.value()),
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "端口" }
                            input {
                                r#type: "number",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: "{port}",
                                oninput: move |e| {
                                    if let Ok(v) = e.value().parse::<i32>() {
                                        port.set(v);
                                    }
                                }
                            }
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "访问地址" }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            placeholder: "http://app.company.com",
                            value: url,
                            oninput: move |e| url.set(e.value()),
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "描述" }
                        textarea {
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            rows: "2",
                            placeholder: "应用描述信息",
                            value: description,
                            oninput: move |e| description.set(e.value()),
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "状态" }
                        select {
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: if matches!(&*status.read(), AppState::Online) { "在线" }
                                  else if matches!(&*status.read(), AppState::Offline) { "离线" }
                                  else { "维护中" },
                            onchange: move |e| {
                                status.set(match e.value().as_str() {
                                    "在线" => AppState::Online,
                                    "离线" => AppState::Offline,
                                    _ => AppState::Maintenance,
                                });
                            },
                            option { value: "在线", "在线" }
                            option { value: "离线", "离线" }
                            option { value: "维护中", "维护中" }
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
                            let new_app = BusinessApplication {
                                id: uuid::Uuid::new_v4().to_string().parse::<i32>().unwrap_or(0),
                                name: name.read().clone(),
                                app_type: app_type.read().clone(),
                                version: version.read().clone(),
                                owner: owner.read().clone(),
                                server: server.read().clone(),
                                port: *port.read(),
                                url: url.read().clone(),
                                description: description.read().clone(),
                                status: status.read().clone(),
                                created_at: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                            };
                            on_save.call(new_app);
                        },
                        "创建"
                    }
                }
            }
        }
    }
}

/// 编辑应用模态框
#[component]
fn EditAppModal(app: BusinessApplication, on_close: EventHandler<()>, on_save: EventHandler<BusinessApplication>) -> Element {
    let mut name = use_signal(|| app.name.clone());
    let mut app_type = use_signal(|| app.app_type.clone());
    let mut version = use_signal(|| app.version.clone());
    let mut owner = use_signal(|| app.owner.clone());
    let mut server = use_signal(|| app.server.clone());
    let mut port = use_signal(|| app.port);
    let mut url = use_signal(|| app.url.clone());
    let mut description = use_signal(|| app.description.clone());
    let mut status = use_signal(|| app.status.clone());

    rsx! {
        div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            div { class: "bg-white rounded-lg shadow-xl w-full max-w-md mx-4",
                div { class: "flex justify-between items-center p-4 border-b",
                    h3 { class: "text-lg font-semibold", "编辑应用" }
                    button {
                        class: "text-gray-400 hover:text-gray-600",
                        onclick: move |_| on_close.call(()),
                        "×"
                    }
                }

                div { class: "p-4 space-y-4",
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "应用名称" }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: name,
                            oninput: move |e| name.set(e.value()),
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "应用类型" }
                        select {
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: "{app_type}",
                            onchange: move |e| app_type.set(e.value()),
                            option { value: "Web应用", "Web应用" }
                            option { value: "企业应用", "企业应用" }
                            option { value: "数据库", "数据库" }
                            option { value: "文件服务", "文件服务" }
                        }
                    }

                    div { class: "grid grid-cols-2 gap-4",
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "版本" }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: version,
                                oninput: move |e| version.set(e.value()),
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "负责人" }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: owner,
                                oninput: move |e| owner.set(e.value()),
                            }
                        }
                    }

                    div { class: "grid grid-cols-2 gap-4",
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "服务器" }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: server,
                                oninput: move |e| server.set(e.value()),
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "端口" }
                            input {
                                r#type: "number",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: "{port}",
                                oninput: move |e| {
                                    if let Ok(v) = e.value().parse::<i32>() {
                                        port.set(v);
                                    }
                                }
                            }
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "访问地址" }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: url,
                            oninput: move |e| url.set(e.value()),
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "描述" }
                        textarea {
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            rows: "2",
                            value: description,
                            oninput: move |e| description.set(e.value()),
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "状态" }
                        select {
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: if matches!(&*status.read(), AppState::Online) { "在线" }
                                  else if matches!(&*status.read(), AppState::Offline) { "离线" }
                                  else { "维护中" },
                            onchange: move |e| {
                                status.set(match e.value().as_str() {
                                    "在线" => AppState::Online,
                                    "离线" => AppState::Offline,
                                    _ => AppState::Maintenance,
                                });
                            },
                            option { value: "在线", "在线" }
                            option { value: "离线", "离线" }
                            option { value: "维护中", "维护中" }
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
                            let updated = BusinessApplication {
                                id: app.id,
                                name: name.read().clone(),
                                app_type: app_type.read().clone(),
                                version: version.read().clone(),
                                owner: owner.read().clone(),
                                server: server.read().clone(),
                                port: *port.read(),
                                url: url.read().clone(),
                                description: description.read().clone(),
                                status: status.read().clone(),
                                created_at: app.created_at.clone(),
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

/// 查看应用模态框
#[component]
fn ViewAppModal(app: BusinessApplication, on_close: EventHandler<()>) -> Element {
    rsx! {
        div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            div { class: "bg-white rounded-lg shadow-xl w-full max-w-md mx-4",
                div { class: "flex justify-between items-center p-4 border-b",
                    h3 { class: "text-lg font-semibold", "应用详情" }
                    button {
                        class: "text-gray-400 hover:text-gray-600",
                        onclick: move |_| on_close.call(()),
                        "×"
                    }
                }

                div { class: "p-4 space-y-4",
                    div { class: "flex items-center",
                        div { class: "flex-shrink-0 h-12 w-12 bg-blue-100 rounded-full flex items-center justify-center",
                            Icon { icon: FaBuilding, width: 24, height: 24, class: "text-blue-600" }
                        }
                        div { class: "ml-4",
                            h4 { class: "text-lg font-semibold text-gray-900", {app.name.clone()} }
                            p { class: "text-sm text-gray-500", {app.app_type.clone()} }
                        }
                    }

                    div { class: "border-t pt-4 space-y-3",
                        div { class: "grid grid-cols-2 gap-4",
                            div {
                                label { class: "block text-xs text-gray-500", "版本" }
                                p { class: "text-sm text-gray-900", {app.version.clone()} }
                            }
                            div {
                                label { class: "block text-xs text-gray-500", "状态" }
                                span {
                                    class: "inline-flex px-2 py-1 text-xs font-semibold rounded-full {app.status.color_class()}",
                                    {app.status.as_str()}
                                }
                            }
                        }
                        div { class: "grid grid-cols-2 gap-4",
                            div {
                                label { class: "block text-xs text-gray-500", "服务器" }
                                p { class: "text-sm text-gray-900", {app.server.clone()} }
                            }
                            div {
                                label { class: "block text-xs text-gray-500", "端口" }
                                p { class: "text-sm text-gray-900", {app.port.to_string()} }
                            }
                        }
                        div {
                            label { class: "block text-xs text-gray-500", "访问地址" }
                            p { class: "text-sm text-blue-600 break-all", {app.url.clone()} }
                        }
                        div {
                            label { class: "block text-xs text-gray-500", "负责人" }
                            p { class: "text-sm text-gray-900", {app.owner.clone()} }
                        }
                        if !app.description.is_empty() {
                            div {
                                label { class: "block text-xs text-gray-500", "描述" }
                                p { class: "text-sm text-gray-900", {app.description.clone()} }
                            }
                        }
                        div {
                            label { class: "block text-xs text-gray-500", "创建时间" }
                            p { class: "text-sm text-gray-900", {app.created_at.clone()} }
                        }
                    }
                }

                div { class: "flex justify-end p-4 border-t",
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
