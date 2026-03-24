use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaArrowLeft, FaBoxOpen, FaCheck, FaCircleCheck, FaCircleXmark, FaClock, FaCloud, FaFile,
    FaFileLines, FaGear, FaMagnifyingGlass, FaPaperPlane, FaPlus, FaServer, FaShieldHalved,
    FaTicket, FaWrench, FaXmark,
};
use dioxus_free_icons::Icon;

use crate::app::CLOUD_PLATFORMS_STATE;
use crate::app::MACHINE_ROOMS_STATE;
use crate::app::PROVIDERS_STATE;
use crate::app::SECURITY_PRODUCTS_STATE;
use crate::components::security_product::security_product_selector::{
    SecurityProductSelector, SelectedSecurityProducts,
};
use crate::services::{
    cloud_platform_api::fetch_cloud_platform_configs,
    ip_zone_api::fetch_ip_zones,
    machine_room_api::fetch_machine_rooms,
    resource_ticket_api::{
        approve_ticket, create_resource_ticket, deliver_ticket, fetch_resource_tickets,
        provision_ticket,
    },
    security_product_api::fetch_security_products,
    service_provider_api::fetch_service_providers,
};
use crate::state::resource_ticket::{ResourceTicket, ResourceType, TicketStatus};
use crate::state::security_product::{SecurityProductCategory, SecurityProductStatus};
use crate::state::user_role::{use_auth, ApplicationTab, UserRole};

// 导入三个模块的表单组件和请求类型
use crate::components::resource_ticket::cloud_service::cloud_service_request::CloudServiceRequest;
use crate::components::resource_ticket::cloud_service::CloudServiceForm;
use crate::components::resource_ticket::network_policy::network_policy_request::NetworkPolicyRequest;
use crate::components::resource_ticket::network_policy::NetworkPolicyForm;
use crate::components::resource_ticket::physical_server::physical_server_request::PhysicalServerRequest;
use crate::components::resource_ticket::physical_server::PhysicalServerForm;

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
                ApplicationTab::MyApplications => true,
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
    current_role: UserRole,
    on_select: Callback<i32>,
) -> Element {
    let all_tickets = tickets.read().clone();
    let filtered_tickets = match current_tab {
        ApplicationTab::MyApplications => all_tickets.clone(),
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
    current_role: UserRole,
    on_back: Callback<()>,
) -> Element {
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
                    if ticket.approver.is_some() || current_role.can_approve() {
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
                    if ticket.provisioner.is_some() || current_role.can_provision() {
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
                    if ticket.deliverer.is_some() || current_role.can_deliver() {
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
    tickets: Signal<Vec<ResourceTicket>>,
    default_resource_type: ResourceType,
    on_cancel: Callback<()>,
    on_submit: Callback<()>,
) -> Element {
    let auth = use_auth();
    let mut ecs_name = use_signal(String::new);
    let mut application_name = use_signal(String::new);
    let mut contract_name = use_signal(String::new);
    let mut customer_name = use_signal(String::new);
    let mut selected_provider_id = use_signal(|| Option::<i32>::None);
    let mut available_zone_names = use_signal(Vec::<String>::new);
    let security_defaults_initialized = use_signal(|| false);

    use_effect(move || {
        spawn(async move {
            if PROVIDERS_STATE.read().is_empty() {
                if let Ok(data) = fetch_service_providers().await {
                    *PROVIDERS_STATE.write() = data;
                }
            }

            if CLOUD_PLATFORMS_STATE.read().is_empty() {
                if let Ok(data) = fetch_cloud_platform_configs().await {
                    *CLOUD_PLATFORMS_STATE.write() = data;
                }
            }

            if MACHINE_ROOMS_STATE.read().is_empty() {
                if let Ok(data) = fetch_machine_rooms().await {
                    *MACHINE_ROOMS_STATE.write() = data;
                }
            }

            if SECURITY_PRODUCTS_STATE.read().is_empty() {
                if let Ok(data) = fetch_security_products().await {
                    *SECURITY_PRODUCTS_STATE.write() = data;
                }
            }

            if available_zone_names.read().is_empty() {
                if let Ok(data) = fetch_ip_zones().await {
                    let mut zone_names = data
                        .into_iter()
                        .map(|zone| zone.name)
                        .filter(|name| !name.trim().is_empty())
                        .collect::<Vec<_>>();
                    zone_names.sort();
                    zone_names.dedup();
                    available_zone_names.set(zone_names);
                }
            }
        });
    });

    let service_providers = PROVIDERS_STATE.read().clone();
    let cloud_platforms_all = CLOUD_PLATFORMS_STATE.read().clone();
    let machine_rooms_all = MACHINE_ROOMS_STATE.read().clone();
    let security_products_all = SECURITY_PRODUCTS_STATE.read().clone();
    let network_zone_options = {
        let zones = available_zone_names.read().clone();
        if zones.is_empty() {
            vec![
                "互联网DMZ".to_string(),
                "政务网DMZ".to_string(),
                "办公网".to_string(),
                "数据中心".to_string(),
                "可信区".to_string(),
            ]
        } else {
            zones
        }
    };
    let selected_provider_value = selected_provider_id
        .read()
        .as_ref()
        .map(|id| id.to_string())
        .unwrap_or_default();

    // 为闭包克隆数据
    let service_providers_for_select = service_providers.clone();

    {
        let provider_ids = service_providers
            .iter()
            .map(|provider| provider.id)
            .collect::<Vec<_>>();
        let mut selected_provider_id = selected_provider_id;
        use_effect(move || {
            let current_provider_id = *selected_provider_id.read();
            let provider_is_valid = current_provider_id
                .map(|id| provider_ids.contains(&id))
                .unwrap_or(false);

            if !provider_is_valid {
                selected_provider_id.set(provider_ids.first().copied());
            }
        });
    }

    // 选中的云平台和机房
    let mut selected_cloud_platform_id = use_signal(|| Option::<i32>::None);
    let mut selected_machine_room_id = use_signal(|| Option::<i32>::None);
    let resource_type = default_resource_type; // 直接使用传入的类型，不可切换
    let zone_name = use_signal(String::new); // 区域
    let mut zone_cabinet = use_signal(String::new); // 机柜
    let mut rack_units = use_signal(|| 0i32); // 机位(U数)
    let mut ecs_type = use_signal(|| "ecs.g6.xlarge".to_string());
    let mut ecs_os = use_signal(|| "CentOS 7.9".to_string());
    let mut data_disk_type = use_signal(String::new);
    let mut data_disk_size = use_signal(String::new);
    let mut cpu_cores = use_signal(|| 4);
    let mut memory_gb = use_signal(|| 16);
    let mut system_disk = use_signal(|| "SSD".to_string());
    let mut system_disk_size = use_signal(|| 100);
    let has_security = use_signal(|| true);
    let selected_security_products = use_signal(SelectedSecurityProducts::default);
    let mut show_security_selector = use_signal(|| false);
    let mut remarks = use_signal(String::new);

    {
        let security_products = security_products_all.clone();
        let mut selected_security_products = selected_security_products;
        let mut security_defaults_initialized = security_defaults_initialized;
        use_effect(move || {
            if *security_defaults_initialized.read() || security_products.is_empty() {
                return;
            }

            let mut defaults = SelectedSecurityProducts::default();
            for category in [
                SecurityProductCategory::Bastion,
                SecurityProductCategory::Vpn,
                SecurityProductCategory::Siem,
            ] {
                let matches = security_products
                    .iter()
                    .filter(|product| {
                        product.category == category
                            && product.status == SecurityProductStatus::Active
                    })
                    .collect::<Vec<_>>();

                if matches.len() == 1 {
                    defaults.set(category, matches[0].id);
                }
            }

            if !defaults.is_empty() {
                selected_security_products.set(defaults);
            }

            security_defaults_initialized.set(true);
        });
    }

    // 网络策略专用字段
    let mut fw_source_zone = use_signal(String::new);
    let mut fw_source_address = use_signal(String::new);
    let mut fw_dest_zone = use_signal(String::new);
    let mut fw_dest_address = use_signal(String::new);
    let mut fw_protocol = use_signal(|| "TCP".to_string());
    let mut fw_port = use_signal(String::new);
    let mut fw_direction = use_signal(|| "入站".to_string());
    let mut fw_valid_until = use_signal(String::new);
    let mut fw_firewall_name = use_signal(String::new);

    rsx! {
        // 模态框背景层
        div {
            class: "fixed inset-0 z-50 flex items-center justify-center bg-black/40",
            onclick: move |_| on_cancel.call(()),

            // 模态框内容
            div {
                class: "bg-white rounded-lg shadow-xl w-full max-w-2xl max-h-[85vh] overflow-y-auto m-4",
                onclick: |e| e.stop_propagation(),

                // 标题栏
                div { class: "flex items-center justify-between px-5 py-3 border-b border-gray-200 bg-gray-50",
                    h2 { class: "text-base font-semibold text-gray-800 flex items-center gap-2",
                        if resource_type == ResourceType::Cloud {
                            Icon { icon: FaCloud, class: "text-blue-500 w-4 h-4" }
                        } else if resource_type == ResourceType::Physical {
                            Icon { icon: FaServer, class: "text-blue-500 w-4 h-4" }
                        } else {
                            Icon { icon: FaShieldHalved, class: "text-blue-500 w-4 h-4" }
                        }
                        "新建{resource_type.display_name()}申请"
                    }
                    button {
                        class: "p-1 text-gray-400 hover:text-gray-600 hover:bg-gray-200 rounded transition-colors",
                        onclick: move |_| on_cancel.call(()),
                        Icon { icon: FaXmark, width: 16, height: 16 }
                    }
                }

                // 表单内容区域
                div { class: "p-5",
                    form {
                        class: "space-y-4",
                onsubmit: move |e: dioxus::prelude::Event<FormData>| {
                    e.prevent_default();
                    let id = (tickets.read().len() + 1) as i32;

                    // 获取选中的服务商信息
                    let provider_id = *selected_provider_id.read();
                    let provider_name = service_providers.iter()
                        .find(|p| Some(p.id) == provider_id)
                        .map(|p| p.short_name.clone())
                        .unwrap_or_default();

                    // 获取选中的云平台信息
                    let cloud_platform_id = *selected_cloud_platform_id.read();
                    let cloud_platform_name = if let Some(platform_id) = cloud_platform_id {
                        cloud_platforms_all.iter()
                            .find(|p| p.id == platform_id)
                            .map(|p| p.platform_name.clone())
                            .unwrap_or_default()
                    } else {
                        String::new()
                    };

                    // 获取选中的机房信息
                    let machine_room_id = *selected_machine_room_id.read();
                    let machine_room_name = if let Some(room_id) = machine_room_id {
                        machine_rooms_all.iter()
                            .find(|r| r.id == room_id)
                            .map(|r| r.room_name.clone())
                            .unwrap_or_default()
                    } else {
                        String::new()
                    };

                    let new_app = ResourceTicket {
                        id,
                        resource_type,
                        ecs_name: ecs_name.read().clone(),
                        ticket_status: TicketStatus::PendingApproval,
                        provider_id,
                        provider_name,
                        cloud_platform_id,
                        cloud_platform_name,
                        machine_room_id: if resource_type == ResourceType::Cloud { None } else { machine_room_id },
                        machine_room_name: if resource_type == ResourceType::Cloud { String::new() } else { machine_room_name },
                        cloud_region: String::new(),
                        cloud_category: match resource_type {
                            ResourceType::Cloud => "云主机".to_string(),
                            ResourceType::Physical => "物理机".to_string(),
                            ResourceType::Network => "网络策略".to_string(),
                        },
                        zone_name: if resource_type == ResourceType::Cloud { String::new() } else { zone_name.read().clone() },
                        zone_cabinet: if resource_type == ResourceType::Physical { zone_cabinet.read().clone() } else { String::new() },
                        rack_units: if resource_type == ResourceType::Physical { *rack_units.read() } else { 0 },
                        customer_name: customer_name.read().clone(),
                        application_name: application_name.read().clone(),
                        contract_name: contract_name.read().clone(),
                        ecs_type: ecs_type.read().clone(),
                        ecs_os: ecs_os.read().clone(),
                        cpu_cores: *cpu_cores.read(),
                        memory_gb: *memory_gb.read(),
                        system_disk: system_disk.read().clone(),
                        system_disk_size_gb: *system_disk_size.read(),
                        data_disk: if data_disk_type.read().is_empty() {
                            String::new()
                        } else {
                            format!("{} {}GB", data_disk_type.read(), data_disk_size.read())
                        },
                        has_security_product: if resource_type == ResourceType::Network {
                            false
                        } else {
                            *has_security.read()
                        },
                        security_products: if resource_type == ResourceType::Network {
                            String::new()
                        } else {
                            selected_security_products.read().to_names_string(&SECURITY_PRODUCTS_STATE.read())
                        },
                        ip_address: String::new(),
                        delivery_status: "未交付".to_string(),
                        remarks: remarks.read().clone(),
                        created_at: chrono::Local::now().format("%Y-%m-%d %H:%M").to_string(),
                        updated_at: chrono::Local::now().format("%Y-%m-%d %H:%M").to_string(),
                        created_by: auth.read().username.clone(),
                        approver: None,
                        approve_time: None,
                        approve_comment: None,
                        provisioner: None,
                        provision_time: None,
                        provision_details: None,
                        deliverer: None,
                        deliver_time: None,
                        deliver_comment: None,

                        // 网络策略字段（网络策略类型时使用）
                        fw_source_zone: if resource_type == ResourceType::Network {
                            Some(fw_source_zone.read().clone())
                        } else {
                            None
                        },
                        fw_source_address: if resource_type == ResourceType::Network {
                            Some(fw_source_address.read().clone())
                        } else {
                            None
                        },
                        fw_dest_zone: if resource_type == ResourceType::Network {
                            Some(fw_dest_zone.read().clone())
                        } else {
                            None
                        },
                        fw_dest_address: if resource_type == ResourceType::Network {
                            Some(fw_dest_address.read().clone())
                        } else {
                            None
                        },
                        fw_protocol: if resource_type == ResourceType::Network {
                            Some(fw_protocol.read().clone())
                        } else {
                            None
                        },
                        fw_port: if resource_type == ResourceType::Network {
                            Some(fw_port.read().clone())
                        } else {
                            None
                        },
                        fw_direction: if resource_type == ResourceType::Network {
                            Some(fw_direction.read().clone())
                        } else {
                            None
                        },
                        fw_valid_until: if resource_type == ResourceType::Network {
                            Some(fw_valid_until.read().clone())
                        } else {
                            None
                        },
                        fw_firewall_name: if resource_type == ResourceType::Network {
                            Some(fw_firewall_name.read().clone())
                        } else {
                            None
                        },
                    };
                    let mut tickets_ref = tickets;
                    let on_submit_callback = on_submit;
                    spawn(async move {
                        match create_resource_ticket(&new_app).await {
                            Ok(created) => {
                                tickets_ref.with_mut(|apps| {
                                    apps.push(created);
                                });
                                on_submit_callback.call(());
                            }
                            Err(e) => {
                                tracing::error!("创建工单失败: {}", e);
                            }
                        }
                    });
                },

                // 基本信息
                div { class: "grid grid-cols-2 gap-3",
                    div {
                        label { class: "block text-xs font-medium text-gray-600 mb-0.5", "申请名称*" }
                        input {
                            class: "w-full px-2.5 py-1.5 text-sm border border-gray-200 rounded focus:ring-1 focus:ring-blue-500 focus:border-blue-500",
                            placeholder: "请输入申请名称",
                            value: "{application_name}",
                            required: true,
                            oninput: move |e| application_name.set(e.value())
                        }
                    }
                    div {
                        label { class: "block text-xs font-medium text-gray-600 mb-0.5", "实例名称*" }
                        input {
                            class: "w-full px-2.5 py-1.5 text-sm border border-gray-200 rounded focus:ring-1 focus:ring-blue-500 focus:border-blue-500",
                            placeholder: "Web服务器-01",
                            value: "{ecs_name}",
                            required: true,
                            oninput: move |e| ecs_name.set(e.value())
                        }
                    }
                    div {
                        label { class: "block text-xs font-medium text-gray-600 mb-0.5", "客户名称*" }
                        input {
                            class: "w-full px-2.5 py-1.5 text-sm border border-gray-200 rounded focus:ring-1 focus:ring-blue-500 focus:border-blue-500",
                            placeholder: "客户单位名称",
                            value: "{customer_name}",
                            required: true,
                            oninput: move |e| customer_name.set(e.value())
                        }
                    }
                    div {
                        label { class: "block text-xs font-medium text-gray-600 mb-0.5", "合同名称*" }
                        input {
                            class: "w-full px-2.5 py-1.5 text-sm border border-gray-200 rounded focus:ring-1 focus:ring-blue-500 focus:border-blue-500",
                            placeholder: "合同-2024-001",
                            value: "{contract_name}",
                            required: true,
                            oninput: move |e| contract_name.set(e.value())
                        }
                    }
                }

                // 资源配置
                div { class: "border-t border-gray-100 pt-3 mt-3",
                    h3 { class: "text-sm font-medium text-gray-700 mb-3",
                        if resource_type == ResourceType::Cloud {
                            "云资源配置"
                        } else if resource_type == ResourceType::Physical {
                            "物理资源配置"
                        } else {
                            "网络策略配置"
                        }
                    }
                    div { class: "grid grid-cols-2 gap-3",
                        div {
                            label { class: "block text-xs font-medium text-gray-600 mb-0.5", "服务商*" }
                            select {
                                class: "w-full px-2.5 py-1.5 text-sm border border-gray-200 rounded focus:ring-1 focus:ring-blue-500 focus:border-blue-500",
                                required: true,
                                value: "{selected_provider_value}",
                                onchange: move |e| {
                                    if let Ok(id) = e.value().parse::<i32>() {
                                        selected_provider_id.set(Some(id));
                                        // 服务商改变时重置已选择的云平台和机房
                                        selected_cloud_platform_id.set(None);
                                        selected_machine_room_id.set(None);
                                    }
                                },
                                option { value: "", disabled: true, "请选择服务商" }
                                for provider in &service_providers_for_select {
                                    option {
                                        value: "{provider.id}",
                                        selected: *selected_provider_id.read() == Some(provider.id),
                                        "{provider.short_name}"
                                    }
                                }
                            }
                        }
                        // 云资源专用字段
                        if resource_type == ResourceType::Cloud {
                            div {
                                label { class: "block text-xs font-medium text-gray-600 mb-0.5", "云平台*" }
                                select {
                                    class: "w-full px-2.5 py-1.5 text-sm border border-gray-200 rounded focus:ring-1 focus:ring-blue-500 focus:border-blue-500",
                                    required: true,
                                    oninput: move |e| {
                                        let platform_id: i32 = e.value().parse().unwrap_or(0);
                                        selected_cloud_platform_id.set(Some(platform_id));
                                    },
                                    option { value: "-1", "请先选择服务商" }
                                    // 根据服务商过滤云平台
                                    for platform in cloud_platforms_all.iter().filter(|p| Some(p.provider_id) == *selected_provider_id.read()) {
                                        option {
                                            value: "{platform.id}",
                                            selected: *selected_cloud_platform_id.read() == Some(platform.id),
                                            "{platform.platform_name}"
                                        }
                                    }
                                }
                            }
                        }
                        // 物理资源专用字段
                        if resource_type == ResourceType::Physical {
                            div {
                                label { class: "block text-xs font-medium text-gray-600 mb-0.5", "服务器型号*" }
                                input {
                                    class: "w-full px-2.5 py-1.5 text-sm border border-gray-200 rounded focus:ring-1 focus:ring-blue-500 focus:border-blue-500",
                                    placeholder: "如：Dell R740",
                                    required: true,
                                    value: "{ecs_type}",
                                    oninput: move |e| ecs_type.set(e.value())
                                }
                            }
                            div {
                                label { class: "block text-xs font-medium text-gray-600 mb-0.5", "机房*" }
                                select {
                                    class: "w-full px-2.5 py-1.5 text-sm border border-gray-200 rounded focus:ring-1 focus:ring-blue-500 focus:border-blue-500",
                                    required: true,
                                    oninput: move |e| {
                                        let room_id: i32 = e.value().parse().unwrap_or(0);
                                        selected_machine_room_id.set(if room_id > 0 { Some(room_id) } else { None });
                                    },
                                    option { value: "-1", "请先选择服务商" }
                                    // 根据服务商过滤机房
                                    for room in machine_rooms_all.iter().filter(|r| Some(r.provider_id) == *selected_provider_id.read()) {
                                        option {
                                            value: "{room.id}",
                                            selected: *selected_machine_room_id.read() == Some(room.id),
                                            "{room}"
                                        }
                                    }
                                }
                            }
                            div {
                                label { class: "block text-xs font-medium text-gray-600 mb-0.5", "机柜" }
                                input {
                                    class: "w-full px-2.5 py-1.5 text-sm border border-gray-200 rounded focus:ring-1 focus:ring-blue-500 focus:border-blue-500",
                                    placeholder: "如：03柜",
                                    value: "{zone_cabinet}",
                                    oninput: move |e| zone_cabinet.set(e.value())
                                }
                            }
                            div {
                                label { class: "block text-xs font-medium text-gray-600 mb-0.5", "机位(U)" }
                                input {
                                    class: "w-full px-2.5 py-1.5 text-sm border border-gray-200 rounded focus:ring-1 focus:ring-blue-500 focus:border-blue-500",
                                    r#type: "number",
                                    min: 1,
                                    max: 42,
                                    placeholder: "如：20",
                                    value: "{rack_units}",
                                    oninput: move |e| rack_units.set(e.value().parse().unwrap_or(0))
                                }
                            }
                        }
                        // 网络策略专用字段
                        if resource_type == ResourceType::Network {
                            div {
                                label { class: "block text-xs font-medium text-gray-600 mb-0.5", "源区域*" }
                                select {
                                    class: "w-full px-2.5 py-1.5 text-sm border border-gray-200 rounded focus:ring-1 focus:ring-blue-500 focus:border-blue-500",
                                    required: true,
                                    value: "{fw_source_zone}",
                                    oninput: move |e| fw_source_zone.set(e.value()),
                                    option { value: "", "请选择" }
                                    for zone_name in network_zone_options.iter() {
                                        option { value: "{zone_name}", "{zone_name}" }
                                    }
                                }
                            }
                            div {
                                label { class: "block text-xs font-medium text-gray-600 mb-0.5", "源地址/IP*" }
                                input {
                                    class: "w-full px-2.5 py-1.5 text-sm border border-gray-200 rounded focus:ring-1 focus:ring-blue-500 focus:border-blue-500",
                                    placeholder: "如: 192.168.1.0/24",
                                    required: true,
                                    value: "{fw_source_address}",
                                    oninput: move |e| fw_source_address.set(e.value())
                                }
                            }
                            div {
                                label { class: "block text-xs font-medium text-gray-600 mb-0.5", "目标区域*" }
                                select {
                                    class: "w-full px-2.5 py-1.5 text-sm border border-gray-200 rounded focus:ring-1 focus:ring-blue-500 focus:border-blue-500",
                                    required: true,
                                    value: "{fw_dest_zone}",
                                    oninput: move |e| fw_dest_zone.set(e.value()),
                                    option { value: "", "请选择" }
                                    for zone_name in network_zone_options.iter() {
                                        option { value: "{zone_name}", "{zone_name}" }
                                    }
                                }
                            }
                            div {
                                label { class: "block text-xs font-medium text-gray-600 mb-0.5", "目标地址/IP*" }
                                input {
                                    class: "w-full px-2.5 py-1.5 text-sm border border-gray-200 rounded focus:ring-1 focus:ring-blue-500 focus:border-blue-500",
                                    placeholder: "如: 10.1.1.100",
                                    required: true,
                                    value: "{fw_dest_address}",
                                    oninput: move |e| fw_dest_address.set(e.value())
                                }
                            }
                            div {
                                label { class: "block text-xs font-medium text-gray-600 mb-0.5", "协议*" }
                                select {
                                    class: "w-full px-2.5 py-1.5 text-sm border border-gray-200 rounded focus:ring-1 focus:ring-blue-500 focus:border-blue-500",
                                    required: true,
                                    value: "{fw_protocol}",
                                    oninput: move |e| fw_protocol.set(e.value()),
                                    option { value: "TCP", "TCP" }
                                    option { value: "UDP", "UDP" }
                                    option { value: "ICMP", "ICMP" }
                                    option { value: "ANY", "ANY" }
                                }
                            }
                            div {
                                label { class: "block text-xs font-medium text-gray-600 mb-0.5", "端口*" }
                                input {
                                    class: "w-full px-2.5 py-1.5 text-sm border border-gray-200 rounded focus:ring-1 focus:ring-blue-500 focus:border-blue-500",
                                    placeholder: "如: 443 或 8080-8090",
                                    required: true,
                                    value: "{fw_port}",
                                    oninput: move |e| fw_port.set(e.value())
                                }
                            }
                            div {
                                label { class: "block text-xs font-medium text-gray-600 mb-0.5", "访问方向*" }
                                select {
                                    class: "w-full px-2.5 py-1.5 text-sm border border-gray-200 rounded focus:ring-1 focus:ring-blue-500 focus:border-blue-500",
                                    required: true,
                                    value: "{fw_direction}",
                                    oninput: move |e| fw_direction.set(e.value()),
                                    option { value: "入站", "入站" }
                                    option { value: "出站", "出站" }
                                    option { value: "双向", "双向" }
                                }
                            }
                            div {
                                label { class: "block text-xs font-medium text-gray-600 mb-0.5", "有效期至*" }
                                input {
                                    class: "w-full px-2.5 py-1.5 text-sm border border-gray-200 rounded focus:ring-1 focus:ring-blue-500 focus:border-blue-500",
                                    r#type: "date",
                                    required: true,
                                    value: "{fw_valid_until}",
                                    oninput: move |e| fw_valid_until.set(e.value())
                                }
                            }
                            div {
                                label { class: "block text-xs font-medium text-gray-600 mb-0.5", "防火墙设备*" }
                                input {
                                    class: "w-full px-2.5 py-1.5 text-sm border border-gray-200 rounded focus:ring-1 focus:ring-blue-500 focus:border-blue-500",
                                    placeholder: "如: 核心防火墙-01",
                                    required: true,
                                    value: "{fw_firewall_name}",
                                    oninput: move |e| fw_firewall_name.set(e.value())
                                }
                            }
                        }
                        div {
                            label { class: "block text-xs font-medium text-gray-600 mb-0.5", "操作系统" }
                            select {
                                class: "w-full px-2.5 py-1.5 text-sm border border-gray-200 rounded focus:ring-1 focus:ring-blue-500 focus:border-blue-500",
                                value: "{ecs_os}",
                                oninput: move |e| ecs_os.set(e.value()),
                                option { value: "CentOS 7.9", "CentOS 7.9" }
                                option { value: "CentOS 8.4", "CentOS 8.4" }
                                option { value: "Ubuntu 20.04", "Ubuntu 20.04" }
                                option { value: "Ubuntu 22.04", "Ubuntu 22.04" }
                            }
                        }
                        div {
                            label { class: "block text-xs font-medium text-gray-600 mb-0.5", "CPU核数" }
                            input {
                                class: "w-full px-2.5 py-1.5 text-sm border border-gray-200 rounded focus:ring-1 focus:ring-blue-500 focus:border-blue-500",
                                r#type: "number",
                                min: 1,
                                max: 128,
                                value: "{cpu_cores}",
                                oninput: move |e| cpu_cores.set(e.value().parse().unwrap_or(4))
                            }
                        }
                        div {
                            label { class: "block text-xs font-medium text-gray-600 mb-0.5", "内存(GB)" }
                            input {
                                class: "w-full px-2.5 py-1.5 text-sm border border-gray-200 rounded focus:ring-1 focus:ring-blue-500 focus:border-blue-500",
                                r#type: "number",
                                min: 1,
                                max: 1024,
                                value: "{memory_gb}",
                                oninput: move |e| memory_gb.set(e.value().parse().unwrap_or(16))
                            }
                        }
                        div {
                            label { class: "block text-xs font-medium text-gray-600 mb-0.5", "系统盘类型" }
                            select {
                                class: "w-full px-2.5 py-1.5 text-sm border border-gray-200 rounded focus:ring-1 focus:ring-blue-500 focus:border-blue-500",
                                value: "{system_disk}",
                                oninput: move |e| system_disk.set(e.value()),
                                option { value: "SSD", "SSD" }
                                option { value: "NVMe", "NVMe" }
                            }
                        }
                        div {
                            label { class: "block text-xs font-medium text-gray-600 mb-0.5", "系统盘大小(GB)" }
                            input {
                                class: "w-full px-2.5 py-1.5 text-sm border border-gray-200 rounded focus:ring-1 focus:ring-blue-500 focus:border-blue-500",
                                r#type: "number",
                                min: 20,
                                max: 5000,
                                value: "{system_disk_size}",
                                oninput: move |e| system_disk_size.set(e.value().parse().unwrap_or(100))
                            }
                        }
                        div {
                            label { class: "block text-xs font-medium text-gray-600 mb-0.5", "数据盘类型" }
                            select {
                                class: "w-full px-2.5 py-1.5 text-sm border border-gray-200 rounded focus:ring-1 focus:ring-blue-500 focus:border-blue-500",
                                value: "{data_disk_type}",
                                oninput: move |e| {
                                    let new_value = e.value();
                                    data_disk_type.set(new_value.clone());
                                    // 当选择数据盘类型时，默认大小设为500；当选择"无"时，清空大小
                                    if new_value.is_empty() {
                                        data_disk_size.set(String::new());
                                    } else if data_disk_size.read().is_empty() || data_disk_size.read().as_str() == "0" {
                                        data_disk_size.set("500".to_string());
                                    }
                                },
                                option { value: "", "无" }
                                option { value: "SSD", "SSD" }
                                option { value: "NVMe", "NVMe" }
                                option { value: "SATA", "SATA" }
                            }
                        }
                        div {
                            label { class: "block text-xs font-medium text-gray-600 mb-0.5", "数据盘大小(GB)" }
                            input {
                                class: format!("w-full px-3 py-2 border rounded-lg {} {}",
                                    if data_disk_type.read().is_empty() {
                                        "bg-gray-100 border-gray-300 text-gray-400 cursor-not-allowed"
                                    } else {
                                        "border-gray-300 focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                                    },
                                    if data_disk_type.read().is_empty() { "disabled" } else { "" }
                                ),
                                r#type: "number",
                                min: 0,
                                max: 10000,
                                placeholder: "500",
                                value: "{data_disk_size}",
                                disabled: data_disk_type.read().is_empty(),
                                oninput: move |e| data_disk_size.set(e.value())
                            }
                        }
                    }
                    // 安全产品选择（仅云服务和物理机显示）
                    if resource_type != ResourceType::Network {
                        div { class: "border-t pt-3 mt-2",
                            div { class: "flex items-center justify-between mb-2",
                                h4 { class: "text-sm font-medium text-gray-700", "安全产品选择" }
                                {
                                    let is_expanded = *show_security_selector.read();
                                    rsx! {
                                        button {
                                            class: "text-sm text-blue-600 hover:text-blue-800",
                                            onclick: move |_| show_security_selector.set(!is_expanded),
                                            if is_expanded {
                                                "收起选择器"
                                            } else {
                                                "展开选择器"
                                            }
                                        }
                                    }
                                }
                            }
                            p { class: "text-xs text-gray-500 mb-2", "选择需要绑定的安全产品，每个分类只能选择一个" }

                            if *show_security_selector.read() {
                                SecurityProductSelector {
                                    selected: selected_security_products,
                                    active_only: true,
                                }
                            } else {
                                // 显示已选择的安全产品摘要
                                div { class: "bg-gray-50 rounded-lg p-3",
                                    if selected_security_products.read().is_empty() {
                                        p { class: "text-sm text-gray-400", "尚未选择安全产品" }
                                    } else {
                                        div { class: "space-y-1",
                                            for (category, product_id) in selected_security_products.read().products.iter() {
                                                {
                                                    let products = crate::app::SECURITY_PRODUCTS_STATE.read();
                                                    let product_name = products.iter()
                                                        .find(|p| p.id == *product_id)
                                                        .map(|p| p.name.clone())
                                                        .unwrap_or_else(|| format!("产品{}", product_id));
                                                    rsx! {
                                                        div { class: "flex items-center text-sm",
                                                            dioxus_free_icons::Icon { icon: FaShieldHalved, width: 14, height: 14, class: "text-indigo-500 mr-2" }
                                                            span { class: "text-gray-600", "{category.display_name()}: " }
                                                            span { class: "font-medium text-gray-800", "{product_name}" }
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

                // 备注
                div { class: "mt-3",
                    label { class: "block text-xs font-medium text-gray-600 mb-0.5", "备注说明" }
                    textarea {
                        class: "w-full px-2.5 py-1.5 text-sm border border-gray-200 rounded focus:ring-1 focus:ring-blue-500 focus:border-blue-500",
                        rows: 2,
                        placeholder: "申请用途、特殊需求等",
                        value: "{remarks}",
                        oninput: move |e| remarks.set(e.value())
                    }
                }

                    // 提交按钮
                    div { class: "flex gap-2 pt-3 mt-3 border-t border-gray-100",
                        button {
                            class: "flex-1 px-4 py-2 bg-blue-600 text-white text-sm rounded hover:bg-blue-700 font-medium flex items-center justify-center gap-1.5 transition-colors",
                            r#type: "submit",
                            Icon { icon: FaPaperPlane, class: "w-3.5 h-3.5" }
                            "提交申请"
                        }
                        button {
                            class: "px-4 py-2 text-sm border border-gray-200 text-gray-600 rounded hover:bg-gray-50 font-medium",
                            r#type: "button",
                            onclick: move |_| on_cancel.call(()),
                            "取消"
                        }
                    }
                }
                } // end p-5 wrapper
            } // end modal content
        } // end backdrop
    }
}
