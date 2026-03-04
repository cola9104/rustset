use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaPlus, FaMagnifyingGlass, FaTicket, FaServer, FaCloud,
    FaCheck, FaWrench, FaBoxOpen, FaPaperPlane, FaXmark, FaFileLines,
    FaCircleCheck, FaCircleXmark, FaGear, FaArrowLeft, FaFile,
    FaShieldHalved, FaClock,
};

use crate::state::user_role::{use_auth, UserRole, ApplicationTab};
use crate::state::resource_ticket::{
    TicketStatus, ResourceTicket, ResourceType, init_test_tickets,
};
use crate::state::cloud_platform::{CloudPlatformConfig, init_cloud_platforms};
use crate::state::machine_room::{MachineRoomConfig, init_machine_rooms};
use crate::state::service_provider::{ServiceProviderConfig, init_service_providers};
use crate::app::PROVIDERS_STATE;

/// 资源工单主页面 - 基于资源类型的标签页导航 + 工作流程
#[allow(non_snake_case)]
pub fn ResourceTicket() -> Element {
    let auth = use_auth();
    let mut tickets = use_signal(init_test_tickets);
    let mut resource_type_tab = use_signal(|| ResourceType::Cloud);
    let mut workflow_tab = use_signal(|| auth.read().role.accessible_tabs().first().copied().unwrap_or(ApplicationTab::MyApplications));
    let mut search_query = use_signal(|| String::new());
    let mut selected_ticket = use_signal(|| Option::<i32>::None);
    let mut show_new_form = use_signal(|| false);

    let accessible_tabs = auth.read().role.accessible_tabs();
    let current_role = auth.read().role;

    // 计算各资源类型的待处理数量（待审批+待配置+待交付）
    let all_tickets = tickets.read().clone();

    let cloud_pending_count = all_tickets.iter()
        .filter(|t| t.resource_type == ResourceType::Cloud &&
            (t.ticket_status == TicketStatus::PendingApproval ||
             t.ticket_status == TicketStatus::PendingProvision ||
             t.ticket_status == TicketStatus::PendingDelivery))
        .count();

    let physical_pending_count = all_tickets.iter()
        .filter(|t| t.resource_type == ResourceType::Physical &&
            (t.ticket_status == TicketStatus::PendingApproval ||
             t.ticket_status == TicketStatus::PendingProvision ||
             t.ticket_status == TicketStatus::PendingDelivery))
        .count();

    let network_pending_count = all_tickets.iter()
        .filter(|t| t.resource_type == ResourceType::Network &&
            (t.ticket_status == TicketStatus::PendingApproval ||
             t.ticket_status == TicketStatus::PendingProvision ||
             t.ticket_status == TicketStatus::PendingDelivery))
        .count();

    // 计算当前资源类型的统计数据
    let current_type_tickets = all_tickets.iter()
        .filter(|t| t.resource_type == *resource_type_tab.read())
        .cloned()
        .collect::<Vec<_>>();

    let total_count = current_type_tickets.len();
    let pending_count = current_type_tickets.iter().filter(|t| t.ticket_status == TicketStatus::PendingApproval).count();
    let pending_provision_count = current_type_tickets.iter().filter(|t| t.ticket_status == TicketStatus::PendingProvision).count();
    let pending_delivery_count = current_type_tickets.iter().filter(|t| t.ticket_status == TicketStatus::PendingDelivery).count();
    let delivered_count = current_type_tickets.iter().filter(|t| t.ticket_status == TicketStatus::Delivered).count();
    let archived_count = current_type_tickets.iter().filter(|t| t.ticket_status == TicketStatus::Archived).count();

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

                    // 角色切换器（开发测试用）
                    div { class: "flex items-center gap-2",
                        label { class: "text-sm text-gray-600", "切换角色:" }
                        RoleSwitcher { current_role: auth }
                    }
                }
            }

            // 资源类型标签页导航（外层）
            div { class: "bg-gradient-to-r from-white to-gray-50 border-b border-gray-200 px-6 py-2 shadow-sm overflow-visible",
                div { class: "flex gap-2 overflow-x-auto overflow-y-visible",
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

            // 工作流程标签页导航（内层）
            div { class: "bg-white border-b border-gray-200 px-6",
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
            }

            // 工具栏
            div { class: "bg-white border-b border-gray-200 px-6 py-3",
                div { class: "flex items-center justify-between",
                    div { class: "flex items-center gap-4 flex-1",
                        // 搜索框
                        div { class: "relative flex-1 max-w-md",
                            input {
                                class: "w-full pl-10 pr-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500",
                                placeholder: "搜索工单名称、申请人、合同...",
                                value: "{search_query}",
                                oninput: move |e| search_query.set(e.value())
                            }
                            Icon { icon: FaMagnifyingGlass, class: "absolute left-3 top-1/2 -translate-y-1/2 text-gray-400" }
                        }

                        // 统计信息
                        div { class: "flex items-center gap-4 text-sm text-gray-600",
                            span { class: "flex items-center gap-1",
                                span { class: "font-semibold text-blue-600" },
                                {format!("共 {} 条", total_count)}
                            }
                        }
                    }

                    // 操作按钮
                    div { class: "flex items-center gap-2",
                        if current_role.can_submit() && *workflow_tab.read() == ApplicationTab::MyApplications {
                            button {
                                class: "px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 flex items-center gap-2 transition-colors",
                                onclick: move |_| show_new_form.set(true),
                                Icon { icon: FaPlus, class: "text-sm" }
                                "新建工单"
                            }
                        }
                    }
                }
            }

            // 主内容区
            div { class: "flex-1 overflow-auto p-6",
                // 新建工单表单
                if *show_new_form.read() {
                    NewTicketForm {
                        tickets: tickets.clone(),
                        default_resource_type: *resource_type_tab.read(),
                        on_cancel: move |_| show_new_form.set(false),
                        on_submit: move |_| {
                            show_new_form.set(false);
                        },
                    }
                } else if let Some(ticket_id) = *selected_ticket.read() {
                    // 详情视图
                    {
                        let all_tickets = tickets.read().clone();
                        if let Some(ticket) = all_tickets.iter().find(|t| t.id == ticket_id).cloned() {
                            rsx! {
                                TicketDetailView {
                                    ticket,
                                    tickets: tickets.clone(),
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
                        tickets: tickets.clone(),
                        search_query: (*search_query.read()).clone(),
                        current_role,
                        on_select: move |id| selected_ticket.set(Some(id)),
                    }
                }
            }
        }
    }
}

/// 角色切换器（开发测试用）
#[component]
fn RoleSwitcher(current_role: Signal<crate::state::user_role::AuthState>) -> Element {
    let mut is_open = use_signal(|| false);

    rsx! {
        div { class: "relative",
            button {
                class: "px-3 py-1.5 text-sm border border-gray-300 rounded-lg hover:bg-gray-50 flex items-center gap-2",
                onclick: move |_| {
                    let current = *is_open.read();
                    is_open.set(!current);
                },
                {current_role.read().role.display_name()}
                i { class: "fa fa-chevron-down text-xs" }
            }

            if *is_open.read() {
                div { class: "absolute top-full right-0 mt-1 bg-white border border-gray-200 rounded-lg shadow-lg z-10 min-w-[120px]",
                    button {
                        class: "w-full px-3 py-2 text-left text-sm hover:bg-gray-50 flex items-center gap-2",
                        onclick: move |_| {
                            current_role.write().role = UserRole::Applicant;
                            is_open.set(false);
                        },
                        "申请人员"
                    }
                    button {
                        class: "w-full px-3 py-2 text-left text-sm hover:bg-gray-50 flex items-center gap-2",
                        onclick: move |_| {
                            current_role.write().role = UserRole::Approver;
                            is_open.set(false);
                        },
                        "审批人员"
                    }
                    button {
                        class: "w-full px-3 py-2 text-left text-sm hover:bg-gray-50 flex items-center gap-2",
                        onclick: move |_| {
                            current_role.write().role = UserRole::Operator;
                            is_open.set(false);
                        },
                        "运维人员"
                    }
                    button {
                        class: "w-full px-3 py-2 text-left text-sm hover:bg-gray-50 flex items-center gap-2",
                        onclick: move |_| {
                            current_role.write().role = UserRole::Deliverer;
                            is_open.set(false);
                        },
                        "交付人员"
                    }
                    button {
                        class: "w-full px-3 py-2 text-left text-sm hover:bg-gray-50 flex items-center gap-2",
                        onclick: move |_| {
                            current_role.write().role = UserRole::Admin;
                            is_open.set(false);
                        },
                        "管理员"
                    }
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
                ApplicationTab::PendingApproval => ticket.ticket_status == TicketStatus::PendingApproval,
                ApplicationTab::PendingProvision => ticket.ticket_status == TicketStatus::PendingProvision,
                ApplicationTab::PendingDelivery => ticket.ticket_status == TicketStatus::PendingDelivery,
                ApplicationTab::Delivered => ticket.ticket_status == TicketStatus::Delivered,
                ApplicationTab::Archived => ticket.ticket_status == TicketStatus::Archived,
                ApplicationTab::NewApplication => false,
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
        ApplicationTab::PendingApproval => all_tickets.clone().into_iter().filter(|ticket| ticket.ticket_status == TicketStatus::PendingApproval).collect(),
        ApplicationTab::PendingProvision => all_tickets.clone().into_iter().filter(|ticket| ticket.ticket_status == TicketStatus::PendingProvision).collect(),
        ApplicationTab::PendingDelivery => all_tickets.clone().into_iter().filter(|ticket| ticket.ticket_status == TicketStatus::PendingDelivery).collect(),
        ApplicationTab::Delivered => all_tickets.clone().into_iter().filter(|ticket| ticket.ticket_status == TicketStatus::Delivered).collect(),
        ApplicationTab::Archived => all_tickets.clone().into_iter().filter(|ticket| ticket.ticket_status == TicketStatus::Archived).collect(),
        ApplicationTab::NewApplication => vec![],
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
                            InfoRow { label: "区域", value: ticket.cloud_region.clone() }
                            InfoRow { label: "可用区", value: ticket.zone_name.clone() }
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
                                    value: if ticket.has_security_product { "是" } else { "否" }
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
                                    tickets: tickets.clone(),
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
                                    tickets: tickets.clone(),
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
                                    tickets: tickets.clone(),
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
    let mut tickets_clone = tickets.clone();
    let ticket_clone2 = ticket.clone();
    let mut tickets_clone2 = tickets.clone();
    let mut comment = use_signal(|| String::new());
    let _is_approving = use_signal(|| false);

    rsx! {
        div { class: "space-y-4",
            p { class: "text-sm text-gray-500", "请审批此申请" }
            textarea {
                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500",
                rows: 3,
                placeholder: "审批意见（可选）",
                value: "{comment}",
                oninput: move |e| comment.set(e.value())
            }
            div { class: "flex gap-2",
                button {
                    class: "flex-1 px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 flex items-center justify-center gap-2 transition-colors",
                    onclick: move |_| {
                        let mut ticket = ticket_clone.clone();
                        ticket.ticket_status = TicketStatus::Approved;
                        ticket.approver = Some("当前用户".to_string());
                        ticket.approve_time = Some(chrono::Local::now().format("%Y-%m-%d %H:%M").to_string());
                        ticket.approve_comment = if !comment.read().is_empty() { Some(comment.read().clone()) } else { None };
                        let id = ticket.id;
                        tickets_clone.with_mut(|tickets| {
                            if let Some(t) = tickets.iter_mut().find(|t| t.id == id) {
                                *t = ticket;
                            }
                        });
                    },
                    Icon { icon: FaCircleCheck, class: "text-sm" }
                    "通过"
                }
                button {
                    class: "flex-1 px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 flex items-center justify-center gap-2 transition-colors",
                    onclick: move |_| {
                        let mut ticket = ticket_clone2.clone();
                        ticket.ticket_status = TicketStatus::Rejected;
                        ticket.approver = Some("当前用户".to_string());
                        ticket.approve_time = Some(chrono::Local::now().format("%Y-%m-%d %H:%M").to_string());
                        ticket.approve_comment = if !comment.read().is_empty() { Some(comment.read().clone()) } else { Some("拒绝".to_string()) };
                        let id = ticket.id;
                        tickets_clone2.with_mut(|tickets| {
                            if let Some(t) = tickets.iter_mut().find(|t| t.id == id) {
                                *t = ticket;
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
    let mut tickets_clone = tickets.clone();
    let mut details = use_signal(|| String::new());
    let mut ip_address = use_signal(|| ticket.ip_address.clone());

    rsx! {
        div { class: "space-y-4",
            p { class: "text-sm text-gray-500", "请配置云资源并填写配置信息" }
            div {
                label { class: "block text-sm font-medium text-gray-700 mb-1", "IP地址" }
                input {
                    class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500",
                    placeholder: "192.168.1.100",
                    value: "{ip_address}",
                    oninput: move |e| ip_address.set(e.value())
                }
            }
            textarea {
                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500",
                rows: 4,
                placeholder: "配置详情（如：实例已创建，安全组已配置等）",
                value: "{details}",
                oninput: move |e| details.set(e.value())
            }
            button {
                class: "w-full px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 flex items-center justify-center gap-2 transition-colors",
                onclick: move |_| {
                    let mut ticket = ticket_clone.clone();
                    ticket.ticket_status = TicketStatus::PendingDelivery;
                    ticket.ip_address = ip_address.read().clone();
                    ticket.provisioner = Some("当前用户".to_string());
                    ticket.provision_time = Some(chrono::Local::now().format("%Y-%m-%d %H:%M").to_string());
                    ticket.provision_details = if !details.read().is_empty() { Some(details.read().clone()) } else { None };
                    let id = ticket.id;
                    tickets_clone.with_mut(|tickets| {
                        if let Some(t) = tickets.iter_mut().find(|t| t.id == id) {
                            *t = ticket;
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
    let mut tickets_clone = tickets.clone();
    let mut comment = use_signal(|| String::new());

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
                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500",
                rows: 3,
                placeholder: "交付备注（可选）",
                value: "{comment}",
                oninput: move |e| comment.set(e.value())
            }
            button {
                class: "w-full px-4 py-2 bg-orange-600 text-white rounded-lg hover:bg-orange-700 flex items-center justify-center gap-2 transition-colors",
                onclick: move |_| {
                    let mut ticket = ticket_clone.clone();
                    ticket.ticket_status = TicketStatus::Delivered;
                    ticket.delivery_status = "已交付".to_string();
                    ticket.deliverer = Some("当前用户".to_string());
                    ticket.deliver_time = Some(chrono::Local::now().format("%Y-%m-%d %H:%M").to_string());
                    ticket.deliver_comment = if !comment.read().is_empty() { Some(comment.read().clone()) } else { None };
                    let id = ticket.id;
                    tickets_clone.with_mut(|tickets| {
                        if let Some(t) = tickets.iter_mut().find(|t| t.id == id) {
                            *t = ticket;
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
fn InfoRow<T: std::fmt::Display + Clone + PartialEq + 'static>(label: &'static str, value: T) -> Element {
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
    let mut ecs_name = use_signal(|| String::new());
    let mut application_name = use_signal(|| String::new());
    let mut contract_name = use_signal(|| String::new());
    let mut customer_name = use_signal(|| String::new());
    // 选中的服务商
    let mut selected_provider_id = use_signal(|| 1i32);

    // 获取服务商数据
    let service_providers = init_service_providers();

    // 为闭包克隆数据
    let service_providers_for_select = service_providers.clone();
    let cloud_platforms_all = init_cloud_platforms();
    let machine_rooms_all = init_machine_rooms();

    // 选中的云平台和机房
    let mut selected_cloud_platform_id = use_signal(|| Option::<i32>::None);
    let mut selected_machine_room_id = use_signal(|| Option::<i32>::None);
    let mut resource_type = use_signal(|| default_resource_type);
    let mut zone_name = use_signal(|| String::new());
    let mut ecs_type = use_signal(|| "ecs.g6.xlarge".to_string());
    let mut ecs_os = use_signal(|| "CentOS 7.9".to_string());
    let mut data_disk_type = use_signal(|| String::new());
    let mut data_disk_size = use_signal(|| String::new());
    let mut cpu_cores = use_signal(|| 4);
    let mut memory_gb = use_signal(|| 16);
    let mut system_disk = use_signal(|| "SSD".to_string());
    let mut system_disk_size = use_signal(|| 100);
    let mut has_security = use_signal(|| true);
    let mut remarks = use_signal(|| String::new());

    // 网络策略专用字段
    let mut fw_source_zone = use_signal(|| String::new());
    let mut fw_source_address = use_signal(|| String::new());
    let mut fw_dest_zone = use_signal(|| String::new());
    let mut fw_dest_address = use_signal(|| String::new());
    let mut fw_protocol = use_signal(|| "TCP".to_string());
    let mut fw_port = use_signal(|| String::new());
    let mut fw_direction = use_signal(|| "入站".to_string());
    let mut fw_valid_until = use_signal(|| String::new());
    let mut fw_firewall_name = use_signal(|| String::new());

    rsx! {
        div { class: "bg-white rounded-xl shadow-sm border border-gray-200 p-6",
            div { class: "flex items-center justify-between mb-6",
                h2 { class: "text-xl font-bold text-gray-800 flex items-center gap-2",
                    Icon { icon: FaPlus, class: "text-blue-600" }
                    "新建资源申请"
                }
                button {
                    class: "text-gray-400 hover:text-gray-600",
                    onclick: move |_| on_cancel.call(()),
                    Icon { icon: FaXmark }
                }
            }

            // Tab 切换
            div { class: "border-b border-gray-200 mb-6",
                div { class: "flex gap-8",
                    button {
                        class: format!("pb-3 px-1 border-b-2 font-medium text-sm transition-colors {}",
                            if *resource_type.read() == ResourceType::Cloud {
                                "border-blue-500 text-blue-600"
                            } else {
                                "border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300"
                            }
                        ),
                        onclick: move |_| resource_type.set(ResourceType::Cloud),
                        div { class: "flex items-center gap-2",
                            Icon { icon: FaCloud, width: 16, height: 16 }
                            span { "云资源" }
                        }
                    }
                    button {
                        class: format!("pb-3 px-1 border-b-2 font-medium text-sm transition-colors {}",
                            if *resource_type.read() == ResourceType::Physical {
                            "border-blue-500 text-blue-600"
                            } else {
                                "border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300"
                            }
                        ),
                        onclick: move |_| resource_type.set(ResourceType::Physical),
                        div { class: "flex items-center gap-2",
                            Icon { icon: FaServer, width: 16, height: 16 }
                            span { "物理资源" }
                        }
                    }
                    button {
                        class: format!("pb-3 px-1 border-b-2 font-medium text-sm transition-colors {}",
                            if *resource_type.read() == ResourceType::Network {
                            "border-blue-500 text-blue-600"
                            } else {
                                "border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300"
                            }
                        ),
                        onclick: move |_| resource_type.set(ResourceType::Network),
                        div { class: "flex items-center gap-2",
                            Icon { icon: FaShieldHalved, width: 16, height: 16 }
                            span { "网络策略" }
                        }
                    }
                }
            }

            form {
                class: "space-y-6",
                onsubmit: move |e: dioxus::prelude::Event<FormData>| {
                    e.prevent_default();
                    let id = (tickets.read().len() + 1) as i32;

                    // 获取选中的服务商信息
                    let provider_id = Some(*selected_provider_id.read());
                    let provider_name = service_providers.iter()
                        .find(|p| p.id == *selected_provider_id.read())
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
                        resource_type: *resource_type.read(),
                        ecs_name: ecs_name.read().clone(),
                        ticket_status: TicketStatus::PendingApproval,
                        provider_id,
                        provider_name,
                        cloud_platform_id,
                        cloud_platform_name,
                        machine_room_id: if *resource_type.read() == ResourceType::Cloud { None } else { machine_room_id },
                        machine_room_name: if *resource_type.read() == ResourceType::Cloud { String::new() } else { machine_room_name },
                        cloud_region: String::new(),
                        cloud_category: match *resource_type.read() {
                            ResourceType::Cloud => "云主机".to_string(),
                            ResourceType::Physical => "物理机".to_string(),
                            ResourceType::Network => "网络策略".to_string(),
                        },
                        zone_name: if *resource_type.read() == ResourceType::Cloud { String::new() } else { zone_name.read().clone() },
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
                        has_security_product: *has_security.read(),
                        ip_address: String::new(),
                        delivery_status: "未交付".to_string(),
                        remarks: remarks.read().clone(),
                        created_at: chrono::Local::now().format("%Y-%m-%d %H:%M").to_string(),
                        updated_at: chrono::Local::now().format("%Y-%m-%d %H:%M").to_string(),
                        created_by: "当前用户".to_string(),
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
                        fw_source_zone: if *resource_type.read() == ResourceType::Network {
                            Some(fw_source_zone.read().clone())
                        } else {
                            None
                        },
                        fw_source_address: if *resource_type.read() == ResourceType::Network {
                            Some(fw_source_address.read().clone())
                        } else {
                            None
                        },
                        fw_dest_zone: if *resource_type.read() == ResourceType::Network {
                            Some(fw_dest_zone.read().clone())
                        } else {
                            None
                        },
                        fw_dest_address: if *resource_type.read() == ResourceType::Network {
                            Some(fw_dest_address.read().clone())
                        } else {
                            None
                        },
                        fw_protocol: if *resource_type.read() == ResourceType::Network {
                            Some(fw_protocol.read().clone())
                        } else {
                            None
                        },
                        fw_port: if *resource_type.read() == ResourceType::Network {
                            Some(fw_port.read().clone())
                        } else {
                            None
                        },
                        fw_direction: if *resource_type.read() == ResourceType::Network {
                            Some(fw_direction.read().clone())
                        } else {
                            None
                        },
                        fw_valid_until: if *resource_type.read() == ResourceType::Network {
                            Some(fw_valid_until.read().clone())
                        } else {
                            None
                        },
                        fw_firewall_name: if *resource_type.read() == ResourceType::Network {
                            Some(fw_firewall_name.read().clone())
                        } else {
                            None
                        },
                    };
                    tickets.with_mut(|apps| {
                        apps.push(new_app);
                    });
                    on_submit.call(());
                },

                // 基本信息
                div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "申请名称*" }
                        input {
                            class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500",
                            placeholder: "如：业务系统A-Web服务器",
                            value: "{application_name}",
                            required: true,
                            oninput: move |e| application_name.set(e.value())
                        }
                    }
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "实例名称*" }
                        input {
                            class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500",
                            placeholder: "如：Web服务器-01",
                            value: "{ecs_name}",
                            required: true,
                            oninput: move |e| ecs_name.set(e.value())
                        }
                    }
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "客户名称*" }
                        input {
                            class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500",
                            placeholder: "客户单位名称",
                            value: "{customer_name}",
                            required: true,
                            oninput: move |e| customer_name.set(e.value())
                        }
                    }
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "合同名称*" }
                        input {
                            class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500",
                            placeholder: "如：合同-2024-001",
                            value: "{contract_name}",
                            required: true,
                            oninput: move |e| contract_name.set(e.value())
                        }
                    }
                }

                // 资源配置
                div { class: "border-t border-gray-200 pt-4",
                    h3 { class: "text-md font-semibold text-gray-800 mb-4",
                        if *resource_type.read() == ResourceType::Cloud {
                            "云资源配置"
                        } else if *resource_type.read() == ResourceType::Physical {
                            "物理资源配置"
                        } else {
                            "网络策略配置"
                        }
                    }
                    div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "服务商*" }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500",
                                required: true,
                                value: "{selected_provider_id}",
                                onchange: move |e| {
                                    if let Ok(id) = e.value().parse::<i32>() {
                                        selected_provider_id.set(id);
                                        // 服务商改变时重置已选择的云平台和机房
                                        selected_cloud_platform_id.set(None);
                                        selected_machine_room_id.set(None);
                                    }
                                },
                                for provider in &service_providers_for_select {
                                    option {
                                        value: "{provider.id}",
                                        selected: *selected_provider_id.read() == provider.id,
                                        "{provider.short_name}"
                                    }
                                }
                            }
                        }
                        // 云资源专用字段
                        if *resource_type.read() == ResourceType::Cloud {
                            div {
                                label { class: "block text-sm font-medium text-gray-700 mb-1", "云平台*" }
                                select {
                                    class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500",
                                    required: true,
                                    oninput: move |e| {
                                        let platform_id: i32 = e.value().parse().unwrap_or(0);
                                        selected_cloud_platform_id.set(Some(platform_id));
                                    },
                                    option { value: "-1", "请先选择服务商" }
                                    // 根据服务商过滤云平台
                                    for platform in cloud_platforms_all.iter().filter(|p| p.provider_id == *selected_provider_id.read()) {
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
                        if *resource_type.read() == ResourceType::Physical {
                            div {
                                label { class: "block text-sm font-medium text-gray-700 mb-1", "服务器型号*" }
                                input {
                                    class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500",
                                    placeholder: "如：Dell R740",
                                    required: true,
                                    value: "{ecs_type}",
                                    oninput: move |e| ecs_type.set(e.value())
                                }
                            }
                            div {
                                label { class: "block text-sm font-medium text-gray-700 mb-1", "机房*" }
                                select {
                                    class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500",
                                    required: true,
                                    oninput: move |e| {
                                        let room_id: i32 = e.value().parse().unwrap_or(0);
                                        selected_machine_room_id.set(if room_id > 0 { Some(room_id) } else { None });
                                    },
                                    option { value: "-1", "请先选择服务商" }
                                    // 根据服务商过滤机房
                                    for room in machine_rooms_all.iter().filter(|r| r.provider_id == *selected_provider_id.read()) {
                                        option {
                                            value: "{room.id}",
                                            selected: *selected_machine_room_id.read() == Some(room.id),
                                            "{room.display_name()}"
                                        }
                                    }
                                }
                            }
                            div {
                                label { class: "block text-sm font-medium text-gray-700 mb-1", "机柜位置" }
                                input {
                                    class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500",
                                    placeholder: "如：A区-03柜",
                                    value: "{zone_name}",
                                    oninput: move |e| zone_name.set(e.value())
                                }
                            }
                        }
                        // 网络策略专用字段
                        if *resource_type.read() == ResourceType::Network {
                            div {
                                label { class: "block text-sm font-medium text-gray-700 mb-1", "源区域*" }
                                select {
                                    class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500",
                                    required: true,
                                    value: "{fw_source_zone}",
                                    oninput: move |e| fw_source_zone.set(e.value()),
                                    option { value: "", "请选择" }
                                    option { value: "互联网DMZ", "互联网DMZ" }
                                    option { value: "政务网DMZ", "政务网DMZ" }
                                    option { value: "办公网", "办公网" }
                                    option { value: "数据中心", "数据中心" }
                                    option { value: "可信区", "可信区" }
                                }
                            }
                            div {
                                label { class: "block text-sm font-medium text-gray-700 mb-1", "源地址/IP*" }
                                input {
                                    class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500",
                                    placeholder: "如: 192.168.1.0/24",
                                    required: true,
                                    value: "{fw_source_address}",
                                    oninput: move |e| fw_source_address.set(e.value())
                                }
                            }
                            div {
                                label { class: "block text-sm font-medium text-gray-700 mb-1", "目标区域*" }
                                select {
                                    class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500",
                                    required: true,
                                    value: "{fw_dest_zone}",
                                    oninput: move |e| fw_dest_zone.set(e.value()),
                                    option { value: "", "请选择" }
                                    option { value: "互联网DMZ", "互联网DMZ" }
                                    option { value: "政务网DMZ", "政务网DMZ" }
                                    option { value: "办公网", "办公网" }
                                    option { value: "数据中心", "数据中心" }
                                    option { value: "可信区", "可信区" }
                                }
                            }
                            div {
                                label { class: "block text-sm font-medium text-gray-700 mb-1", "目标地址/IP*" }
                                input {
                                    class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500",
                                    placeholder: "如: 10.1.1.100",
                                    required: true,
                                    value: "{fw_dest_address}",
                                    oninput: move |e| fw_dest_address.set(e.value())
                                }
                            }
                            div {
                                label { class: "block text-sm font-medium text-gray-700 mb-1", "协议*" }
                                select {
                                    class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500",
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
                                label { class: "block text-sm font-medium text-gray-700 mb-1", "端口*" }
                                input {
                                    class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500",
                                    placeholder: "如: 443 或 8080-8090",
                                    required: true,
                                    value: "{fw_port}",
                                    oninput: move |e| fw_port.set(e.value())
                                }
                            }
                            div {
                                label { class: "block text-sm font-medium text-gray-700 mb-1", "访问方向*" }
                                select {
                                    class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500",
                                    required: true,
                                    value: "{fw_direction}",
                                    oninput: move |e| fw_direction.set(e.value()),
                                    option { value: "入站", "入站" }
                                    option { value: "出站", "出站" }
                                    option { value: "双向", "双向" }
                                }
                            }
                            div {
                                label { class: "block text-sm font-medium text-gray-700 mb-1", "有效期至*" }
                                input {
                                    class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500",
                                    r#type: "date",
                                    required: true,
                                    value: "{fw_valid_until}",
                                    oninput: move |e| fw_valid_until.set(e.value())
                                }
                            }
                            div {
                                label { class: "block text-sm font-medium text-gray-700 mb-1", "防火墙设备*" }
                                input {
                                    class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500",
                                    placeholder: "如: 核心防火墙-01",
                                    required: true,
                                    value: "{fw_firewall_name}",
                                    oninput: move |e| fw_firewall_name.set(e.value())
                                }
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "实例类型" }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500",
                                value: "{ecs_type}",
                                oninput: move |e| ecs_type.set(e.value()),
                                option { value: "ecs.g6.large", "ecs.g6.large (2核8GB)" }
                                option { value: "ecs.g6.xlarge", "ecs.g6.xlarge (4核16GB)" }
                                option { value: "ecs.g6.2xlarge", "ecs.g6.2xlarge (8核32GB)" }
                                option { value: "Dell R740", "Dell R740 (物理机)" }
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "操作系统" }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500",
                                value: "{ecs_os}",
                                oninput: move |e| ecs_os.set(e.value()),
                                option { value: "CentOS 7.9", "CentOS 7.9" }
                                option { value: "CentOS 8.4", "CentOS 8.4" }
                                option { value: "Ubuntu 20.04", "Ubuntu 20.04" }
                                option { value: "Ubuntu 22.04", "Ubuntu 22.04" }
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "CPU核数" }
                            input {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500",
                                r#type: "number",
                                min: 1,
                                max: 128,
                                value: "{cpu_cores}",
                                oninput: move |e| cpu_cores.set(e.value().parse().unwrap_or(4))
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "内存(GB)" }
                            input {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500",
                                r#type: "number",
                                min: 1,
                                max: 1024,
                                value: "{memory_gb}",
                                oninput: move |e| memory_gb.set(e.value().parse().unwrap_or(16))
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "系统盘类型" }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500",
                                value: "{system_disk}",
                                oninput: move |e| system_disk.set(e.value()),
                                option { value: "SSD", "SSD" }
                                option { value: "NVMe", "NVMe" }
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "系统盘大小(GB)" }
                            input {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500",
                                r#type: "number",
                                min: 20,
                                max: 5000,
                                value: "{system_disk_size}",
                                oninput: move |e| system_disk_size.set(e.value().parse().unwrap_or(100))
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "数据盘类型" }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500",
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
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "数据盘大小(GB)" }
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
                    div { class: "mt-4",
                        label { class: "flex items-center gap-2 cursor-pointer",
                            input {
                                r#type: "checkbox",
                                class: "w-4 h-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500",
                                checked: *has_security.read(),
                                oninput: move |e| has_security.set(e.checked())
                            }
                            span { class: "text-sm text-gray-700", "需要安全产品（防火墙、WAF等）" }
                        }
                    }
                }

                // 备注
                div {
                    label { class: "block text-sm font-medium text-gray-700 mb-1", "备注说明" }
                    textarea {
                        class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500",
                        rows: 3,
                        placeholder: "申请用途、特殊需求等",
                        value: "{remarks}",
                        oninput: move |e| remarks.set(e.value())
                    }
                }

                // 提交按钮
                div { class: "flex gap-3 pt-4 border-t border-gray-200",
                    button {
                        class: "flex-1 px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 font-medium flex items-center justify-center gap-2 transition-colors",
                        r#type: "submit",
                        Icon { icon: FaPaperPlane, class: "text-sm" }
                        "提交申请"
                    }
                    button {
                        class: "px-6 py-3 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 font-medium",
                        r#type: "button",
                        onclick: move |_| on_cancel.call(()),
                        "取消"
                    }
                }
            }
        }
    }
}
