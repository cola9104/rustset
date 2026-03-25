use crate::components::common::VirtualScroller;
use crate::services::audit_api::{fetch_audit_logs, AuditLogRecord};
use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaFileLines, FaFilter, FaMagnifyingGlass, FaShield, FaUser,
};
use dioxus_free_icons::Icon;

#[derive(Clone, PartialEq)]
struct AuditDetailField {
    label: String,
    value: String,
}

#[allow(non_snake_case)]
pub fn AuditLogs() -> Element {
    let logs = use_signal(Vec::<AuditLogRecord>::new);
    let mut search_query = use_signal(String::new);
    let mut action_filter = use_signal(|| "".to_string());
    let mut category_filter = use_signal(|| "all".to_string());
    let mut outcome_filter = use_signal(|| "all".to_string());
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
    let role_change_count = logs
        .read()
        .iter()
        .filter(|log| audit_category_key(&log.action) == "role")
        .count() as i32;

    let query = search_query.read().to_lowercase();
    let action_filter_value = action_filter.read().clone();
    let category_filter_value = category_filter.read().clone();
    let outcome_filter_value = outcome_filter.read().clone();

    let filtered_logs: Vec<AuditLogRecord> = logs
        .read()
        .iter()
        .filter(|log| {
            let query_match = query.is_empty()
                || log.username.to_lowercase().contains(&query)
                || log.action.to_lowercase().contains(&query)
                || log.target.to_lowercase().contains(&query)
                || audit_category_label(&log.action)
                    .to_lowercase()
                    .contains(&query)
                || log.details.to_lowercase().contains(&query);
            let action_match = action_filter_value.is_empty() || log.action == action_filter_value;
            let category_match = category_filter_value == "all"
                || audit_category_key(&log.action) == category_filter_value;
            let outcome_match = match outcome_filter_value.as_str() {
                "normal" => !is_failed_log(log),
                "failed" => is_failed_log(log),
                _ => true,
            };
            query_match && action_match && category_match && outcome_match
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

            div { class: "grid grid-cols-1 md:grid-cols-2 xl:grid-cols-4 gap-4",
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
                        div { class: "p-2 rounded-full bg-amber-500",
                            Icon { icon: FaFilter, width: 20, height: 20, class: "text-white" }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "角色变更" }
                            p { class: "text-xl font-bold text-gray-800", {role_change_count.to_string()} }
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
                div { class: "grid grid-cols-1 gap-4 xl:grid-cols-[minmax(0,1fr),200px,180px,180px]",
                    div { class: "flex-1 flex items-center",
                        Icon { icon: FaMagnifyingGlass, width: 20, height: 20, class: "text-gray-400" }
                        input {
                            r#type: "text",
                            class: "ml-2 flex-1 border-0 focus:outline-none",
                            placeholder: "搜索用户、动作、目标或详情...",
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
                    select {
                        class: "px-3 py-2 border border-gray-300 rounded-md text-sm text-gray-700",
                        value: category_filter,
                        onchange: move |e| category_filter.set(e.value()),
                        option { value: "all", "全部类别" }
                        option { value: "auth", "登录认证" }
                        option { value: "role", "角色权限" }
                        option { value: "user", "用户账号" }
                        option { value: "asset", "资产与扫描" }
                        option { value: "task", "任务与处置" }
                        option { value: "other", "其他事件" }
                    }
                    select {
                        class: "px-3 py-2 border border-gray-300 rounded-md text-sm text-gray-700",
                        value: outcome_filter,
                        onchange: move |e| outcome_filter.set(e.value()),
                        option { value: "all", "全部结果" }
                        option { value: "normal", "仅正常" }
                        option { value: "failed", "仅异常" }
                    }
                }
                div { class: "mt-3 flex flex-wrap gap-2 text-xs text-gray-500",
                    span { class: "rounded-full bg-gray-100 px-3 py-1", "当前结果 {filtered_logs.len()} 条" }
                    span { class: "rounded-full bg-amber-50 px-3 py-1 text-amber-700", "角色日志会显示数据范围、权限数量、工单能力和影响用户" }
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
                        div { class: "rounded-lg border border-gray-100 bg-white px-5 py-4 shadow-sm",
                            div { class: "grid grid-cols-1 gap-4 lg:grid-cols-[190px,220px,minmax(0,1fr)]",
                                div { class: "space-y-2",
                                    p { class: "text-sm font-medium text-gray-900", "{format_time(&log.timestamp)}" }
                                    p { class: "text-xs text-gray-500", "目标: {display_target(&log)}" }
                                }
                                div { class: "space-y-3",
                                    div {
                                        span { class: "text-sm font-medium text-gray-900", "{log.username}" }
                                    }
                                    div { class: "flex flex-wrap gap-2",
                                        span { class: "{action_badge_class(&log.action)}",
                                            "{log.action}"
                                        }
                                        span { class: "px-2 py-1 inline-flex text-xs leading-5 font-semibold rounded-full bg-slate-100 text-slate-700",
                                            "{audit_category_label(&log.action)}"
                                        }
                                        span {
                                            class: if is_failed_log(&log) {
                                                "px-2 py-1 inline-flex text-xs leading-5 font-semibold rounded-full bg-red-100 text-red-800"
                                            } else {
                                                "px-2 py-1 inline-flex text-xs leading-5 font-semibold rounded-full bg-green-100 text-green-800"
                                            },
                                            if is_failed_log(&log) { "异常" } else { "正常" }
                                        }
                                    }
                                }
                                div { class: "space-y-3",
                                    {render_log_details(&log)}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn audit_category_key(action: &str) -> &'static str {
    if action.contains("LOGIN") || action.contains("LOGOUT") || action.contains("REFRESH") {
        "auth"
    } else if action.starts_with("ROLE_") {
        "role"
    } else if action.contains("USER") || action.contains("PASSWORD") {
        "user"
    } else if action.contains("ASSET") || action.contains("ZONE") || action.contains("SCAN") {
        "asset"
    } else if action.contains("TASK") || action.contains("RISK") {
        "task"
    } else {
        "other"
    }
}

fn audit_category_label(action: &str) -> &'static str {
    match audit_category_key(action) {
        "auth" => "登录认证",
        "role" => "角色权限",
        "user" => "用户账号",
        "asset" => "资产与扫描",
        "task" => "任务与处置",
        _ => "其他事件",
    }
}

fn action_badge_class(action: &str) -> &'static str {
    match audit_category_key(action) {
        "auth" => "px-2 py-1 inline-flex text-xs leading-5 font-semibold rounded-full bg-green-100 text-green-800",
        "role" => "px-2 py-1 inline-flex text-xs leading-5 font-semibold rounded-full bg-amber-100 text-amber-800",
        "user" => "px-2 py-1 inline-flex text-xs leading-5 font-semibold rounded-full bg-indigo-100 text-indigo-800",
        "asset" => "px-2 py-1 inline-flex text-xs leading-5 font-semibold rounded-full bg-sky-100 text-sky-800",
        "task" => "px-2 py-1 inline-flex text-xs leading-5 font-semibold rounded-full bg-violet-100 text-violet-800",
        _ => "px-2 py-1 inline-flex text-xs leading-5 font-semibold rounded-full bg-gray-100 text-gray-800",
    }
}

fn display_target(log: &AuditLogRecord) -> String {
    if log.target.trim().is_empty() {
        "未指定".to_string()
    } else {
        log.target.clone()
    }
}

fn render_log_details(log: &AuditLogRecord) -> Element {
    let role_fields = parse_role_audit_fields(log);
    if !role_fields.is_empty() {
        return rsx! {
            div { class: "space-y-3",
                p { class: "text-sm text-gray-600", "{role_log_summary(log)}" }
                div { class: "flex flex-wrap gap-2",
                    for field in role_fields {
                        span {
                            key: "{field.label}-{field.value}",
                            class: role_field_badge_class(&field.label),
                            strong { "{field.label}" }
                            " {field.value}"
                        }
                    }
                }
                if !log.details.trim().is_empty() {
                    p { class: "text-xs text-gray-400", "{log.details}" }
                }
            }
        };
    }

    rsx! {
        div { class: "space-y-2",
            p { class: "text-sm text-gray-700 break-all", "{log.details}" }
        }
    }
}

fn role_log_summary(log: &AuditLogRecord) -> String {
    match log.action.as_str() {
        "ROLE_CREATED" => "创建了一个自定义角色，以下是本次生效的权限范围和流程能力。".to_string(),
        "ROLE_UPDATED" => "更新了自定义角色，以下是角色当前权限配置和影响范围。".to_string(),
        "ROLE_DELETED" => "删除了一个自定义角色。".to_string(),
        _ => log.details.clone(),
    }
}

fn parse_role_audit_fields(log: &AuditLogRecord) -> Vec<AuditDetailField> {
    if audit_category_key(&log.action) != "role" {
        return Vec::new();
    }

    log.details
        .replace('，', ",")
        .split(',')
        .filter_map(|segment| {
            let item = segment.trim();
            let (label, value) = item.split_once(':')?;
            let label = label.trim();
            let value = value.trim();
            if label.is_empty() || value.is_empty() {
                return None;
            }
            Some(AuditDetailField {
                label: label.to_string(),
                value: value.to_string(),
            })
        })
        .collect()
}

fn role_field_badge_class(label: &str) -> &'static str {
    match label {
        "角色名" | "旧名称" => {
            "inline-flex items-center gap-1 rounded-full bg-slate-100 px-3 py-1 text-xs font-medium text-slate-700"
        }
        "数据范围" => {
            "inline-flex items-center gap-1 rounded-full bg-blue-100 px-3 py-1 text-xs font-medium text-blue-800"
        }
        "布尔权限数" => {
            "inline-flex items-center gap-1 rounded-full bg-emerald-100 px-3 py-1 text-xs font-medium text-emerald-800"
        }
        "工单能力" => {
            "inline-flex items-center gap-1 rounded-full bg-amber-100 px-3 py-1 text-xs font-medium text-amber-800"
        }
        "影响用户" => {
            "inline-flex items-center gap-1 rounded-full bg-rose-100 px-3 py-1 text-xs font-medium text-rose-800"
        }
        _ => "inline-flex items-center gap-1 rounded-full bg-gray-100 px-3 py-1 text-xs font-medium text-gray-700",
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
