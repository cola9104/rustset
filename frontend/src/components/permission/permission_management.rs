use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaPlus, FaPenToSquare, FaTrash, FaMagnifyingGlass, FaShield, FaKey, FaUserGear, FaUsers, FaServer, FaTriangleExclamation
};

/// 权限数据模型
#[derive(Clone, Debug, PartialEq)]
pub struct Permission {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub category: String,
    pub users_count: i32,
    pub created_at: String,
}

/// 角色数据模型
#[derive(Clone, Debug, PartialEq)]
pub struct Role {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub permissions: Vec<String>, // 格式: "模块:操作" 如 "用户管理:查看"
    pub users_count: i32,
    pub created_at: String,
}

/// 模块权限组
#[derive(Clone, Debug)]
struct ModulePermissionGroup {
    module: String,
    permissions: Vec<(String, String, String)>, // (id, action, description)
}

/// 获取所有模块权限定义（按模块分组）
fn get_module_permission_groups() -> Vec<ModulePermissionGroup> {
    vec![
        ModulePermissionGroup {
            module: "用户管理".to_string(),
            permissions: vec![
                ("user:view".to_string(), "查看".to_string(), "查看用户列表".to_string()),
                ("user:add".to_string(), "添加".to_string(), "添加新用户".to_string()),
                ("user:edit".to_string(), "编辑".to_string(), "编辑用户信息".to_string()),
                ("user:delete".to_string(), "删除".to_string(), "删除用户".to_string()),
                ("user:reset".to_string(), "重置密码".to_string(), "重置用户密码".to_string()),
            ],
        },
        ModulePermissionGroup {
            module: "角色管理".to_string(),
            permissions: vec![
                ("role:view".to_string(), "查看".to_string(), "查看角色列表".to_string()),
                ("role:add".to_string(), "添加".to_string(), "添加新角色".to_string()),
                ("role:edit".to_string(), "编辑".to_string(), "编辑角色信息".to_string()),
                ("role:delete".to_string(), "删除".to_string(), "删除角色".to_string()),
                ("role:assign".to_string(), "分配权限".to_string(), "为角色分配权限".to_string()),
            ],
        },
        ModulePermissionGroup {
            module: "资产管理".to_string(),
            permissions: vec![
                ("asset:view".to_string(), "查看".to_string(), "查看资产列表".to_string()),
                ("asset:add".to_string(), "添加".to_string(), "添加新资产".to_string()),
                ("asset:edit".to_string(), "编辑".to_string(), "编辑资产信息".to_string()),
                ("asset:delete".to_string(), "删除".to_string(), "删除资产".to_string()),
                ("asset:import".to_string(), "导入导出".to_string(), "导入导出资产数据".to_string()),
            ],
        },
        ModulePermissionGroup {
            module: "服务商管理".to_string(),
            permissions: vec![
                ("provider:view".to_string(), "查看".to_string(), "查看服务商列表".to_string()),
                ("provider:add".to_string(), "添加".to_string(), "添加服务商".to_string()),
                ("provider:edit".to_string(), "编辑".to_string(), "编辑服务商信息".to_string()),
                ("provider:delete".to_string(), "删除".to_string(), "删除服务商".to_string()),
            ],
        },
        ModulePermissionGroup {
            module: "云平台管理".to_string(),
            permissions: vec![
                ("cloud:view".to_string(), "查看".to_string(), "查看云平台列表".to_string()),
                ("cloud:add".to_string(), "添加".to_string(), "添加云平台".to_string()),
                ("cloud:edit".to_string(), "编辑".to_string(), "编辑云平台信息".to_string()),
                ("cloud:delete".to_string(), "删除".to_string(), "删除云平台".to_string()),
            ],
        },
        ModulePermissionGroup {
            module: "机房管理".to_string(),
            permissions: vec![
                ("room:view".to_string(), "查看".to_string(), "查看机房列表".to_string()),
                ("room:add".to_string(), "添加".to_string(), "添加机房".to_string()),
                ("room:edit".to_string(), "编辑".to_string(), "编辑机房信息".to_string()),
                ("room:delete".to_string(), "删除".to_string(), "删除机房".to_string()),
            ],
        },
        ModulePermissionGroup {
            module: "工单管理".to_string(),
            permissions: vec![
                ("ticket:view".to_string(), "查看".to_string(), "查看工单列表".to_string()),
                ("ticket:submit".to_string(), "提交".to_string(), "提交资源申请".to_string()),
                ("ticket:approve".to_string(), "审批".to_string(), "审批工单".to_string()),
                ("ticket:provision".to_string(), "配置".to_string(), "配置资源".to_string()),
                ("ticket:deliver".to_string(), "交付".to_string(), "交付资源".to_string()),
            ],
        },
        ModulePermissionGroup {
            module: "扫描任务".to_string(),
            permissions: vec![
                ("scan:view".to_string(), "查看".to_string(), "查看扫描任务".to_string()),
                ("scan:create".to_string(), "创建".to_string(), "创建扫描任务".to_string()),
                ("scan:execute".to_string(), "执行".to_string(), "执行扫描任务".to_string()),
                ("scan:delete".to_string(), "删除".to_string(), "删除扫描任务".to_string()),
            ],
        },
        ModulePermissionGroup {
            module: "风险管理".to_string(),
            permissions: vec![
                ("risk:view".to_string(), "查看".to_string(), "查看风险列表".to_string()),
                ("risk:handle".to_string(), "处理".to_string(), "处理风险".to_string()),
                ("risk:export".to_string(), "导出报告".to_string(), "导出风险报告".to_string()),
            ],
        },
        ModulePermissionGroup {
            module: "审计日志".to_string(),
            permissions: vec![
                ("audit:view".to_string(), "查看".to_string(), "查看审计日志".to_string()),
                ("audit:export".to_string(), "导出".to_string(), "导出审计日志".to_string()),
            ],
        },
        ModulePermissionGroup {
            module: "系统配置".to_string(),
            permissions: vec![
                ("config:view".to_string(), "查看".to_string(), "查看系统配置".to_string()),
                ("config:modify".to_string(), "修改".to_string(), "修改系统配置".to_string()),
                ("config:security".to_string(), "安全策略".to_string(), "配置安全策略".to_string()),
            ],
        },
    ]
}

/// 权限管理页面
#[allow(non_snake_case)]
pub fn PermissionManagement() -> Element {
    let mut active_tab = use_signal(|| "permissions".to_string());

    let mut permissions = use_signal(|| vec![
        Permission { id: 1, name: "用户管理".to_string(), description: "管理系统用户的增删改查".to_string(), category: "用户管理".to_string(), users_count: 2, created_at: "2024-01-01".to_string() },
        Permission { id: 2, name: "角色管理".to_string(), description: "管理系统角色和权限分配".to_string(), category: "用户管理".to_string(), users_count: 1, created_at: "2024-01-01".to_string() },
        Permission { id: 3, name: "资产管理".to_string(), description: "管理硬件和云服务资产".to_string(), category: "资产管理".to_string(), users_count: 3, created_at: "2024-01-01".to_string() },
        Permission { id: 4, name: "工单管理".to_string(), description: "管理资源申请和交付".to_string(), category: "工单管理".to_string(), users_count: 2, created_at: "2024-01-01".to_string() },
        Permission { id: 5, name: "扫描任务".to_string(), description: "执行端口和漏洞扫描".to_string(), category: "扫描管理".to_string(), users_count: 2, created_at: "2024-01-01".to_string() },
        Permission { id: 6, name: "风险管理".to_string(), description: "查看安全风险和漏洞".to_string(), category: "风险管理".to_string(), users_count: 4, created_at: "2024-01-01".to_string() },
        Permission { id: 7, name: "审计日志".to_string(), description: "查看系统操作日志".to_string(), category: "系统管理".to_string(), users_count: 3, created_at: "2024-01-01".to_string() },
    ]);

    let mut roles = use_signal(|| vec![
        Role {
            id: 1,
            name: "系统管理员".to_string(),
            description: "拥有所有权限".to_string(),
            permissions: vec![
                "用户管理:查看".to_string(), "用户管理:添加".to_string(), "用户管理:编辑".to_string(), "用户管理:删除".to_string(),
                "角色管理:查看".to_string(), "角色管理:添加".to_string(), "角色管理:编辑".to_string(), "角色管理:删除".to_string(),
                "资产管理:查看".to_string(), "资产管理:添加".to_string(), "资产管理:编辑".to_string(), "资产管理:删除".to_string(),
            ],
            users_count: 1,
            created_at: "2024-01-01".to_string(),
        },
        Role {
            id: 2,
            name: "安全管理员".to_string(),
            description: "负责安全扫描和风险处理".to_string(),
            permissions: vec![
                "资产管理:查看".to_string(), "扫描任务:查看".to_string(), "扫描任务:创建".to_string(),
                "风险管理:查看".to_string(), "风险管理:处理".to_string(), "审计日志:查看".to_string(),
            ],
            users_count: 1,
            created_at: "2024-01-01".to_string(),
        },
        Role {
            id: 3,
            name: "审计员".to_string(),
            description: "查看审计日志和系统状态".to_string(),
            permissions: vec!["审计日志:查看".to_string(), "审计日志:导出".to_string()],
            users_count: 1,
            created_at: "2024-01-01".to_string(),
        },
        Role {
            id: 4,
            name: "操作员".to_string(),
            description: "基础查看权限".to_string(),
            permissions: vec!["资产管理:查看".to_string(), "工单管理:查看".to_string()],
            users_count: 1,
            created_at: "2024-01-01".to_string(),
        },
    ]);

    let mut show_add_modal = use_signal(|| false);
    let mut editing_permission = use_signal(|| None::<Permission>);
    let mut editing_role = use_signal(|| None::<Role>);
    let mut delete_confirm = use_signal(|| None::<(String, i32)>);

    let is_permissions = *active_tab.read() == "permissions";

    rsx! {
        div { class: "space-y-6",
            div { class: "flex justify-between items-center",
                h1 { class: "text-2xl font-bold text-gray-800", "权限管理" }
            }

            div { class: "bg-white rounded-lg shadow p-1 inline-flex",
                div { class: "flex space-x-1",
                    button {
                        class: format!("flex items-center px-6 py-2.5 rounded-md text-sm font-medium transition-all duration-200 {}",
                            if is_permissions { "bg-blue-600 text-white shadow-md" } else { "text-gray-600 hover:bg-gray-100" }
                        ),
                        onclick: move |_| active_tab.set("permissions".to_string()),
                        Icon { icon: FaKey, width: 16, height: 16, class: "mr-2" }
                        "权限"
                    }
                    button {
                        class: format!("flex items-center px-6 py-2.5 rounded-md text-sm font-medium transition-all duration-200 {}",
                            if !is_permissions { "bg-blue-600 text-white shadow-md" } else { "text-gray-600 hover:bg-gray-100" }
                        ),
                        onclick: move |_| active_tab.set("roles".to_string()),
                        Icon { icon: FaUserGear, width: 16, height: 16, class: "mr-2" }
                        "角色"
                    }
                }
            }

            if is_permissions {
                PermissionsTab {
                    permissions: permissions,
                    on_add: move |_| show_add_modal.set(true),
                    on_edit: move |id| {
                        let perm = permissions.read().iter().find(|p| p.id == id).cloned();
                        editing_permission.set(perm);
                    },
                    on_delete: move |id| delete_confirm.set(Some(("permission".to_string(), id)))
                }
            } else {
                RolesTab {
                    roles: roles,
                    on_add: move |_| show_add_modal.set(true),
                    on_edit: move |id| {
                        let role = roles.read().iter().find(|r| r.id == id).cloned();
                        editing_role.set(role);
                    },
                    on_delete: move |id| delete_confirm.set(Some(("role".to_string(), id)))
                }
            }

            if *show_add_modal.read() && *active_tab.read() == "permissions" {
                AddPermissionModal {
                    on_close: move |_| show_add_modal.set(false),
                    on_save: move |new_perm: Permission| {
                        permissions.write().push(new_perm);
                        show_add_modal.set(false);
                    }
                }
            }

            if let Some(perm) = editing_permission.read().clone() {
                EditPermissionModal {
                    permission: perm,
                    on_close: move |_| editing_permission.set(None),
                    on_save: move |updated: Permission| {
                        let mut perms = permissions.write();
                        if let Some(p) = perms.iter_mut().find(|p| p.id == updated.id) { *p = updated; }
                        editing_permission.set(None);
                    }
                }
            }

            if *show_add_modal.read() && *active_tab.read() == "roles" {
                AddRoleModal {
                    on_close: move |_| show_add_modal.set(false),
                    on_save: move |new_role: Role| {
                        roles.write().push(new_role);
                        show_add_modal.set(false);
                    }
                }
            }

            if let Some(role) = editing_role.read().clone() {
                EditRoleModal {
                    role: role,
                    on_close: move |_| editing_role.set(None),
                    on_save: move |updated: Role| {
                        let mut roles_list = roles.write();
                        if let Some(r) = roles_list.iter_mut().find(|r| r.id == updated.id) { *r = updated; }
                        editing_role.set(None);
                    }
                }
            }

            {delete_confirm_dialog_component(delete_confirm, permissions, roles)}
        }
    }
}

fn delete_confirm_dialog_component(
    mut delete_confirm: Signal<Option<(String, i32)>>,
    mut permissions: Signal<Vec<Permission>>,
    mut roles: Signal<Vec<Role>>,
) -> Element {
    if let Some(data) = delete_confirm.read().as_ref() {
        let item_type = data.0.clone();
        let id = data.1;
        rsx! {
            DeleteConfirmDialog {
                item_type: item_type.clone(),
                id,
                on_close: move |_| delete_confirm.set(None),
                on_confirm: move || {
                    if item_type == "permission" {
                        permissions.write().retain(|p| p.id != id);
                    } else {
                        roles.write().retain(|r| r.id != id);
                    }
                    delete_confirm.set(None);
                }
            }
        }
    } else {
        rsx! { {""} }
    }
}

#[component]
fn PermissionsTab(
    permissions: Signal<Vec<Permission>>,
    on_add: EventHandler<()>,
    on_edit: EventHandler<i32>,
    on_delete: EventHandler<i32>,
) -> Element {
    let mut search_query = use_signal(String::new);
    let total_count = permissions.read().len() as i32;

    let filtered_permissions: Vec<Permission> = permissions.read()
        .iter()
        .filter(|perm| {
            let query = search_query.read().to_lowercase();
            query.is_empty() || perm.name.to_lowercase().contains(&query) || perm.category.to_lowercase().contains(&query)
        })
        .cloned()
        .collect();

    rsx! {
        div { class: "space-y-6",
            div { class: "flex justify-between items-center bg-white rounded-lg shadow p-4",
                div { class: "flex items-center flex-1",
                    Icon { icon: FaMagnifyingGlass, width: 20, height: 20, class: "text-gray-400" }
                    input {
                        r#type: "text",
                        class: "ml-2 flex-1 border-0 focus:outline-none",
                        placeholder: "搜索权限...",
                        value: search_query,
                        oninput: move |e| search_query.set(e.value()),
                    }
                }
                button {
                    class: "flex items-center px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700",
                    onclick: move |_| on_add.call(()),
                    Icon { icon: FaPlus, width: 16, height: 16 }
                    span { class: "ml-2", "添加权限" }
                }
            }

            div { class: "grid grid-cols-1 md:grid-cols-4 gap-4",
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-blue-500", Icon { icon: FaKey, width: 20, height: 20, class: "text-white" } }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "总权限" }
                            p { class: "text-xl font-bold text-gray-800", {total_count.to_string()} }
                        }
                    }
                }
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-purple-500", Icon { icon: FaUsers, width: 20, height: 20, class: "text-white" } }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "用户管理" }
                            p { class: "text-xl font-bold text-gray-800", {permissions.read().iter().filter(|p| p.category == "用户管理").count().to_string()} }
                        }
                    }
                }
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-green-500", Icon { icon: FaServer, width: 20, height: 20, class: "text-white" } }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "资产管理" }
                            p { class: "text-xl font-bold text-gray-800", {permissions.read().iter().filter(|p| p.category == "资产管理").count().to_string()} }
                        }
                    }
                }
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-orange-500", Icon { icon: FaShield, width: 20, height: 20, class: "text-white" } }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "扫描管理" }
                            p { class: "text-xl font-bold text-gray-800", {permissions.read().iter().filter(|p| p.category == "扫描管理").count().to_string()} }
                        }
                    }
                }
            }

            div { class: "bg-white rounded-lg shadow overflow-hidden",
                table { class: "min-w-full divide-y divide-gray-200",
                    thead { class: "bg-gray-50",
                        tr {
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase", "权限名称" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase", "分类" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase", "关联用户" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase", "创建时间" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase", "操作" }
                        }
                    }
                    tbody { class: "bg-white divide-y divide-gray-200",
                        for perm in filtered_permissions {
                            tr { class: "hover:bg-gray-50",
                                td { class: "px-6 py-4",
                                    div { class: "flex items-center",
                                        Icon { icon: FaKey, width: 20, height: 20, class: "text-blue-500 mr-3" }
                                        div {
                                            div { class: "text-sm font-medium text-gray-900", {perm.name.clone()} }
                                            div { class: "text-sm text-gray-500", {perm.description.clone()} }
                                        }
                                    }
                                }
                                td { class: "px-6 py-4",
                                    span { class: "px-2 py-1 text-xs bg-gray-100 text-gray-800 rounded-full", {perm.category.clone()} }
                                }
                                td { class: "px-6 py-4 text-sm text-gray-500", {perm.users_count.to_string()} }
                                td { class: "px-6 py-4 text-sm text-gray-500", {perm.created_at.clone()} }
                                td { class: "px-6 py-4 text-sm font-medium",
                                    button { class: "text-blue-600 hover:text-blue-900 mr-3", onclick: move |_| on_edit.call(perm.id), Icon { icon: FaPenToSquare, width: 16, height: 16 } }
                                    button { class: "text-red-600 hover:text-red-900", onclick: move |_| on_delete.call(perm.id), Icon { icon: FaTrash, width: 16, height: 16 } }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn RolesTab(
    roles: Signal<Vec<Role>>,
    on_add: EventHandler<()>,
    on_edit: EventHandler<i32>,
    on_delete: EventHandler<i32>,
) -> Element {
    let mut search_query = use_signal(String::new);
    let total_count = roles.read().len() as i32;

    let filtered_roles: Vec<Role> = roles.read()
        .iter()
        .filter(|role| {
            let query = search_query.read().to_lowercase();
            query.is_empty() || role.name.to_lowercase().contains(&query)
        })
        .cloned()
        .collect();

    rsx! {
        div { class: "space-y-6",
            div { class: "flex justify-between items-center bg-white rounded-lg shadow p-4",
                div { class: "flex items-center flex-1",
                    Icon { icon: FaMagnifyingGlass, width: 20, height: 20, class: "text-gray-400" }
                    input {
                        r#type: "text",
                        class: "ml-2 flex-1 border-0 focus:outline-none",
                        placeholder: "搜索角色...",
                        value: search_query,
                        oninput: move |e| search_query.set(e.value()),
                    }
                }
                button {
                    class: "flex items-center px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700",
                    onclick: move |_| on_add.call(()),
                    Icon { icon: FaPlus, width: 16, height: 16 }
                    span { class: "ml-2", "添加角色" }
                }
            }

            div { class: "bg-white rounded-lg shadow p-4",
                div { class: "flex items-center",
                    div { class: "p-2 rounded-full bg-purple-500", Icon { icon: FaUserGear, width: 20, height: 20, class: "text-white" } }
                    div { class: "ml-3",
                        p { class: "text-sm text-gray-500", "总角色数" }
                        p { class: "text-xl font-bold text-gray-800", {total_count.to_string()} }
                    }
                }
            }

            div { class: "grid grid-cols-1 lg:grid-cols-2 gap-6",
                for role in filtered_roles {
                    div { class: "bg-white rounded-lg shadow p-6 hover:shadow-lg transition-shadow",
                        div { class: "flex items-start justify-between mb-4",
                            div { class: "flex items-center",
                                div { class: "p-3 rounded-full bg-purple-100", Icon { icon: FaUserGear, width: 24, height: 24, class: "text-purple-600" } }
                                div { class: "ml-4",
                                    h3 { class: "text-lg font-semibold text-gray-900", {role.name.clone()} }
                                    p { class: "text-sm text-gray-500", {role.description.clone()} }
                                }
                            }
                            div { class: "flex space-x-2",
                                button { class: "text-blue-600 hover:text-blue-900", onclick: move |_| on_edit.call(role.id), Icon { icon: FaPenToSquare, width: 16, height: 16 } }
                                button { class: "text-red-600 hover:text-red-900", onclick: move |_| on_delete.call(role.id), Icon { icon: FaTrash, width: 16, height: 16 } }
                            }
                        }

                        div { class: "space-y-3",
                            p { class: "text-xs font-medium text-gray-500 uppercase", "权限配置" }
                            {render_role_permissions_grouped(role.permissions.clone())}
                        }

                        div { class: "flex items-center justify-between pt-4 mt-4 border-t border-gray-100",
                            div { class: "text-sm text-gray-500",
                                Icon { icon: FaUsers, width: 16, height: 16, class: "text-gray-400 mr-1" }
                                span { "{role.users_count} 位用户" }
                            }
                            div { class: "text-xs text-gray-400", {role.created_at.clone()} }
                        }
                    }
                }
            }
        }
    }
}

fn render_role_permissions_grouped(permissions: Vec<String>) -> Element {
    let mut grouped: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    for perm in permissions {
        let parts: Vec<&str> = perm.split(':').collect();
        if parts.len() == 2 {
            grouped.entry(parts[0].to_string()).or_insert_with(Vec::new).push(parts[1].to_string());
        }
    }
    let modules: Vec<String> = grouped.keys().cloned().collect();

    rsx! {
        for module in modules {
            div { class: "border border-gray-200 rounded-lg p-3",
                div { class: "flex items-center justify-between mb-2",
                    h4 { class: "text-sm font-medium text-gray-700", {module.clone()} }
                    span { class: "text-xs text-gray-500", "{grouped.get(&module).map(|v| v.len()).unwrap_or(0)} 项" }
                }
                div { class: "flex flex-wrap gap-1",
                    for action in grouped.get(&module).unwrap_or(&vec![]) {
                        span { class: "px-2 py-0.5 text-xs bg-blue-50 text-blue-700 rounded", {action.clone()} }
                    }
                }
            }
        }
    }
}

#[component]
fn AddPermissionModal(on_close: EventHandler<()>, on_save: EventHandler<Permission>) -> Element {
    let mut name = use_signal(String::new);
    let mut description = use_signal(String::new);
    let mut category = use_signal(|| "用户管理".to_string());

    rsx! {
        div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            div { class: "bg-white rounded-lg shadow-xl w-full max-w-md mx-4",
                div { class: "flex justify-between items-center p-4 border-b",
                    h3 { class: "text-lg font-semibold", "添加权限" }
                    button { class: "text-gray-400 hover:text-gray-600", onclick: move |_| on_close.call(()), "×" }
                }
                div { class: "p-4 space-y-4",
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "权限名称" }
                        input { r#type: "text", class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500", placeholder: "如：用户管理", value: name, oninput: move |e| name.set(e.value()) }
                    }
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "权限分类" }
                        select { class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500", value: "{category}", onchange: move |e| category.set(e.value()),
                            option { value: "用户管理", "用户管理" } option { value: "资产管理", "资产管理" }
                            option { value: "扫描管理", "扫描管理" } option { value: "风险管理", "风险管理" }
                            option { value: "系统管理", "系统管理" } option { value: "工单管理", "工单管理" }
                        }
                    }
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "权限描述" }
                        textarea { class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500", rows: 3, placeholder: "描述该权限的作用范围", value: description, oninput: move |e| description.set(e.value()) }
                    }
                }
                div { class: "flex justify-end space-x-3 p-4 border-t",
                    button { class: "px-4 py-2 text-gray-600 border border-gray-300 rounded-md hover:bg-gray-50", onclick: move |_| on_close.call(()), "取消" }
                    button { class: "px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700", onclick: move |_| {
                        on_save.call(Permission { id: (uuid::Uuid::new_v4().as_u128() & 0xFFFFFFFF) as i32, name: name.read().clone(), description: description.read().clone(), category: category.read().clone(), users_count: 0, created_at: chrono::Utc::now().format("%Y-%m-%d").to_string() });
                    }, "创建" }
                }
            }
        }
    }
}

#[component]
fn EditPermissionModal(permission: Permission, on_close: EventHandler<()>, on_save: EventHandler<Permission>) -> Element {
    let mut name = use_signal(|| permission.name.clone());
    let mut description = use_signal(|| permission.description.clone());
    let mut category = use_signal(|| permission.category.clone());

    rsx! {
        div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            div { class: "bg-white rounded-lg shadow-xl w-full max-w-md mx-4",
                div { class: "flex justify-between items-center p-4 border-b",
                    h3 { class: "text-lg font-semibold", "编辑权限" }
                    button { class: "text-gray-400 hover:text-gray-600", onclick: move |_| on_close.call(()), "×" }
                }
                div { class: "p-4 space-y-4",
                    div { label { class: "block text-sm font-medium text-gray-700 mb-1", "权限名称" } input { r#type: "text", class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500", value: name, oninput: move |e| name.set(e.value()) } }
                    div { label { class: "block text-sm font-medium text-gray-700 mb-1", "权限分类" } select { class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500", value: "{category}", onchange: move |e| category.set(e.value()),
                        option { value: "用户管理", "用户管理" } option { value: "资产管理", "资产管理" }
                        option { value: "扫描管理", "扫描管理" } option { value: "风险管理", "风险管理" }
                        option { value: "系统管理", "系统管理" } option { value: "工单管理", "工单管理" }
                    } }
                    div { label { class: "block text-sm font-medium text-gray-700 mb-1", "权限描述" } textarea { class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500", rows: 3, value: description, oninput: move |e| description.set(e.value()) } }
                }
                div { class: "flex justify-end space-x-3 p-4 border-t",
                    button { class: "px-4 py-2 text-gray-600 border border-gray-300 rounded-md hover:bg-gray-50", onclick: move |_| on_close.call(()), "取消" }
                    button { class: "px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700", onclick: move |_| {
                        on_save.call(Permission { id: permission.id, name: name.read().clone(), description: description.read().clone(), category: category.read().clone(), users_count: permission.users_count, created_at: permission.created_at.clone() });
                    }, "保存" }
                }
            }
        }
    }
}

#[component]
fn AddRoleModal(on_close: EventHandler<()>, on_save: EventHandler<Role>) -> Element {
    let mut name = use_signal(String::new);
    let mut description = use_signal(String::new);
    let mut selected_permissions = use_signal(|| Vec::<String>::new());
    let module_groups = get_module_permission_groups();

    rsx! {
        div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            div { class: "bg-white rounded-lg shadow-xl w-full max-w-4xl mx-4 max-h-[90vh] overflow-y-auto",
                div { class: "flex justify-between items-center p-4 border-b",
                    h3 { class: "text-lg font-semibold", "添加角色" }
                    button { class: "text-gray-400 hover:text-gray-600", onclick: move |_| on_close.call(()), "×" }
                }
                div { class: "p-4 space-y-4",
                    div { class: "grid grid-cols-2 gap-4",
                        div { label { class: "block text-sm font-medium text-gray-700 mb-1", "角色名称" } input { r#type: "text", class: "w-full px-3 py-2 border border-gray-300 rounded-md", placeholder: "如：审计员", value: name, oninput: move |e| name.set(e.value()) } }
                        div { label { class: "block text-sm font-medium text-gray-700 mb-1", "角色描述" } input { r#type: "text", class: "w-full px-3 py-2 border border-gray-300 rounded-md", placeholder: "描述职责", value: description, oninput: move |e| description.set(e.value()) } }
                    }

                    div { class: "border-t border-gray-200 pt-4" },
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-2", "选择权限（按模块分组）" }
                        div { class: "space-y-4 max-h-96 overflow-y-auto",
                            for group in module_groups.iter() {
                                {render_module_permission_group(group.module.clone(), group.permissions.clone(), selected_permissions.clone())}
                            }
                        }
                    }
                }
                div { class: "flex justify-between items-center p-4 border-t bg-gray-50",
                    div { class: "text-sm text-gray-600", "已选择 {selected_permissions.read().len()} 项权限" }
                    div { class: "flex space-x-3",
                        button { class: "px-4 py-2 text-gray-600 border border-gray-300 rounded-md hover:bg-gray-50", onclick: move |_| on_close.call(()), "取消" }
                        button { class: "px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700", onclick: move |_| {
                            on_save.call(Role { id: (uuid::Uuid::new_v4().as_u128() & 0xFFFFFFFF) as i32, name: name.read().clone(), description: description.read().clone(), permissions: selected_permissions.read().clone(), users_count: 0, created_at: chrono::Utc::now().format("%Y-%m-%d").to_string() });
                        }, "创建" }
                    }
                }
            }
        }
    }
}

#[component]
fn EditRoleModal(role: Role, on_close: EventHandler<()>, on_save: EventHandler<Role>) -> Element {
    let mut name = use_signal(|| role.name.clone());
    let mut description = use_signal(|| role.description.clone());
    let mut selected_permissions = use_signal(|| role.permissions.clone());
    let module_groups = get_module_permission_groups();

    rsx! {
        div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            div { class: "bg-white rounded-lg shadow-xl w-full max-w-4xl mx-4 max-h-[90vh] overflow-y-auto",
                div { class: "flex justify-between items-center p-4 border-b",
                    h3 { class: "text-lg font-semibold", "编辑角色" }
                    button { class: "text-gray-400 hover:text-gray-600", onclick: move |_| on_close.call(()), "×" }
                }
                div { class: "p-4 space-y-4",
                    div { class: "grid grid-cols-2 gap-4",
                        div { label { class: "block text-sm font-medium text-gray-700 mb-1", "角色名称" } input { r#type: "text", class: "w-full px-3 py-2 border border-gray-300 rounded-md", value: name, oninput: move |e| name.set(e.value()) } }
                        div { label { class: "block text-sm font-medium text-gray-700 mb-1", "角色描述" } input { r#type: "text", class: "w-full px-3 py-2 border border-gray-300 rounded-md", value: description, oninput: move |e| description.set(e.value()) } }
                    }

                    div { class: "border-t border-gray-200 pt-4" },
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-2", "选择权限（按模块分组）" }
                        div { class: "space-y-4 max-h-96 overflow-y-auto",
                            for group in module_groups.iter() {
                                {render_module_permission_group(group.module.clone(), group.permissions.clone(), selected_permissions.clone())}
                            }
                        }
                    }
                }
                div { class: "flex justify-between items-center p-4 border-t bg-gray-50",
                    div { class: "text-sm text-gray-600", "已选择 {selected_permissions.read().len()} 项权限" }
                    div { class: "flex space-x-3",
                        button { class: "px-4 py-2 text-gray-600 border border-gray-300 rounded-md hover:bg-gray-50", onclick: move |_| on_close.call(()), "取消" }
                        button { class: "px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700", onclick: move |_| {
                            on_save.call(Role { id: role.id, name: name.read().clone(), description: description.read().clone(), permissions: selected_permissions.read().clone(), users_count: role.users_count, created_at: role.created_at.clone() });
                        }, "保存" }
                    }
                }
            }
        }
    }
}

fn render_module_permission_group(module: String, permissions: Vec<(String, String, String)>, mut selected_permissions: Signal<Vec<String>>) -> Element {
    // Clone for use in closures
    let permissions_for_all = permissions.clone();
    let module_for_all = module.clone();
    let perms_count = permissions.len();

    rsx! {
        div { class: "border border-gray-200 rounded-lg p-3",
            div { class: "flex items-center justify-between mb-2",
                h4 { class: "text-sm font-semibold text-gray-800", {module.clone()} }
                label { class: "flex items-center text-xs text-gray-600",
                    input {
                        r#type: "checkbox",
                        class: "h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded mr-1",
                        checked: {
                            let selected_count = selected_permissions.read().iter().filter(|p| p.starts_with(&(format!("{}:", module.clone()) + ":"))).count();
                            perms_count > 0 && selected_count == perms_count
                        },
                        onchange: move |e| {
                            let module = module_for_all.clone();
                            if e.checked() {
                                for perm in &permissions_for_all {
                                    let perm_key = format!("{}:{}", module, perm.1);
                                    selected_permissions.with_mut(|v| { if !v.contains(&perm_key) { v.push(perm_key); } });
                                }
                            } else {
                                selected_permissions.with_mut(|v| { v.retain(|p| !p.starts_with(&format!("{}:", module))); });
                            }
                        },
                    }
                    span { "全选" }
                }
            }
            div { class: "grid grid-cols-2 md:grid-cols-3 gap-2",
                for perm in permissions {
                    div { class: "flex flex-col p-2 border border-gray-200 rounded hover:bg-gray-50",
                        div { class: "flex items-center",
                            input {
                                r#type: "checkbox",
                                class: "h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded",
                                checked: selected_permissions.read().contains(&format!("{}:{}", module, perm.1)),
                                onchange: {
                                    let perm_key = format!("{}:{}", module.clone(), perm.1.clone());
                                    move |e| {
                                        let key = perm_key.clone();
                                        if e.checked() {
                                            selected_permissions.with_mut(|v| { if !v.contains(&key) { v.push(key); } });
                                        } else {
                                            selected_permissions.with_mut(|v| { v.retain(|p| p != &key); });
                                        }
                                    }
                                },
                            }
                            label { class: "ml-2 text-xs font-medium text-gray-700", {perm.1.clone()} }
                        }
                        span { class: "text-xs text-gray-400", {perm.2.clone()} }
                    }
                }
            }
        }
    }
}

#[component]
fn DeleteConfirmDialog(item_type: String, id: i32, on_close: EventHandler<()>, on_confirm: EventHandler<()>) -> Element {
    let item_name = if item_type == "permission" { "权限" } else { "角色" };

    rsx! {
        div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            div { class: "bg-white rounded-lg shadow-xl w-full max-w-md mx-4",
                div { class: "p-6",
                    div { class: "flex items-center",
                        div { class: "flex-shrink-0",
                            div { class: "mx-auto flex items-center justify-center h-12 w-12 rounded-full bg-red-100",
                                Icon { icon: FaTriangleExclamation, width: 24, height: 24, class: "text-red-600" }
                            }
                        }
                        div { class: "mt-3 text-center sm:mt-0 sm:ml-4 sm:text-left",
                            h3 { class: "text-lg font-medium text-gray-900", "确认删除" }
                            div { class: "mt-2", p { class: "text-sm text-gray-500", "确定要删除这个{item_name}吗？此操作无法撤销。" } }
                        }
                    }
                }
                div { class: "mt-6 flex justify-end space-x-3",
                    button { class: "px-4 py-2 text-gray-600 border border-gray-300 rounded-md hover:bg-gray-50", onclick: move |_| on_close.call(()), "取消" }
                    button { class: "px-4 py-2 bg-red-600 text-white rounded-md hover:bg-red-700", onclick: move |_| on_confirm.call(()), "删除" }
                }
            }
        }
    }
}
