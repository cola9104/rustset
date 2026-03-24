use crate::components::common::VirtualScroller;
use crate::services::business_resource_api::{fetch_business_resources, BusinessResourceRecord};
use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaCloud, FaFilter, FaMagnifyingGlass, FaServer, FaShieldHalved,
};
use dioxus_free_icons::Icon;

#[allow(non_snake_case)]
pub fn BusinessApplication() -> Element {
    let resources = use_signal(Vec::<BusinessResourceRecord>::new);
    let mut search_query = use_signal(String::new);
    let mut type_filter = use_signal(|| "".to_string());
    let mut status_filter = use_signal(|| "".to_string());
    let loading = use_signal(|| true);
    let error = use_signal(String::new);

    {
        let mut resources = resources;
        let mut loading = loading;
        let mut error = error;
        use_effect(move || {
            spawn(async move {
                match fetch_business_resources().await {
                    Ok(data) => {
                        resources.set(data);
                        error.set(String::new());
                    }
                    Err(err) => error.set(err),
                }
                loading.set(false);
            });
        });
    }

    let total_count = resources.read().len() as i32;
    let cloud_count = resources
        .read()
        .iter()
        .filter(|item| item.resource_type == "cloud")
        .count() as i32;
    let physical_count = resources
        .read()
        .iter()
        .filter(|item| item.resource_type == "physical")
        .count() as i32;
    let secured_count = resources
        .read()
        .iter()
        .filter(|item| item.has_security_product)
        .count() as i32;

    let query = search_query.read().to_lowercase();
    let active_type = type_filter.read().clone();
    let active_status = status_filter.read().clone();
    let filtered_resources: Vec<BusinessResourceRecord> = resources
        .read()
        .iter()
        .filter(|item| {
            let matches_query = query.is_empty()
                || item.ecs_name.to_lowercase().contains(&query)
                || item.customer_name.to_lowercase().contains(&query)
                || item.ip_address.to_lowercase().contains(&query)
                || item
                    .application_name
                    .as_deref()
                    .unwrap_or_default()
                    .to_lowercase()
                    .contains(&query);
            let matches_type = active_type.is_empty() || item.resource_type == active_type;
            let delivery_status = item.delivery_status.as_deref().unwrap_or_default();
            let matches_status = active_status.is_empty() || delivery_status == active_status;
            matches_query && matches_type && matches_status
        })
        .cloned()
        .collect();

    let mut delivery_statuses: Vec<String> = resources
        .read()
        .iter()
        .filter_map(|item| item.delivery_status.clone())
        .collect();
    delivery_statuses.sort();
    delivery_statuses.dedup();

    rsx! {
        div { class: "space-y-6",
            div { class: "flex justify-between items-center",
                h1 { class: "text-2xl font-bold text-gray-800", "业务应用与资源" }
            }

            div { class: "grid grid-cols-1 md:grid-cols-4 gap-4",
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-blue-500",
                            Icon { icon: FaServer, width: 20, height: 20, class: "text-white" }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "资源总数" }
                            p { class: "text-xl font-bold text-gray-800", {total_count.to_string()} }
                        }
                    }
                }
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-cyan-500",
                            Icon { icon: FaCloud, width: 20, height: 20, class: "text-white" }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "云资源" }
                            p { class: "text-xl font-bold text-gray-800", {cloud_count.to_string()} }
                        }
                    }
                }
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-slate-600",
                            Icon { icon: FaServer, width: 20, height: 20, class: "text-white" }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "物理机" }
                            p { class: "text-xl font-bold text-gray-800", {physical_count.to_string()} }
                        }
                    }
                }
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-green-500",
                            Icon { icon: FaShieldHalved, width: 20, height: 20, class: "text-white" }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "已挂安全产品" }
                            p { class: "text-xl font-bold text-gray-800", {secured_count.to_string()} }
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
                            placeholder: "搜索资源名称、业务名称、客户或 IP...",
                            value: search_query,
                            oninput: move |e| search_query.set(e.value()),
                        }
                    }
                    div { class: "flex items-center",
                        Icon { icon: FaFilter, width: 18, height: 18, class: "text-gray-400" }
                        select {
                            class: "ml-2 px-3 py-2 border border-gray-300 rounded-md",
                            value: type_filter,
                            onchange: move |e| type_filter.set(e.value()),
                            option { value: "", "全部类型" }
                            option { value: "cloud", "云资源" }
                            option { value: "physical", "物理机" }
                            option { value: "network", "网络资源" }
                        }
                    }
                    select {
                        class: "px-3 py-2 border border-gray-300 rounded-md",
                        value: status_filter,
                        onchange: move |e| status_filter.set(e.value()),
                        option { value: "", "全部交付状态" }
                        for status in delivery_statuses {
                            option { value: "{status}", "{delivery_label(&status)}" }
                        }
                    }
                }
            }

            if *loading.read() {
                div { class: "bg-white rounded-lg shadow p-6 text-gray-500", "正在加载业务资源数据..." }
            } else if !error.read().is_empty() {
                div { class: "bg-red-50 border border-red-200 text-red-700 rounded-lg p-4", "{error}" }
            } else {
                VirtualScroller {
                    items: filtered_resources,
                    page_size: 20,
                    render_item: move |item: BusinessResourceRecord| rsx! {
                        div { class: "grid grid-cols-1 lg:grid-cols-6 gap-4 px-6 py-4 border-b border-gray-100 items-center",
                            div {
                                p { class: "text-sm font-medium text-gray-900", "{item.ecs_name}" }
                                p { class: "text-xs text-gray-500", "{item.application_name.clone().unwrap_or_else(|| item.customer_name.clone())}" }
                            }
                            div {
                                span { class: "px-2 inline-flex text-xs leading-5 font-semibold rounded-full {resource_type_class(&item.resource_type)}",
                                    "{resource_type_label(&item.resource_type)}"
                                }
                            }
                            div {
                                p { class: "text-sm text-gray-700", "{resource_location(&item)}" }
                                p { class: "text-xs text-gray-500", "{item.ip_address}" }
                            }
                            div {
                                p { class: "text-sm text-gray-700", "{item.customer_name}" }
                                p { class: "text-xs text-gray-500", "{item.ecs_type} / {item.ecs_os}" }
                            }
                            div {
                                span { class: "px-2 inline-flex text-xs leading-5 font-semibold rounded-full {delivery_class(item.delivery_status.as_deref())}",
                                    "{delivery_label(item.delivery_status.as_deref().unwrap_or(\"未配置\"))}"
                                }
                            }
                            div {
                                p { class: "text-sm text-gray-700", "{security_label(item.has_security_product)}" }
                                p { class: "text-xs text-gray-500", "{format_time(item.updated_at.as_deref().or(item.created_at.as_deref()))}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn resource_type_label(value: &str) -> &str {
    match value {
        "cloud" => "云资源",
        "physical" => "物理机",
        "network" => "网络资源",
        other => other,
    }
}

fn resource_type_class(value: &str) -> &'static str {
    match value {
        "cloud" => "bg-cyan-100 text-cyan-800",
        "physical" => "bg-slate-100 text-slate-800",
        "network" => "bg-purple-100 text-purple-800",
        _ => "bg-gray-100 text-gray-800",
    }
}

fn resource_location(item: &BusinessResourceRecord) -> String {
    item.platform_name
        .clone()
        .or_else(|| item.zone_name.clone())
        .or_else(|| item.county_city.clone())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| item.cloud_region.clone())
}

fn delivery_label(value: &str) -> &str {
    match value {
        "Pending" => "待交付",
        "Provisioning" => "配置中",
        "Delivered" => "已交付",
        "Confirmed" => "已确认",
        "Completed" => "已完成",
        "Rejected" => "已驳回",
        "未配置" => "未配置",
        other => other,
    }
}

fn delivery_class(value: Option<&str>) -> &'static str {
    match value.unwrap_or_default() {
        "Delivered" | "Confirmed" | "Completed" => "bg-green-100 text-green-800",
        "Provisioning" => "bg-blue-100 text-blue-800",
        "Rejected" => "bg-red-100 text-red-800",
        _ => "bg-amber-100 text-amber-800",
    }
}

fn security_label(has_security_product: bool) -> &'static str {
    if has_security_product {
        "已接入安全产品"
    } else {
        "未接入安全产品"
    }
}

fn format_time(value: Option<&str>) -> String {
    value
        .map(|raw| {
            raw.split('.')
                .next()
                .unwrap_or(raw)
                .replace('T', " ")
                .replace("+00:00", " UTC")
        })
        .unwrap_or_else(|| "暂无时间".to_string())
}
