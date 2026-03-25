use crate::services::department_api::fetch_departments;
use crate::services::organization_api::fetch_organizations;
use crate::services::role_api::{fetch_roles, RoleRecord};
use crate::services::user_api::{
    create_user, delete_user, fetch_users, update_user, CreateUserPayload, UpdateUserPayload,
    UserRecord,
};
use crate::state::department::DepartmentRecord;
use crate::state::organization::OrganizationRecord;
use crate::state::user_role::use_auth;
use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaArrowRotateRight, FaKey, FaMagnifyingGlass, FaPenToSquare, FaPlus, FaShield, FaTrash, FaUser,
    FaUsers,
};
use dioxus_free_icons::Icon;
use serde_json::{json, Value};

#[allow(non_snake_case)]
pub fn UserManagement() -> Element {
    let auth = use_auth();
    let current_auth = auth.read().clone();
    let mut users = use_signal(Vec::<UserRecord>::new);
    let mut roles = use_signal(Vec::<RoleRecord>::new);
    let mut organizations = use_signal(Vec::<OrganizationRecord>::new);
    let mut departments = use_signal(Vec::<DepartmentRecord>::new);
    let mut search_query = use_signal(String::new);
    let mut editing_user = use_signal(|| None::<UserRecord>);
    let mut creating = use_signal(|| false);
    let mut loading = use_signal(|| true);
    let mut error = use_signal(String::new);

    let load_all = move || async move {
        loading.set(true);
        match (
            fetch_users().await,
            fetch_roles().await,
            fetch_organizations().await,
            fetch_departments().await,
        ) {
            (Ok(users_data), Ok(role_data), Ok(organizations_data), Ok(departments_data)) => {
                users.set(users_data);
                roles.set(role_data);
                organizations.set(organizations_data);
                departments.set(departments_data);
                error.set(String::new());
            }
            (Err(err), _, _, _)
            | (_, Err(err), _, _)
            | (_, _, Err(err), _)
            | (_, _, _, Err(err)) => error.set(err),
        }
        loading.set(false);
    };

    use_effect(move || {
        spawn(load_all());
    });

    let refresh = move |_| {
        spawn(load_all());
    };

    let filtered_users: Vec<UserRecord> = users
        .read()
        .iter()
        .filter(|user| {
            let query = search_query.read().to_lowercase();
            query.is_empty()
                || user.username.to_lowercase().contains(&query)
                || user.real_name.to_lowercase().contains(&query)
                || user.email.to_lowercase().contains(&query)
                || user.role_label.to_lowercase().contains(&query)
                || organization_name(&organizations.read(), user.organization_id)
                    .to_lowercase()
                    .contains(&query)
                || department_name(&departments.read(), user.department_id)
                    .to_lowercase()
                    .contains(&query)
        })
        .cloned()
        .collect();
    let is_empty = filtered_users.is_empty();
    let role_catalog = roles.read().clone();

    let total_count = users.read().len() as i32;
    let active_count = users
        .read()
        .iter()
        .filter(|u| u.status.as_str() == "active")
        .count() as i32;
    let locked_count = users
        .read()
        .iter()
        .filter(|u| u.status.as_str() == "locked")
        .count() as i32;
    let admin_count = users
        .read()
        .iter()
        .filter(|u| matches!(u.role_value.as_str(), "SysAdmin" | "SecAdmin"))
        .count() as i32;
    let can_create_user = current_auth.can_create_user();
    let can_update_user = current_auth.can_update_user();
    let can_delete_user = current_auth.can_delete_user();
    let is_read_only = !can_create_user && !can_update_user && !can_delete_user;

    rsx! {
        div { class: "space-y-6",
            div { class: "flex justify-between items-center",
                h1 { class: "text-2xl font-bold text-gray-800", "用户管理" }
                div { class: "flex items-center gap-3",
                    button {
                        class: "flex items-center px-4 py-2 border border-gray-300 text-gray-700 rounded-md hover:bg-gray-50 transition-colors",
                        onclick: refresh,
                        Icon { icon: FaArrowRotateRight, width: 16, height: 16 }
                        span { class: "ml-2", "刷新" }
                    }
                    if can_create_user {
                        button {
                            class: "flex items-center px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors",
                            onclick: move |_| creating.set(true),
                            Icon { icon: FaPlus, width: 16, height: 16 }
                            span { class: "ml-2", "添加用户" }
                        }
                    }
                }
            }

            if is_read_only {
                div { class: "rounded-lg border border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-800",
                    "当前账号只有用户查看权限，新增、编辑、删除操作已禁用。"
                }
            }

            div { class: "bg-white rounded-lg shadow p-4",
                div { class: "flex items-center",
                    Icon { icon: FaMagnifyingGlass, width: 20, height: 20, class: "text-gray-400" }
                    input {
                        r#type: "text",
                        class: "ml-2 flex-1 border-0 focus:outline-none",
                        placeholder: "搜索账号、姓名、组织、部门或角色...",
                        value: search_query,
                        oninput: move |e| search_query.set(e.value()),
                    }
                }
            }

            div { class: "grid grid-cols-1 md:grid-cols-4 gap-4",
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-blue-500",
                            Icon { icon: FaUsers, width: 20, height: 20, class: "text-white" }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "总用户" }
                            p { class: "text-xl font-bold text-gray-800", "{total_count}" }
                        }
                    }
                }
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-green-500",
                            Icon { icon: FaUser, width: 20, height: 20, class: "text-white" }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "活跃" }
                            p { class: "text-xl font-bold text-gray-800", "{active_count}" }
                        }
                    }
                }
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-red-500",
                            Icon { icon: FaShield, width: 20, height: 20, class: "text-white" }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "锁定" }
                            p { class: "text-xl font-bold text-gray-800", "{locked_count}" }
                        }
                    }
                }
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-purple-500",
                            Icon { icon: FaKey, width: 20, height: 20, class: "text-white" }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "管理员" }
                            p { class: "text-xl font-bold text-gray-800", "{admin_count}" }
                        }
                    }
                }
            }

            if *loading.read() {
                div { class: "bg-white rounded-lg shadow p-6 text-gray-500", "正在加载用户数据..." }
            } else if !error.read().is_empty() {
                div { class: "bg-red-50 border border-red-200 text-red-700 rounded-lg p-4", "{error}" }
            } else {
                div { class: "bg-white rounded-lg shadow overflow-hidden",
                    table { class: "min-w-full divide-y divide-gray-200",
                        thead { class: "bg-gray-50",
                            tr {
                                th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "账号" }
                                th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "姓名" }
                                th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "角色" }
                                th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "组织/部门" }
                                th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "联系方式" }
                                th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "状态" }
                                th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "最后登录" }
                                th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "创建时间" }
                                th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "操作" }
                            }
                        }
                        tbody { class: "bg-white divide-y divide-gray-200",
                            for user in filtered_users {
                                tr { class: "hover:bg-gray-50",
                                    td { class: "px-6 py-4 whitespace-nowrap",
                                        div { class: "text-sm font-medium text-gray-900", "{user.username}" }
                                        div { class: "text-xs text-gray-500", "ID: {user.id}" }
                                    }
                                    td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-900", "{user.real_name}" }
                                    td { class: "px-6 py-4 whitespace-nowrap",
                                        span { class: "px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-purple-100 text-purple-800",
                                            "{user.role_label}"
                                        }
                                        if let Some(role_scope) = user_role_scope(&role_catalog, &user) {
                                            div { class: "mt-1 text-xs text-indigo-600", "工单范围: {role_scope}" }
                                        }
                                        {
                                            let capability_summary = user_role_capability_summary(&role_catalog, &user);
                                            if !capability_summary.is_empty() {
                                                rsx! {
                                                    div { class: "mt-1 text-xs text-gray-500",
                                                        "{capability_summary}"
                                                    }
                                                }
                                            } else {
                                                rsx! {}
                                            }
                                        }
                                    }
                                    td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-500",
                                        div { "{organization_name(&organizations.read(), user.organization_id)}" }
                                        div { class: "text-xs text-gray-400", "{department_name(&departments.read(), user.department_id)}" }
                                    }
                                    td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-500",
                                        div { "{user.email}" }
                                        div { class: "text-xs text-gray-400", "{user.phone}" }
                                    }
                                    td { class: "px-6 py-4 whitespace-nowrap",
                                        span {
                                            class: if user.status == "active" {
                                                "px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-green-100 text-green-800"
                                            } else if user.status == "locked" {
                                                "px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-red-100 text-red-800"
                                            } else {
                                                "px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-gray-100 text-gray-800"
                                            },
                                            "{status_label(&user.status)}"
                                        }
                                    }
                                    td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-500", "{user.last_login}" }
                                    td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-500", "{user.created_at}" }
                                    td { class: "px-6 py-4 whitespace-nowrap text-sm font-medium",
                                        if can_update_user {
                                            button {
                                                class: "mr-3 text-blue-600 hover:text-blue-900",
                                                onclick: {
                                                    let user = user.clone();
                                                    move |_| editing_user.set(Some(user.clone()))
                                                },
                                                Icon { icon: FaPenToSquare, width: 16, height: 16 }
                                            }
                                        }
                                        if can_delete_user {
                                            button {
                                                class: "text-red-600 hover:text-red-900",
                                                onclick: {
                                                    let user_id = user.id.clone();
                                                    let mut users = users;
                                                    let mut error = error;
                                                    move |_| {
                                                        let user_id = user_id.clone();
                                                        spawn(async move {
                                                            match delete_user(&user_id).await {
                                                                Ok(()) => users.write().retain(|u| u.id != user_id),
                                                                Err(err) => error.set(err),
                                                            }
                                                        });
                                                    }
                                                },
                                                Icon { icon: FaTrash, width: 16, height: 16 }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if is_empty {
                        div { class: "text-center py-12 text-gray-500", "没有找到匹配的用户" }
                    }
                }
            }
        }

        if can_create_user && *creating.read() {
            UserModal {
                mode: UserModalMode::Create,
                user: None,
                roles: roles.read().clone(),
                organizations: organizations.read().clone(),
                departments: departments.read().clone(),
                on_close: move |_| creating.set(false),
                on_create: {
                    let mut users = users;
                    let mut error = error;
                    let mut creating = creating;
                    move |payload: CreateUserPayload| {
                        spawn(async move {
                            match create_user(&payload).await {
                                Ok(user) => {
                                    users.write().push(user);
                                    creating.set(false);
                                    error.set(String::new());
                                }
                                Err(err) => error.set(err),
                            }
                        });
                    }
                },
                on_update: move |_| {}
            }
        }

        if can_update_user {
            if let Some(user) = editing_user() {
            UserModal {
                mode: UserModalMode::Edit,
                user: Some(user),
                roles: roles.read().clone(),
                organizations: organizations.read().clone(),
                departments: departments.read().clone(),
                on_close: move |_| editing_user.set(None),
                on_create: move |_| {},
                on_update: {
                    let mut users = users;
                    let mut error = error;
                    let mut editing_user = editing_user;
                    move |payload: (String, UpdateUserPayload)| {
                        spawn(async move {
                            match update_user(&payload.0, &payload.1).await {
                                Ok(updated) => {
                                    if let Some(existing) = users.write().iter_mut().find(|item| item.id == updated.id) {
                                        *existing = updated;
                                    }
                                    editing_user.set(None);
                                    error.set(String::new());
                                }
                                Err(err) => error.set(err),
                            }
                        });
                    }
                }
            }
        }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum UserModalMode {
    Create,
    Edit,
}

#[component]
fn UserModal(
    mode: UserModalMode,
    user: Option<UserRecord>,
    roles: Vec<RoleRecord>,
    organizations: Vec<OrganizationRecord>,
    departments: Vec<DepartmentRecord>,
    on_close: EventHandler<()>,
    on_create: EventHandler<CreateUserPayload>,
    on_update: EventHandler<(String, UpdateUserPayload)>,
) -> Element {
    let initial = user.clone();
    let mut username = use_signal(|| {
        initial
            .as_ref()
            .map(|item| item.username.clone())
            .unwrap_or_default()
    });
    let mut real_name = use_signal(|| {
        initial
            .as_ref()
            .map(|item| item.real_name.clone())
            .unwrap_or_default()
    });
    let mut password = use_signal(String::new);
    let mut role = use_signal(|| {
        initial
            .as_ref()
            .map(|item| item.role_value.clone())
            .unwrap_or_else(|| "SecAdmin".to_string())
    });
    let mut email = use_signal(|| {
        initial
            .as_ref()
            .map(|item| empty_as_blank(&item.email))
            .unwrap_or_default()
    });
    let mut phone = use_signal(|| {
        initial
            .as_ref()
            .map(|item| empty_as_blank(&item.phone))
            .unwrap_or_default()
    });
    let mut status = use_signal(|| {
        initial
            .as_ref()
            .map(|item| item.status.clone())
            .unwrap_or_else(|| "active".to_string())
    });
    let mut organization_id = use_signal(|| {
        initial
            .as_ref()
            .and_then(|item| item.organization_id)
            .map(|value| value.to_string())
            .unwrap_or_default()
    });
    let mut department_id = use_signal(|| {
        initial
            .as_ref()
            .and_then(|item| item.department_id)
            .map(|value| value.to_string())
            .unwrap_or_default()
    });
    let mut form_error = use_signal(String::new);

    let username_value = username.read().clone();
    let builtin = is_builtin_username(&username_value);
    let filtered_departments = departments
        .iter()
        .filter(|item| organization_id.read().parse::<i32>().ok() == Some(item.organization_id))
        .cloned()
        .collect::<Vec<_>>();
    let selected_role_value = role.read().clone();
    let selected_role_record = roles
        .iter()
        .find(|item| role_key(item) == selected_role_value)
        .cloned();
    let role_scope = selected_role_record
        .as_ref()
        .and_then(role_scope_label)
        .unwrap_or_else(|| "仅自己".to_string());
    let role_capabilities = selected_role_record
        .as_ref()
        .map(role_capabilities)
        .unwrap_or_default();

    rsx! {
        div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            div { class: "bg-white rounded-lg shadow-xl w-full max-w-2xl mx-4",
                div { class: "flex justify-between items-center p-4 border-b",
                    h3 { class: "text-lg font-semibold",
                        if mode == UserModalMode::Create { "添加用户" } else { "编辑用户" }
                    }
                    button { class: "text-gray-400 hover:text-gray-600", onclick: move |_| on_close.call(()), "×" }
                }

                div { class: "p-4 space-y-4",
                    if !form_error.read().is_empty() {
                        div { class: "rounded-md border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700",
                            "{form_error}"
                        }
                    }

                    div { class: "grid grid-cols-2 gap-4",
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "账号" }
                            input {
                                r#type: "text",
                                disabled: mode == UserModalMode::Edit,
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:bg-gray-100",
                                value: username,
                                oninput: move |e| username.set(e.value()),
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "姓名" }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: real_name,
                                oninput: move |e| real_name.set(e.value()),
                            }
                        }
                        if mode == UserModalMode::Create {
                            div {
                                label { class: "block text-sm font-medium text-gray-700 mb-1", "密码" }
                                input {
                                    r#type: "password",
                                    class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                    value: password,
                                    oninput: move |e| password.set(e.value()),
                                }
                            }
                        } else {
                            div {
                                label { class: "block text-sm font-medium text-gray-700 mb-1", "重置密码" }
                                input {
                                    r#type: "password",
                                    class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                    placeholder: "留空则不修改",
                                    value: password,
                                    oninput: move |e| password.set(e.value()),
                                }
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "角色" }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: role,
                                onchange: move |e| role.set(e.value()),
                                option { value: "SysAdmin", "系统管理员" }
                                option { value: "SecAdmin", "安全管理员" }
                                option { value: "Auditor", "审计员" }
                                for role_item in roles.iter().filter(|item| !item.is_system) {
                                    option { value: "{role_item.name}", "{role_item.name}" }
                                }
                            }
                            if let Some(role_item) = selected_role_record.clone() {
                                div { class: "mt-2 rounded-md border border-slate-200 bg-slate-50 px-3 py-3 text-sm",
                                    div { class: "flex flex-wrap items-center gap-2",
                                        span { class: "font-medium text-slate-800", "{role_item.name}" }
                                        span {
                                            class: if role_item.is_system {
                                                "rounded-full bg-emerald-100 px-2 py-0.5 text-xs font-medium text-emerald-700"
                                            } else {
                                                "rounded-full bg-amber-100 px-2 py-0.5 text-xs font-medium text-amber-700"
                                            },
                                            if role_item.is_system { "系统角色" } else { "自定义角色" }
                                        }
                                        span { class: "rounded-full bg-indigo-100 px-2 py-0.5 text-xs font-medium text-indigo-700",
                                            "工单范围: {role_scope}"
                                        }
                                    }
                                    if !role_capabilities.is_empty() {
                                        div { class: "mt-2 flex flex-wrap gap-2",
                                            for capability in role_capabilities.iter() {
                                                span { class: "rounded-full bg-blue-100 px-2 py-0.5 text-xs font-medium text-blue-700",
                                                    "{capability}"
                                                }
                                            }
                                        }
                                    }
                                    if let Some(description) = role_item.description.as_deref() {
                                        if !description.trim().is_empty() {
                                            p { class: "mt-2 text-xs leading-5 text-slate-500", "{description}" }
                                        }
                                    }
                                }
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "邮箱" }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: email,
                                oninput: move |e| email.set(e.value()),
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "手机号" }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: phone,
                                oninput: move |e| phone.set(e.value()),
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "状态" }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: status,
                                onchange: move |e| status.set(e.value()),
                                option { value: "active", "活跃" }
                                option { value: "disabled", "禁用" }
                                option { value: "locked", "锁定" }
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "组织/单位" }
                            select {
                                disabled: builtin,
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:bg-gray-100",
                                value: organization_id,
                                onchange: move |e| {
                                    organization_id.set(e.value());
                                    department_id.set(String::new());
                                },
                                option { value: "", if builtin { "系统内置账号可不绑定" } else { "请选择组织/单位" } }
                                for item in organizations.iter() {
                                    option { value: "{item.id}", "{item.name}" }
                                }
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "部门" }
                            select {
                                disabled: builtin || organization_id.read().is_empty(),
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:bg-gray-100",
                                value: department_id,
                                onchange: move |e| department_id.set(e.value()),
                                option { value: "", if builtin { "系统内置账号可不绑定" } else { "请选择部门" } }
                                for item in filtered_departments.iter() {
                                    option { value: "{item.id}", "{item.name}" }
                                }
                            }
                        }
                    }
                    if builtin {
                        div { class: "rounded-md bg-amber-50 px-3 py-2 text-sm text-amber-700",
                            "默认系统账号 admin / sec / audit 允许不绑定组织和部门。"
                        }
                    }
                }

                div { class: "flex justify-end space-x-3 p-4 border-t",
                    button {
                        class: "px-4 py-2 text-gray-600 border border-gray-300 rounded-md hover:bg-gray-50",
                        onclick: move |_| on_close.call(()),
                        "取消"
                    }
                    button {
                        class: "px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700",
                        onclick: move |_| {
                            let username_value = username.read().trim().to_string();
                            let real_name_value = optional_text(&real_name.read());
                            let password_value = optional_text(&password.read());
                            let organization_value = organization_id.read().parse::<i32>().ok();
                            let department_value = department_id.read().parse::<i32>().ok();

                            if username_value.is_empty() {
                                form_error.set("账号不能为空".to_string());
                                return;
                            }
                            if mode == UserModalMode::Create && password_value.is_none() {
                                form_error.set("密码不能为空".to_string());
                                return;
                            }
                            if !is_builtin_username(&username_value) {
                                if real_name_value.is_none() {
                                    form_error.set("普通账号必须填写姓名".to_string());
                                    return;
                                }
                                if organization_value.is_none() || department_value.is_none() {
                                    form_error.set("普通账号必须绑定组织和部门".to_string());
                                    return;
                                }
                            }

                            form_error.set(String::new());
                            if mode == UserModalMode::Create {
                                on_create.call(CreateUserPayload {
                                    username: username_value,
                                    real_name: real_name_value,
                                    password: password_value.unwrap_or_default(),
                                    role: role_request_value(&role.read(), &roles),
                                    email: optional_text(&email.read()),
                                    phone: optional_text(&phone.read()),
                                    status: Some(status.read().clone()),
                                    organization_id: organization_value,
                                    department_id: department_value,
                                });
                            } else if let Some(user) = &user {
                                on_update.call((user.id.clone(), UpdateUserPayload {
                                    real_name: real_name_value,
                                    password: password_value,
                                    role: Some(role_request_value(&role.read(), &roles)),
                                    email: optional_text(&email.read()),
                                    phone: optional_text(&phone.read()),
                                    status: Some(status.read().clone()),
                                    organization_id: organization_value,
                                    department_id: department_value,
                                }));
                            }
                        },
                        "保存"
                    }
                }
            }
        }
    }
}

fn is_builtin_username(username: &str) -> bool {
    matches!(username, "admin" | "sec" | "audit")
}

fn optional_text(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn empty_as_blank(value: &str) -> String {
    if value == "-" {
        String::new()
    } else {
        value.to_string()
    }
}

fn organization_name(organizations: &[OrganizationRecord], organization_id: Option<i32>) -> String {
    organization_id
        .and_then(|id| organizations.iter().find(|item| item.id == id))
        .map(|item| item.name.clone())
        .unwrap_or_else(|| "-".to_string())
}

fn department_name(departments: &[DepartmentRecord], department_id: Option<i32>) -> String {
    department_id
        .and_then(|id| departments.iter().find(|item| item.id == id))
        .map(|item| item.name.clone())
        .unwrap_or_else(|| "-".to_string())
}

fn status_label(status: &str) -> &str {
    match status {
        "active" => "活跃",
        "locked" => "锁定",
        "disabled" => "禁用",
        _ => "未知",
    }
}

fn role_key(role: &RoleRecord) -> String {
    role.role.clone().unwrap_or_else(|| role.name.clone())
}

fn role_request_value(selected_role: &str, roles: &[RoleRecord]) -> serde_json::Value {
    if roles
        .iter()
        .any(|role| !role.is_system && role.name == selected_role)
    {
        json!({ "Custom": selected_role })
    } else {
        json!(selected_role)
    }
}

fn role_scope_label(role: &RoleRecord) -> Option<String> {
    role.permissions
        .as_ref()
        .and_then(Value::as_object)
        .and_then(|map| map.get("resource_ticket_scope"))
        .and_then(Value::as_str)
        .map(|scope| match scope {
            "self" => "仅自己".to_string(),
            "department" => "本部门".to_string(),
            "organization" => "本公司/组织".to_string(),
            "all" => "全部".to_string(),
            other => other.to_string(),
        })
}

fn user_role_record<'a>(roles: &'a [RoleRecord], user: &UserRecord) -> Option<&'a RoleRecord> {
    roles.iter().find(|role| {
        if role.is_system {
            role.role
                .as_deref()
                .map(|value| value == user.role_value)
                .unwrap_or(false)
        } else {
            role.name == user.role_value
        }
    })
}

fn user_role_scope(roles: &[RoleRecord], user: &UserRecord) -> Option<String> {
    user_role_record(roles, user).and_then(role_scope_label)
}

fn user_role_capability_summary(roles: &[RoleRecord], user: &UserRecord) -> String {
    user_role_record(roles, user)
        .map(role_capabilities)
        .unwrap_or_default()
        .into_iter()
        .take(3)
        .collect::<Vec<_>>()
        .join(" / ")
}

fn role_capabilities(role: &RoleRecord) -> Vec<String> {
    let permissions = role.permissions.as_ref().and_then(Value::as_object);
    let mut items = Vec::new();

    if permission_enabled(permissions, "can_create_resource_tickets") {
        items.push("可提交资源工单".to_string());
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
    if permission_enabled(permissions, "can_manage_permissions") {
        items.push("可管理权限".to_string());
    }
    if permission_enabled(permissions, "can_view_users") {
        items.push("可查看用户".to_string());
    }
    if permission_enabled(permissions, "can_manage_operations") {
        items.push("可运维操作".to_string());
    }

    if items.is_empty() && role.is_system {
        match role.role.as_deref().unwrap_or_default() {
            "SysAdmin" => {
                items.push("全局管理".to_string());
                items.push("可提交资源工单".to_string());
                items.push("可审批".to_string());
                items.push("可配置".to_string());
                items.push("可交付".to_string());
            }
            "SecAdmin" => {
                items.push("资产与运维管理".to_string());
                items.push("可提交资源工单".to_string());
                items.push("可审批".to_string());
                items.push("可配置".to_string());
                items.push("可交付".to_string());
            }
            "Auditor" => {
                items.push("全局审计只读".to_string());
            }
            _ => {}
        }
    }

    items
}

fn permission_enabled(permissions: Option<&serde_json::Map<String, Value>>, key: &str) -> bool {
    permissions
        .and_then(|map| map.get(key))
        .and_then(Value::as_bool)
        .unwrap_or(false)
}
