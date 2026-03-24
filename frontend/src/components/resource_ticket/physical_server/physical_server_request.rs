use crate::components::common::VirtualScroller;
use crate::components::security_product::security_product_selector::SelectedSecurityProducts;
use crate::services::resource_ticket_api::fetch_resource_tickets;
use crate::state::resource_ticket::{ResourceTicket, ResourceType, TicketStatus};
use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaClock, FaMagnifyingGlass, FaMicrochip, FaServer, FaWarehouse,
};
use dioxus_free_icons::Icon;

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub enum PhysicalServerStatus {
    Pending,
    Approved,
    Rejected,
    Processing,
    Deployed,
    Completed,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PhysicalServerRequest {
    pub id: i32,
    pub title: String,
    pub organization: String,
    pub applicant: String,
    pub department: String,
    pub provider_id: Option<i32>,
    pub machine_room_id: Option<i32>,
    pub server_type: String,
    pub cpu_cores: String,
    pub memory: String,
    pub storage: String,
    pub server_count: i32,
    pub purpose: String,
    pub security_products: SelectedSecurityProducts,
    pub status: PhysicalServerStatus,
    pub created_at: String,
}

#[allow(non_snake_case)]
pub fn PhysicalServerRequest() -> Element {
    let tickets = use_signal(Vec::<ResourceTicket>::new);
    let mut search_query = use_signal(String::new);
    let mut status_filter = use_signal(|| "".to_string());
    let loading = use_signal(|| true);
    let error = use_signal(String::new);

    {
        let mut tickets = tickets;
        let mut loading = loading;
        let mut error = error;
        use_effect(move || {
            spawn(async move {
                match fetch_resource_tickets().await {
                    Ok(data) => {
                        let physical_tickets: Vec<ResourceTicket> = data
                            .into_iter()
                            .filter(|ticket| ticket.resource_type == ResourceType::Physical)
                            .collect();
                        tickets.set(physical_tickets);
                        error.set(String::new());
                    }
                    Err(err) => error.set(err),
                }
                loading.set(false);
            });
        });
    }

    let total_count = tickets.read().len() as i32;
    let pending_count = tickets
        .read()
        .iter()
        .filter(|ticket| ticket.ticket_status == TicketStatus::PendingApproval)
        .count() as i32;
    let provisioning_count = tickets
        .read()
        .iter()
        .filter(|ticket| {
            ticket.ticket_status == TicketStatus::PendingProvision
                || ticket.ticket_status == TicketStatus::Provisioning
        })
        .count() as i32;
    let delivered_count = tickets
        .read()
        .iter()
        .filter(|ticket| ticket.ticket_status == TicketStatus::Delivered)
        .count() as i32;

    let query = search_query.read().to_lowercase();
    let active_status = status_filter.read().clone();
    let filtered_tickets: Vec<ResourceTicket> = tickets
        .read()
        .iter()
        .filter(|ticket| {
            let matches_query = query.is_empty()
                || ticket.ecs_name.to_lowercase().contains(&query)
                || ticket.customer_name.to_lowercase().contains(&query)
                || ticket.machine_room_name.to_lowercase().contains(&query)
                || ticket.applicant_name.to_lowercase().contains(&query)
                || ticket.created_by.to_lowercase().contains(&query)
                || ticket.department_name.to_lowercase().contains(&query)
                || ticket.organization_name.to_lowercase().contains(&query);
            let matches_status =
                active_status.is_empty() || ticket.ticket_status.to_api_str() == active_status;
            matches_query && matches_status
        })
        .cloned()
        .collect();

    rsx! {
        div { class: "space-y-6",
            div {
                h1 { class: "text-2xl font-bold text-gray-800", "物理机申请" }
                p { class: "text-sm text-gray-500 mt-1", "直接从后端资源工单接口读取物理服务器申请数据" }
            }

            div { class: "grid grid-cols-1 md:grid-cols-4 gap-4",
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-slate-600",
                            Icon { icon: FaServer, width: 20, height: 20, class: "text-white" }
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
                        div { class: "p-2 rounded-full bg-indigo-500",
                            Icon { icon: FaMicrochip, width: 20, height: 20, class: "text-white" }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "配置中" }
                            p { class: "text-xl font-bold text-gray-800", {provisioning_count.to_string()} }
                        }
                    }
                }
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-green-500",
                            Icon { icon: FaWarehouse, width: 20, height: 20, class: "text-white" }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "已交付" }
                            p { class: "text-xl font-bold text-gray-800", {delivered_count.to_string()} }
                        }
                    }
                }
            }

            div { class: "bg-white rounded-lg shadow p-4",
                div { class: "flex flex-col gap-4 lg:flex-row lg:items-center",
                    div { class: "flex-1 flex items-center",
                        Icon { icon: FaMagnifyingGlass, width: 18, height: 18, class: "text-gray-400" }
                        input {
                            r#type: "text",
                            class: "ml-2 w-full border-0 focus:outline-none",
                            placeholder: "搜索工单名、客户、机房或创建人...",
                            value: search_query,
                            oninput: move |e| search_query.set(e.value()),
                        }
                    }
                    select {
                        class: "px-3 py-2 border border-gray-300 rounded-md",
                        value: status_filter,
                        onchange: move |e| status_filter.set(e.value()),
                        option { value: "", "全部状态" }
                        option { value: "pending_approval", "待审批" }
                        option { value: "approved", "已通过" }
                        option { value: "pending_provision", "待配置" }
                        option { value: "provisioning", "配置中" }
                        option { value: "pending_delivery", "待交付" }
                        option { value: "delivered", "已交付" }
                        option { value: "rejected", "已拒绝" }
                        option { value: "archived", "已归档" }
                    }
                }
            }

            if *loading.read() {
                div { class: "bg-white rounded-lg shadow p-6 text-gray-500", "正在加载物理机申请..." }
            } else if !error.read().is_empty() {
                div { class: "bg-red-50 border border-red-200 text-red-700 rounded-lg p-4", "{error}" }
            } else {
                VirtualScroller {
                    items: filtered_tickets,
                    page_size: 20,
                    render_item: move |ticket: ResourceTicket| rsx! {
                        div { class: "grid grid-cols-1 lg:grid-cols-6 gap-4 px-6 py-4 border-b border-gray-100 items-center",
                            div {
                                p { class: "text-sm font-medium text-gray-900", "{ticket.ecs_name}" }
                                p { class: "text-xs text-gray-500", "{ticket.customer_name}" }
                            }
                            div {
                                p { class: "text-sm text-gray-700", "{ticket.machine_room_name}" }
                                p { class: "text-xs text-gray-500", "{ticket.zone_name}" }
                            }
                            div {
                                p { class: "text-sm text-gray-700", "{ticket.cpu_cores}C / {ticket.memory_gb}GB" }
                                p { class: "text-xs text-gray-500", "{ticket.system_disk} {ticket.system_disk_size_gb}GB" }
                            }
                            div {
                                span { class: "px-2 inline-flex text-xs leading-5 font-semibold rounded-full {ticket.ticket_status.color_class()}",
                                    "{ticket.ticket_status.display_name()}"
                                }
                            }
                            div {
                                p { class: "text-sm text-gray-700", "{ticket.provider_name}" }
                                p { class: "text-xs text-gray-500", "{rack_label(&ticket)}" }
                            }
                            div {
                                p { class: "text-sm text-gray-700",
                                    {if ticket.applicant_name.is_empty() { ticket.created_by.clone() } else { ticket.applicant_name.clone() }}
                                }
                                p { class: "text-xs text-gray-500", "{format_time(&ticket.created_at)}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn rack_label(ticket: &ResourceTicket) -> String {
    if ticket.zone_cabinet.is_empty() && ticket.rack_units == 0 {
        "未填写机柜信息".to_string()
    } else {
        format!("{} / {}U", ticket.zone_cabinet, ticket.rack_units)
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
