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
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
struct RoleSummary {
    id: String,
    name: String,
    description: String,
    is_system: bool,
    user_count: usize,
    permissions: Vec<String>,
    created_at: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
struct RoleEditorState {
    id: Option<String>,
    name: String,
    description: String,
    can_view_resource_tickets: bool,
    can_create_resource_tickets: bool,
    can_approve_resource_tickets: bool,
    can_provision_resource_tickets: bool,
    can_deliver_resource_tickets: bool,
    can_delete_resource_tickets: bool,
    resource_ticket_scope: String,
    base_permissions: Value,
}

impl Default for RoleEditorState {
    fn default() -> Self {
        Self {
            id: None,
            name: String::new(),
            description: String::new(),
            can_view_resource_tickets: true,
            can_create_resource_tickets: false,
            can_approve_resource_tickets: false,
            can_provision_resource_tickets: false,
            can_deliver_resource_tickets: false,
            can_delete_resource_tickets: false,
            resource_ticket_scope: "self".to_string(),
            base_permissions: Value::Object(Default::default()),
        }
    }
}

impl RoleEditorState {
    fn from_role(role: &RoleRecord) -> Self {
        let permissions = role
            .permissions
            .clone()
            .unwrap_or_else(|| Value::Object(Default::default()));
        let get_bool = |key: &str| {
            permissions
                .get(key)
                .and_then(Value::as_bool)
                .unwrap_or(false)
        };

        Self {
            id: Some(role_id(&role.id)),
            name: role.name.clone(),
            description: role.description.clone().unwrap_or_default(),
            can_view_resource_tickets: get_bool("can_view_resource_tickets"),
            can_create_resource_tickets: get_bool("can_create_resource_tickets"),
            can_approve_resource_tickets: get_bool("can_approve_resource_tickets"),
            can_provision_resource_tickets: get_bool("can_provision_resource_tickets"),
            can_deliver_resource_tickets: get_bool("can_deliver_resource_tickets"),
            can_delete_resource_tickets: get_bool("can_delete_resource_tickets"),
            resource_ticket_scope: permissions
                .get("resource_ticket_scope")
                .and_then(Value::as_str)
                .unwrap_or("self")
                .to_string(),
            base_permissions: permissions,
        }
    }

    fn to_payload(&self) -> RolePayload {
        let mut permissions = self
            .base_permissions
            .as_object()
            .cloned()
            .unwrap_or_default();

        permissions.insert(
            "can_view_resource_tickets".to_string(),
            Value::Bool(self.can_view_resource_tickets),
        );
        permissions.insert(
            "can_create_resource_tickets".to_string(),
            Value::Bool(self.can_create_resource_tickets),
        );
        permissions.insert(
            "can_approve_resource_tickets".to_string(),
            Value::Bool(self.can_approve_resource_tickets),
        );
        permissions.insert(
            "can_provision_resource_tickets".to_string(),
            Value::Bool(self.can_provision_resource_tickets),
        );
        permissions.insert(
            "can_deliver_resource_tickets".to_string(),
            Value::Bool(self.can_deliver_resource_tickets),
        );
        permissions.insert(
            "can_delete_resource_tickets".to_string(),
            Value::Bool(self.can_delete_resource_tickets),
        );
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
    let mut show_editor = use_signal(|| false);
    let mut editor_state = use_signal(RoleEditorState::default);
    let mut delete_target = use_signal(|| Option::<(String, String)>::None);
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
    let filtered_summaries: Vec<RoleSummary> = summaries
        .iter()
        .filter(|role| {
            query.is_empty()
                || role.name.to_lowercase().contains(&query)
                || role.description.to_lowercase().contains(&query)
                || role
                    .permissions
                    .iter()
                    .any(|permission| permission.to_lowercase().contains(&query))
        })
        .cloned()
        .collect();

    let mut permission_counts: BTreeMap<String, usize> = BTreeMap::new();
    for role in &summaries {
        for permission in &role.permissions {
            *permission_counts.entry(permission.clone()).or_default() += 1;
        }
    }
    let permission_rows: Vec<(String, usize)> = permission_counts.into_iter().collect();

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
                div { class: "flex items-center",
                    Icon { icon: FaMagnifyingGlass, width: 18, height: 18, class: "text-gray-400" }
                    input {
                        r#type: "text",
                        class: "ml-2 w-full border-0 focus:outline-none",
                        placeholder: "搜索角色名、描述或权限关键字...",
                        value: search_query,
                        oninput: move |e| search_query.set(e.value()),
                    }
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
                                                class: "text-red-600 hover:text-red-800",
                                                onclick: {
                                                    let role_id = role.id.clone();
                                                    let role_name = role.name.clone();
                                                    move |_| delete_target.set(Some((role_id.clone(), role_name.clone())))
                                                },
                                                Icon { icon: FaTrash, width: 16, height: 16 }
                                            }
                                        }
                                    }
                                }
                                div { class: "flex flex-wrap gap-2",
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
                title: if editor_state.read().id.is_some() { "编辑自定义角色".to_string() } else { "新建自定义角色".to_string() },
                size: "xl".to_string(),
                on_close: move |_| show_editor.set(false),
                footer: rsx! {
                    ModalFooter {
                        save_text: if editor_state.read().id.is_some() { "保存".to_string() } else { "创建".to_string() },
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
                    div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "角色名称" }
                            input {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg",
                                value: "{editor_state.read().name}",
                                oninput: move |e| editor_state.write().name = e.value(),
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "资源工单数据范围" }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg",
                                value: "{editor_state.read().resource_ticket_scope}",
                                onchange: move |e| editor_state.write().resource_ticket_scope = e.value(),
                                option { value: "self", "仅自己" }
                                option { value: "department", "本部门" }
                                option { value: "organization", "本公司/组织" }
                                option { value: "all", "全部" }
                            }
                        }
                    }
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "角色描述" }
                        textarea {
                            class: "w-full px-3 py-2 border border-gray-300 rounded-lg",
                            rows: "3",
                            value: "{editor_state.read().description}",
                            oninput: move |e| editor_state.write().description = e.value(),
                        }
                    }
                    div { class: "space-y-3",
                        h3 { class: "text-sm font-semibold text-gray-800", "资源工单权限" }
                        PermissionCheckbox {
                            checked: editor_state.read().can_view_resource_tickets,
                            label: "查看资源工单".to_string(),
                            on_toggle: move |value| editor_state.write().can_view_resource_tickets = value,
                        }
                        PermissionCheckbox {
                            checked: editor_state.read().can_create_resource_tickets,
                            label: "创建资源工单".to_string(),
                            on_toggle: move |value| editor_state.write().can_create_resource_tickets = value,
                        }
                        PermissionCheckbox {
                            checked: editor_state.read().can_approve_resource_tickets,
                            label: "审批资源工单".to_string(),
                            on_toggle: move |value| editor_state.write().can_approve_resource_tickets = value,
                        }
                        PermissionCheckbox {
                            checked: editor_state.read().can_provision_resource_tickets,
                            label: "配置资源工单".to_string(),
                            on_toggle: move |value| editor_state.write().can_provision_resource_tickets = value,
                        }
                        PermissionCheckbox {
                            checked: editor_state.read().can_deliver_resource_tickets,
                            label: "交付资源工单".to_string(),
                            on_toggle: move |value| editor_state.write().can_deliver_resource_tickets = value,
                        }
                        PermissionCheckbox {
                            checked: editor_state.read().can_delete_resource_tickets,
                            label: "删除资源工单".to_string(),
                            on_toggle: move |value| editor_state.write().can_delete_resource_tickets = value,
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
                    .map(|(_, name)| format!("确定要删除角色“{}”吗？", name))
                    .unwrap_or_default(),
                confirm_type: ConfirmType::Danger,
                confirm_text: "删除".to_string(),
                cancel_text: "取消".to_string(),
                on_confirm: move |_| {
                    if let Some((role_id, _)) = delete_target.read().clone() {
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
        created_at: role.created_at.clone(),
    }
}

#[component]
fn PermissionCheckbox(checked: bool, label: String, on_toggle: EventHandler<bool>) -> Element {
    rsx! {
        label { class: "flex items-center gap-3 text-sm text-gray-700",
            input {
                r#type: "checkbox",
                checked: checked,
                onchange: move |e| on_toggle.call(e.checked()),
            }
            "{label}"
        }
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
    match key {
        "can_access_general" => "访问通用模块".to_string(),
        "can_view_dashboard" => "查看仪表盘".to_string(),
        "can_view_tasks" => "查看任务中心".to_string(),
        "can_create_task" => "创建任务".to_string(),
        "can_delete_task" => "删除任务".to_string(),
        "can_update_task" => "更新任务".to_string(),
        "can_view_advanced_scan" => "查看高级扫描".to_string(),
        "can_create_scan" => "创建扫描".to_string(),
        "can_delete_scan" => "删除扫描".to_string(),
        "can_export_scan" => "导出扫描结果".to_string(),
        "can_access_assets_risks" => "访问资产与风险".to_string(),
        "can_view_cloud_assets" => "查看云资产".to_string(),
        "can_create_cloud_asset" => "创建云资产".to_string(),
        "can_update_cloud_asset" => "更新云资产".to_string(),
        "can_delete_cloud_asset" => "删除云资产".to_string(),
        "can_view_risks" => "查看风险".to_string(),
        "can_resolve_risk" => "处置风险".to_string(),
        "can_delete_risk" => "删除风险".to_string(),
        "can_view_business_process" => "查看业务流程".to_string(),
        "can_view_business_applications" => "查看业务申请".to_string(),
        "can_create_business_application" => "创建业务申请".to_string(),
        "can_approve_business_application" => "审批业务申请".to_string(),
        "can_supplement_business_application" => "补充业务申请".to_string(),
        "can_delete_business_application" => "删除业务申请".to_string(),
        "can_view_operations_management" => "查看运维管理".to_string(),
        "can_manage_operations" => "执行运维管理".to_string(),
        "can_view_automation_orchestration" => "查看自动化编排".to_string(),
        "can_execute_orchestration" => "执行自动化编排".to_string(),
        "can_manage_orchestration" => "管理自动化编排".to_string(),
        "can_access_cloud" => "访问 Cloud 模块".to_string(),
        "can_view_cloud_providers" => "查看云厂商配置".to_string(),
        "can_manage_cloud_providers" => "管理云厂商配置".to_string(),
        "can_access_user_management" => "访问用户管理".to_string(),
        "can_view_users" => "查看用户".to_string(),
        "can_create_user" => "创建用户".to_string(),
        "can_update_user" => "更新用户".to_string(),
        "can_delete_user" => "删除用户".to_string(),
        "can_manage_permissions" => "管理权限".to_string(),
        "can_view_password_policy" => "查看密码策略".to_string(),
        "can_manage_password_policy" => "管理密码策略".to_string(),
        "can_access_audit" => "访问审计模块".to_string(),
        "can_view_audit_logs" => "查看审计日志".to_string(),
        "can_view_resource_tickets" => "查看资源工单".to_string(),
        "can_create_resource_tickets" => "创建资源工单".to_string(),
        "can_approve_resource_tickets" => "审批资源工单".to_string(),
        "can_provision_resource_tickets" => "配置资源工单".to_string(),
        "can_deliver_resource_tickets" => "交付资源工单".to_string(),
        "can_delete_resource_tickets" => "删除资源工单".to_string(),
        other => other
            .trim_start_matches("can_")
            .replace('_', " ")
            .to_uppercase(),
    }
}

fn permission_value_label(key: &str, value: &str) -> String {
    match key {
        "resource_ticket_scope" => format!(
            "资源工单范围: {}",
            match value {
                "self" => "仅自己",
                "department" => "本部门",
                "organization" => "本公司/组织",
                "all" => "全部",
                _ => value,
            }
        ),
        _ => format!("{}: {}", permission_label(key), value),
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
