use crate::app::NETWORK_POLICIES_STATE;
use crate::services::resource_ticket_api::fetch_resource_tickets;
use crate::state::resource_ticket::{ResourceTicket, ResourceType, TicketStatus};
use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaArrowLeft, FaArrowRight, FaCheck, FaCircleCheck, FaClock, FaEye, FaMagnifyingGlass, FaPen,
    FaPlus, FaShieldHalved, FaXmark,
};
use dioxus_free_icons::Icon;

/// 网络策略申请状态
#[derive(Clone, Debug, PartialEq)]
pub enum NetworkPolicyStatus {
    Pending,     // 待审批
    Approved,    // 已批准
    Rejected,    // 已拒绝
    Configuring, // 配置中
    Active,      // 已交付
    Expired,     // 已过期
}

impl NetworkPolicyStatus {
    fn display_name(&self) -> &'static str {
        match self {
            NetworkPolicyStatus::Pending => "待审批",
            NetworkPolicyStatus::Approved => "已批准",
            NetworkPolicyStatus::Rejected => "已拒绝",
            NetworkPolicyStatus::Configuring => "配置中",
            NetworkPolicyStatus::Active => "已交付",
            NetworkPolicyStatus::Expired => "已过期",
        }
    }

    fn color_class(&self) -> &'static str {
        match self {
            NetworkPolicyStatus::Pending => "bg-yellow-100 text-yellow-800",
            NetworkPolicyStatus::Approved => "bg-green-100 text-green-800",
            NetworkPolicyStatus::Rejected => "bg-red-100 text-red-800",
            NetworkPolicyStatus::Configuring => "bg-blue-100 text-blue-800",
            NetworkPolicyStatus::Active => "bg-green-100 text-green-800",
            NetworkPolicyStatus::Expired => "bg-gray-100 text-gray-800",
        }
    }
}

/// 访问方向
#[derive(Clone, Debug, PartialEq)]
pub enum AccessDirection {
    Inbound,       // 入站
    Outbound,      // 出站
    Bidirectional, // 双向
}

impl AccessDirection {
    pub fn display_name(&self) -> &'static str {
        match self {
            AccessDirection::Inbound => "入站",
            AccessDirection::Outbound => "出站",
            AccessDirection::Bidirectional => "双向",
        }
    }
}

/// 协议类型
#[derive(Clone, Debug, PartialEq)]
pub enum PolicyProtocol {
    Tcp,
    Udp,
    Icmp,
    Any,
}

impl PolicyProtocol {
    pub fn display_name(&self) -> &'static str {
        match self {
            PolicyProtocol::Tcp => "TCP",
            PolicyProtocol::Udp => "UDP",
            PolicyProtocol::Icmp => "ICMP",
            PolicyProtocol::Any => "ANY",
        }
    }
}

/// 网络策略申请数据模型
#[derive(Clone, Debug, PartialEq)]
pub struct NetworkPolicyRequest {
    pub id: i32,
    pub title: String,
    pub organization: String,
    pub applicant: String,
    pub department: String,
    pub source_zone: String,
    pub destination_zone: String,
    pub direction: AccessDirection,
    pub protocol: PolicyProtocol,
    pub port_range: String,
    pub description: String,
    pub valid_until: String,
    pub status: NetworkPolicyStatus,
    pub created_at: String,
}

/// 从后端 ResourceTicket 转换为前端 NetworkPolicyRequest
fn ticket_to_policy_request(ticket: &ResourceTicket) -> NetworkPolicyRequest {
    // 映射状态
    let status = match ticket.ticket_status {
        TicketStatus::PendingApproval => NetworkPolicyStatus::Pending,
        TicketStatus::Rejected => NetworkPolicyStatus::Rejected,
        TicketStatus::Approved | TicketStatus::PendingProvision | TicketStatus::Provisioning => {
            NetworkPolicyStatus::Configuring
        }
        TicketStatus::PendingDelivery | TicketStatus::Delivered => NetworkPolicyStatus::Active,
        _ => NetworkPolicyStatus::Pending,
    };

    // 映射协议
    let protocol = ticket
        .fw_protocol
        .as_ref()
        .map(|p| match p.to_lowercase().as_str() {
            "tcp" => PolicyProtocol::Tcp,
            "udp" => PolicyProtocol::Udp,
            "icmp" => PolicyProtocol::Icmp,
            "any" => PolicyProtocol::Any,
            _ => PolicyProtocol::Tcp,
        })
        .unwrap_or(PolicyProtocol::Tcp);

    // 映射方向
    let direction = ticket
        .fw_direction
        .as_ref()
        .map(|d| match d.as_str() {
            "入站" => AccessDirection::Inbound,
            "出站" => AccessDirection::Outbound,
            "双向" => AccessDirection::Bidirectional,
            _ => AccessDirection::Inbound,
        })
        .unwrap_or(AccessDirection::Inbound);

    NetworkPolicyRequest {
        id: ticket.id,
        title: ticket.ecs_name.clone(),
        organization: ticket.organization_name.clone(),
        applicant: if ticket.applicant_name.is_empty() {
            ticket.created_by.clone()
        } else {
            ticket.applicant_name.clone()
        },
        department: ticket.department_name.clone(),
        source_zone: ticket.fw_source_zone.clone().unwrap_or_default(),
        destination_zone: ticket.fw_dest_zone.clone().unwrap_or_default(),
        direction,
        protocol,
        port_range: ticket.fw_port.clone().unwrap_or_default(),
        description: ticket.remarks.clone(),
        valid_until: ticket.fw_valid_until.clone().unwrap_or_default(),
        status,
        created_at: ticket.created_at.clone(),
    }
}

/// 网络策略申请页面
#[allow(non_snake_case)]
pub fn NetworkPolicyRequest() -> Element {
    // 使用全局网络策略状态的引用
    let requests = &NETWORK_POLICIES_STATE;
    let mut search_query = use_signal(String::new);
    let mut status_filter = use_signal(|| "全部".to_string());
    let mut show_new_form = use_signal(|| false);
    let mut show_edit_form = use_signal(|| None::<i32>);
    let mut is_loaded = use_signal(|| false);

    // 在组件挂载时从 API 获取数据
    use_effect(move || {
        if *is_loaded.read() {
            return;
        }

        spawn(async move {
            match fetch_resource_tickets().await {
                Ok(tickets) => {
                    // 过滤出网络策略类型的工单
                    let network_policies: Vec<NetworkPolicyRequest> = tickets
                        .iter()
                        .filter(|t| t.resource_type == ResourceType::Network)
                        .map(ticket_to_policy_request)
                        .collect();

                    // 更新全局状态
                    NETWORK_POLICIES_STATE.write().clear();
                    for policy in network_policies {
                        NETWORK_POLICIES_STATE.write().push(policy.into());
                    }

                    is_loaded.set(true);
                }
                Err(e) => {
                    tracing::error!("获取网络策略数据失败: {}", e);
                    // 失败时显示空列表
                    NETWORK_POLICIES_STATE.write().clear();
                    is_loaded.set(true);
                }
            }
        });
    });

    // 统计数据
    let total_count = requests.read().len() as i32;
    let pending_count = requests
        .read()
        .iter()
        .filter(|r| r.status == NetworkPolicyStatus::Pending)
        .count() as i32;
    let configuring_count = requests
        .read()
        .iter()
        .filter(|r| r.status == NetworkPolicyStatus::Configuring)
        .count() as i32;
    let active_count = requests
        .read()
        .iter()
        .filter(|r| r.status == NetworkPolicyStatus::Active)
        .count() as i32;

    // 过滤请求
    let filtered_requests: Vec<NetworkPolicyRequest> = requests
        .read()
        .iter()
        .filter(|req| {
            let query = search_query.read().to_lowercase();
            let matches_search = query.is_empty()
                || req.title.to_lowercase().contains(&query)
                || req.applicant.to_lowercase().contains(&query)
                || req.department.to_lowercase().contains(&query);

            let matches_status = status_filter.read().as_str() == "全部"
                || match status_filter.read().as_str() {
                    "待审批" => req.status == NetworkPolicyStatus::Pending,
                    "已批准" => req.status == NetworkPolicyStatus::Approved,
                    "已拒绝" => req.status == NetworkPolicyStatus::Rejected,
                    "配置中" => req.status == NetworkPolicyStatus::Configuring,
                    "已交付" => req.status == NetworkPolicyStatus::Active,
                    "已过期" => req.status == NetworkPolicyStatus::Expired,
                    _ => true,
                };

            matches_search && matches_status
        })
        .map(|r| r.clone().into())
        .collect();

    rsx! {
        div { class: "space-y-6",
            // 页面标题
            div { class: "flex justify-between items-center",
                div {
                    div { class: "flex items-center gap-3",
                        Icon { icon: FaShieldHalved, class: "text-purple-600 text-2xl" }
                        div {
                            h1 { class: "text-2xl font-bold text-gray-800", "网络策略申请" }
                            p { class: "text-sm text-gray-500 mt-1", "申请网络访问控制、防火墙策略等" }
                        }
                    }
                }
                button {
                    class: "flex items-center px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 transition-colors",
                    onclick: move |_| show_new_form.set(true),
                    Icon { icon: FaPlus, width: 16, height: 16, class: "mr-2" }
                    "申请新策略"
                }
            }

            // 统计卡片
            div { class: "grid grid-cols-1 md:grid-cols-4 gap-4",
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-purple-500",
                            Icon { icon: FaShieldHalved, width: 20, height: 20, class: "text-white" }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "总申请" }
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
                        div { class: "p-2 rounded-full bg-blue-600",
                            Icon { icon: FaCheck, width: 20, height: 20, class: "text-white" }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "配置中" }
                            p { class: "text-xl font-bold text-gray-800", {configuring_count.to_string()} }
                        }
                    }
                }
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-green-500",
                            Icon { icon: FaCircleCheck, width: 20, height: 20, class: "text-white" }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "已交付" }
                            p { class: "text-xl font-bold text-gray-800", {active_count.to_string()} }
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
                            class: "w-full pl-10 pr-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-purple-500",
                            placeholder: "搜索策略名称、申请人或部门...",
                            value: "{search_query}",
                            oninput: move |e| search_query.set(e.value())
                        }
                    }
                    select {
                        class: "px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-purple-500",
                        value: status_filter,
                        onchange: move |e| status_filter.set(e.value()),
                        option { value: "全部", "全部状态" }
                        option { value: "待审批", "待审批" }
                        option { value: "已批准", "已批准" }
                        option { value: "配置中", "配置中" }
                        option { value: "已交付", "已交付" }
                        option { value: "已过期", "已过期" }
                        option { value: "已拒绝", "已拒绝" }
                    }
                }
            }

            // 策略列表表格
            div { class: "bg-white rounded-lg shadow overflow-hidden",
                table { class: "min-w-full divide-y divide-gray-200",
                    thead { class: "bg-gray-50",
                        tr {
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "策略名称" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "申请人" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "网络流向" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "协议" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "端口" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "有效期至" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "状态" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "申请时间" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "操作" }
                        }
                    }
                    tbody { class: "bg-white divide-y divide-gray-200",
                        if filtered_requests.is_empty() {
                            tr {
                                td { colspan: "9", class: "px-6 py-12 text-center text-gray-500",
                                    "没有找到匹配的策略"
                                }
                            }
                        } else {
                            for req in filtered_requests.iter() {
                                tr {
                                    key: "{req.id}",
                                    class: "hover:bg-gray-50",
                                    td { class: "px-6 py-4",
                                        div { class: "text-sm font-medium text-gray-900", {req.title.clone()} }
                                        div { class: "text-sm text-gray-500", {format!("{} / {}", req.organization, req.department)} }
                                    }
                                    td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-900",
                                        {req.applicant.clone()}
                                    }
                                    td { class: "px-6 py-4",
                                        div { class: "flex items-center gap-2 text-sm",
                                            span { class: "text-gray-700", {req.source_zone.clone()} }
                                            match req.direction {
                                                AccessDirection::Outbound => rsx! {
                                                    Icon { icon: FaArrowRight, width: 14, height: 14, class: "text-gray-400" }
                                                },
                                                AccessDirection::Inbound => rsx! {
                                                    Icon { icon: FaArrowLeft, width: 14, height: 14, class: "text-gray-400" }
                                                },
                                                AccessDirection::Bidirectional => rsx! {
                                                    Icon { icon: FaArrowRight, width: 14, height: 14, class: "text-blue-400" }
                                                    Icon { icon: FaArrowLeft, width: 14, height: 14, class: "text-blue-400" }
                                                },
                                            }
                                            span { class: "text-gray-700", {req.destination_zone.clone()} }
                                        }
                                        div { class: "text-xs text-gray-500", {req.direction.display_name()} }
                                    }
                                    td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-500",
                                        {req.protocol.display_name()}
                                    }
                                    td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-500",
                                        span { class: "font-mono text-xs bg-gray-100 px-2 py-1 rounded", {req.port_range.clone()} }
                                    }
                                    td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-500",
                                        {req.valid_until.clone()}
                                    }
                                    td { class: "px-6 py-4 whitespace-nowrap",
                                        span {
                                            class: "px-2 inline-flex text-xs leading-5 font-semibold rounded-full {req.status.color_class()}",
                                            {req.status.display_name()}
                                        }
                                    }
                                    td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-500",
                                        {req.created_at.clone()}
                                    }
                                    td { class: "px-6 py-4 whitespace-nowrap text-sm font-medium",
                                        button {
                                            class: "text-blue-600 hover:text-blue-900 mr-3",
                                            title: "查看详情",
                                            Icon { icon: FaEye, width: 16, height: 16 }
                                        }
                                        {
                                            let edit_id = req.id;
                                            rsx! {
                                                button {
                                                    class: "text-green-600 hover:text-green-900 mr-3",
                                                    title: "编辑",
                                                    onclick: move |_| show_edit_form.set(Some(edit_id)),
                                                    Icon { icon: FaPen, width: 16, height: 16 }
                                                }
                                            }
                                        }
                                        button {
                                            class: "text-red-600 hover:text-red-900",
                                            title: "删除",
                                            Icon { icon: FaXmark, width: 16, height: 16 }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // 新建策略表单
            if *show_new_form.read() {
                super::policy_form::NetworkPolicyForm {
                    mode: crate::components::common::FormMode::New,
                    request: None,
                    on_save: move |new_request: NetworkPolicyRequest| {
                        let mut reqs = requests.write();
                        let new_id = reqs.iter().map(|r| r.id).max().unwrap_or(0) + 1;
                        let mut new_request = new_request;
                        new_request.id = new_id;
                        reqs.push(new_request.into());
                        show_new_form.set(false);
                    },
                    on_close: move |_| show_new_form.set(false),
                }
            }

            // 编辑策略表单
            {
                let edit_id = *show_edit_form.read();
                edit_id.and_then(|id| {
                    let all_requests = requests.read().clone();
                    let editing_request = all_requests.iter().find(|r| r.id == id).cloned();
                    editing_request.map(|req| rsx! {
                        super::policy_form::NetworkPolicyForm {
                            mode: crate::components::common::FormMode::Edit,
                            request: Some(req.into()),
                            on_save: move |updated_request: NetworkPolicyRequest| {
                                let mut reqs = requests.write();
                                if let Some(r) = reqs.iter_mut().find(|r| r.id == id) {
                                    *r = updated_request.into();
                                }
                                show_edit_form.set(None);
                            },
                            on_close: move |_| show_edit_form.set(None),
                        }
                    })
                })
            }
        }
    }
}
