//! IP Zones Management Page

use dioxus::prelude::*;
use shared::ZoneConfig;

/// IP Zones 页面
#[component]
pub fn IpZonesPage() -> Element {
    let mut zones = use_signal(Vec::<ZoneConfig>::new);
    let mut loading = use_signal(|| true);
    let mut search_ip = use_signal(String::new);
    let mut found_zone = use_signal(|| Option::<(String, String)>::None);

    // 加载 zones
    let _load_zones = move |_: dioxus::events::MouseEvent| async move {
        loading.set(true);
        // API 调用
        if let Ok(response) = gloo_net::http::Request::get("http://localhost:3003/api/ip-zones")
            .header("Authorization", "admin")
            .send()
            .await
        {
            if let Ok(data) = response.json::<Vec<ZoneConfig>>().await {
                zones.set(data);
            }
        }
        loading.set(false);
    };

    // 查找 IP 所属区域
    let find_zone = move |_| {
        let ip = search_ip();
        async move {
            if ip.is_empty() { return; }
            
            if let Ok(response) = gloo_net::http::Request::get(
                &format!("http://localhost:3003/api/ip-zones/find?ip={}", ip)
            )
            .header("Authorization", "admin")
            .send()
            .await
            {
                if let Ok(data) = response.json::<serde_json::Value>().await {
                    found_zone.set(Some((
                        data["zone"].as_str().unwrap_or("").to_string(),
                        data["matched_cidr"].as_str().unwrap_or("N/A").to_string()
                    )));
                }
            }
        }
    };

    rsx! {
        div { class: "ip-zones-page",
            h1 { "IP Zones Management" }
            
            // IP 查找
            div { class: "search-section",
                h3 { "Find Zone by IP" }
                input {
                    r#type: "text",
                    placeholder: "Enter IP address (e.g., 192.168.1.100)",
                    value: "{search_ip}",
                    oninput: move |e| search_ip.set(e.value())
                }
                button { onclick: find_zone, "Find Zone" }
                
                if let Some((zone, cidr)) = found_zone() {
                    div { class: "result",
                        p { "Zone: {zone}" }
                        p { "Matched CIDR: {cidr}" }
                    }
                }
            }
            
            // Zones 列表
            div { class: "zones-list",
                h3 { "Configured Zones" }
                
                if *loading.read() {
                    p { "Loading..." }
                } else {
                    table {
                        thead {
                            tr {
                                th { "Name" }
                                th { "CIDR" }
                                th { "Priority" }
                            }
                        }
                        tbody {
                            for zone in zones.read().iter() {
                                tr {
                                    td { "{zone.name}" }
                                    td { "{zone.cidr}" }
                                    td { "{zone.priority}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
