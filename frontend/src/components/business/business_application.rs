use crate::components::common::{ErrorMessage, Modal, ModalFooter};
use crate::services::business_resource_api::{
    create_application_endpoint, create_business_application, delete_application_endpoint,
    delete_business_application, fetch_business_applications, update_application_endpoint,
    update_business_application, ApplicationEndpointRecord, BusinessApplicationRecord,
};
use crate::state::user_role::use_auth;
use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaDiagramProject, FaGlobe, FaNetworkWired, FaPen, FaPlus, FaServer, FaTrash,
};
use dioxus_free_icons::Icon;
use shared::{
    CreateApplicationEndpointRequest, CreateBusinessApplicationRequest,
    UpdateApplicationEndpointRequest, UpdateBusinessApplicationRequest,
};

#[allow(non_snake_case)]
pub fn BusinessApplication() -> Element {
    let auth = use_auth();
    let can_create = auth
        .read()
        .has_permission("can_create_business_application");
    let can_edit = can_create
        || auth
            .read()
            .has_permission("can_supplement_business_application");
    let can_delete = auth
        .read()
        .has_permission("can_delete_business_application");

    let applications = use_signal(Vec::<BusinessApplicationRecord>::new);
    let mut selected_application_id = use_signal(|| None::<i32>);
    let mut search_query = use_signal(String::new);
    let loading = use_signal(|| true);
    let error = use_signal(String::new);
    let mut success = use_signal(String::new);
    let needs_reload = use_signal(|| true);

    let mut show_create_application = use_signal(|| false);
    let mut editing_application = use_signal(|| None::<BusinessApplicationRecord>);
    let mut show_create_endpoint = use_signal(|| false);
    let mut editing_endpoint = use_signal(|| None::<ApplicationEndpointRecord>);

    {
        let mut applications = applications;
        let mut selected_application_id = selected_application_id;
        let mut loading = loading;
        let mut error = error;
        let mut needs_reload = needs_reload;
        use_effect(move || {
            let should_reload = *needs_reload.read();
            if !should_reload {
                return;
            }

            let current_selected_application_id = *selected_application_id.read();
            needs_reload.set(false);
            spawn(async move {
                loading.set(true);
                match fetch_business_applications().await {
                    Ok(data) => {
                        let next_selected = current_selected_application_id
                            .filter(|id| data.iter().any(|item| item.id == Some(*id)))
                            .or_else(|| data.first().and_then(|item| item.id));
                        applications.set(data);
                        selected_application_id.set(next_selected);
                        error.set(String::new());
                    }
                    Err(err) => {
                        applications.set(Vec::new());
                        selected_application_id.set(None);
                        error.set(err);
                    }
                }
                loading.set(false);
            });
        });
    }

    let query = search_query.read().trim().to_ascii_lowercase();
    let filtered_applications = applications
        .read()
        .iter()
        .filter(|item| {
            query.is_empty()
                || item.name.to_ascii_lowercase().contains(&query)
                || item
                    .description
                    .as_deref()
                    .unwrap_or_default()
                    .to_ascii_lowercase()
                    .contains(&query)
        })
        .cloned()
        .collect::<Vec<_>>();

    let selected_application = selected_application_id.read().and_then(|id| {
        applications
            .read()
            .iter()
            .find(|item| item.id == Some(id))
            .cloned()
    });
    let total_application_count = applications.read().len();
    let total_endpoint_count = applications
        .read()
        .iter()
        .map(|item| item.endpoints.len())
        .sum::<usize>();
    let total_domain_count = applications
        .read()
        .iter()
        .flat_map(|item| item.endpoints.iter())
        .filter(|endpoint| endpoint.domain.as_deref().is_some_and(|value| !value.is_empty()))
        .count();

    rsx! {
        div { class: "space-y-6",
            div { class: "flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between",
                div {
                    h1 { class: "text-2xl font-bold text-gray-800", "业务应用管理" }
                    p { class: "mt-1 text-sm text-gray-500", "应用主档和端点独立维护。互联网地址表示公网访问地址，NAT地址表示内网地址，且 NAT 地址必填。" }
                }
                if can_create {
                    div { class: "flex flex-wrap gap-3",
                        button {
                            class: "inline-flex items-center gap-2 rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700 transition-colors",
                            onclick: move |_| {
                                success.set(String::new());
                                show_create_application.set(true);
                            },
                            Icon { icon: FaPlus, width: 16, height: 16 }
                            "新建应用"
                        }
                    }
                }
            }

            if !success.read().is_empty() {
                div { class: "rounded-lg border border-green-200 bg-green-50 px-4 py-3 text-sm text-green-700",
                    "{success}"
                }
            }

            if !error.read().is_empty() {
                ErrorMessage { message: error.read().clone() }
            }

            div { class: "grid grid-cols-1 gap-4 md:grid-cols-3",
                div { class: "rounded-xl border border-gray-200 bg-white p-5 shadow-sm",
                    div { class: "flex items-center gap-4",
                        div { class: "flex h-11 w-11 items-center justify-center rounded-full bg-blue-500 text-white",
                            Icon { icon: FaDiagramProject, width: 18, height: 18 }
                        }
                        div {
                            p { class: "text-sm text-gray-500", "业务应用数" }
                            p { class: "mt-1 text-2xl font-semibold text-gray-900", "{total_application_count}" }
                        }
                    }
                }
                div { class: "rounded-xl border border-gray-200 bg-white p-5 shadow-sm",
                    div { class: "flex items-center gap-4",
                        div { class: "flex h-11 w-11 items-center justify-center rounded-full bg-cyan-500 text-white",
                            Icon { icon: FaNetworkWired, width: 18, height: 18 }
                        }
                        div {
                            p { class: "text-sm text-gray-500", "应用端点数" }
                            p { class: "mt-1 text-2xl font-semibold text-gray-900", "{total_endpoint_count}" }
                        }
                    }
                }
                div { class: "rounded-xl border border-gray-200 bg-white p-5 shadow-sm",
                    div { class: "flex items-center gap-4",
                        div { class: "flex h-11 w-11 items-center justify-center rounded-full bg-emerald-500 text-white",
                            Icon { icon: FaGlobe, width: 18, height: 18 }
                        }
                        div {
                            p { class: "text-sm text-gray-500", "已配置域名端点" }
                            p { class: "mt-1 text-2xl font-semibold text-gray-900", "{total_domain_count}" }
                        }
                    }
                }
            }

            div { class: "rounded-lg border border-gray-200 bg-white p-4 shadow-sm",
                div { class: "flex items-center gap-3",
                    Icon { icon: FaServer, width: 16, height: 16, class: "text-gray-400" }
                    input {
                        r#type: "text",
                        class: "w-full border-0 p-0 text-sm text-gray-700 placeholder:text-gray-400 focus:outline-none",
                        placeholder: "搜索业务应用名称或说明",
                        value: "{search_query.read()}",
                        oninput: move |e| search_query.set(e.value()),
                    }
                }
            }

            if *loading.read() {
                div { class: "rounded-lg border border-gray-200 bg-white p-8 text-sm text-gray-500 shadow-sm",
                    "正在加载业务应用数据..."
                }
            } else {
                div { class: "grid grid-cols-1 gap-6 xl:grid-cols-[360px_minmax(0,1fr)]",
                    div { class: "rounded-xl border border-gray-200 bg-white shadow-sm",
                        div { class: "border-b border-gray-100 px-5 py-4",
                            h2 { class: "text-base font-semibold text-gray-800", "业务应用列表" }
                            p { class: "mt-1 text-xs text-gray-500", "可以先建应用，再持续给这个应用追加新的端口端点。" }
                        }
                        div { class: "max-h-[720px] overflow-y-auto",
                            if filtered_applications.is_empty() {
                                div { class: "px-5 py-8 text-sm text-gray-500",
                                    if applications.read().is_empty() {
                                        "还没有业务应用。"
                                    } else {
                                        "没有符合搜索条件的业务应用。"
                                    }
                                }
                            } else {
                                for application in filtered_applications {
                                    {
                                        let app_id = application.id.unwrap_or_default();
                                        let app_name = application.name.clone();
                                        let app_name_for_delete = app_name.clone();
                                        let app_for_edit = application.clone();
                                        let is_selected = Some(app_id) == *selected_application_id.read();
                                        rsx! {
                                            div {
                                                class: if is_selected {
                                                    "border-b border-gray-100 bg-blue-50 px-5 py-4 transition-colors"
                                                } else {
                                                    "border-b border-gray-100 px-5 py-4 transition-colors hover:bg-gray-50"
                                                },
                                                div { class: "flex items-start justify-between gap-3",
                                                    button {
                                                        class: "min-w-0 flex-1 text-left",
                                                        onclick: move |_| selected_application_id.set(Some(app_id)),
                                                        div {
                                                            p { class: "text-sm font-semibold text-gray-900", "{application.name}" }
                                                            p { class: "mt-1 text-xs text-gray-500",
                                                                "{application.description.clone().filter(|value| !value.is_empty()).unwrap_or_else(|| \"暂无说明\".to_string())}"
                                                            }
                                                        }
                                                        div { class: "mt-3 flex flex-wrap gap-2 text-xs text-gray-500",
                                                            span { class: "rounded bg-gray-100 px-2 py-1", "端点数：{application.endpoints.len()}" }
                                                            span { class: "rounded bg-gray-100 px-2 py-1", "创建时间：{format_time(&application.created_at)}" }
                                                        }
                                                    }
                                                    if can_edit || can_delete {
                                                        div { class: "flex items-center gap-2",
                                                            if can_edit {
                                                                button {
                                                                    class: "rounded border border-gray-200 p-2 text-gray-600 hover:bg-white hover:text-blue-700",
                                                                    title: "编辑应用",
                                                                    onclick: move |_| editing_application.set(Some(app_for_edit.clone())),
                                                                    Icon { icon: FaPen, width: 14, height: 14 }
                                                                }
                                                            }
                                                            if can_delete {
                                                                button {
                                                                    class: "rounded border border-red-200 p-2 text-red-600 hover:bg-red-50",
                                                                    title: "删除应用",
                                                                    onclick: move |_| {
                                                                        if !confirm_action(&format!("确认删除业务应用“{}”以及其下所有端点吗？", app_name)) {
                                                                            return;
                                                                        }
                                                                        let app_name_for_success = app_name_for_delete.clone();
                                                                        let mut error = error;
                                                                        let mut success = success;
                                                                        let mut selected_application_id = selected_application_id;
                                                                        let mut needs_reload = needs_reload;
                                                                        spawn(async move {
                                                                            match delete_business_application(app_id).await {
                                                                                Ok(_) => {
                                                                                    success.set(format!("业务应用“{}”已删除", app_name_for_success));
                                                                                    error.set(String::new());
                                                                                    if *selected_application_id.read() == Some(app_id) {
                                                                                        selected_application_id.set(None);
                                                                                    }
                                                                                    needs_reload.set(true);
                                                                                }
                                                                                Err(err) => error.set(normalize_error_message(err)),
                                                                            }
                                                                        });
                                                                    },
                                                                    Icon { icon: FaTrash, width: 14, height: 14 }
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

                    div { class: "rounded-xl border border-gray-200 bg-white shadow-sm",
                        if let Some(application) = selected_application {
                            {
                                let application_for_edit = application.clone();
                                rsx! {
                            div {
                                div { class: "border-b border-gray-100 px-6 py-5",
                                    div { class: "flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between",
                                        div {
                                            h2 { class: "text-xl font-semibold text-gray-900", "{application.name}" }
                                            p { class: "mt-2 text-sm text-gray-500",
                                                "{application.description.clone().filter(|value| !value.is_empty()).unwrap_or_else(|| \"暂无业务说明\".to_string())}"
                                            }
                                        }
                                        div { class: "flex flex-wrap items-center gap-2",
                                            span { class: "rounded bg-gray-100 px-2 py-1 text-xs text-gray-500", "端点数：{application.endpoints.len()}" }
                                            span { class: "rounded bg-gray-100 px-2 py-1 text-xs text-gray-500", "创建人：{application.created_by.clone().unwrap_or_else(|| \"未知\".to_string())}" }
                                            span { class: "rounded bg-gray-100 px-2 py-1 text-xs text-gray-500", "创建时间：{format_time(&application.created_at)}" }
                                            if can_edit {
                                                button {
                                                    class: "inline-flex items-center gap-2 rounded border border-gray-200 px-3 py-1.5 text-xs font-medium text-gray-700 hover:bg-gray-50",
                                                    onclick: move |_| editing_application.set(Some(application_for_edit.clone())),
                                                    Icon { icon: FaPen, width: 12, height: 12 }
                                                    "编辑应用"
                                                }
                                            }
                                            if can_create {
                                                button {
                                                    class: "inline-flex items-center gap-2 rounded border border-blue-200 px-3 py-1.5 text-xs font-medium text-blue-700 hover:bg-blue-50",
                                                    onclick: move |_| show_create_endpoint.set(true),
                                                    Icon { icon: FaPlus, width: 12, height: 12 }
                                                    "新增端点"
                                                }
                                            }
                                        }
                                    }
                                }

                                div { class: "px-6 py-5",
                                    h3 { class: "text-sm font-semibold text-gray-800", "应用端点" }
                                    p { class: "mt-1 text-xs text-gray-500", "互联网地址表示公网访问地址，NAT地址表示内网地址且必填。一个应用可以挂多个端点。带域名时按 域名 + 协议 + 端口 识别，不带域名时按 互联网地址 + 协议 + 端口 识别。" }

                                    if application.endpoints.is_empty() {
                                        div { class: "mt-4 rounded-lg border border-dashed border-gray-200 bg-gray-50 px-4 py-8 text-sm text-gray-500",
                                            "当前应用还没有端点。"
                                        }
                                    } else {
                                        div { class: "mt-4 overflow-x-auto",
                                            table { class: "min-w-full divide-y divide-gray-200",
                                                thead { class: "bg-gray-50",
                                                    tr {
                                                        th { class: "px-4 py-3 text-left text-xs font-semibold uppercase tracking-wide text-gray-500", "协议" }
                                                        th { class: "px-4 py-3 text-left text-xs font-semibold uppercase tracking-wide text-gray-500", "互联网地址" }
                                                        th { class: "px-4 py-3 text-left text-xs font-semibold uppercase tracking-wide text-gray-500", "NAT地址" }
                                                        th { class: "px-4 py-3 text-left text-xs font-semibold uppercase tracking-wide text-gray-500", "端口" }
                                                        th { class: "px-4 py-3 text-left text-xs font-semibold uppercase tracking-wide text-gray-500", "域名" }
                                                        th { class: "px-4 py-3 text-left text-xs font-semibold uppercase tracking-wide text-gray-500", "创建信息" }
                                                        if can_edit || can_delete {
                                                            th { class: "px-4 py-3 text-left text-xs font-semibold uppercase tracking-wide text-gray-500", "操作" }
                                                        }
                                                    }
                                                }
                                                tbody { class: "divide-y divide-gray-100 bg-white",
                                                    for endpoint in application.endpoints {
                                                        {
                                                            let endpoint_id = endpoint.id.unwrap_or_default();
                                                            let app_id = application.id.unwrap_or_default();
                                                            let app_name = application.name.clone();
                                                            let endpoint_dest_ip = endpoint.dest_ip.clone();
                                                            let endpoint_dest_port = endpoint.dest_port.clone();
                                                            let endpoint_for_edit = endpoint.clone();
                                                            rsx! {
                                                                tr {
                                                                    td { class: "px-4 py-3 text-sm font-medium text-gray-800", "{endpoint.protocol}" }
                                                                    td { class: "px-4 py-3 text-sm text-gray-700 font-mono", "{endpoint.dest_ip}" }
                                                                    td { class: "px-4 py-3 text-sm text-gray-700 font-mono",
                                                                        "{endpoint.nat_ip.clone().unwrap_or_else(|| \"-\".to_string())}"
                                                                    }
                                                                    td { class: "px-4 py-3 text-sm text-gray-700 font-mono", "{endpoint.dest_port}" }
                                                                    td { class: "px-4 py-3 text-sm text-gray-600",
                                                                        "{endpoint.domain.clone().unwrap_or_else(|| \"-\".to_string())}"
                                                                    }
                                                                    td { class: "px-4 py-3 text-xs text-gray-500",
                                                                        div { "{endpoint.created_by.clone().unwrap_or_else(|| \"未知\".to_string())}" }
                                                                        div { class: "mt-1", "{format_time(&endpoint.updated_at.clone().unwrap_or_else(|| endpoint.created_at.clone()))}" }
                                                                    }
                                                                    if can_edit || can_delete {
                                                                        td { class: "px-4 py-3 text-sm",
                                                                            div { class: "flex items-center gap-2",
                                                                                if can_edit {
                                                                                    button {
                                                                                        class: "rounded border border-gray-200 p-2 text-gray-600 hover:bg-gray-50 hover:text-blue-700",
                                                                                        title: "编辑端点",
                                                                                        onclick: move |_| editing_endpoint.set(Some(endpoint_for_edit.clone())),
                                                                                        Icon { icon: FaPen, width: 14, height: 14 }
                                                                                    }
                                                                                }
                                                                                if can_delete {
                                                                                    button {
                                                                                        class: "rounded border border-red-200 p-2 text-red-600 hover:bg-red-50",
                                                                                        title: "删除端点",
                                                                                        onclick: move |_| {
                                                                                            if !confirm_action(&format!("确认删除应用“{}”下的端点 {}:{} 吗？", app_name, endpoint.dest_ip, endpoint.dest_port)) {
                                                                                                return;
                                                                                            }
                                                                                            let endpoint_dest_ip_for_success = endpoint_dest_ip.clone();
                                                                                            let endpoint_dest_port_for_success = endpoint_dest_port.clone();
                                                                                            let mut error = error;
                                                                                            let mut success = success;
                                                                                            let mut selected_application_id = selected_application_id;
                                                                                            let mut needs_reload = needs_reload;
                                                                                            spawn(async move {
                                                                                                match delete_application_endpoint(endpoint_id).await {
                                                                                                    Ok(_) => {
                                                                                                        success.set(format!("端点 {}:{} 已删除", endpoint_dest_ip_for_success, endpoint_dest_port_for_success));
                                                                                                        error.set(String::new());
                                                                                                        selected_application_id.set(Some(app_id));
                                                                                                        needs_reload.set(true);
                                                                                                    }
                                                                                                    Err(err) => error.set(normalize_error_message(err)),
                                                                                                }
                                                                                            });
                                                                                        },
                                                                                        Icon { icon: FaTrash, width: 14, height: 14 }
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
                            }
                        } else {
                            div { class: "flex min-h-[420px] items-center justify-center px-6 py-10 text-sm text-gray-500",
                                "请选择左侧业务应用查看端点。"
                            }
                        }
                    }
                }
            }

            if *show_create_application.read() {
                BusinessApplicationModal {
                    initial: None,
                    on_close: move |_| show_create_application.set(false),
                    on_save: move |submit: BusinessApplicationFormSubmit| {
                        let mut error = error;
                        let mut success = success;
                        let mut show_create_application = show_create_application;
                        let mut selected_application_id = selected_application_id;
                        let mut needs_reload = needs_reload;
                        spawn(async move {
                            let req = CreateBusinessApplicationRequest {
                                name: submit.name.clone(),
                                description: submit.description.clone(),
                            };
                            match create_business_application(&req).await {
                                Ok(created) => {
                                    success.set(format!("业务应用“{}”已创建", created.name));
                                    error.set(String::new());
                                    selected_application_id.set(created.id);
                                    show_create_application.set(false);
                                    needs_reload.set(true);
                                }
                                Err(err) => error.set(normalize_error_message(err)),
                            }
                        });
                    },
                }
            }

            if let Some(application) = editing_application.read().clone() {
                BusinessApplicationModal {
                    initial: Some(application),
                    on_close: move |_| editing_application.set(None),
                    on_save: move |submit: BusinessApplicationFormSubmit| {
                        let mut error = error;
                        let mut success = success;
                        let mut editing_application = editing_application;
                        let mut selected_application_id = selected_application_id;
                        let mut needs_reload = needs_reload;
                        spawn(async move {
                            let Some(id) = submit.id else {
                                error.set("缺少业务应用 ID".to_string());
                                return;
                            };
                            let req = UpdateBusinessApplicationRequest {
                                name: submit.name.clone(),
                                description: submit.description.clone(),
                            };
                            match update_business_application(id, &req).await {
                                Ok(updated) => {
                                    success.set(format!("业务应用“{}”已更新", updated.name));
                                    error.set(String::new());
                                    selected_application_id.set(updated.id);
                                    editing_application.set(None);
                                    needs_reload.set(true);
                                }
                                Err(err) => error.set(normalize_error_message(err)),
                            }
                        });
                    },
                }
            }

            if *show_create_endpoint.read() {
                ApplicationEndpointModal {
                    applications: applications.read().clone(),
                    initial: None,
                    selected_application_id: *selected_application_id.read(),
                    on_close: move |_| show_create_endpoint.set(false),
                    on_save: move |submit: ApplicationEndpointFormSubmit| {
                        let mut error = error;
                        let mut success = success;
                        let mut show_create_endpoint = show_create_endpoint;
                        let mut selected_application_id = selected_application_id;
                        let mut needs_reload = needs_reload;
                        spawn(async move {
                            let req = CreateApplicationEndpointRequest {
                                business_application_id: submit.business_application_id,
                                protocol: submit.protocol.clone(),
                                dest_ip: submit.dest_ip.clone(),
                                nat_ip: submit.nat_ip.clone(),
                                dest_port: submit.dest_port.clone(),
                                domain: submit.domain.clone(),
                            };
                            match create_application_endpoint(&req).await {
                                Ok(created) => {
                                    success.set(format!(
                                        "端点 {}:{} 已加入应用“{}”",
                                        created.dest_ip,
                                        created.dest_port,
                                        created.business_application_name
                                    ));
                                    error.set(String::new());
                                    selected_application_id.set(Some(created.business_application_id));
                                    show_create_endpoint.set(false);
                                    needs_reload.set(true);
                                }
                                Err(err) => error.set(normalize_error_message(err)),
                            }
                        });
                    },
                }
            }

            if let Some(endpoint) = editing_endpoint.read().clone() {
                ApplicationEndpointModal {
                    applications: applications.read().clone(),
                    initial: Some(endpoint),
                    selected_application_id: *selected_application_id.read(),
                    on_close: move |_| editing_endpoint.set(None),
                    on_save: move |submit: ApplicationEndpointFormSubmit| {
                        let mut error = error;
                        let mut success = success;
                        let mut editing_endpoint = editing_endpoint;
                        let mut selected_application_id = selected_application_id;
                        let mut needs_reload = needs_reload;
                        spawn(async move {
                            let Some(id) = submit.id else {
                                error.set("缺少端点 ID".to_string());
                                return;
                            };
                            let req = UpdateApplicationEndpointRequest {
                                business_application_id: submit.business_application_id,
                                protocol: submit.protocol.clone(),
                                dest_ip: submit.dest_ip.clone(),
                                nat_ip: submit.nat_ip.clone(),
                                dest_port: submit.dest_port.clone(),
                                domain: submit.domain.clone(),
                            };
                            match update_application_endpoint(id, &req).await {
                                Ok(updated) => {
                                    success.set(format!(
                                        "端点 {}:{} 已更新",
                                        updated.dest_ip, updated.dest_port
                                    ));
                                    error.set(String::new());
                                    selected_application_id.set(Some(updated.business_application_id));
                                    editing_endpoint.set(None);
                                    needs_reload.set(true);
                                }
                                Err(err) => error.set(normalize_error_message(err)),
                            }
                        });
                    },
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
struct BusinessApplicationFormSubmit {
    id: Option<i32>,
    name: String,
    description: Option<String>,
}

#[derive(Clone, Debug, Default)]
struct BusinessApplicationFormData {
    id: Option<i32>,
    name: String,
    description: String,
}

impl BusinessApplicationFormData {
    fn from_initial(initial: Option<&BusinessApplicationRecord>) -> Self {
        initial.map_or_else(Self::default, |application| Self {
            id: application.id,
            name: application.name.clone(),
            description: application.description.clone().unwrap_or_default(),
        })
    }

    fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("业务应用名称不能为空".to_string());
        }
        Ok(())
    }

    fn to_submit(&self) -> BusinessApplicationFormSubmit {
        BusinessApplicationFormSubmit {
            id: self.id,
            name: self.name.trim().to_string(),
            description: optional_trimmed(&self.description),
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct BusinessApplicationModalProps {
    initial: Option<BusinessApplicationRecord>,
    on_save: EventHandler<BusinessApplicationFormSubmit>,
    on_close: EventHandler<()>,
}

#[component]
fn BusinessApplicationModal(props: BusinessApplicationModalProps) -> Element {
    let is_edit = props.initial.is_some();
    let mut form_data = use_signal(|| BusinessApplicationFormData::from_initial(props.initial.as_ref()));
    let mut error = use_signal(String::new);

    rsx! {
        Modal {
            show: true,
            title: if is_edit { "编辑业务应用".to_string() } else { "新建业务应用".to_string() },
            size: "xl".to_string(),
            on_close: move |_| props.on_close.call(()),
            footer: rsx! {
                ModalFooter {
                    save_text: if is_edit { "保存".to_string() } else { "创建".to_string() },
                    cancel_text: "取消".to_string(),
                    on_save: move |_| {
                        let data = form_data.read().clone();
                        if let Err(err) = data.validate() {
                            error.set(err);
                            return;
                        }
                        error.set(String::new());
                        props.on_save.call(data.to_submit());
                    },
                    on_cancel: move |_| props.on_close.call(()),
                }
            },
            div { class: "space-y-4",
                if !error.read().is_empty() {
                    ErrorMessage { message: error.read().clone() }
                }
                div {
                    label { class: "mb-1 block text-sm font-medium text-gray-700",
                        "业务应用名称 "
                        span { class: "text-red-500", "*" }
                    }
                    input {
                        r#type: "text",
                        class: "w-full rounded-lg border border-gray-300 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none focus:ring-2 focus:ring-blue-100",
                        placeholder: "如：支付平台、OA 门户、ERP 核心服务",
                        value: "{form_data.read().name}",
                        oninput: move |e| form_data.write().name = e.value(),
                    }
                }
                div {
                    label { class: "mb-1 block text-sm font-medium text-gray-700", "业务说明" }
                    textarea {
                        class: "w-full rounded-lg border border-gray-300 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none focus:ring-2 focus:ring-blue-100",
                        rows: 4,
                        placeholder: "补充该业务应用的用途、负责人或边界说明",
                        value: "{form_data.read().description}",
                        oninput: move |e| form_data.write().description = e.value(),
                    }
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
struct ApplicationEndpointFormSubmit {
    id: Option<i32>,
    business_application_id: i32,
    protocol: String,
    dest_ip: String,
    nat_ip: String,
    dest_port: String,
    domain: Option<String>,
}

#[derive(Clone, Debug)]
struct ApplicationEndpointFormData {
    id: Option<i32>,
    business_application_id: Option<i32>,
    protocol: String,
    dest_ip: String,
    nat_ip: String,
    dest_port: String,
    domain: String,
}

impl ApplicationEndpointFormData {
    fn from_initial(
        initial: Option<&ApplicationEndpointRecord>,
        selected_application_id: Option<i32>,
    ) -> Self {
        if let Some(endpoint) = initial {
            Self {
                id: endpoint.id,
                business_application_id: Some(endpoint.business_application_id),
                protocol: endpoint.protocol.clone(),
                dest_ip: endpoint.dest_ip.clone(),
                nat_ip: endpoint.nat_ip.clone().unwrap_or_default(),
                dest_port: endpoint.dest_port.clone(),
                domain: endpoint.domain.clone().unwrap_or_default(),
            }
        } else {
            Self {
                id: None,
                business_application_id: selected_application_id,
                protocol: "TCP".to_string(),
                dest_ip: String::new(),
                nat_ip: String::new(),
                dest_port: String::new(),
                domain: String::new(),
            }
        }
    }

    fn validate(&self) -> Result<(), String> {
        if self.business_application_id.is_none() {
            return Err("请选择业务应用".to_string());
        }
        if self.protocol.trim().is_empty() {
            return Err("协议不能为空".to_string());
        }
        if self.dest_ip.trim().is_empty() {
            return Err("互联网地址不能为空".to_string());
        }
        if self.nat_ip.trim().is_empty() {
            return Err("NAT地址不能为空".to_string());
        }
        if self.dest_port.trim().is_empty() {
            return Err("目的端口不能为空".to_string());
        }
        Ok(())
    }

    fn to_submit(&self) -> Option<ApplicationEndpointFormSubmit> {
        Some(ApplicationEndpointFormSubmit {
            id: self.id,
            business_application_id: self.business_application_id?,
            protocol: self.protocol.trim().to_ascii_uppercase(),
            dest_ip: self.dest_ip.trim().to_string(),
            nat_ip: self.nat_ip.trim().to_string(),
            dest_port: self.dest_port.trim().to_string(),
            domain: optional_trimmed(&self.domain),
        })
    }
}

#[derive(Props, Clone, PartialEq)]
struct ApplicationEndpointModalProps {
    applications: Vec<BusinessApplicationRecord>,
    initial: Option<ApplicationEndpointRecord>,
    selected_application_id: Option<i32>,
    on_save: EventHandler<ApplicationEndpointFormSubmit>,
    on_close: EventHandler<()>,
}

#[component]
fn ApplicationEndpointModal(props: ApplicationEndpointModalProps) -> Element {
    let is_edit = props.initial.is_some();
    let initial_application_id = props
        .initial
        .as_ref()
        .map(|endpoint| endpoint.business_application_id)
        .or(props.selected_application_id)
        .or_else(|| {
            props
                .applications
                .first()
                .and_then(|application| application.id)
        });
    let has_applications = !props.applications.is_empty();
    let locked_application_name = initial_application_id
        .and_then(|id| {
            props
                .applications
                .iter()
                .find(|application| application.id == Some(id))
                .map(|application| application.name.clone())
        })
        .unwrap_or_else(|| "未选择业务应用".to_string());
    let mut form_data = use_signal(|| {
        ApplicationEndpointFormData::from_initial(props.initial.as_ref(), initial_application_id)
    });
    let mut error = use_signal(String::new);

    rsx! {
        Modal {
            show: true,
            title: if is_edit { "编辑应用端点".to_string() } else { "新增应用端点".to_string() },
            size: "xl".to_string(),
            on_close: move |_| props.on_close.call(()),
            footer: rsx! {
                ModalFooter {
                    save_text: if is_edit { "保存".to_string() } else { "创建".to_string() },
                    cancel_text: "取消".to_string(),
                    save_disabled: !has_applications,
                    on_save: move |_| {
                        if !has_applications {
                            error.set("请先创建业务应用".to_string());
                            return;
                        }

                        let data = form_data.read().clone();
                        if let Err(err) = data.validate() {
                            error.set(err);
                            return;
                        }
                        let Some(request) = data.to_submit() else {
                            error.set("请选择业务应用".to_string());
                            return;
                        };
                        error.set(String::new());
                        props.on_save.call(request);
                    },
                    on_cancel: move |_| props.on_close.call(()),
                }
            },
            div { class: "space-y-4",
                if !error.read().is_empty() {
                    ErrorMessage { message: error.read().clone() }
                }

                if !has_applications {
                    div { class: "rounded-lg border border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-800",
                        "当前没有业务应用，请先创建业务应用。"
                    }
                } else {
                    div {
                        label { class: "mb-1 block text-sm font-medium text-gray-700",
                            "所属业务应用"
                            span { class: "text-red-500", "*" }
                        }
                        input {
                            r#type: "text",
                            class: "w-full rounded-lg border border-gray-200 bg-gray-50 px-3 py-2 text-sm text-gray-500 cursor-not-allowed",
                            value: "{locked_application_name}",
                            readonly: true,
                            disabled: true,
                        }
                    }

                    div { class: "grid grid-cols-1 gap-4 md:grid-cols-2",
                        div {
                            label { class: "mb-1 block text-sm font-medium text-gray-700",
                                "协议 "
                                span { class: "text-red-500", "*" }
                            }
                            select {
                                class: "w-full rounded-lg border border-gray-300 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none focus:ring-2 focus:ring-blue-100",
                                value: "{form_data.read().protocol}",
                                onchange: move |e| form_data.write().protocol = e.value(),
                                option { value: "TCP", "TCP" }
                                option { value: "UDP", "UDP" }
                                option { value: "ICMP", "ICMP" }
                                option { value: "ANY", "ANY" }
                            }
                        }
                        div {
                            label { class: "mb-1 block text-sm font-medium text-gray-700",
                                "目的端口 "
                                span { class: "text-red-500", "*" }
                            }
                            input {
                                r#type: "text",
                                class: "w-full rounded-lg border border-gray-300 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none focus:ring-2 focus:ring-blue-100",
                                placeholder: "如：443、8443、8080-8090",
                                value: "{form_data.read().dest_port}",
                                oninput: move |e| form_data.write().dest_port = e.value(),
                            }
                        }
                    }

                    div {
                        label { class: "mb-1 block text-sm font-medium text-gray-700",
                            "互联网地址 "
                            span { class: "text-red-500", "*" }
                        }
                        input {
                            r#type: "text",
                            class: "w-full rounded-lg border border-gray-300 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none focus:ring-2 focus:ring-blue-100",
                            placeholder: "如：pay.example.com 对应公网 IP，或直接填公网 IP",
                            value: "{form_data.read().dest_ip}",
                            oninput: move |e| form_data.write().dest_ip = e.value(),
                        }
                    }

                    div {
                        label { class: "mb-1 block text-sm font-medium text-gray-700",
                            "NAT地址（内网地址） "
                            span { class: "text-red-500", "*" }
                        }
                        input {
                            r#type: "text",
                            class: "w-full rounded-lg border border-gray-300 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none focus:ring-2 focus:ring-blue-100",
                            placeholder: "如：192.168.1.1",
                            value: "{form_data.read().nat_ip}",
                            oninput: move |e| form_data.write().nat_ip = e.value(),
                        }
                    }

                    div {
                        label { class: "mb-1 block text-sm font-medium text-gray-700", "域名" }
                        input {
                            r#type: "text",
                            class: "w-full rounded-lg border border-gray-300 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none focus:ring-2 focus:ring-blue-100",
                            placeholder: "如：pay.tzdtv.com；不填则按 互联网地址 + 端口 唯一",
                            value: "{form_data.read().domain}",
                            oninput: move |e| form_data.write().domain = e.value(),
                        }
                    }
                }
            }
        }
    }
}

fn confirm_action(message: &str) -> bool {
    web_sys::window()
        .and_then(|window| window.confirm_with_message(message).ok())
        .unwrap_or(false)
}

fn optional_trimmed(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn format_time(value: &str) -> String {
    value
        .split('.')
        .next()
        .unwrap_or(value)
        .replace('T', " ")
        .replace("+00:00", " UTC")
}

fn normalize_error_message(raw: String) -> String {
    serde_json::from_str::<serde_json::Value>(&raw)
        .ok()
        .and_then(|value| {
            value
                .get("message")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string)
                .or_else(|| {
                    value
                        .get("error")
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_string)
                })
        })
        .unwrap_or(raw)
}
