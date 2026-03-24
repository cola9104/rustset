use crate::components::common::VirtualScroller;
use crate::services::asset_api::{fetch_assets, AssetRecord, NetworkZoneRecord};
use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaFilter, FaLock, FaMagnifyingGlass, FaNetworkWired, FaServer,
};
use dioxus_free_icons::Icon;

#[allow(non_snake_case)]
pub fn AssetManagement() -> Element {
    let assets = use_signal(Vec::<AssetRecord>::new);
    let mut search_query = use_signal(String::new);
    let mut zone_filter = use_signal(|| "".to_string());
    let loading = use_signal(|| true);
    let error = use_signal(String::new);

    {
        let mut assets = assets;
        let mut loading = loading;
        let mut error = error;
        use_effect(move || {
            spawn(async move {
                match fetch_assets().await {
                    Ok(data) => {
                        assets.set(data);
                        error.set(String::new());
                    }
                    Err(err) => error.set(err),
                }
                loading.set(false);
            });
        });
    }

    let total_count = assets.read().len() as i32;
    let internet_count = assets
        .read()
        .iter()
        .filter(|asset| zone_key(&asset.zone) == "Internet")
        .count() as i32;
    let open_port_count = assets
        .read()
        .iter()
        .map(|asset| asset.ports.iter().filter(|port| port.is_open).count() as i32)
        .sum::<i32>();
    let high_weight_count = assets
        .read()
        .iter()
        .filter(|asset| asset.weight >= 80)
        .count() as i32;

    let query = search_query.read().to_lowercase();
    let active_zone = zone_filter.read().clone();
    let filtered_assets: Vec<AssetRecord> = assets
        .read()
        .iter()
        .filter(|asset| {
            let matches_query = query.is_empty()
                || asset.name.to_lowercase().contains(&query)
                || asset.ip.to_lowercase().contains(&query)
                || asset
                    .owner
                    .as_deref()
                    .unwrap_or_default()
                    .to_lowercase()
                    .contains(&query)
                || asset
                    .labels
                    .iter()
                    .any(|label| label.to_lowercase().contains(&query));
            let matches_zone = active_zone.is_empty() || zone_key(&asset.zone) == active_zone;
            matches_query && matches_zone
        })
        .cloned()
        .collect();

    let mut zones: Vec<String> = assets
        .read()
        .iter()
        .map(|asset| zone_key(&asset.zone).to_string())
        .collect();
    zones.sort();
    zones.dedup();

    rsx! {
        div { class: "space-y-6",
            div { class: "flex justify-between items-center",
                h1 { class: "text-2xl font-bold text-gray-800", "资产管理" }
            }

            div { class: "grid grid-cols-1 md:grid-cols-4 gap-4",
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-blue-500",
                            Icon { icon: FaServer, width: 20, height: 20, class: "text-white" }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "资产总数" }
                            p { class: "text-xl font-bold text-gray-800", {total_count.to_string()} }
                        }
                    }
                }
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-cyan-500",
                            Icon { icon: FaNetworkWired, width: 20, height: 20, class: "text-white" }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "互联网区资产" }
                            p { class: "text-xl font-bold text-gray-800", {internet_count.to_string()} }
                        }
                    }
                }
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-amber-500",
                            Icon { icon: FaFilter, width: 20, height: 20, class: "text-white" }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "开放端口数" }
                            p { class: "text-xl font-bold text-gray-800", {open_port_count.to_string()} }
                        }
                    }
                }
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-red-500",
                            Icon { icon: FaLock, width: 20, height: 20, class: "text-white" }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "高权重资产" }
                            p { class: "text-xl font-bold text-gray-800", {high_weight_count.to_string()} }
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
                            placeholder: "搜索名称、IP、负责人或标签...",
                            value: search_query,
                            oninput: move |e| search_query.set(e.value()),
                        }
                    }
                    div { class: "flex items-center",
                        Icon { icon: FaFilter, width: 18, height: 18, class: "text-gray-400" }
                        select {
                            class: "ml-2 px-3 py-2 border border-gray-300 rounded-md",
                            value: zone_filter,
                            onchange: move |e| zone_filter.set(e.value()),
                            option { value: "", "全部区域" }
                            for zone in zones {
                                option { value: "{zone}", "{zone_label_text(&zone)}" }
                            }
                        }
                    }
                }
            }

            if *loading.read() {
                div { class: "bg-white rounded-lg shadow p-6 text-gray-500", "正在加载资产数据..." }
            } else if !error.read().is_empty() {
                div { class: "bg-red-50 border border-red-200 text-red-700 rounded-lg p-4", "{error}" }
            } else {
                VirtualScroller {
                    items: filtered_assets,
                    page_size: 20,
                    render_item: move |asset: AssetRecord| rsx! {
                        div { class: "grid grid-cols-1 lg:grid-cols-6 gap-4 px-6 py-4 border-b border-gray-100 items-center",
                            div {
                                p { class: "text-sm font-medium text-gray-900", "{asset.name}" }
                                p { class: "text-xs text-gray-500", "{asset.ip}" }
                            }
                            div {
                                span { class: "px-2 inline-flex text-xs leading-5 font-semibold rounded-full {zone_class(&asset.zone)}",
                                    "{zone_label(&asset.zone)}"
                                }
                            }
                            div { class: "text-sm text-gray-600", "{port_summary(&asset)}" }
                            div {
                                p { class: "text-sm text-gray-700", "{asset.owner.clone().unwrap_or_else(|| \"未分配\".to_string())}" }
                                p { class: "text-xs text-gray-500", "{asset.device_type.clone().unwrap_or_else(|| \"未知设备\".to_string())}" }
                            }
                            div {
                                p { class: "text-sm text-gray-700", "权重 {asset.weight}" }
                                p { class: "text-xs text-gray-500", "{asset.os.clone().unwrap_or_else(|| \"未识别系统\".to_string())}" }
                            }
                            div {
                                p { class: "text-sm text-gray-700", "{format_time(asset.last_scanned.as_deref())}" }
                                p { class: "text-xs text-gray-500", "{format_labels(&asset.labels)}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn zone_key(zone: &NetworkZoneRecord) -> &str {
    match zone {
        NetworkZoneRecord::Named(value) => value.as_str(),
        NetworkZoneRecord::Custom { custom } => custom.as_str(),
    }
}

fn zone_label(zone: &NetworkZoneRecord) -> String {
    zone_label_text(zone_key(zone)).to_string()
}

fn zone_label_text(value: &str) -> &str {
    match value {
        "Internet" => "互联网区",
        "DMZ" => "DMZ 区",
        "Intranet" => "内网区",
        other => other,
    }
}

fn zone_class(zone: &NetworkZoneRecord) -> &'static str {
    match zone_key(zone) {
        "Internet" => "bg-red-100 text-red-800",
        "DMZ" => "bg-amber-100 text-amber-800",
        "Intranet" => "bg-green-100 text-green-800",
        _ => "bg-gray-100 text-gray-800",
    }
}

fn port_summary(asset: &AssetRecord) -> String {
    let open_ports: Vec<String> = asset
        .ports
        .iter()
        .filter(|port| port.is_open)
        .map(|port| match &port.service {
            Some(service) => format!("{}({})", port.port, service),
            None => port.port.to_string(),
        })
        .collect();

    if open_ports.is_empty() {
        "无开放端口".to_string()
    } else {
        open_ports.join(", ")
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
        .unwrap_or_else(|| "未扫描".to_string())
}

fn format_labels(labels: &[String]) -> String {
    if labels.is_empty() {
        "无标签".to_string()
    } else {
        labels.join(" / ")
    }
}
