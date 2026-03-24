use crate::services::user_api::{
    create_user, delete_user, fetch_users, CreateUserPayload, UserRecord,
};
use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaArrowRotateRight, FaKey, FaMagnifyingGlass, FaPlus, FaShield, FaTrash, FaUser, FaUsers,
};
use dioxus_free_icons::Icon;

#[allow(non_snake_case)]
pub fn UserManagement() -> Element {
    let users = use_signal(Vec::<UserRecord>::new);
    let mut search_query = use_signal(String::new);
    let mut show_add_modal = use_signal(|| false);
    let loading = use_signal(|| true);
    let error = use_signal(String::new);

    {
        let mut users = users;
        let mut loading = loading;
        let mut error = error;
        use_effect(move || {
            spawn(async move {
                match fetch_users().await {
                    Ok(data) => {
                        users.set(data);
                        error.set(String::new());
                    }
                    Err(err) => error.set(err),
                }
                loading.set(false);
            });
        });
    }

    let refresh = move |_| {
        let mut users = users;
        let mut loading = loading;
        let mut error = error;
        spawn(async move {
            loading.set(true);
            match fetch_users().await {
                Ok(data) => {
                    users.set(data);
                    error.set(String::new());
                }
                Err(err) => error.set(err),
            }
            loading.set(false);
        });
    };

    let filtered_users: Vec<UserRecord> = users
        .read()
        .iter()
        .filter(|user| {
            let query = search_query.read().to_lowercase();
            query.is_empty()
                || user.username.to_lowercase().contains(&query)
                || user.email.to_lowercase().contains(&query)
                || user.role_label.to_lowercase().contains(&query)
        })
        .cloned()
        .collect();
    let is_empty = filtered_users.is_empty();

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
                    button {
                        class: "flex items-center px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors",
                        onclick: move |_| show_add_modal.set(true),
                        Icon { icon: FaPlus, width: 16, height: 16 }
                        span { class: "ml-2", "添加用户" }
                    }
                }
            }

            div { class: "bg-white rounded-lg shadow p-4",
                div { class: "flex items-center",
                    Icon { icon: FaMagnifyingGlass, width: 20, height: 20, class: "text-gray-400" }
                    input {
                        r#type: "text",
                        class: "ml-2 flex-1 border-0 focus:outline-none",
                        placeholder: "搜索用户名、邮箱或角色...",
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
                            p { class: "text-sm text-gray-500", "活跃" }
                            p { class: "text-xl font-bold text-gray-800", {active_count.to_string()} }
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
                            p { class: "text-xl font-bold text-gray-800", {locked_count.to_string()} }
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
                            p { class: "text-xl font-bold text-gray-800", {admin_count.to_string()} }
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
                                th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "用户名" }
                                th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "角色" }
                                th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "邮箱" }
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
                                    td { class: "px-6 py-4 whitespace-nowrap",
                                        span { class: "px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-purple-100 text-purple-800",
                                            "{user.role_label}"
                                        }
                                    }
                                    td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-500", "{user.email}" }
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

                    if is_empty {
                        div { class: "text-center py-12 text-gray-500", "没有找到匹配的用户" }
                    }
                }
            }
        }

        if *show_add_modal.read() {
            AddUserModal {
                on_close: move |_| show_add_modal.set(false),
                on_save: {
                    let mut users = users;
                    let mut error = error;
                    let mut show_add_modal = show_add_modal;
                    move |payload: CreateUserPayload| {
                        spawn(async move {
                            match create_user(&payload).await {
                                Ok(user) => {
                                    users.write().push(user);
                                    show_add_modal.set(false);
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

#[component]
fn AddUserModal(on_close: EventHandler<()>, on_save: EventHandler<CreateUserPayload>) -> Element {
    let mut username = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut role = use_signal(|| "SecAdmin".to_string());

    rsx! {
        div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            div { class: "bg-white rounded-lg shadow-xl w-full max-w-md mx-4",
                div { class: "flex justify-between items-center p-4 border-b",
                    h3 { class: "text-lg font-semibold", "添加用户" }
                    button { class: "text-gray-400 hover:text-gray-600", onclick: move |_| on_close.call(()), "×" }
                }

                div { class: "p-4 space-y-4",
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "用户名" }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: username,
                            oninput: move |e| username.set(e.value()),
                        }
                    }
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "密码" }
                        input {
                            r#type: "password",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: password,
                            oninput: move |e| password.set(e.value()),
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
                            on_save.call(CreateUserPayload {
                                username: username.read().clone(),
                                password: password.read().clone(),
                                role: role.read().clone(),
                            });
                        },
                        "保存"
                    }
                }
            }
        }
    }
}

fn status_label(status: &str) -> &str {
    match status {
        "active" => "活跃",
        "locked" => "锁定",
        "disabled" => "禁用",
        _ => "未知",
    }
}
