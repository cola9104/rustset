use crate::components::common::confirm_dialog::{ConfirmDialog, ConfirmType};
use crate::components::common::{ErrorMessage, Modal, ModalFooter};
use crate::services::role_api::{
    create_role, delete_role, fetch_roles, update_role, RolePayload, RoleRecord,
};
use crate::services::user_api::{fetch_users, UserRecord};
use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaKey, FaMagnifyingGlass, FaPenToSquare, FaPlus, FaShield, FaTrash, FaUserGear, FaUsers,
};
use dioxus_free_icons::Icon;
use serde_json::{Map, Value};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
struct RoleSummary {
    id: String,
    name: String,
    description: String,
    is_system: bool,
    user_count: usize,
    permissions: Vec<String>,
    workflow_permissions: Vec<String>,
    resource_ticket_scope: String,
    created_at: Option<String>,
}

#[derive(Clone, Copy)]
struct PermissionGroupConfig {
    title: &'static str,
    description: &'static str,
    permissions: &'static [(&'static str, &'static str)],
}

const GENERAL_PERMISSION_ITEMS: &[(&str, &str)] = &[
    ("can_access_general", "访问通用模块"),
    ("can_view_dashboard", "查看仪表盘"),
    ("can_view_tasks", "查看任务中心"),
    ("can_create_task", "创建任务"),
    ("can_update_task", "更新任务"),
    ("can_delete_task", "删除任务"),
    ("can_view_advanced_scan", "查看高级扫描"),
    ("can_create_scan", "创建扫描"),
    ("can_delete_scan", "删除扫描"),
    ("can_export_scan", "导出扫描结果"),
];

const ASSET_RISK_PERMISSION_ITEMS: &[(&str, &str)] = &[
    ("can_access_assets_risks", "访问资产与风险"),
    ("can_view_cloud_assets", "查看云资产"),
    ("can_create_cloud_asset", "创建云资产"),
    ("can_update_cloud_asset", "更新云资产"),
    ("can_delete_cloud_asset", "删除云资产"),
    ("can_view_risks", "查看风险"),
    ("can_resolve_risk", "处置风险"),
    ("can_delete_risk", "删除风险"),
    ("can_view_business_process", "查看业务流程"),
    ("can_view_business_applications", "查看业务申请"),
    ("can_create_business_application", "创建业务申请"),
    ("can_approve_business_application", "审批业务申请"),
    ("can_supplement_business_application", "补充业务申请"),
    ("can_delete_business_application", "删除业务申请"),
    ("can_view_operations_management", "查看运维管理"),
    ("can_manage_operations", "执行运维管理"),
    ("can_view_automation_orchestration", "查看自动化编排"),
    ("can_execute_orchestration", "执行自动化编排"),
    ("can_manage_orchestration", "管理自动化编排"),
];

const CLOUD_PERMISSION_ITEMS: &[(&str, &str)] = &[
    ("can_access_cloud", "访问 Cloud 模块"),
    ("can_view_cloud_providers", "查看云厂商配置"),
    ("can_manage_cloud_providers", "管理云厂商配置"),
];

const USER_MANAGEMENT_PERMISSION_ITEMS: &[(&str, &str)] = &[
    ("can_access_user_management", "访问用户管理"),
    ("can_view_users", "查看用户"),
    ("can_create_user", "创建用户"),
    ("can_update_user", "更新用户"),
    ("can_delete_user", "删除用户"),
    ("can_manage_permissions", "管理权限"),
    ("can_view_password_policy", "查看密码策略"),
    ("can_manage_password_policy", "管理密码策略"),
];

const AUDIT_PERMISSION_ITEMS: &[(&str, &str)] = &[
    ("can_access_audit", "访问审计模块"),
    ("can_view_audit_logs", "查看审计日志"),
];

const RESOURCE_TICKET_PERMISSION_ITEMS: &[(&str, &str)] = &[
    ("can_view_resource_tickets", "查看资源工单"),
    ("can_create_resource_tickets", "创建资源工单"),
    ("can_approve_resource_tickets", "审批资源工单"),
    ("can_provision_resource_tickets", "配置资源工单"),
    ("can_deliver_resource_tickets", "交付资源工单"),
    ("can_delete_resource_tickets", "删除资源工单"),
];

const PERMISSION_GROUPS: &[PermissionGroupConfig] = &[
    PermissionGroupConfig {
        title: "通用模块",
        description: "仪表盘、任务中心和扫描能力。",
        permissions: GENERAL_PERMISSION_ITEMS,
    },
    PermissionGroupConfig {
        title: "资产与风险",
        description: "资产、风险、业务流程与运维编排。",
        permissions: ASSET_RISK_PERMISSION_ITEMS,
    },
    PermissionGroupConfig {
        title: "Cloud 模块",
        description: "云厂商对接配置能力。",
        permissions: CLOUD_PERMISSION_ITEMS,
    },
    PermissionGroupConfig {
        title: "用户管理",
        description: "用户、权限和密码策略配置。",
        permissions: USER_MANAGEMENT_PERMISSION_ITEMS,
    },
    PermissionGroupConfig {
        title: "审计模块",
        description: "审计日志相关查看能力。",
        permissions: AUDIT_PERMISSION_ITEMS,
    },
    PermissionGroupConfig {
        title: "资源工单",
        description: "资源申请、审批、交付及删除控制。",
        permissions: RESOURCE_TICKET_PERMISSION_ITEMS,
    },
];

#[derive(Clone, Debug, PartialEq)]
struct RoleEditorState {
    id: Option<String>,
    name: String,
    description: String,
    permission_values: BTreeMap<String, bool>,
    resource_ticket_scope: String,
    base_permissions: Map<String, Value>,
}

impl Default for RoleEditorState {
    fn default() -> Self {
        Self {
            id: None,
            name: String::new(),
            description: String::new(),
            permission_values: default_permission_values(),
            resource_ticket_scope: "self".to_string(),
            base_permissions: Map::new(),
        }
    }
}

impl RoleEditorState {
    fn from_role(role: &RoleRecord) -> Self {
        let permissions = role
            .permissions
            .as_ref()
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        let mut permission_values = default_permission_values();

        for key in all_permission_keys() {
            let enabled = permissions
                .get(key)
                .and_then(Value::as_bool)
                .unwrap_or(false);
            permission_values.insert(key.to_string(), enabled);
        }

        Self {
            id: Some(role_id(&role.id)),
            name: role.name.clone(),
            description: role.description.clone().unwrap_or_default(),
            permission_values,
            resource_ticket_scope: permissions
                .get("resource_ticket_scope")
                .and_then(Value::as_str)
                .unwrap_or("self")
                .to_string(),
            base_permissions: permissions,
        }
    }

    fn permission_enabled(&self, key: &str) -> bool {
        self.permission_values.get(key).copied().unwrap_or(false)
    }

    fn set_permission(&mut self, key: &str, enabled: bool) {
        self.permission_values.insert(key.to_string(), enabled);
    }

    fn set_all_permissions(&mut self, enabled: bool) {
        for key in all_permission_keys() {
            self.permission_values.insert(key.to_string(), enabled);
        }
    }

    fn selected_permission_count(&self) -> usize {
        self.permission_values
            .values()
            .filter(|enabled| **enabled)
            .count()
    }

    fn to_payload(&self) -> RolePayload {
        let mut permissions = self.base_permissions.clone();
        let mut normalized_permissions = self.permission_values.clone();
        apply_permission_dependencies(&mut normalized_permissions);

        for (key, enabled) in normalized_permissions {
            permissions.insert(key, Value::Bool(enabled));
        }

        permissions.insert(
            "resource_ticket_scope".to_string(),
            Value::String(self.resource_ticket_scope.clone()),
        );

        RolePayload {
            name: self.name.trim().to_string(),
            description: optional_text(&self.description),
            permissions: Value::Object(permissions),
        }
    }
}

#[allow(non_snake_case)]
pub fn PermissionManagement() -> Element {
    let roles = use_signal(Vec::<RoleRecord>::new);
    let users = use_signal(Vec::<UserRecord>::new);
    let mut search_query = use_signal(String::new);
    let mut role_type_filter = use_signal(|| "all".to_string());
    let mut scope_filter = use_signal(|| "all".to_string());
    let mut capability_filter = use_signal(|| "all".to_string());
    let mut show_editor = use_signal(|| false);
    let mut editor_state = use_signal(RoleEditorState::default);
    let mut delete_target = use_signal(|| Option::<(String, String, usize)>::None);
    let loading = use_signal(|| true);
    let mut error = use_signal(String::new);
    let mut success = use_signal(String::new);

    {
        let mut roles = roles;
        let mut users = users;
        let mut loading = loading;
        let mut error = error;
        use_effect(move || {
            spawn(async move {
                match fetch_roles().await {
                    Ok(role_data) => {
                        roles.set(role_data);
                        match fetch_users().await {
                            Ok(user_data) => {
                                users.set(user_data);
                                error.set(String::new());
                            }
                            Err(err) => error.set(err),
                        }
                    }
                    Err(err) => error.set(err),
                }
                loading.set(false);
            });
        });
    }

    let summaries: Vec<RoleSummary> = roles
        .read()
        .iter()
        .map(|role| build_role_summary(role, &users.read()))
        .collect();

    let total_roles = summaries.len() as i32;
    let system_roles = summaries.iter().filter(|role| role.is_system).count() as i32;
    let custom_roles = summaries.iter().filter(|role| !role.is_system).count() as i32;
    let total_users = users.read().len() as i32;

    let query = search_query.read().to_lowercase();
    let role_type_filter_value = role_type_filter.read().clone();
    let scope_filter_value = scope_filter.read().clone();
    let capability_filter_value = capability_filter.read().clone();
    let filtered_summaries: Vec<RoleSummary> = summaries
        .iter()
        .filter(|role| {
            query.is_empty()
                || role.name.to_lowercase().contains(&query)
                || role.description.to_lowercase().contains(&query)
                || role.resource_ticket_scope.to_lowercase().contains(&query)
                || role
                    .workflow_permissions
                    .iter()
                    .any(|permission| permission.to_lowercase().contains(&query))
                || role
                    .permissions
                    .iter()
                    .any(|permission| permission.to_lowercase().contains(&query))
        })
        .filter(|role| role_type_matches(role, &role_type_filter_value))
        .filter(|role| scope_matches(role, &scope_filter_value))
        .filter(|role| capability_matches(role, &capability_filter_value))
        .cloned()
        .collect();

    let mut permission_counts: BTreeMap<String, usize> = BTreeMap::new();
    for role in &summaries {
        for permission in &role.permissions {
            *permission_counts.entry(permission.clone()).or_default() += 1;
        }
    }
    let permission_rows: Vec<(String, usize)> = permission_counts.into_iter().collect();
    let editor_snapshot = editor_state.read().clone();
    let editing_role_summary = editor_snapshot
        .id
        .as_ref()
        .and_then(|id| summaries.iter().find(|summary| summary.id == *id).cloned());

    rsx! {
        div { class: "space-y-6",
            div { class: "flex justify-between items-center",
                h1 { class: "text-2xl font-bold text-gray-800", "权限管理" }
                button {
                    class: "px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 flex items-center gap-2",
                    onclick: move |_| {
                        editor_state.set(RoleEditorState::default());
                        success.set(String::new());
                        error.set(String::new());
                        show_editor.set(true);
                    },
                    Icon { icon: FaPlus, width: 16, height: 16 }
                    "新建自定义角色"
                }
            }

            if !success.read().is_empty() {
                div { class: "bg-green-50 border border-green-200 text-green-700 rounded-lg p-4", "{success}" }
            }

            div { class: "grid grid-cols-1 md:grid-cols-4 gap-4",
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-blue-500",
                            Icon { icon: FaUserGear, width: 20, height: 20, class: "text-white" }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "角色总数" }
                            p { class: "text-xl font-bold text-gray-800", {total_roles.to_string()} }
                        }
                    }
                }
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-emerald-500",
                            Icon { icon: FaShield, width: 20, height: 20, class: "text-white" }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "系统角色" }
                            p { class: "text-xl font-bold text-gray-800", {system_roles.to_string()} }
                        }
                    }
                }
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-amber-500",
                            Icon { icon: FaKey, width: 20, height: 20, class: "text-white" }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "自定义角色" }
                            p { class: "text-xl font-bold text-gray-800", {custom_roles.to_string()} }
                        }
                    }
                }
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-slate-600",
                            Icon { icon: FaUsers, width: 20, height: 20, class: "text-white" }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "系统用户" }
                            p { class: "text-xl font-bold text-gray-800", {total_users.to_string()} }
                        }
                    }
                }
            }

            div { class: "bg-white rounded-lg shadow p-4",
                div { class: "grid grid-cols-1 gap-4 xl:grid-cols-[minmax(0,1fr),160px,180px,180px]",
                    div { class: "flex items-center",
                        Icon { icon: FaMagnifyingGlass, width: 18, height: 18, class: "text-gray-400" }
                        input {
                            r#type: "text",
                            class: "ml-2 w-full border-0 focus:outline-none",
                            placeholder: "搜索角色名、描述、范围或权限关键字...",
                            value: search_query,
                            oninput: move |e| search_query.set(e.value()),
                        }
                    }
                    select {
                        class: "rounded-md border border-gray-300 px-3 py-2 text-sm text-gray-700",
                        value: role_type_filter,
                        onchange: move |e| role_type_filter.set(e.value()),
                        option { value: "all", "全部角色" }
                        option { value: "system", "系统角色" }
                        option { value: "custom", "自定义角色" }
                    }
                    select {
                        class: "rounded-md border border-gray-300 px-3 py-2 text-sm text-gray-700",
                        value: scope_filter,
                        onchange: move |e| scope_filter.set(e.value()),
                        option { value: "all", "全部范围" }
                        option { value: "仅自己", "仅自己" }
                        option { value: "本部门", "本部门" }
                        option { value: "本公司/组织", "本公司/组织" }
                        option { value: "全部", "全部" }
                    }
                    select {
                        class: "rounded-md border border-gray-300 px-3 py-2 text-sm text-gray-700",
                        value: capability_filter,
                        onchange: move |e| capability_filter.set(e.value()),
                        option { value: "all", "全部流程能力" }
                        option { value: "可提交", "可提交" }
                        option { value: "可审批", "可审批" }
                        option { value: "可配置", "可配置" }
                        option { value: "可交付", "可交付" }
                        option { value: "可删除工单", "可删除工单" }
                        option { value: "全局只读", "全局只读" }
                    }
                }
                div { class: "mt-3 flex flex-wrap gap-2 text-xs text-gray-500",
                    span { class: "rounded-full bg-gray-100 px-3 py-1", "当前结果 {filtered_summaries.len()} 个角色" }
                    span { class: "rounded-full bg-indigo-50 px-3 py-1 text-indigo-700", "搜索支持工单范围和流程能力关键字" }
                }
            }

            if *loading.read() {
                div { class: "bg-white rounded-lg shadow p-6 text-gray-500", "正在加载角色与权限数据..." }
            } else if !error.read().is_empty() {
                div { class: "bg-red-50 border border-red-200 text-red-700 rounded-lg p-4", "{error}" }
            } else {
                div { class: "grid grid-cols-1 xl:grid-cols-3 gap-6",
                    div { class: "xl:col-span-2 bg-white rounded-lg shadow overflow-hidden",
                        div { class: "px-6 py-4 border-b border-gray-100",
                            h2 { class: "text-lg font-semibold text-gray-800", "角色清单" }
                        }
                        for role in filtered_summaries {
                            div { class: "px-6 py-4 border-b border-gray-100 space-y-3",
                                div { class: "flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between",
                                    div {
                                        div { class: "flex items-center gap-3",
                                            h3 { class: "text-sm font-semibold text-gray-900", "{role.name}" }
                                            span {
                                                class: if role.is_system {
                                                    "px-2 py-1 text-xs font-semibold rounded-full bg-emerald-100 text-emerald-800"
                                                } else {
                                                    "px-2 py-1 text-xs font-semibold rounded-full bg-amber-100 text-amber-800"
                                                },
                                                if role.is_system { "系统角色" } else { "自定义角色" }
                                            }
                                        }
                                        p { class: "text-sm text-gray-500 mt-1", "{role.description}" }
                                    }
                                    div { class: "flex items-center gap-3",
                                        div { class: "text-sm text-gray-500",
                                            "{role.user_count} 个用户"
                                        }
                                        if !role.is_system {
                                            button {
                                                class: "text-blue-600 hover:text-blue-800",
                                                onclick: {
                                                    let role_record = roles
                                                        .read()
                                                        .iter()
                                                        .find(|item| role_id(&item.id) == role.id)
                                                        .cloned();
                                                    move |_| {
                                                        if let Some(item) = role_record.clone() {
                                                            editor_state.set(RoleEditorState::from_role(&item));
                                                            success.set(String::new());
                                                            error.set(String::new());
                                                            show_editor.set(true);
                                                        }
                                                    }
                                                },
                                                Icon { icon: FaPenToSquare, width: 16, height: 16 }
                                            }
                                            button {
                                                class: if role.user_count > 0 {
                                                    "text-gray-300 cursor-not-allowed"
                                                } else {
                                                    "text-red-600 hover:text-red-800"
                                                },
                                                disabled: role.user_count > 0,
                                                onclick: {
                                                    let role_id = role.id.clone();
                                                    let role_name = role.name.clone();
                                                    let role_user_count = role.user_count;
                                                    move |_| delete_target.set(Some((role_id.clone(), role_name.clone(), role_user_count)))
                                                },
                                                Icon { icon: FaTrash, width: 16, height: 16 }
                                            }
                                        }
                                    }
                                }
                                div { class: "flex flex-wrap gap-2",
                                    span { class: "px-2 py-1 text-xs rounded-full border border-indigo-100 bg-indigo-50 text-indigo-700",
                                        "工单范围: {role.resource_ticket_scope}"
                                    }
                                    if role.workflow_permissions.is_empty() {
                                        span { class: "px-2 py-1 text-xs rounded-full border border-slate-200 bg-slate-50 text-slate-500",
                                            "无工单流程能力"
                                        }
                                    } else {
                                        for capability in role.workflow_permissions.iter() {
                                            span { class: "px-2 py-1 text-xs rounded-full border border-emerald-100 bg-emerald-50 text-emerald-700",
                                                "{capability}"
                                            }
                                        }
                                    }
                                    for permission in role.permissions.iter().take(10) {
                                        span { class: "px-2 py-1 text-xs rounded-full bg-slate-100 text-slate-700",
                                            "{permission}"
                                        }
                                    }
                                    if role.permissions.len() > 10 {
                                        span { class: "px-2 py-1 text-xs rounded-full bg-slate-50 text-slate-500",
                                            "+{role.permissions.len() - 10} 项"
                                        }
                                    }
                                }
                                if let Some(created_at) = role.created_at.as_deref() {
                                    p { class: "text-xs text-gray-400", "创建时间: {format_time(created_at)}" }
                                }
                            }
                        }
                    }

                    div { class: "bg-white rounded-lg shadow overflow-hidden",
                        div { class: "px-6 py-4 border-b border-gray-100",
                            h2 { class: "text-lg font-semibold text-gray-800", "权限覆盖" }
                        }
                        for (permission, count) in permission_rows {
                            div { class: "px-6 py-3 border-b border-gray-100 flex items-center justify-between",
                                span { class: "text-sm text-gray-700", "{permission}" }
                                span { class: "text-xs font-semibold rounded-full bg-blue-100 text-blue-700 px-2 py-1",
                                    "{count} 个角色"
                                }
                            }
                        }
                    }
                }
            }

            Modal {
                show: *show_editor.read(),
                title: if editor_snapshot.id.is_some() { "编辑自定义角色".to_string() } else { "新建自定义角色".to_string() },
                size: "xl".to_string(),
                on_close: move |_| show_editor.set(false),
                footer: rsx! {
                    ModalFooter {
                        save_text: if editor_snapshot.id.is_some() { "保存".to_string() } else { "创建".to_string() },
                        cancel_text: "取消".to_string(),
                        save_disabled: false,
                        on_save: move |_| {
                            let payload = editor_state.read().to_payload();
                            if payload.name.trim().is_empty() {
                                error.set("角色名称不能为空".to_string());
                                return;
                            }

                            let role_id = editor_state.read().id.clone();
                            let mut roles = roles;
                            let mut error = error;
                            let mut success = success;
                            let mut show_editor = show_editor;
                            spawn(async move {
                                let result = if let Some(id) = role_id {
                                    update_role(&id, &payload).await
                                } else {
                                    create_role(&payload).await
                                };

                                match result {
                                    Ok(()) => {
                                        match fetch_roles().await {
                                            Ok(items) => roles.set(items),
                                            Err(err) => error.set(err),
                                        }
                                        success.set("角色配置已保存".to_string());
                                        error.set(String::new());
                                        show_editor.set(false);
                                    }
                                    Err(err) => error.set(err),
                                }
                            });
                        },
                        on_cancel: move |_| show_editor.set(false),
                    }
                },
                div { class: "space-y-6",
                    if !error.read().is_empty() {
                        ErrorMessage { message: error.read().clone() }
                    }
                    div { class: "rounded-lg border border-blue-100 bg-blue-50 px-4 py-3 text-sm text-blue-700",
                        "系统角色保持内置策略不开放编辑；自定义角色支持按模块勾选权限。保存时会自动补齐必要的模块访问和查看权限，避免出现子权限已开但页面无法访问的配置。"
                    }
                    if let Some(summary) = editing_role_summary.clone() {
                        div { class: "rounded-lg border border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-800",
                            "当前有 {summary.user_count} 个用户绑定此角色。保存后，这些用户的有效权限、工单范围和前端可见页面会立即按新配置生效。"
                        }
                    } else {
                        div { class: "rounded-lg border border-emerald-200 bg-emerald-50 px-4 py-3 text-sm text-emerald-700",
                            "新建角色后，可在用户管理里分配给具体账号；分配后该角色的数据范围和流程能力会立即影响对应用户。"
                        }
                    }
                    div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "角色名称" }
                            input {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg",
                                value: "{editor_snapshot.name}",
                                oninput: move |e| editor_state.write().name = e.value(),
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "资源工单数据范围" }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg",
                                value: "{editor_snapshot.resource_ticket_scope}",
                                onchange: move |e| editor_state.write().resource_ticket_scope = e.value(),
                                option { value: "self", "仅自己" }
                                option { value: "department", "本部门" }
                                option { value: "organization", "本公司/组织" }
                                option { value: "all", "全部" }
                            }
                            p { class: "mt-2 text-xs leading-5 text-slate-500",
                                "这里控制该角色在资源工单里默认能查看本人、部门、公司/组织还是全部数据。审批、配置、交付类角色通常应设置为“全部”或至少“本公司/组织”。"
                            }
                        }
                    }
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "角色描述" }
                        textarea {
                            class: "w-full px-3 py-2 border border-gray-300 rounded-lg",
                            rows: "3",
                            value: "{editor_snapshot.description}",
                            oninput: move |e| editor_state.write().description = e.value(),
                        }
                    }
                    div { class: "flex flex-col gap-3 rounded-lg border border-slate-200 bg-slate-50 p-4 md:flex-row md:items-center md:justify-between",
                        div {
                            h3 { class: "text-sm font-semibold text-slate-800", "权限配置" }
                            p { class: "text-sm text-slate-500", "已启用 {editor_snapshot.selected_permission_count()} 项布尔权限，资源工单范围单独控制。" }
                        }
                        div { class: "flex gap-2",
                            button {
                                class: "px-3 py-2 text-sm rounded-lg border border-slate-300 text-slate-700 hover:bg-white",
                                onclick: move |_| editor_state.write().set_all_permissions(true),
                                "全部勾选"
                            }
                            button {
                                class: "px-3 py-2 text-sm rounded-lg border border-slate-300 text-slate-700 hover:bg-white",
                                onclick: move |_| editor_state.write().set_all_permissions(false),
                                "全部清空"
                            }
                        }
                    }
                    div { class: "space-y-4",
                        for group in PERMISSION_GROUPS.iter() {
                            div { class: "rounded-lg border border-gray-200 bg-white p-4 shadow-sm",
                                div { class: "mb-3",
                                    h3 { class: "text-sm font-semibold text-gray-800", "{group.title}" }
                                    p { class: "mt-1 text-xs text-gray-500", "{group.description}" }
                                }
                                div { class: "grid grid-cols-1 md:grid-cols-2 gap-3",
                                    for (key, label) in group.permissions.iter().copied() {
                                        PermissionCheckbox {
                                            checked: editor_snapshot.permission_enabled(key),
                                            label: label.to_string(),
                                            on_toggle: {
                                                let key = key.to_string();
                                                move |value| editor_state.write().set_permission(&key, value)
                                            },
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            ConfirmDialog {
                show: delete_target.read().is_some(),
                title: "确认删除角色".to_string(),
                message: delete_target
                    .read()
                    .as_ref()
                    .map(|(_, name, user_count)| {
                        if *user_count > 0 {
                            format!(
                                "角色“{}”当前仍分配给 {} 个用户，需先把这些用户切换到其它角色后才能删除。",
                                name, user_count
                            )
                        } else {
                            format!("确定要删除角色“{}”吗？", name)
                        }
                    })
                    .unwrap_or_default(),
                confirm_type: ConfirmType::Danger,
                confirm_text: "删除".to_string(),
                cancel_text: "取消".to_string(),
                on_confirm: move |_| {
                    let target = delete_target.read().clone();
                    if let Some((role_id, _, user_count)) = target {
                        if user_count > 0 {
                            error.set("该角色仍绑定用户，请先迁移用户后再删除".to_string());
                            delete_target.set(None);
                            return;
                        }
                        let mut roles = roles;
                        let mut delete_target = delete_target;
                        let mut error = error;
                        let mut success = success;
                        spawn(async move {
                            match delete_role(&role_id).await {
                                Ok(()) => {
                                    match fetch_roles().await {
                                        Ok(items) => roles.set(items),
                                        Err(err) => error.set(err),
                                    }
                                    success.set("角色已删除".to_string());
                                    error.set(String::new());
                                    delete_target.set(None);
                                }
                                Err(err) => error.set(err),
                            }
                        });
                    }
                },
                on_cancel: move |_| delete_target.set(None),
            }
        }
    }
}

fn build_role_summary(role: &RoleRecord, users: &[UserRecord]) -> RoleSummary {
    let permissions_value = role.permissions.as_ref().and_then(Value::as_object);
    let user_count = users
        .iter()
        .filter(|user| user_matches_role(user, role))
        .count();
    RoleSummary {
        id: role_id(&role.id),
        name: role.name.clone(),
        description: role
            .description
            .clone()
            .unwrap_or_else(|| "无描述".to_string()),
        is_system: role.is_system,
        user_count,
        permissions: permissions_for_role(role),
        workflow_permissions: workflow_permissions_for_role(role),
        resource_ticket_scope: permissions_value
            .and_then(|map| map.get("resource_ticket_scope"))
            .and_then(Value::as_str)
            .map(resource_ticket_scope_label)
            .unwrap_or_else(|| default_scope_for_role(role).to_string()),
        created_at: role.created_at.clone(),
    }
}

fn role_type_matches(role: &RoleSummary, filter: &str) -> bool {
    match filter {
        "system" => role.is_system,
        "custom" => !role.is_system,
        _ => true,
    }
}

fn scope_matches(role: &RoleSummary, filter: &str) -> bool {
    match filter {
        "all" => true,
        value => role.resource_ticket_scope == value,
    }
}

fn capability_matches(role: &RoleSummary, filter: &str) -> bool {
    match filter {
        "all" => true,
        value => role
            .workflow_permissions
            .iter()
            .any(|permission| permission == value),
    }
}

#[component]
fn PermissionCheckbox(checked: bool, label: String, on_toggle: EventHandler<bool>) -> Element {
    rsx! {
        label { class: "flex items-center gap-3 rounded-lg border border-gray-200 px-3 py-2 text-sm text-gray-700 hover:border-blue-300 hover:bg-blue-50",
            input {
                r#type: "checkbox",
                checked: checked,
                onchange: move |e| on_toggle.call(e.checked()),
            }
            "{label}"
        }
    }
}

fn default_permission_values() -> BTreeMap<String, bool> {
    all_permission_keys()
        .into_iter()
        .map(|key| (key.to_string(), false))
        .collect()
}

fn all_permission_keys() -> Vec<&'static str> {
    PERMISSION_GROUPS
        .iter()
        .flat_map(|group| group.permissions.iter().map(|(key, _)| *key))
        .collect()
}

fn apply_permission_dependencies(permission_values: &mut BTreeMap<String, bool>) {
    enable_if_any(
        permission_values,
        "can_view_tasks",
        &["can_create_task", "can_update_task", "can_delete_task"],
    );
    enable_if_any(
        permission_values,
        "can_view_advanced_scan",
        &["can_create_scan", "can_delete_scan", "can_export_scan"],
    );
    enable_if_any(
        permission_values,
        "can_view_cloud_assets",
        &[
            "can_create_cloud_asset",
            "can_update_cloud_asset",
            "can_delete_cloud_asset",
        ],
    );
    enable_if_any(
        permission_values,
        "can_view_risks",
        &["can_resolve_risk", "can_delete_risk"],
    );
    enable_if_any(
        permission_values,
        "can_view_business_applications",
        &[
            "can_create_business_application",
            "can_approve_business_application",
            "can_supplement_business_application",
            "can_delete_business_application",
        ],
    );
    enable_if_any(
        permission_values,
        "can_view_operations_management",
        &["can_manage_operations"],
    );
    enable_if_any(
        permission_values,
        "can_view_automation_orchestration",
        &["can_execute_orchestration", "can_manage_orchestration"],
    );
    enable_if_any(
        permission_values,
        "can_view_cloud_providers",
        &["can_manage_cloud_providers"],
    );
    enable_if_any(
        permission_values,
        "can_view_users",
        &[
            "can_create_user",
            "can_update_user",
            "can_delete_user",
            "can_manage_permissions",
        ],
    );
    enable_if_any(
        permission_values,
        "can_view_password_policy",
        &["can_manage_password_policy"],
    );
    enable_if_any(
        permission_values,
        "can_view_resource_tickets",
        &[
            "can_create_resource_tickets",
            "can_approve_resource_tickets",
            "can_provision_resource_tickets",
            "can_deliver_resource_tickets",
            "can_delete_resource_tickets",
        ],
    );

    enable_if_any(
        permission_values,
        "can_access_general",
        &[
            "can_view_dashboard",
            "can_view_tasks",
            "can_create_task",
            "can_update_task",
            "can_delete_task",
            "can_view_advanced_scan",
            "can_create_scan",
            "can_delete_scan",
            "can_export_scan",
        ],
    );
    enable_if_any(
        permission_values,
        "can_access_assets_risks",
        &[
            "can_view_cloud_assets",
            "can_create_cloud_asset",
            "can_update_cloud_asset",
            "can_delete_cloud_asset",
            "can_view_risks",
            "can_resolve_risk",
            "can_delete_risk",
            "can_view_business_process",
            "can_view_business_applications",
            "can_create_business_application",
            "can_approve_business_application",
            "can_supplement_business_application",
            "can_delete_business_application",
            "can_view_operations_management",
            "can_manage_operations",
            "can_view_automation_orchestration",
            "can_execute_orchestration",
            "can_manage_orchestration",
        ],
    );
    enable_if_any(
        permission_values,
        "can_access_cloud",
        &["can_view_cloud_providers", "can_manage_cloud_providers"],
    );
    enable_if_any(
        permission_values,
        "can_access_user_management",
        &[
            "can_view_users",
            "can_create_user",
            "can_update_user",
            "can_delete_user",
            "can_manage_permissions",
            "can_view_password_policy",
            "can_manage_password_policy",
        ],
    );
    enable_if_any(
        permission_values,
        "can_access_audit",
        &["can_view_audit_logs"],
    );
}

fn enable_if_any(permission_values: &mut BTreeMap<String, bool>, target: &str, sources: &[&str]) {
    if sources
        .iter()
        .any(|key| permission_values.get(*key).copied().unwrap_or(false))
    {
        permission_values.insert(target.to_string(), true);
    }
}

fn role_id(value: &Value) -> String {
    match value {
        Value::String(value) => value.clone(),
        Value::Number(value) => value.to_string(),
        _ => String::new(),
    }
}

fn user_matches_role(user: &UserRecord, role: &RoleRecord) -> bool {
    if role.is_system {
        role.role
            .as_deref()
            .map(|value| value == user.role_value)
            .unwrap_or(false)
    } else {
        user.role_value == role.name
    }
}

fn permissions_for_role(role: &RoleRecord) -> Vec<String> {
    let permissions = role
        .permissions
        .as_ref()
        .and_then(Value::as_object)
        .map(|map| {
            let mut items: Vec<String> = map
                .iter()
                .filter_map(|(key, value)| {
                    if value.as_bool().is_some_and(|enabled| enabled) {
                        Some(permission_label(key))
                    } else {
                        value
                            .as_str()
                            .filter(|text| !text.trim().is_empty())
                            .map(|text| permission_value_label(key, text))
                    }
                })
                .collect();
            items.sort();
            if items.is_empty() {
                items.push("无启用权限".to_string());
            }
            items
        });

    if role.is_system {
        permissions.unwrap_or_else(|| system_role_permissions(role.role.as_deref()))
    } else {
        permissions.unwrap_or_else(|| vec!["无权限数据".to_string()])
    }
}

fn workflow_permissions_for_role(role: &RoleRecord) -> Vec<String> {
    let permissions = role.permissions.as_ref().and_then(Value::as_object);
    let mut items = Vec::new();

    if permission_enabled(permissions, "can_create_resource_tickets") {
        items.push("可提交".to_string());
    }
    if permission_enabled(permissions, "can_approve_resource_tickets") {
        items.push("可审批".to_string());
    }
    if permission_enabled(permissions, "can_provision_resource_tickets") {
        items.push("可配置".to_string());
    }
    if permission_enabled(permissions, "can_deliver_resource_tickets") {
        items.push("可交付".to_string());
    }
    if permission_enabled(permissions, "can_delete_resource_tickets") {
        items.push("可删除工单".to_string());
    }

    if items.is_empty() && role.is_system {
        match role.role.as_deref().unwrap_or_default() {
            "SysAdmin" => {
                items.push("可提交".to_string());
                items.push("可审批".to_string());
                items.push("可配置".to_string());
                items.push("可交付".to_string());
                items.push("可删除工单".to_string());
            }
            "SecAdmin" => {
                items.push("可提交".to_string());
                items.push("可审批".to_string());
                items.push("可配置".to_string());
                items.push("可交付".to_string());
            }
            "Auditor" => items.push("全局只读".to_string()),
            _ => {}
        }
    }

    items
}

fn system_role_permissions(role: Option<&str>) -> Vec<String> {
    match role.unwrap_or_default() {
        "SysAdmin" => vec![
            "全局管理".to_string(),
            "用户与权限管理".to_string(),
            "资产与风险管理".to_string(),
            "任务与扫描管理".to_string(),
            "审计日志查看".to_string(),
        ],
        "SecAdmin" => vec![
            "资产管理".to_string(),
            "风险处置".to_string(),
            "业务资源管理".to_string(),
            "扫描任务管理".to_string(),
            "审计日志查看".to_string(),
            "资源工单范围: 全部".to_string(),
        ],
        "Auditor" => vec![
            "仪表盘查看".to_string(),
            "任务与扫描只读".to_string(),
            "审计日志查看".to_string(),
            "资源工单范围: 全部".to_string(),
        ],
        _ => vec!["系统权限".to_string()],
    }
}

fn permission_label(key: &str) -> String {
    permission_label_text(key)
        .map(str::to_string)
        .unwrap_or_else(|| {
            key.trim_start_matches("can_")
                .replace('_', " ")
                .to_uppercase()
        })
}

fn permission_label_text(key: &str) -> Option<&'static str> {
    PERMISSION_GROUPS.iter().find_map(|group| {
        group
            .permissions
            .iter()
            .find(|(permission_key, _)| *permission_key == key)
            .map(|(_, label)| *label)
    })
}

fn permission_value_label(key: &str, value: &str) -> String {
    match key {
        "resource_ticket_scope" => {
            format!("资源工单范围: {}", resource_ticket_scope_label(value))
        }
        _ => format!("{}: {}", permission_label(key), value),
    }
}

fn permission_enabled(permissions: Option<&Map<String, Value>>, key: &str) -> bool {
    permissions
        .and_then(|map| map.get(key))
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn resource_ticket_scope_label(value: &str) -> String {
    match value {
        "self" => "仅自己".to_string(),
        "department" => "本部门".to_string(),
        "organization" => "本公司/组织".to_string(),
        "all" => "全部".to_string(),
        _ => value.to_string(),
    }
}

fn default_scope_for_role(role: &RoleRecord) -> &'static str {
    match role.role.as_deref().unwrap_or_default() {
        "SysAdmin" | "SecAdmin" | "Auditor" => "全部",
        _ => "仅自己",
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

fn optional_text(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}
