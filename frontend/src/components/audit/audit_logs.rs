use crate::components::common::VirtualScroller;
use crate::services::audit_api::{fetch_audit_logs, AuditLogRecord};
use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaFileLines, FaFilter, FaMagnifyingGlass, FaShield, FaUser,
};
use dioxus_free_icons::Icon;

#[allow(non_snake_case)]
pub fn AuditLogs() -> Element {
    let logs = use_signal(Vec::<AuditLogRecord>::new);
    let mut search_query = use_signal(String::new);
    let mut action_filter = use_signal(|| "".to_string());
    let loading = use_signal(|| true);
    let error = use_signal(String::new);

    {
        let mut logs = logs;
        let mut loading = loading;
        let mut error = error;
        use_effect(move || {
            spawn(async move {
                match fetch_audit_logs().await {
                    Ok(data) => {
                        logs.set(data);
                        error.set(String::new());
                    }
                    Err(err) => error.set(err),
                }
                loading.set(false);
            });
        });
    }

    let total_count = logs.read().len() as i32;
    let login_count = logs
        .read()
        .iter()
        .filter(|log| log.action.contains("LOGIN"))
        .count() as i32;
    let failed_count = logs.read().iter().filter(|log| is_failed_log(log)).count() as i32;

    let filtered_logs: Vec<AuditLogRecord> = logs
        .read()
        .iter()
        .filter(|log| {
            let query = search_query.read().to_lowercase();
            let action_filter = action_filter.read();
            let query_match = query.is_empty()
                || log.username.to_lowercase().contains(&query)
                || log.target.to_lowercase().contains(&query)
                || log.details.to_lowercase().contains(&query);
            let action_match = action_filter.is_empty() || log.action == *action_filter;
            query_match && action_match
        })
        .cloned()
        .collect();

    let mut action_options: Vec<String> =
        logs.read().iter().map(|log| log.action.clone()).collect();
    action_options.sort();
    action_options.dedup();

    rsx! {
        div { class: "space-y-6",
            div { class: "flex justify-between items-center",
                h1 { class: "text-2xl font-bold text-gray-800", "审计日志" }
            }

            div { class: "grid grid-cols-1 md:grid-cols-3 gap-4",
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-blue-500",
                            Icon { icon: FaFileLines, width: 20, height: 20, class: "text-white" }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "总日志数" }
                            p { class: "text-xl font-bold text-gray-800", {total_count.to_string()} }
                        }
                    }
                }
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-green-500",
                            Icon { icon: FaUser, width: 20, height: 20, class: "text-white" }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "登录事件" }
                            p { class: "text-xl font-bold text-gray-800", {login_count.to_string()} }
                        }
                    }
                }
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-red-500",
                            Icon { icon: FaShield, width: 20, height: 20, class: "text-white" }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "异常事件" }
                            p { class: "text-xl font-bold text-gray-800", {failed_count.to_string()} }
                        }
                    }
                }
            }

            div { class: "bg-white rounded-lg shadow p-4",
                div { class: "flex items-center gap-4",
                    div { class: "flex-1 flex items-center",
                        Icon { icon: FaMagnifyingGlass, width: 20, height: 20, class: "text-gray-400" }
                        input {
                            r#type: "text",
                            class: "ml-2 flex-1 border-0 focus:outline-none",
                            placeholder: "搜索用户、目标或详情...",
                            value: search_query,
                            oninput: move |e| search_query.set(e.value()),
                        }
                    }
                    div { class: "flex items-center",
                        Icon { icon: FaFilter, width: 18, height: 18, class: "text-gray-400" }
                        select {
                            class: "ml-2 px-3 py-2 border border-gray-300 rounded-md",
                            value: action_filter,
                            onchange: move |e| action_filter.set(e.value()),
                            option { value: "", "全部动作" }
                            for action in action_options {
                                option { value: "{action}", "{action}" }
                            }
                        }
                    }
                }
            }

            if *loading.read() {
                div { class: "bg-white rounded-lg shadow p-6 text-gray-500", "正在加载审计日志..." }
            } else if !error.read().is_empty() {
                div { class: "bg-red-50 border border-red-200 text-red-700 rounded-lg p-4", "{error}" }
            } else {
                VirtualScroller {
                    items: filtered_logs,
                    page_size: 20,
                    render_item: move |log: AuditLogRecord| rsx! {
                        div { class: "grid grid-cols-1 lg:grid-cols-6 gap-4 px-6 py-4 border-b border-gray-100 items-center",
                            div { class: "text-sm text-gray-500", "{format_time(&log.timestamp)}" }
                            div { class: "text-sm font-medium text-gray-900", "{log.username}" }
                            div {
                                span { class: "px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-blue-100 text-blue-800",
                                    "{log.action}"
                                }
                            }
                            div { class: "text-sm text-gray-600", "{log.target}" }
                            div { class: "text-sm text-gray-500", "{log.details}" }
                            div {
                                span {
                                    class: if is_failed_log(&log) {
                                        "px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-red-100 text-red-800"
                                    } else {
                                        "px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-green-100 text-green-800"
                                    },
                                    if is_failed_log(&log) { "异常" } else { "正常" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn is_failed_log(log: &AuditLogRecord) -> bool {
    let details = log.details.to_lowercase();
    details.contains("failed")
        || details.contains("error")
        || details.contains("lock")
        || details.contains("失败")
        || details.contains("错误")
}

fn format_time(value: &str) -> String {
    value
        .split('.')
        .next()
        .unwrap_or(value)
        .replace('T', " ")
        .replace("+00:00", " UTC")
}
