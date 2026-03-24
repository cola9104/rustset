use crate::services::risk_api::{fetch_risks, update_risk_status, RiskRecord};
use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaCircleCheck, FaClock, FaFilter, FaTriangleExclamation,
};
use dioxus_free_icons::Icon;

#[allow(non_snake_case)]
pub fn RiskCenter() -> Element {
    let risks = use_signal(Vec::<RiskRecord>::new);
    let mut severity_filter = use_signal(|| "".to_string());
    let mut status_filter = use_signal(|| "".to_string());
    let loading = use_signal(|| true);
    let error = use_signal(String::new);

    {
        let mut risks = risks;
        let mut loading = loading;
        let mut error = error;
        use_effect(move || {
            spawn(async move {
                match fetch_risks().await {
                    Ok(data) => {
                        risks.set(data);
                        error.set(String::new());
                    }
                    Err(err) => error.set(err),
                }
                loading.set(false);
            });
        });
    }

    let critical_count = risks
        .read()
        .iter()
        .filter(|risk| risk.severity.eq_ignore_ascii_case("critical"))
        .count() as i32;
    let high_count = risks
        .read()
        .iter()
        .filter(|risk| risk.severity.eq_ignore_ascii_case("high"))
        .count() as i32;
    let open_count = risks
        .read()
        .iter()
        .filter(|risk| risk.status == "Open" || risk.status == "PendingReview")
        .count() as i32;
    let resolved_count = risks
        .read()
        .iter()
        .filter(|risk| risk.status == "Resolved")
        .count() as i32;

    let filtered_risks: Vec<RiskRecord> = risks
        .read()
        .iter()
        .filter(|risk| {
            let severity_match = severity_filter.read().is_empty()
                || risk
                    .severity
                    .eq_ignore_ascii_case(severity_filter.read().as_str());
            let status_match =
                status_filter.read().is_empty() || risk.status == *status_filter.read();
            severity_match && status_match
        })
        .cloned()
        .collect();

    rsx! {
        div { class: "space-y-6",
            div { class: "flex justify-between items-center",
                h1 { class: "text-2xl font-bold text-gray-800", "风险中心" }
            }

            div { class: "grid grid-cols-1 md:grid-cols-4 gap-4",
                SimpleStatCard { title: "严重风险", value: critical_count.to_string(), icon_name: "critical".to_string() }
                SimpleStatCard { title: "高危风险", value: high_count.to_string(), icon_name: "high".to_string() }
                SimpleStatCard { title: "待处理", value: open_count.to_string(), icon_name: "open".to_string() }
                SimpleStatCard { title: "已解决", value: resolved_count.to_string(), icon_name: "resolved".to_string() }
            }

            div { class: "bg-white rounded-lg shadow p-4",
                div { class: "flex items-center gap-4",
                    Icon { icon: FaFilter, width: 18, height: 18, class: "text-gray-400" }
                    select {
                        class: "px-3 py-2 border border-gray-300 rounded-md",
                        value: severity_filter,
                        onchange: move |e| severity_filter.set(e.value()),
                        option { value: "", "全部等级" }
                        option { value: "critical", "严重" }
                        option { value: "high", "高危" }
                        option { value: "medium", "中危" }
                        option { value: "low", "低危" }
                    }
                    select {
                        class: "px-3 py-2 border border-gray-300 rounded-md",
                        value: status_filter,
                        onchange: move |e| status_filter.set(e.value()),
                        option { value: "", "全部状态" }
                        option { value: "Open", "待处理" }
                        option { value: "PendingReview", "待复核" }
                        option { value: "Verified", "已验证" }
                        option { value: "Resolved", "已解决" }
                        option { value: "Ignored", "已忽略" }
                        option { value: "FalsePositive", "误报" }
                    }
                }
            }

            if *loading.read() {
                div { class: "bg-white rounded-lg shadow p-6 text-gray-500", "正在加载风险数据..." }
            } else if !error.read().is_empty() {
                div { class: "bg-red-50 border border-red-200 text-red-700 rounded-lg p-4", "{error}" }
            } else {
                div { class: "bg-white rounded-lg shadow overflow-hidden",
                    for risk in filtered_risks {
                        div { class: "grid grid-cols-1 lg:grid-cols-7 gap-4 px-6 py-4 border-b border-gray-100 items-center",
                            div {
                                p { class: "text-sm font-medium text-gray-900", "{risk.description}" }
                                p { class: "text-xs text-gray-500", "{risk.asset_ip}:{risk.port}" }
                            }
                            div {
                                span { class: "px-2 inline-flex text-xs leading-5 font-semibold rounded-full {severity_class(&risk.severity)}",
                                    "{severity_label(&risk.severity)}"
                                }
                            }
                            div {
                                span { class: "px-2 inline-flex text-xs leading-5 font-semibold rounded-full {status_class(&risk.status)}",
                                    "{status_label(&risk.status)}"
                                }
                            }
                            div { class: "text-sm text-gray-600", "{risk.solution.clone().unwrap_or_else(|| \"-\".to_string())}" }
                            div { class: "text-sm text-gray-500", "{risk.assigned_to.clone().unwrap_or_else(|| \"未分配\".to_string())}" }
                            div { class: "text-sm text-gray-500",
                                "{risk.updated_at.clone().or(risk.created_at.clone()).unwrap_or_else(|| \"-\".to_string())}"
                            }
                            div { class: "flex items-center gap-2",
                                RiskActionButton { label: "已解决".to_string(), status: "resolved".to_string(), risk_id: risk.id.clone(), risks }
                                RiskActionButton { label: "忽略".to_string(), status: "ignored".to_string(), risk_id: risk.id.clone(), risks }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn SimpleStatCard(title: String, value: String, icon_name: String) -> Element {
    rsx! {
        div { class: "bg-white rounded-lg shadow p-4",
            div { class: "flex items-center",
                div {
                    class: match icon_name.as_str() {
                        "critical" => "p-2 rounded-full bg-red-100",
                        "high" => "p-2 rounded-full bg-orange-100",
                        "open" => "p-2 rounded-full bg-blue-100",
                        _ => "p-2 rounded-full bg-green-100",
                    },
                    if icon_name == "resolved" {
                        Icon { icon: FaCircleCheck, width: 20, height: 20 }
                    } else if icon_name == "open" {
                        Icon { icon: FaClock, width: 20, height: 20 }
                    } else {
                        Icon { icon: FaTriangleExclamation, width: 20, height: 20 }
                    }
                }
                div { class: "ml-3",
                    p { class: "text-sm text-gray-500", "{title}" }
                    p { class: "text-xl font-bold text-gray-800", "{value}" }
                }
            }
        }
    }
}

#[component]
fn RiskActionButton(
    label: String,
    status: String,
    risk_id: String,
    mut risks: Signal<Vec<RiskRecord>>,
) -> Element {
    rsx! {
        button {
            class: "px-2 py-1 text-xs rounded border border-gray-300 text-gray-700 hover:bg-gray-50",
            onclick: move |_| {
                let risk_id = risk_id.clone();
                let status = status.clone();
                spawn(async move {
                    if update_risk_status(&risk_id, &status).await.is_ok() {
                        if let Some(item) = risks.write().iter_mut().find(|item| item.id == risk_id) {
                            item.status = match status.as_str() {
                                "resolved" => "Resolved".to_string(),
                                "ignored" => "Ignored".to_string(),
                                other => other.to_string(),
                            };
                        }
                    }
                });
            },
            "{label}"
        }
    }
}

fn severity_label(value: &str) -> &str {
    match value.to_ascii_lowercase().as_str() {
        "critical" => "严重",
        "high" => "高危",
        "medium" => "中危",
        _ => "低危",
    }
}

fn severity_class(value: &str) -> &'static str {
    match value.to_ascii_lowercase().as_str() {
        "critical" => "bg-red-100 text-red-800",
        "high" => "bg-orange-100 text-orange-800",
        "medium" => "bg-yellow-100 text-yellow-800",
        _ => "bg-blue-100 text-blue-800",
    }
}

fn status_label(value: &str) -> &str {
    match value {
        "Open" => "待处理",
        "PendingReview" => "待复核",
        "Verified" => "已验证",
        "Resolved" => "已解决",
        "Ignored" => "已忽略",
        "FalsePositive" => "误报",
        _ => value,
    }
}

fn status_class(value: &str) -> &'static str {
    match value {
        "Open" | "PendingReview" => "bg-red-50 text-red-700",
        "Verified" => "bg-blue-50 text-blue-700",
        "Resolved" => "bg-green-50 text-green-700",
        _ => "bg-gray-50 text-gray-700",
    }
}
