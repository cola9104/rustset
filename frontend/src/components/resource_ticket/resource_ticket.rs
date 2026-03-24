use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaArrowLeft, FaBoxOpen, FaCheck, FaCircleCheck, FaCircleXmark, FaClock, FaCloud, FaFile,
    FaFileLines, FaGear, FaMagnifyingGlass, FaPlus, FaServer, FaShieldHalved, FaTicket, FaWrench,
};
use dioxus_free_icons::Icon;

use crate::app::CLOUD_PLATFORMS_STATE;
use crate::app::MACHINE_ROOMS_STATE;
use crate::app::PROVIDERS_STATE;
use crate::app::SECURITY_PRODUCTS_STATE;
use crate::services::resource_ticket_api::{
    approve_ticket, create_resource_ticket, deliver_ticket, fetch_resource_tickets,
    provision_ticket,
};
use crate::state::resource_ticket::{ResourceTicket, ResourceType, TicketStatus};
use crate::state::user_role::{use_auth, ApplicationTab, UserRole};

// 导入三个模块的表单组件和请求类型
use crate::components::resource_ticket::cloud_service::cloud_service_request::CloudServiceRequest;
use crate::components::resource_ticket::cloud_service::CloudServiceForm;
use crate::components::resource_ticket::network_policy::network_policy_request::NetworkPolicyRequest;
use crate::components::resource_ticket::network_policy::NetworkPolicyForm;
use crate::components::resource_ticket::physical_server::physical_server_request::PhysicalServerRequest;
use crate::components::resource_ticket::physical_server::PhysicalServerForm;

fn parse_number_with_suffix(value: &str, suffix: &str) -> i32 {
    value
        .trim()
        .strip_suffix(suffix)
        .unwrap_or(value.trim())
        .trim()
        .parse::<i32>()
        .unwrap_or(0)
}

fn parse_storage_config(value: &str) -> (String, i32) {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return (String::new(), 0);
    }

    let mut parts = trimmed.split_whitespace();
    let size_part = parts.next().unwrap_or_default().to_uppercase();
    let disk_type = parts.collect::<Vec<_>>().join(" ");

    let size_gb = if let Some(size) = size_part.strip_suffix("TB") {
        size.parse::<i32>().unwrap_or(0) * 1024
    } else if let Some(size) = size_part.strip_suffix("GB") {
        size.parse::<i32>().unwrap_or(0)
    } else {
        0
    };

    (disk_type, size_gb)
}

fn is_my_ticket(ticket: &ResourceTicket, current_username: &str) -> bool {
    !current_username.is_empty() && ticket.created_by.eq_ignore_ascii_case(current_username)
}

/// 资源工单主页面 - 基于资源类型的标签页导航 + 工作流程
#[allow(non_snake_case)]
pub fn ResourceTicket() -> Element {
    let auth = use_auth();
    let tickets = use_signal(Vec::new);
    let is_loading = use_signal(|| true);
    let mut resource_type_tab = use_signal(|| ResourceType::Cloud);
    let mut workflow_tab = use_signal(|| {
        auth.read()
            .role
            .accessible_tabs()
            .first()
            .copied()
            .unwrap_or(ApplicationTab::MyApplications)
    });
    let mut search_query = use_signal(String::new);
    let mut selected_ticket = use_signal(|| Option::<i32>::None);
    let mut show_new_form = use_signal(|| false);

    // 组件挂载时从API加载数据
    {
        let mut tickets_clone = tickets;
        let mut is_loading_clone = is_loading;
        use_effect(move || {
            spawn(async move {
                match fetch_resource_tickets().await {
                    Ok(data) => {
                        tickets_clone.set(data);
                        is_loading_clone.set(false);
                    }
                    Err(e) => {
                        tracing::error!("加载工单数据失败: {}", e);
                        // 加载失败时显示空列表
                        tickets_clone.set(Vec::new());
                        is_loading_clone.set(false);
                    }
                }
            });
        });
    }

    let accessible_tabs = auth.read().role.accessible_tabs();
    let current_role = auth.read().role;

    // 计算各资源类型的待处理数量（待审批+待配置+待交付）
    let all_tickets = tickets.read().clone();

    let cloud_pending_count = all_tickets
        .iter()
        .filter(|t| {
            t.resource_type == ResourceType::Cloud
                && (t.ticket_status == TicketStatus::PendingApproval
                    || t.ticket_status == TicketStatus::PendingProvision
                    || t.ticket_status == TicketStatus::PendingDelivery)
        })
        .count();

    let physical_pending_count = all_tickets
        .iter()
        .filter(|t| {
            t.resource_type == ResourceType::Physical
                && (t.ticket_status == TicketStatus::PendingApproval
                    || t.ticket_status == TicketStatus::PendingProvision
                    || t.ticket_status == TicketStatus::PendingDelivery)
        })
        .count();

    let network_pending_count = all_tickets
        .iter()
        .filter(|t| {
            t.resource_type == ResourceType::Network
                && (t.ticket_status == TicketStatus::PendingApproval
                    || t.ticket_status == TicketStatus::PendingProvision
                    || t.ticket_status == TicketStatus::PendingDelivery)
        })
        .count();

    // 计算当前资源类型的统计数据
    let current_type_tickets = all_tickets
        .iter()
        .filter(|t| t.resource_type == *resource_type_tab.read())
        .cloned()
        .collect::<Vec<_>>();

    let total_count = current_type_tickets.len();
    let pending_count = current_type_tickets
        .iter()
        .filter(|t| t.ticket_status == TicketStatus::PendingApproval)
        .count();
    let pending_provision_count = current_type_tickets
        .iter()
        .filter(|t| t.ticket_status == TicketStatus::PendingProvision)
        .count();
    let pending_delivery_count = current_type_tickets
        .iter()
        .filter(|t| t.ticket_status == TicketStatus::PendingDelivery)
        .count();
    let delivered_count = current_type_tickets
        .iter()
        .filter(|t| t.ticket_status == TicketStatus::Delivered)
        .count();
    let archived_count = current_type_tickets
        .iter()
        .filter(|t| t.ticket_status == TicketStatus::Archived)
        .count();

    rsx! {
        div { class: "flex flex-col h-full bg-gray-50",
            // 顶部标题栏
            div { class: "bg-white border-b border-gray-200 px-6 py-4",
                div { class: "flex items-center justify-between",
                    div { class: "flex items-center gap-3",
                        Icon { icon: FaTicket, class: "text-blue-600 text-2xl" }
                        div {
                            h1 { class: "text-xl font-bold text-gray-800", "资源工单管理" }
                            p { class: "text-sm text-gray-500 mt-0.5",
                                "当前角色: {current_role.display_name()}"
                            }
                        }
                    }

                    span { class: "text-sm text-gray-500", "登录角色由后端会话决定" }
                }
            }

            // 资源类型标签页导航（外层）
            div { class: "bg-gradient-to-r from-white to-gray-50 border-b border-gray-200 px-6 py-2 shadow-sm overflow-visible",
                div { class: "flex items-center justify-between overflow-x-auto overflow-y-visible",
                    div { class: "flex gap-2",
                        button {
                            class: format!(
                                "px-5 py-3 text-sm font-semibold rounded-t-lg whitespace-nowrap transition-all duration-200 flex items-center gap-2.5 relative {}",
                                if *resource_type_tab.read() == ResourceType::Cloud {
                                    "bg-gradient-to-r from-blue-500 to-blue-600 text-white shadow-lg shadow-blue-200"
                                } else {
                                    "text-gray-600 hover:bg-white hover:text-blue-600 hover:shadow-md"
                                }
                            ),
                            onclick: move |_| {
                                resource_type_tab.set(ResourceType::Cloud);
                                selected_ticket.set(None);
                                show_new_form.set(false);
                            },
                            Icon { icon: FaCloud, width: 16, height: 16, class: if *resource_type_tab.read() == ResourceType::Cloud { "text-white" } else { "text-gray-400" } }
                            "云资源"
                            if cloud_pending_count > 0 {
                                span { class: "absolute top-0 -right-3 h-5 min-w-[20px] px-1.5 flex items-center justify-center rounded-full bg-red-500 text-[11px] font-bold text-white shadow-md z-50 border-2 border-white",
                                    {cloud_pending_count.to_string()}
                                }
                            }
                        }
                        button {
                            class: format!(
                                "px-5 py-3 text-sm font-semibold rounded-t-lg whitespace-nowrap transition-all duration-200 flex items-center gap-2.5 relative {}",
                                if *resource_type_tab.read() == ResourceType::Physical {
                                    "bg-gradient-to-r from-blue-500 to-blue-600 text-white shadow-lg shadow-blue-200"
                                } else {
                                    "text-gray-600 hover:bg-white hover:text-blue-600 hover:shadow-md"
                                }
                            ),
                            onclick: move |_| {
                                resource_type_tab.set(ResourceType::Physical);
                                selected_ticket.set(None);
                                show_new_form.set(false);
                            },
                            Icon { icon: FaServer, width: 16, height: 16, class: if *resource_type_tab.read() == ResourceType::Physical { "text-white" } else { "text-gray-400" } }
                            "物理资源"
                            if physical_pending_count > 0 {
                                span { class: "absolute top-0 -right-3 h-5 min-w-[20px] px-1.5 flex items-center justify-center rounded-full bg-red-500 text-[11px] font-bold text-white shadow-md z-50 border-2 border-white",
                                    {physical_pending_count.to_string()}
                                }
                            }
                        }
                        button {
                            class: format!(
                                "px-5 py-3 text-sm font-semibold rounded-t-lg whitespace-nowrap transition-all duration-200 flex items-center gap-2.5 relative {}",
                                if *resource_type_tab.read() == ResourceType::Network {
                                    "bg-gradient-to-r from-blue-500 to-blue-600 text-white shadow-lg shadow-blue-200"
                                } else {
                                    "text-gray-600 hover:bg-white hover:text-blue-600 hover:shadow-md"
                                }
                            ),
                            onclick: move |_| {
                                resource_type_tab.set(ResourceType::Network);
                                selected_ticket.set(None);
                                show_new_form.set(false);
                            },
                            Icon { icon: FaShieldHalved, width: 16, height: 16, class: if *resource_type_tab.read() == ResourceType::Network { "text-white" } else { "text-gray-400" } }
                            "网络策略"
                            if network_pending_count > 0 {
                                span { class: "absolute top-0 -right-3 h-5 min-w-[20px] px-1.5 flex items-center justify-center rounded-full bg-red-500 text-[11px] font-bold text-white shadow-md z-50 border-2 border-white",
                                    {network_pending_count.to_string()}
                                }
                            }
                        }
                    }

                    // 新建工单按钮
                    if current_role.can_submit() {
                        button {
                            class: "px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 flex items-center gap-2 transition-colors shadow-sm",
                            onclick: move |_| show_new_form.set(true),
                            Icon { icon: FaPlus, class: "text-sm" }
                            "新建工单"
                        }
                    }
                }
            }

            // 统计概览卡片（移到资源类型标签页下面）
            div { class: "bg-gray-50 border-b border-gray-200 px-6 py-4",
                div { class: "grid grid-cols-1 md:grid-cols-6 gap-4",
                    div { class: "bg-white rounded-lg shadow p-4",
                        div { class: "flex items-center",
                            div { class: "p-2 rounded-full bg-blue-500",
                                Icon { icon: FaTicket, width: 20, height: 20, class: "text-white" }
                            }
                            div { class: "ml-3",
                                p { class: "text-sm text-gray-500", "总工单" }
                                p { class: "text-xl font-bold text-gray-800", {total_count.to_string()} }
                            }
                        }
                    }
                    div { class: "bg-white rounded-lg shadow p-4",
                        div { class: "flex items-center",
                            div { class: "p-2 rounded-full bg-yellow-500",
                                Icon { icon: FaClock, width: 20, height: 20, class: "text-white" }
                            }
                            div { class: "ml-3",
                                p { class: "text-sm text-gray-500", "待审批" }
                                p { class: "text-xl font-bold text-gray-800", {pending_count.to_string()} }
                            }
                        }
                    }
                    div { class: "bg-white rounded-lg shadow p-4",
                        div { class: "flex items-center",
                            div { class: "p-2 rounded-full bg-purple-500",
                                Icon { icon: FaWrench, width: 20, height: 20, class: "text-white" }
                            }
                            div { class: "ml-3",
                                p { class: "text-sm text-gray-500", "待配置" }
                                p { class: "text-xl font-bold text-gray-800", {pending_provision_count.to_string()} }
                            }
                        }
                    }
                    div { class: "bg-white rounded-lg shadow p-4",
                        div { class: "flex items-center",
                            div { class: "p-2 rounded-full bg-orange-500",
                                Icon { icon: FaBoxOpen, width: 20, height: 20, class: "text-white" }
                            }
                            div { class: "ml-3",
                                p { class: "text-sm text-gray-500", "待交付" }
                                p { class: "text-xl font-bold text-gray-800", {pending_delivery_count.to_string()} }
                            }
                        }
                    }
                    div { class: "bg-white rounded-lg shadow p-4",
                        div { class: "flex items-center",
                            div { class: "p-2 rounded-full bg-teal-500",
                                Icon { icon: FaCircleCheck, width: 20, height: 20, class: "text-white" }
                            }
                            div { class: "ml-3",
                                p { class: "text-sm text-gray-500", "已交付" }
                                p { class: "text-xl font-bold text-gray-800", {delivered_count.to_string()} }
                            }
                        }
                    }
                    div { class: "bg-white rounded-lg shadow p-4",
                        div { class: "flex items-center",
                            div { class: "p-2 rounded-full bg-gray-500",
                                Icon { icon: FaFileLines, width: 20, height: 20, class: "text-white" }
                            }
                            div { class: "ml-3",
                                p { class: "text-sm text-gray-500", "已归档" }
                                p { class: "text-xl font-bold text-gray-800", {archived_count.to_string()} }
                            }
                        }
                    }
                }
            }

            // 工作流程标签页导航 + 搜索框
            div { class: "bg-white border-b border-gray-200 px-6",
                div { class: "flex items-center justify-between gap-4",
                    // 工作流标签
                    div { class: "flex gap-1 overflow-x-auto",
                        for tab in accessible_tabs {
                            button {
                                class: format!(
                                    "px-4 py-2 text-sm font-medium border-b-2 whitespace-nowrap transition-colors {}",
                                    if *workflow_tab.read() == tab {
                                        "border-blue-500 text-blue-600 bg-white"
                                    } else {
                                        "border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300"
                                    }
                                ),
                                onclick: move |_| {
                                    workflow_tab.set(tab);
                                    selected_ticket.set(None);
                                    show_new_form.set(false);
                                },
                                i { class: format!("fa {} mr-2", tab.icon_name()) }
                                {tab.display_name()}
                            }
                        }
                    }

                    // 搜索框和统计
                    div { class: "flex items-center gap-3",
                        // 统计信息
                        span { class: "text-sm text-gray-500 whitespace-nowrap",
                            {format!("共 {} 条", total_count)}
                        }
                        // 搜索框
                        div { class: "relative w-40",
                            input {
                                class: "w-full pl-7 pr-2 py-1 border border-gray-200 rounded text-xs focus:ring-1 focus:ring-blue-500 focus:border-blue-500 bg-gray-50 focus:bg-white",
                                placeholder: "搜索...",
                                value: "{search_query}",
                                oninput: move |e| search_query.set(e.value())
                            }
                            Icon { icon: FaMagnifyingGlass, class: "absolute left-2 top-1/2 -translate-y-1/2 text-gray-400 w-3 h-3" }
                        }
                    }
                }
            }

            // 主内容区
            div { class: "flex-1 overflow-auto p-6",
                if let Some(ticket_id) = *selected_ticket.read() {
                    // 详情视图
                    {
                        let all_tickets = tickets.read().clone();
                        if let Some(ticket) = all_tickets.iter().find(|t| t.id == ticket_id).cloned() {
                            rsx! {
                                TicketDetailView {
                                    ticket,
                                    tickets: tickets,
                                    workflow_tab: *workflow_tab.read(),
                                    current_role,
                                    on_back: move |_| selected_ticket.set(None),
                                }
                            }
                        } else {
                            rsx! { div { "工单不存在" } }
                        }
                    }
                } else {
                    // 工单列表视图
                    TicketListViewByTypeAndWorkflow {
                        resource_type: *resource_type_tab.read(),
                        workflow_tab: *workflow_tab.read(),
                        tickets: tickets,
                        search_query: (*search_query.read()).clone(),
                        current_username: auth.read().username.clone(),
                        current_role,
                        on_select: move |id| selected_ticket.set(Some(id)),
                    }
                }
            }

            // 新建工单模态框（放在最外层）- 根据资源类型使用不同的表单组件
            {
                let current_resource_type = *resource_type_tab.read();
                if *show_new_form.read() {
                    Some(match current_resource_type {
                        ResourceType::Cloud => rsx! {
                            CloudServiceForm {
                                mode: crate::components::common::FormMode::New,
                                request: None,
                                on_save: move |req: CloudServiceRequest| {
                                    // 获取服务商名称
                                    let provider_name = req.provider_id
                                        .and_then(|pid| {
                                            PROVIDERS_STATE
                                                .read()
                                                .iter()
                                                .find(|p| p.id == pid)
                                                .map(|p| p.short_name.clone())
                                        })
                                        .unwrap_or_default();
                                    let selected_platform = CLOUD_PLATFORMS_STATE
                                        .read()
                                        .iter()
                                        .find(|platform| {
                                            platform.platform_name == req.cloud_platform
                                                && req.provider_id
                                                    .map(|pid| platform.provider_id == pid)
                                                    .unwrap_or(true)
                                        })
                                        .cloned();

                                    // 转换为 ResourceTicket 并添加
                                    let new_ticket = ResourceTicket {
                                        id: (tickets.read().len() + 1) as i32,
                                        resource_type: ResourceType::Cloud,
                                        ecs_name: req.title.clone(),
                                        ticket_status: TicketStatus::PendingApproval,
                                        provider_id: req.provider_id,
                                        provider_name,
                                        cloud_platform_id: selected_platform
                                            .as_ref()
                                            .map(|platform| platform.id),
                                        cloud_platform_name: selected_platform
                                            .as_ref()
                                            .map(|platform| platform.platform_name.clone())
                                            .unwrap_or_else(|| req.cloud_platform.clone()),
                                        machine_room_id: None,
                                        machine_room_name: String::new(),
                                        cloud_region: String::new(),
                                        cloud_category: "云主机".to_string(),
                                        zone_name: String::new(),
                                        zone_cabinet: String::new(),
                                        rack_units: 0,
                                        customer_name: String::new(),
                                        application_name: req.title.clone(),
                                        contract_name: String::new(),
                                        ecs_type: req.instance_type.clone(),
                                        ecs_os: String::new(),
                                        cpu_cores: 0,
                                        memory_gb: 0,
                                        system_disk: String::new(),
                                        system_disk_size_gb: 0,
                                        data_disk: String::new(),
                                        has_security_product: !req.security_products.is_empty(),
                                        security_products: req.security_products.to_names_string(&SECURITY_PRODUCTS_STATE.read()),
                                        ip_address: String::new(),
                                        delivery_status: "未交付".to_string(),
                                        remarks: req.purpose.clone(),
                                        created_at: chrono::Local::now().format("%Y-%m-%d %H:%M").to_string(),
                                        updated_at: chrono::Local::now().format("%Y-%m-%d %H:%M").to_string(),
                                        created_by: req.applicant.clone(),
                                        approver: None,
                                        approve_time: None,
                                        approve_comment: None,
                                        provisioner: None,
                                        provision_time: None,
                                        provision_details: None,
                                        deliverer: None,
                                        deliver_time: None,
                                        deliver_comment: None,
                                        fw_source_zone: None,
                                        fw_source_address: None,
                                        fw_dest_zone: None,
                                        fw_dest_address: None,
                                        fw_protocol: None,
                                        fw_port: None,
                                        fw_direction: None,
                                        fw_valid_until: None,
                                        fw_firewall_name: None,
                                    };
                                    let mut tickets_ref = tickets;
                                    let mut form_close = show_new_form;
                                    spawn(async move {
                                        match create_resource_ticket(&new_ticket).await {
                                            Ok(created) => {
                                                tickets_ref.with_mut(|t| t.push(created));
                                                form_close.set(false);
                                            }
                                            Err(e) => {
                                                tracing::error!("创建云服务工单失败: {}", e);
                                            }
                                        }
                                    });
                                },
                                on_close: move |_| show_new_form.set(false),
                            }
                        },
                        ResourceType::Physical => rsx! {
                            PhysicalServerForm {
                                mode: crate::components::common::FormMode::New,
                                request: None,
                                on_save: move |req: PhysicalServerRequest| {
                                    let cpu_cores = parse_number_with_suffix(&req.cpu_cores, "核");
                                    let memory_gb = parse_number_with_suffix(&req.memory, "GB");
                                    let (system_disk, system_disk_size_gb) =
                                        parse_storage_config(&req.storage);

                                    // 获取服务商名称
                                    let provider_name = req.provider_id
                                        .and_then(|pid| PROVIDERS_STATE.read()
                                            .iter()
                                            .find(|p| p.id == pid)
                                            .map(|p| p.short_name.clone()))
                                        .unwrap_or_default();

                                    // 获取机房名称
                                    let machine_room_name = req.machine_room_id
                                        .and_then(|rid| MACHINE_ROOMS_STATE.read()
                                            .iter()
                                            .find(|r| r.id == rid)
                                            .map(|r| r.room_name.clone()))
                                        .unwrap_or_default();

                                    // 转换为 ResourceTicket 并添加
                                    let new_ticket = ResourceTicket {
                                        id: (tickets.read().len() + 1) as i32,
                                        resource_type: ResourceType::Physical,
                                        ecs_name: req.title.clone(),
                                        ticket_status: TicketStatus::PendingApproval,
                                        provider_id: req.provider_id,
                                        provider_name,
                                        cloud_platform_id: None,
                                        cloud_platform_name: String::new(),
                                        machine_room_id: req.machine_room_id,
                                        machine_room_name,
                                        cloud_region: String::new(),
                                        cloud_category: "物理机".to_string(),
                                        zone_name: String::new(),
                                        zone_cabinet: String::new(),
                                        rack_units: 0,
                                        customer_name: String::new(),
                                        application_name: req.title.clone(),
                                        contract_name: String::new(),
                                        ecs_type: req.server_type.clone(),
                                        ecs_os: String::new(),
                                        cpu_cores,
                                        memory_gb,
                                        system_disk,
                                        system_disk_size_gb,
                                        data_disk: String::new(),
                                        has_security_product: !req.security_products.is_empty(),
                                        security_products: req.security_products.to_names_string(&SECURITY_PRODUCTS_STATE.read()),
                                        ip_address: String::new(),
                                        delivery_status: "未交付".to_string(),
                                        remarks: req.purpose.clone(),
                                        created_at: chrono::Local::now().format("%Y-%m-%d %H:%M").to_string(),
                                        updated_at: chrono::Local::now().format("%Y-%m-%d %H:%M").to_string(),
                                        created_by: req.applicant.clone(),
                                        approver: None,
                                        approve_time: None,
                                        approve_comment: None,
                                        provisioner: None,
                                        provision_time: None,
                                        provision_details: None,
                                        deliverer: None,
                                        deliver_time: None,
                                        deliver_comment: None,
                                        fw_source_zone: None,
                                        fw_source_address: None,
                                        fw_dest_zone: None,
                                        fw_dest_address: None,
                                        fw_protocol: None,
                                        fw_port: None,
                                        fw_direction: None,
                                        fw_valid_until: None,
                                        fw_firewall_name: None,
                                    };
                                    let mut tickets_ref = tickets;
                                    let mut form_close = show_new_form;
                                    spawn(async move {
                                        match create_resource_ticket(&new_ticket).await {
                                            Ok(created) => {
                                                tickets_ref.with_mut(|t| t.push(created));
                                                form_close.set(false);
                                            }
                                            Err(e) => {
                                                tracing::error!("创建物理机工单失败: {}", e);
                                            }
                                        }
                                    });
                                },
                                on_close: move |_| show_new_form.set(false),
                            }
                        },
                        ResourceType::Network => rsx! {
                            NetworkPolicyForm {
                                mode: crate::components::common::FormMode::New,
                                request: None,
                                on_save: move |req: NetworkPolicyRequest| {
                                    // 转换为 ResourceTicket 并添加
                                    let new_ticket = ResourceTicket {
                                        id: (tickets.read().len() + 1) as i32,
                                        resource_type: ResourceType::Network,
                                        ecs_name: req.title.clone(),
                                        ticket_status: TicketStatus::PendingApproval,
                                        provider_id: None,
                                        provider_name: String::new(),
                                        cloud_platform_id: None,
                                        cloud_platform_name: String::new(),
                                        machine_room_id: None,
                                        machine_room_name: String::new(),
                                        cloud_region: String::new(),
                                        cloud_category: "网络策略".to_string(),
                                        zone_name: String::new(),
                                        zone_cabinet: String::new(),
                                        rack_units: 0,
                                        customer_name: String::new(),
                                        application_name: req.title.clone(),
                                        contract_name: String::new(),
                                        ecs_type: String::new(),
                                        ecs_os: String::new(),
                                        cpu_cores: 0,
                                        memory_gb: 0,
                                        system_disk: String::new(),
                                        system_disk_size_gb: 0,
                                        data_disk: String::new(),
                                        has_security_product: false,
                                        security_products: String::new(),
                                        ip_address: String::new(),
                                        delivery_status: "未交付".to_string(),
                                        remarks: String::new(),
                                        created_at: chrono::Local::now().format("%Y-%m-%d %H:%M").to_string(),
                                        updated_at: chrono::Local::now().format("%Y-%m-%d %H:%M").to_string(),
                                        created_by: req.applicant.clone(),
                                        approver: None,
                                        approve_time: None,
                                        approve_comment: None,
                                        provisioner: None,
                                        provision_time: None,
                                        provision_details: None,
                                        deliverer: None,
                                        deliver_time: None,
                                        deliver_comment: None,
                                        fw_source_zone: Some(req.source_zone.clone()),
                                        fw_source_address: None,
                                        fw_dest_zone: Some(req.destination_zone.clone()),
                                        fw_dest_address: None,
                                        fw_protocol: Some(req.protocol.display_name().to_string()),
                                        fw_port: Some(req.port_range.clone()),
                                        fw_direction: Some(req.direction.display_name().to_string()),
                                        fw_valid_until: Some(req.valid_until.clone()),
                                        fw_firewall_name: None,
                                    };
                                    let mut tickets_ref = tickets;
                                    let mut form_close = show_new_form;
                                    spawn(async move {
                                        match create_resource_ticket(&new_ticket).await {
                                            Ok(created) => {
                                                tickets_ref.with_mut(|t| t.push(created));
                                                form_close.set(false);
                                            }
                                            Err(e) => {
                                                tracing::error!("创建网络策略工单失败: {}", e);
                                            }
                                        }
                                    });
                                },
                                on_close: move |_| show_new_form.set(false),
                            }
                        },
                    })
                } else {
                    None
                }
            }
        }
    }
}

/// 按资源类型和工作流程筛选的工单列表视图
#[component]
fn TicketListViewByTypeAndWorkflow(
    resource_type: ResourceType,
    workflow_tab: ApplicationTab,
    tickets: Signal<Vec<ResourceTicket>>,
    search_query: String,
    current_username: String,
    current_role: UserRole,
    on_select: Callback<i32>,
) -> Element {
    let all_tickets = tickets.read().clone();

    // 先按资源类型筛选
    let filtered_tickets: Vec<_> = all_tickets
        .into_iter()
        .filter(|ticket| ticket.resource_type == resource_type)
        .filter(|ticket| {
            // 工作流程筛选
            match workflow_tab {
                ApplicationTab::MyApplications => is_my_ticket(ticket, &current_username),
                ApplicationTab::PendingApproval => {
                    ticket.ticket_status == TicketStatus::PendingApproval
                }
                ApplicationTab::PendingProvision => {
                    ticket.ticket_status == TicketStatus::PendingProvision
                }
                ApplicationTab::PendingDelivery => {
                    ticket.ticket_status == TicketStatus::PendingDelivery
                }
                ApplicationTab::Delivered => ticket.ticket_status == TicketStatus::Delivered,
                ApplicationTab::Archived => ticket.ticket_status == TicketStatus::Archived,
            }
        })
        .filter(|ticket| {
            // 搜索查询筛选
            if search_query.is_empty() {
                true
            } else {
                let query = search_query.to_lowercase();
                ticket.application_name.to_lowercase().contains(&query)
                    || ticket.created_by.to_lowercase().contains(&query)
                    || ticket.contract_name.to_lowercase().contains(&query)
            }
        })
        .collect();

    rsx! {
        div { class: "bg-white rounded-xl shadow-sm border border-gray-200 overflow-hidden",
            if filtered_tickets.is_empty() {
                div { class: "flex flex-col items-center justify-center py-16 text-gray-500",
                    Icon { icon: FaFile, class: "text-5xl text-gray-300 mb-4" }
                    p { class: "text-lg", "暂无数据" }
                }
            } else {
                div { class: "overflow-x-auto",
                    table { class: "w-full",
                        thead { class: "bg-gray-50 border-b border-gray-200",
                            tr {
                                th { class: "px-6 py-3 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider", "ID" }
                                th { class: "px-6 py-3 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider", "申请名称" }
                                th { class: "px-6 py-3 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider", "配置" }
                                th { class: "px-6 py-3 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider", "申请人" }
                                th { class: "px-6 py-3 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider", "状态" }
                                th { class: "px-6 py-3 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider", "创建时间" }
                                th { class: "px-6 py-3 text-right text-xs font-semibold text-gray-600 uppercase tracking-wider", "操作" }
                            }
                        }
                        tbody { class: "divide-y divide-gray-200",
                            for ticket in filtered_tickets {
                                tr {
                                    class: "hover:bg-gray-50 transition-colors cursor-pointer",
                                    onclick: move |_| on_select.call(ticket.id),
                                    td { class: "px-6 py-4 text-sm text-gray-900", "{ticket.id}" }
                                    td { class: "px-6 py-4",
                                        div { class: "text-sm font-medium text-gray-900", {ticket.application_name} }
                                        div { class: "text-sm text-gray-500", {ticket.contract_name} }
                                    }
                                    td { class: "px-6 py-4 text-sm text-gray-600",
                                        // 根据资源类型显示不同的配置信息
                                        if resource_type == ResourceType::Network {
                                            div { class: "text-xs",
                                                "{ticket.fw_source_zone.as_ref().unwrap_or(&String::new())} → {ticket.fw_dest_zone.as_ref().unwrap_or(&String::new())}"
                                            }
                                            div { class: "text-xs text-gray-400",
                                                "{ticket.fw_protocol.as_ref().unwrap_or(&String::new())} / {ticket.fw_port.as_ref().unwrap_or(&String::new())}"
                                            }
                                        } else {
                                            div { {ticket.ecs_type.clone()} }
                                            div { class: "text-xs text-gray-400",
                                                "{ticket.cpu_cores}核 / {ticket.memory_gb}GB"
                                            }
                                        }
                                    }
                                    td { class: "px-6 py-4 text-sm text-gray-600", {ticket.created_by} }
                                    td { class: "px-6 py-4",
                                        span {
                                            class: format!("px-2.5 py-1 text-xs font-medium rounded-full {}", ticket.ticket_status.color_class()),
                                            {ticket.ticket_status.display_name()}
                                        }
                                    }
                                    td { class: "px-6 py-4 text-sm text-gray-600", {ticket.created_at} }
                                    td { class: "px-6 py-4 text-right",
                                        button {
                                            class: "text-blue-600 hover:text-blue-800 text-sm font-medium",
                                            onclick: move |e| {
                                                e.stop_propagation();
                                                on_select.call(ticket.id);
                                            },
                                            "查看详情"
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

/// 申请列表视图（保留用于其他地方）
#[component]
fn TicketListView(
    current_tab: ApplicationTab,
    tickets: Signal<Vec<ResourceTicket>>,
    search_query: String,
    resource_type_filter: String,
    current_username: String,
    current_role: UserRole,
    on_select: Callback<i32>,
) -> Element {
    let all_tickets = tickets.read().clone();
    let filtered_tickets = match current_tab {
        ApplicationTab::MyApplications => all_tickets
            .clone()
            .into_iter()
            .filter(|ticket| is_my_ticket(ticket, &current_username))
            .collect::<Vec<_>>(),
        ApplicationTab::PendingApproval => all_tickets
            .clone()
            .into_iter()
            .filter(|ticket| ticket.ticket_status == TicketStatus::PendingApproval)
            .collect(),
        ApplicationTab::PendingProvision => all_tickets
            .clone()
            .into_iter()
            .filter(|ticket| ticket.ticket_status == TicketStatus::PendingProvision)
            .collect(),
        ApplicationTab::PendingDelivery => all_tickets
            .clone()
            .into_iter()
            .filter(|ticket| ticket.ticket_status == TicketStatus::PendingDelivery)
            .collect(),
        ApplicationTab::Delivered => all_tickets
            .clone()
            .into_iter()
            .filter(|ticket| ticket.ticket_status == TicketStatus::Delivered)
            .collect(),
        ApplicationTab::Archived => all_tickets
            .clone()
            .into_iter()
            .filter(|ticket| ticket.ticket_status == TicketStatus::Archived)
            .collect(),
    };

    let filtered_tickets: Vec<_> = filtered_tickets
        .into_iter()
        .filter(|ticket| {
            // 搜索查询筛选
            let matches_search = if search_query.is_empty() {
                true
            } else {
                let query = search_query.to_lowercase();
                ticket.application_name.to_lowercase().contains(&query)
                    || ticket.created_by.to_lowercase().contains(&query)
                    || ticket.contract_name.to_lowercase().contains(&query)
            };

            // 资源类型筛选
            let matches_type = match resource_type_filter.as_str() {
                "all" => true,
                "cloud" => ticket.resource_type == ResourceType::Cloud,
                "physical" => ticket.resource_type == ResourceType::Physical,
                "network" => ticket.resource_type == ResourceType::Network,
                _ => true,
            };

            matches_search && matches_type
        })
        .collect();

    rsx! {
        div { class: "bg-white rounded-xl shadow-sm border border-gray-200 overflow-hidden",
            if filtered_tickets.is_empty() {
                div { class: "flex flex-col items-center justify-center py-16 text-gray-500",
                    Icon { icon: FaFile, class: "text-5xl text-gray-300 mb-4" }
                    p { class: "text-lg", "暂无数据" }
                }
            } else {
                div { class: "overflow-x-auto",
                    table { class: "w-full",
                        thead { class: "bg-gray-50 border-b border-gray-200",
                            tr {
                                th { class: "px-6 py-3 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider", "ID" }
                                th { class: "px-6 py-3 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider", "申请名称" }
                                th { class: "px-6 py-3 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider", "资源类型" }
                                th { class: "px-6 py-3 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider", "配置" }
                                th { class: "px-6 py-3 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider", "申请人" }
                                th { class: "px-6 py-3 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider", "状态" }
                                th { class: "px-6 py-3 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider", "创建时间" }
                                th { class: "px-6 py-3 text-right text-xs font-semibold text-gray-600 uppercase tracking-wider", "操作" }
                            }
                        }
                        tbody { class: "divide-y divide-gray-200",
                            for ticket in filtered_tickets {
                                tr {
                                    class: "hover:bg-gray-50 transition-colors cursor-pointer",
                                    onclick: move |_| on_select.call(ticket.id),
                                    td { class: "px-6 py-4 text-sm text-gray-900", "{ticket.id}" }
                                    td { class: "px-6 py-4",
                                        div { class: "text-sm font-medium text-gray-900", {ticket.application_name} }
                                        div { class: "text-sm text-gray-500", {ticket.contract_name} }
                                    }
                                    td { class: "px-6 py-4",
                                        div { class: "flex items-center gap-2",
                                            match ticket.resource_type {
                                                ResourceType::Cloud => rsx! {
                                                    Icon {
                                                        icon: FaCloud,
                                                        class: "text-blue-500"
                                                    }
                                                },
                                                ResourceType::Physical => rsx! {
                                                    Icon {
                                                        icon: FaServer,
                                                        class: "text-gray-500"
                                                    }
                                                },
                                                ResourceType::Network => rsx! {
                                                    Icon {
                                                        icon: FaShieldHalved,
                                                        class: "text-purple-500"
                                                    }
                                                },
                                            }
                                            span { class: "text-sm text-gray-700",
                                                {ticket.resource_type.display_name()}
                                            }
                                        }
                                    }
                                    td { class: "px-6 py-4 text-sm text-gray-600",
                                        "{ticket.ecs_type} / {ticket.cpu_cores}核 / {ticket.memory_gb}GB"
                                    }
                                    td { class: "px-6 py-4 text-sm text-gray-600", {ticket.created_by} }
                                    td { class: "px-6 py-4",
                                        span {
                                            class: format!("px-2.5 py-1 text-xs font-medium rounded-full {}", ticket.ticket_status.color_class()),
                                            {ticket.ticket_status.display_name()}
                                        }
                                    }
                                    td { class: "px-6 py-4 text-sm text-gray-600", {ticket.created_at} }
                                    td { class: "px-6 py-4 text-right",
                                        button {
                                            class: "text-blue-600 hover:text-blue-800 text-sm font-medium",
                                            onclick: move |e| {
                                                e.stop_propagation();
                                                on_select.call(ticket.id);
                                            },
                                            "查看详情"
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

/// 申请详情视图
#[component]
fn TicketDetailView(
    ticket: ResourceTicket,
    tickets: Signal<Vec<ResourceTicket>>,
    workflow_tab: ApplicationTab,
    current_role: UserRole,
    on_back: Callback<()>,
) -> Element {
    let show_approval_section = matches!(workflow_tab, ApplicationTab::PendingApproval)
        && (ticket.approver.is_some() || current_role.can_approve());
    let show_provision_section = matches!(workflow_tab, ApplicationTab::PendingProvision)
        && (ticket.provisioner.is_some() || current_role.can_provision());
    let show_delivery_section = matches!(
        workflow_tab,
        ApplicationTab::PendingDelivery | ApplicationTab::Delivered | ApplicationTab::Archived
    ) && (ticket.deliverer.is_some() || current_role.can_deliver());

    rsx! {
        div { class: "space-y-6",
            // 返回按钮
            button {
                class: "flex items-center gap-2 text-gray-600 hover:text-gray-800 mb-4",
                onclick: move |_| on_back.call(()),
                Icon { icon: FaArrowLeft, class: "text-sm" }
                "返回列表"
            }

            // 主要内容卡片
            div { class: "grid grid-cols-1 lg:grid-cols-3 gap-6",
                // 左侧：申请详情
                div { class: "lg:col-span-2 space-y-6",
                    // 基本信息
                    div { class: "bg-white rounded-xl shadow-sm border border-gray-200 p-6",
                        h3 { class: "text-lg font-semibold text-gray-800 mb-4 flex items-center gap-2",
                            Icon { icon: FaFileLines, class: "text-blue-600" }
                            "申请信息"
                        }
                        div { class: "grid grid-cols-2 gap-4",
                            InfoRow { label: "申请ID", value: ticket.id }
                            InfoRow { label: "申请名称", value: ticket.application_name.clone() }
                            InfoRow { label: "合同名称", value: ticket.contract_name.clone() }
                            InfoRow { label: "客户名称", value: ticket.customer_name.clone() }
                            InfoRow { label: "资源类型", value: ticket.resource_type.display_name() }
                            InfoRow { label: "云平台", value: ticket.cloud_platform_name.clone() }
                            InfoRow { label: "云服务商", value: ticket.provider_name.clone() }
                            InfoRow { label: "申请人", value: ticket.created_by.clone() }
                            InfoRowElement { label: "申请状态",
                                value: rsx! {
                                    span {
                                        class: format!("px-2.5 py-1 text-xs font-medium rounded-full {}", ticket.ticket_status.color_class()),
                                        {ticket.ticket_status.display_name()}
                                    }
                                }
                            }
                            InfoRow { label: "创建时间", value: ticket.created_at.clone() }
                            InfoRow { label: "更新时间", value: ticket.updated_at.clone() }
                        }
                    }

                    // 配置信息
                    div { class: "bg-white rounded-xl shadow-sm border border-gray-200 p-6",
                        h3 { class: "text-lg font-semibold text-gray-800 mb-4 flex items-center gap-2",
                            Icon { icon: FaGear, class: "text-blue-600" }
                            if ticket.resource_type == ResourceType::Network {
                                "网络策略配置"
                            } else {
                                "资源配置"
                            }
                        }
                        div { class: "grid grid-cols-2 gap-4",
                            // 网络策略专用字段
                            if ticket.resource_type == ResourceType::Network {
                                InfoRow { label: "策略名称", value: ticket.ecs_name.clone() }
                                InfoRow { label: "源区域", value: ticket.fw_source_zone.clone().unwrap_or_default() }
                                InfoRow { label: "源地址", value: ticket.fw_source_address.clone().unwrap_or_default() }
                                InfoRow { label: "目标区域", value: ticket.fw_dest_zone.clone().unwrap_or_default() }
                                InfoRow { label: "目标地址", value: ticket.fw_dest_address.clone().unwrap_or_default() }
                                InfoRow { label: "协议", value: ticket.fw_protocol.clone().unwrap_or_default() }
                                InfoRow { label: "端口", value: ticket.fw_port.clone().unwrap_or_default() }
                                InfoRow { label: "访问方向", value: ticket.fw_direction.clone().unwrap_or_default() }
                                InfoRow { label: "有效期至", value: ticket.fw_valid_until.clone().unwrap_or_default() }
                                InfoRow { label: "防火墙设备", value: ticket.fw_firewall_name.clone().unwrap_or_default() }
                            }
                            // 云资源和物理资源字段
                            if ticket.resource_type != ResourceType::Network {
                                InfoRow { label: "实例名称", value: ticket.ecs_name.clone() }
                                InfoRow { label: "实例类型", value: ticket.ecs_type.clone() }
                                InfoRow { label: "操作系统", value: ticket.ecs_os.clone() }
                                InfoRow { label: "CPU", value: format!("{} 核", ticket.cpu_cores) }
                                InfoRow { label: "内存", value: format!("{} GB", ticket.memory_gb) }
                                InfoRow { label: "系统盘", value: format!("{} {}GB", ticket.system_disk, ticket.system_disk_size_gb) }
                                InfoRow { label: "数据盘", value: ticket.data_disk.clone() }
                                InfoRow { label: "IP地址", value: if ticket.ip_address.is_empty() { "未分配".to_string() } else { ticket.ip_address.clone() } }
                                InfoRow { label: "安全产品",
                                    value: if ticket.security_products.is_empty() {
                                        "无".to_string()
                                    } else {
                                        ticket.security_products.clone()
                                    }
                                }
                            }
                        }
                    }

                    // 审批信息
                    if show_approval_section {
                        div { class: "bg-white rounded-xl shadow-sm border border-gray-200 p-6",
                            h3 { class: "text-lg font-semibold text-gray-800 mb-4 flex items-center gap-2",
                                Icon { icon: FaCheck, class: "text-green-600" }
                                "审批信息"
                            }
                            if let Some(approver) = &ticket.approver {
                                div { class: "space-y-3",
                                    InfoRow { label: "审批人", value: approver.clone() }
                                    InfoRow { label: "审批时间", value: ticket.approve_time.clone().unwrap_or_default() }
                                    InfoRow { label: "审批意见",
                                        value: ticket.approve_comment.clone().unwrap_or_else(|| "无".to_string())
                                    }
                                }
                            } else {
                                ApprovalPanel {
                                    ticket: ticket.clone(),
                                    tickets: tickets,
                                }
                            }
                        }
                    }

                    // 配置信息
                    if show_provision_section {
                        div { class: "bg-white rounded-xl shadow-sm border border-gray-200 p-6",
                            h3 { class: "text-lg font-semibold text-gray-800 mb-4 flex items-center gap-2",
                                Icon { icon: FaWrench, class: "text-purple-600" }
                                "配置信息"
                            }
                            if let Some(provisioner) = &ticket.provisioner {
                                div { class: "space-y-3",
                                    InfoRow { label: "配置人员", value: provisioner.clone() }
                                    InfoRow { label: "配置时间", value: ticket.provision_time.clone().unwrap_or_default() }
                                    InfoRow { label: "配置详情",
                                        value: ticket.provision_details.clone().unwrap_or_else(|| "无".to_string())
                                    }
                                }
                            } else {
                                ProvisionPanel {
                                    ticket: ticket.clone(),
                                    tickets: tickets,
                                }
                            }
                        }
                    }

                    // 交付信息
                    if show_delivery_section {
                        div { class: "bg-white rounded-xl shadow-sm border border-gray-200 p-6",
                            h3 { class: "text-lg font-semibold text-gray-800 mb-4 flex items-center gap-2",
                                Icon { icon: FaBoxOpen, class: "text-orange-600" }
                                "交付信息"
                            }
                            if let Some(deliverer) = &ticket.deliverer {
                                div { class: "space-y-3",
                                    InfoRow { label: "交付人员", value: deliverer.clone() }
                                    InfoRow { label: "交付时间", value: ticket.deliver_time.clone().unwrap_or_default() }
                                    InfoRow { label: "交付备注",
                                        value: ticket.deliver_comment.clone().unwrap_or_else(|| "无".to_string())
                                    }
                                }
                            } else {
                                DeliveryPanel {
                                    ticket: ticket.clone(),
                                    tickets: tickets,
                                }
                            }
                        }
                    }
                }

                // 右侧：状态时间线
                div { class: "lg:col-span-1",
                    StatusTimeline { ticket: ticket.clone() }
                }
            }
        }
    }
}

/// 审批面板
#[component]
fn ApprovalPanel(ticket: ResourceTicket, tickets: Signal<Vec<ResourceTicket>>) -> Element {
    let ticket_clone = ticket.clone();
    let tickets_clone = tickets;
    let ticket_clone2 = ticket.clone();
    let tickets_clone2 = tickets;
    let mut comment = use_signal(String::new);
    let mut is_approving = use_signal(|| false);

    rsx! {
        div { class: "space-y-4",
            p { class: "text-sm text-gray-500", "请审批此申请" }
            textarea {
                class: "w-full px-2.5 py-1.5 text-sm border border-gray-200 rounded focus:ring-1 focus:ring-blue-500 focus:border-blue-500",
                rows: 3,
                placeholder: "审批意见（可选）",
                value: "{comment}",
                oninput: move |e| comment.set(e.value())
            }
            div { class: "flex gap-2",
                button {
                    class: "flex-1 px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 flex items-center justify-center gap-2 transition-colors disabled:opacity-50",
                    disabled: *is_approving.read(),
                    onclick: move |_| {
                        let ticket_id = ticket_clone.id;
                        let comment_val = if !comment.read().is_empty() { Some(comment.read().clone()) } else { None };
                        let mut tickets_ref = tickets_clone;
                        is_approving.set(true);
                        spawn(async move {
                            let req = crate::services::resource_ticket_api::ApproveTicketRequest {
                                approved: true,
                                comment: comment_val,
                            };
                            match approve_ticket(ticket_id, req).await {
                                Ok(updated) => {
                                    tickets_ref.with_mut(|tickets| {
                                        if let Some(t) = tickets.iter_mut().find(|t| t.id == ticket_id) {
                                            *t = updated;
                                        }
                                    });
                                }
                                Err(e) => {
                                    tracing::error!("审批失败: {}", e);
                                }
                            }
                        });
                    },
                    Icon { icon: FaCircleCheck, class: "text-sm" }
                    "通过"
                }
                button {
                    class: "flex-1 px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 flex items-center justify-center gap-2 transition-colors disabled:opacity-50",
                    disabled: *is_approving.read(),
                    onclick: move |_| {
                        let ticket_id = ticket_clone2.id;
                        let comment_val = if !comment.read().is_empty() { Some(comment.read().clone()) } else { Some("拒绝".to_string()) };
                        let mut tickets_ref = tickets_clone2;
                        is_approving.set(true);
                        spawn(async move {
                            let req = crate::services::resource_ticket_api::ApproveTicketRequest {
                                approved: false,
                                comment: comment_val,
                            };
                            match approve_ticket(ticket_id, req).await {
                                Ok(updated) => {
                                    tickets_ref.with_mut(|tickets| {
                                        if let Some(t) = tickets.iter_mut().find(|t| t.id == ticket_id) {
                                            *t = updated;
                                        }
                                    });
                                }
                                Err(e) => {
                                    tracing::error!("审批拒绝失败: {}", e);
                                }
                            }
                        });
                    },
                    Icon { icon: FaCircleXmark, class: "text-sm" }
                    "拒绝"
                }
            }
        }
    }
}

/// 配置面板
#[component]
fn ProvisionPanel(ticket: ResourceTicket, tickets: Signal<Vec<ResourceTicket>>) -> Element {
    let ticket_clone = ticket.clone();
    let tickets_clone = tickets;
    let mut details = use_signal(String::new);
    let mut ip_address = use_signal(|| ticket.ip_address.clone());
    let mut is_provisioning = use_signal(|| false);

    rsx! {
        div { class: "space-y-4",
            p { class: "text-sm text-gray-500", "请配置云资源并填写配置信息" }
            div {
                label { class: "block text-xs font-medium text-gray-600 mb-0.5", "IP地址" }
                input {
                    class: "w-full px-2.5 py-1.5 text-sm border border-gray-200 rounded focus:ring-1 focus:ring-blue-500 focus:border-blue-500",
                    placeholder: "192.168.1.100",
                    value: "{ip_address}",
                    oninput: move |e| ip_address.set(e.value())
                }
            }
            textarea {
                class: "w-full px-2.5 py-1.5 text-sm border border-gray-200 rounded focus:ring-1 focus:ring-blue-500 focus:border-blue-500",
                rows: 4,
                placeholder: "配置详情（如：实例已创建，安全组已配置等）",
                value: "{details}",
                oninput: move |e| details.set(e.value())
            }
            button {
                class: "w-full px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 flex items-center justify-center gap-2 transition-colors disabled:opacity-50",
                disabled: *is_provisioning.read(),
                onclick: move |_| {
                    let ticket_id = ticket_clone.id;
                    let details_val = if !details.read().is_empty() { Some(details.read().clone()) } else { None };
                    let mut tickets_ref = tickets_clone;
                    is_provisioning.set(true);
                    spawn(async move {
                        let req = crate::services::resource_ticket_api::ProvisionTicketRequest {
                            details: details_val,
                        };
                        match provision_ticket(ticket_id, req).await {
                            Ok(updated) => {
                                tickets_ref.with_mut(|tickets| {
                                    if let Some(t) = tickets.iter_mut().find(|t| t.id == ticket_id) {
                                        *t = updated;
                                    }
                                });
                            }
                            Err(e) => {
                                tracing::error!("配置失败: {}", e);
                            }
                        }
                    });
                },
                Icon { icon: FaCheck, class: "text-sm" }
                "完成配置，提交交付"
            }
        }
    }
}

/// 交付面板
#[component]
fn DeliveryPanel(ticket: ResourceTicket, tickets: Signal<Vec<ResourceTicket>>) -> Element {
    let ticket_clone = ticket.clone();
    let tickets_clone = tickets;
    let mut comment = use_signal(String::new);
    let mut is_delivering = use_signal(|| false);

    rsx! {
        div { class: "space-y-4",
            p { class: "text-sm text-gray-500", "请复核资源配置，确认无误后交付" }
            div { class: "bg-blue-50 border border-blue-200 rounded-lg p-3",
                h4 { class: "text-sm font-semibold text-blue-800 mb-2", "配置信息复核" }
                div { class: "text-sm text-blue-700 space-y-1",
                    p { "IP地址: {ticket.ip_address}" }
                    if let Some(details) = &ticket.provision_details {
                        p { "配置详情: {details}" }
                    }
                }
            }
            textarea {
                class: "w-full px-2.5 py-1.5 text-sm border border-gray-200 rounded focus:ring-1 focus:ring-blue-500 focus:border-blue-500",
                rows: 3,
                placeholder: "交付备注（可选）",
                value: "{comment}",
                oninput: move |e| comment.set(e.value())
            }
            button {
                class: "w-full px-4 py-2 bg-orange-600 text-white rounded-lg hover:bg-orange-700 flex items-center justify-center gap-2 transition-colors disabled:opacity-50",
                disabled: *is_delivering.read(),
                onclick: move |_| {
                    let ticket_id = ticket_clone.id;
                    let comment_val = if !comment.read().is_empty() { Some(comment.read().clone()) } else { None };
                    let mut tickets_ref = tickets_clone;
                    is_delivering.set(true);
                    spawn(async move {
                        let req = crate::services::resource_ticket_api::DeliverTicketRequest {
                            comment: comment_val,
                        };
                        match deliver_ticket(ticket_id, req).await {
                            Ok(updated) => {
                                tickets_ref.with_mut(|tickets| {
                                    if let Some(t) = tickets.iter_mut().find(|t| t.id == ticket_id) {
                                        *t = updated;
                                    }
                                });
                            }
                            Err(e) => {
                                tracing::error!("交付失败: {}", e);
                            }
                        }
                    });
                },
                Icon { icon: FaBoxOpen, class: "text-sm" }
                "确认交付"
            }
        }
    }
}

/// 状态时间线
#[component]
fn StatusTimeline(ticket: ResourceTicket) -> Element {
    rsx! {
        div { class: "bg-white rounded-xl shadow-sm border border-gray-200 p-6",
            h3 { class: "text-lg font-semibold text-gray-800 mb-4", "流程进度" }
            div { class: "space-y-4",
                TimelineItem {
                    icon_data: "fa-paper-plane",
                    label: "提交申请",
                    time: "{ticket.created_at}",
                    user: "{ticket.created_by}",
                    completed: true,
                    current: false,
                }
                TimelineItem {
                    icon_data: "fa-check",
                    label: "审批通过",
                    time: ticket.approve_time.clone().unwrap_or_else(|| "待审批".to_string()),
                    user: ticket.approver.clone().unwrap_or_else(|| "-".to_string()),
                    completed: ticket.approver.is_some(),
                    current: ticket.ticket_status == TicketStatus::PendingApproval,
                }
                TimelineItem {
                    icon_data: "fa-wrench",
                    label: "配置完成",
                    time: ticket.provision_time.clone().unwrap_or_else(|| "待配置".to_string()),
                    user: ticket.provisioner.clone().unwrap_or_else(|| "-".to_string()),
                    completed: ticket.provisioner.is_some(),
                    current: ticket.ticket_status == TicketStatus::PendingProvision
                        || ticket.ticket_status == TicketStatus::Provisioning,
                }
                TimelineItem {
                    icon_data: "fa-box-open",
                    label: "已交付",
                    time: ticket.deliver_time.clone().unwrap_or_else(|| "待交付".to_string()),
                    user: ticket.deliverer.clone().unwrap_or_else(|| "-".to_string()),
                    completed: ticket.deliverer.is_some(),
                    current: ticket.ticket_status == TicketStatus::PendingDelivery,
                }
            }
        }
    }
}

/// 时间线项
#[component]
fn TimelineItem(
    #[props(!optional)] icon_data: &'static str,
    label: String,
    time: String,
    user: String,
    completed: bool,
    current: bool,
) -> Element {
    rsx! {
        div { class: "flex gap-3",
            div { class: "flex flex-col items-center",
                div {
                    class: format!(
                        "w-10 h-10 rounded-full flex items-center justify-center {}",
                        if completed { "bg-green-500 text-white" }
                        else if current { "bg-blue-500 text-white animate-pulse" }
                        else { "bg-gray-200 text-gray-400" }
                    ),
                    i { class: "fa {icon_data} text-sm" }
                }
                if !completed {
                    div { class: "w-0.5 h-full bg-gray-200 mt-1" }
                }
            }
            div { class: "flex-1 pb-4",
                div { class: "flex items-center justify-between",
                    p { class: "font-medium text-gray-800", "{label}" }
                    if completed || current {
                        span { class: "text-xs text-gray-500", "{time}" }
                    }
                }
                p { class: "text-sm text-gray-500", "操作人: {user}" }
            }
        }
    }
}

/// 信息行组件
#[component]
fn InfoRow<T: std::fmt::Display + Clone + PartialEq + 'static>(
    label: &'static str,
    value: T,
) -> Element {
    let value_str = value.to_string();
    rsx! {
        div {
            p { class: "text-xs text-gray-500 uppercase tracking-wider mb-0.5", "{label}" }
            div { class: "text-sm text-gray-900", "{value_str}" }
        }
    }
}

/// 信息行组件（Element版本）
#[component]
fn InfoRowElement(label: &'static str, value: Element) -> Element {
    rsx! {
        div {
            p { class: "text-xs text-gray-500 uppercase tracking-wider mb-0.5", "{label}" }
            div { class: "text-sm text-gray-900", {value} }
        }
    }
}

/// 新建申请表单
#[component]
fn NewTicketForm(
    _tickets: Signal<Vec<ResourceTicket>>,
    _default_resource_type: ResourceType,
    _on_cancel: Callback<()>,
    _on_submit: Callback<()>,
) -> Element {
    rsx! {}
}
