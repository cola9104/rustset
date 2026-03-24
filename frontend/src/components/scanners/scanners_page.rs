//! Scanners Management Page

use crate::config::scan_ip_url;
use crate::utils::storage::authorization_header;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use web_sys::RequestCredentials;

/// 扫描结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub id: String,
    pub target: String,
    pub status: String,
    pub ports: Vec<PortResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortResult {
    pub port: u16,
    pub is_open: bool,
    pub service: Option<String>,
}

/// Scanners 页面
#[component]
pub fn ScannersPage() -> Element {
    let mut target_ip = use_signal(String::new);
    let mut ports_input = use_signal(String::new);
    let mut scan_results = use_signal(Vec::<ScanResult>::new);
    let mut scanning = use_signal(|| false);
    let mut error_msg = use_signal(String::new);

    // 执行扫描
    let do_scan = move |_| {
        let target = target_ip();
        let ports = ports_input();
        async move {
            if target.is_empty() {
                error_msg.set("Please enter target IP".to_string());
                return;
            }

            scanning.set(true);
            error_msg.set(String::new());

            // 构建请求
            let mut body = serde_json::json!({
                "target": target
            });

            if !ports.is_empty() {
                if let Ok(port_list) = serde_json::from_str::<Vec<u16>>(&format!("[{}]", ports)) {
                    body["ports"] = port_list.into();
                }
            }

            // API 调用
            let mut request = gloo_net::http::Request::post(&scan_ip_url())
                .credentials(RequestCredentials::Include)
                .header("Content-Type", "application/json");
            if let Some(header) = authorization_header() {
                request = request.header("Authorization", &header);
            }

            match request.body(serde_json::to_string(&body).unwrap_or_default()) {
                Ok(request) => {
                    if let Ok(response) = request.send().await {
                        if let Ok(result) = response.json::<ScanResult>().await {
                            let mut results = scan_results();
                            results.insert(0, result);
                            if results.len() > 10 {
                                results.truncate(10);
                            }
                            scan_results.set(results);
                        }
                    }
                }
                Err(e) => {
                    error_msg.set(format!("Failed to scan: {}", e));
                }
            }

            scanning.set(false);
        }
    };

    rsx! {
        div { class: "scanners-page",
            h1 { "Port Scanner" }

            // 扫描输入
            div { class: "scan-input",
                div { class: "input-group",
                    label { "Target IP:" }
                    input {
                        r#type: "text",
                        placeholder: "192.168.1.1",
                        value: "{target_ip}",
                        oninput: move |e| target_ip.set(e.value())
                    }
                }

                div { class: "input-group",
                    label { "Ports (comma-separated, optional):" }
                    input {
                        r#type: "text",
                        placeholder: "22,80,443",
                        value: "{ports_input}",
                        oninput: move |e| ports_input.set(e.value())
                    }
                }

                button {
                    onclick: do_scan,
                    disabled: *scanning.read(),
                    if *scanning.read() {
                        "Scanning..."
                    } else {
                        "Start Scan"
                    }
                }

                if !error_msg.read().is_empty() {
                    p { class: "error", "{error_msg}" }
                }
            }

            // 扫描结果
            div { class: "results",
                h2 { "Scan Results" }

                for result in scan_results.read().iter() {
                    div { class: "result-card",
                        div { class: "result-header",
                            h3 { "{result.target}" }
                            span { class: "status", "{result.status}" }
                        }

                        table {
                            thead {
                                tr {
                                    th { "Port" }
                                    th { "Status" }
                                    th { "Service" }
                                }
                            }
                            tbody {
                                for port in result.ports.iter() {
                                    tr {
                                        td { "{port.port}" }
                                        td {
                                            if port.is_open {
                                                "Open"
                                            } else {
                                                "Closed"
                                            }
                                        }
                                        td { "{port.service.as_deref().unwrap_or(\"-\")}" }
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
