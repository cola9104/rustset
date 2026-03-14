use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaPlus, FaPenToSquare, FaTrash, FaMagnifyingGlass, FaUser, FaShield, FaKey, FaUsers
};

/// 用户数据模型
#[derive(Clone, Debug, PartialEq)]
pub struct User {
    pub id: String,
    pub username: String,
    pub role: String,
    pub email: String,
    pub status: String,
    pub last_login: String,
    pub created_at: String,
}

/// 用户管理页面
#[allow(non_snake_case)]
pub fn UserManagement() -> Element {
    let mut users = use_signal(|| vec![
        User {
            id: "1".to_string(),
            username: "admin".to_string(),
            role: "系统管理员".to_string(),
            email: "admin@rustset.local".to_string(),
            status: "active".to_string(),
            last_login: "2024-01-15 10:30".to_string(),
            created_at: "2024-01-01 09:00".to_string(),
        },
        User {
            id: "2".to_string(),
            username: "sec_admin".to_string(),
            role: "安全管理员".to_string(),
            email: "sec@rustset.local".to_string(),
            status: "active".to_string(),
            last_login: "2024-01-15 09:15".to_string(),
            created_at: "2024-01-02 14:20".to_string(),
        },
        User {
            id: "3".to_string(),
            username: "auditor".to_string(),
            role: "审计员".to_string(),
            email: "audit@rustset.local".to_string(),
            status: "active".to_string(),
            last_login: "2024-01-14 16:45".to_string(),
            created_at: "2024-01-03 11:00".to_string(),
        },
        User {
            id: "4".to_string(),
            username: "operator".to_string(),
            role: "操作员".to_string(),
            email: "operator@rustset.local".to_string(),
            status: "locked".to_string(),
            last_login: "2024-01-10 08:20".to_string(),
            created_at: "2024-01-05 15:30".to_string(),
        },
    ]);

    let mut search_query = use_signal(String::new);
    let mut show_add_modal = use_signal(|| false);
    let mut editing_user = use_signal(|| None::<User>);

    // 计算统计数据
    let total_count = users.read().len() as i32;
    let active_count = users.read().iter().filter(|u| u.status == "active").count() as i32;
    let locked_count = users.read().iter().filter(|u| u.status == "locked").count() as i32;
    let admin_count = users.read().iter().filter(|u| u.role.contains("管理员")).count() as i32;

    // 过滤用户
    let filtered_users: Vec<User> = users.read()
        .iter()
        .filter(|user| {
            let query = search_query.read().to_lowercase();
            query.is_empty() ||
            user.username.to_lowercase().contains(&query) ||
            user.email.to_lowercase().contains(&query) ||
            user.role.contains(&query)
        })
        .cloned()
        .collect();

    let is_empty = filtered_users.is_empty();

    rsx! {
        div { class: "space-y-6",
            // 页面标题和操作栏
            div { class: "flex justify-between items-center",
                h1 { class: "text-2xl font-bold text-gray-800", "用户管理" }
                button {
                    class: "flex items-center px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors",
                    onclick: move |_| show_add_modal.set(true),
                    Icon { icon: FaPlus, width: 16, height: 16 }
                    span { class: "ml-2", "添加用户" }
                }
            }

            // 搜索栏
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

            // 统计卡片
            div { class: "grid grid-cols-1 md:grid-cols-4 gap-4",
                // 总用户
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
                // 活跃用户
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
                // 锁定用户
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
                // 管理员
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

            // 用户列表表格
            div { class: "bg-white rounded-lg shadow overflow-hidden",
                table { class: "min-w-full divide-y divide-gray-200",
                    thead { class: "bg-gray-50",
                        tr {
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "用户名" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "角色" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "邮箱" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "状态" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "最后登录" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "操作" }
                        }
                    }
                    tbody { class: "bg-white divide-y divide-gray-200",
                        for user in filtered_users {
                            tr { class: "hover:bg-gray-50",
                                td { class: "px-6 py-4 whitespace-nowrap",
                                    div { class: "flex items-center",
                                        div { class: "flex-shrink-0 h-10 w-10 bg-blue-100 rounded-full flex items-center justify-center",
                                            Icon { icon: FaUser, width: 20, height: 20, class: "text-blue-600" }
                                        }
                                        div { class: "ml-4",
                                            div { class: "text-sm font-medium text-gray-900", {user.username.clone()} }
                                            div { class: "text-sm text-gray-500", "ID: {user.id}" }
                                        }
                                    }
                                }
                                td { class: "px-6 py-4 whitespace-nowrap",
                                    span { class: "px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-purple-100 text-purple-800",
                                        {user.role.clone()}
                                    }
                                }
                                td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-500",
                                    {user.email.clone()}
                                }
                                td { class: "px-6 py-4 whitespace-nowrap",
                                    span {
                                        class: if user.status == "active" {
                                            "px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-green-100 text-green-800"
                                        } else {
                                            "px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-red-100 text-red-800"
                                        },
                                        if user.status == "active" { "活跃" } else { "锁定" }
                                    }
                                }
                                td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-500",
                                    {user.last_login.clone()}
                                }
                                td { class: "px-6 py-4 whitespace-nowrap text-sm font-medium",
                                    button {
                                        class: "text-blue-600 hover:text-blue-900 mr-3",
                                        onclick: {
                                            let user = user.clone();
                                            move |_| editing_user.set(Some(user.clone()))
                                        },
                                        Icon { icon: FaPenToSquare, width: 16, height: 16 }
                                    }
                                    button {
                                        class: "text-red-600 hover:text-red-900",
                                        onclick: {
                                            let user_id = user.id.clone();
                                            move |_| {
                                                let mut list = users.write();
                                                list.retain(|u| u.id != user_id);
                                            }
                                        },
                                        Icon { icon: FaTrash, width: 16, height: 16 }
                                    }
                                }
                            }
                        }
                    }
                }

                // 空状态
                if is_empty {
                    div { class: "text-center py-12 text-gray-500",
                        "没有找到匹配的用户"
                    }
                }
            }
        }

        // 添加用户模态框
        if *show_add_modal.read() {
            AddUserModal {
                on_close: move |_| show_add_modal.set(false),
                on_save: move |new_user: User| {
                    users.write().push(new_user);
                    show_add_modal.set(false);
                }
            }
        }

        // 编辑用户模态框
        if let Some(user) = editing_user.read().clone() {
            EditUserModal {
                user: user.clone(),
                on_close: move |_| editing_user.set(None),
                on_save: {
                    let mut editing_user_signal = editing_user;
                    move |updated: User| {
                        let mut list = users.write();
                        if let Some(u) = list.iter_mut().find(|u| u.id == updated.id) {
                            *u = updated;
                        }
                        editing_user_signal.set(None);
                    }
                }
            }
        }
    }
}

/// 添加用户模态框
#[component]
fn AddUserModal(on_close: EventHandler<()>, on_save: EventHandler<User>) -> Element {
    let mut username = use_signal(String::new);
    let mut role = use_signal(|| "操作员".to_string());
    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut status = use_signal(|| "active".to_string());

    rsx! {
        div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            div { class: "bg-white rounded-lg shadow-xl w-full max-w-md mx-4",
                div { class: "flex justify-between items-center p-4 border-b",
                    h3 { class: "text-lg font-semibold", "添加用户" }
                    button {
                        class: "text-gray-400 hover:text-gray-600",
                        onclick: move |_| on_close.call(()),
                        "×"
                    }
                }

                div { class: "p-4 space-y-4",
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "用户名" }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            placeholder: "输入用户名",
                            value: username,
                            oninput: move |e| username.set(e.value()),
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "角色" }
                        select {
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: role,
                            onchange: move |e| role.set(e.value()),
                            option { value: "系统管理员", "系统管理员" }
                            option { value: "安全管理员", "安全管理员" }
                            option { value: "审计员", "审计员" }
                            option { value: "操作员", "操作员" }
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "邮箱" }
                        input {
                            r#type: "email",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            placeholder: "user@example.com",
                            value: email,
                            oninput: move |e| email.set(e.value()),
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "密码" }
                        input {
                            r#type: "password",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            placeholder: "输入密码",
                            value: password,
                            oninput: move |e| password.set(e.value()),
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "状态" }
                        select {
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: status,
                            onchange: move |e| status.set(e.value()),
                            option { value: "active", "活跃" }
                            option { value: "locked", "锁定" }
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
                            let new_user = User {
                                id: uuid::Uuid::new_v4().to_string(),
                                username: username.read().clone(),
                                role: role.read().clone(),
                                email: email.read().clone(),
                                status: status.read().clone(),
                                last_login: "从未登录".to_string(),
                                created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M").to_string(),
                            };
                            on_save.call(new_user);
                        },
                        "创建"
                    }
                }
            }
        }
    }
}

/// 编辑用户模态框
#[component]
fn EditUserModal(user: User, on_close: EventHandler<()>, on_save: EventHandler<User>) -> Element {
    let mut username = use_signal(|| user.username.clone());
    let mut role = use_signal(|| user.role.clone());
    let mut email = use_signal(|| user.email.clone());
    let mut status = use_signal(|| user.status.clone());

    rsx! {
        div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            div { class: "bg-white rounded-lg shadow-xl w-full max-w-md mx-4",
                div { class: "flex justify-between items-center p-4 border-b",
                    h3 { class: "text-lg font-semibold", "编辑用户" }
                    button {
                        class: "text-gray-400 hover:text-gray-600",
                        onclick: move |_| on_close.call(()),
                        "×"
                    }
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
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "角色" }
                        select {
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: role,
                            onchange: move |e| role.set(e.value()),
                            option { value: "系统管理员", "系统管理员" }
                            option { value: "安全管理员", "安全管理员" }
                            option { value: "审计员", "审计员" }
                            option { value: "操作员", "操作员" }
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "邮箱" }
                        input {
                            r#type: "email",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: email,
                            oninput: move |e| email.set(e.value()),
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "状态" }
                        select {
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: status,
                            onchange: move |e| status.set(e.value()),
                            option { value: "active", "活跃" }
                            option { value: "locked", "锁定" }
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
                        onclick: {
                            let user_id = user.id.clone();
                            let last_login = user.last_login.clone();
                            let created_at = user.created_at.clone();
                            move |_| {
                                let updated = User {
                                    id: user_id.clone(),
                                    username: username.read().clone(),
                                    role: role.read().clone(),
                                    email: email.read().clone(),
                                    status: status.read().clone(),
                                    last_login: last_login.clone(),
                                    created_at: created_at.clone(),
                                };
                                on_save.call(updated);
                            }
                        },
                        "保存"
                    }
                }
            }
        }
    }
}
