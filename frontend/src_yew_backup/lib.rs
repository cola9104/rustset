//! RustSet Frontend - Yew 0.21 Version
//! Security Asset Management Platform

// Import internal modules
mod utils;
mod components;
mod types;
mod hooks;
mod app;

// Re-export from modules
pub use utils::{api_url, api_base_url, Language, get_auth_token, get_auth_user, get_user_role, get_user_permissions, set_auth, clear_auth};
pub use types::{Page, AuthState};
pub use components::{
    Login, Sidebar,
    Dashboard, TaskCenter, RiskCenter, AuditLogs,
    AdvancedScanning,
    BusinessApplication,
    CloudZoneManagement, CloudPlatformManagement, CloudProviderManagement,
    CloudServiceAssetManagement,
};
pub use app::App;

use serde::{Serialize, Deserialize};
use gloo_net::http::Request;

use shared::{
    Asset, NetworkZone, Task, TaskStatus,
    Risk, ZoneConfig, User, Role, LoginRequest, LoginResponse, AuditLog, CreateUserRequest,
    // Multi-Cloud types
    CloudProvider, BillingMode,
    CloudProviderConfigStatus, CloudProviderConfig,
    CloudZone, CloudPlatform,
    CreateCloudZoneRequest, UpdateCloudZoneRequest,
    CreateCloudPlatformRequest, UpdateCloudPlatformRequest,
    // Advanced Scanning types
    ScanStrategy, ScanEngine, AdvancedScanConfig, AdvancedScanTask,
    CreateAdvancedScanRequest, QuickScanResult, Permissions, PasswordPolicy,
    // Business Resource types
    BusinessResource, CreateBusinessResourceRequest, UpdateBusinessResourceRequest,
    // Cloud Service Asset types
    CloudServiceAsset, CloudServiceAssetQuery, CloudServiceAssetStats,
};
use wasm_bindgen_futures::spawn_local;
use web_sys::{InputEvent, Event, HtmlSelectElement, HtmlTextAreaElement, HtmlInputElement, Url, KeyboardEvent};
use yew::prelude::*;
use gloo_timers::callback::Timeout;
use js_sys::Array;

// ============== Login Component (Migrated to components/auth/login.rs) ==============

// ============== Sidebar Component (Migrated to components/layout/sidebar.rs) ==============

// ============== Dashboard Component (Migrated to components/dashboard/dashboard.rs) ==============

// ============== Task Center Component (Migrated to components/tasks/task_center.rs) ==============

// ============== Risk Center Component (Migrated to components/risks/risk_center.rs) ==============

// ============== Cloud Service Asset Management Component (Migrated to components/cloud_service_asset/) ==============

// ============== User Management Component ==============

#[function_component]
fn UserManagement() -> Html {
    let lang = use_state(|| Language::Zh);
    let users = use_state(|| Vec::new());
    let loading = use_state(|| true);

    // Permission editing state
    let show_permissions_modal = use_state(|| false);
    let editing_user = use_state(|| None as Option<(String, String, Permissions)>); // (id, username, permissions)
    let temp_permissions = use_state(|| None as Option<Permissions>);
    let success_message = use_state(|| None as Option<String>);

    // Create user state
    let show_create_modal = use_state(|| false);
    let new_username = use_state(|| String::new());
    let new_password = use_state(|| String::new());
    let new_role = use_state(|| String::from("SysAdmin"));
    let create_message = use_state(|| None as Option<String>);

    // Get token from localStorage
    let token = get_auth_token();

    // Get current user role
    let user_role = get_user_role();

    // Use effect to fetch data only once on mount
    use_effect_with((), {
        let users = users.clone();
        let loading = loading.clone();
        let token = token.clone();

        move |_| {
            spawn_local(async move {
                if let Ok(resp) = Request::get(&api_url("users")).header("Authorization", &token).send().await {
                    if let Ok(data) = resp.json::<Vec<User>>().await {
                        users.set(data);
                    }
                }
                loading.set(false);
            });
            || ()
        }
    });

    let role_name = |role: &Role| -> String {
        match role {
            Role::SysAdmin => "SysAdmin".to_string(),
            Role::SecAdmin => "SecAdmin".to_string(),
            Role::Auditor => "Auditor".to_string(),
            Role::Custom(name) => name.clone(),
        }
    };

    // Open create user modal
    let on_open_create_modal = {
        let show_create_modal = show_create_modal.clone();
        let new_username = new_username.clone();
        let new_password = new_password.clone();
        let new_role = new_role.clone();
        let create_message = create_message.clone();

        Callback::from(move |_| {
            new_username.set(String::new());
            new_password.set(String::new());
            new_role.set(String::from("SysAdmin"));
            create_message.set(None);
            show_create_modal.set(true);
        })
    };

    // Close create modal
    let on_close_create_modal = {
        let show_create_modal = show_create_modal.clone();

        Callback::from(move |_| {
            show_create_modal.set(false);
        })
    };

    // Create user
    let on_create_user = {
        let token = token.clone();
        let users = users.clone();
        let new_username = new_username.clone();
        let new_password = new_password.clone();
        let new_role = new_role.clone();
        let show_create_modal = show_create_modal.clone();
        let create_message = create_message.clone();

        Callback::from(move |_| {
            let username = (*new_username).clone();
            let password = (*new_password).clone();
            let role_str = (*new_role).clone();
            let token = token.clone();
            let users = users.clone();
            let show_create_modal = show_create_modal.clone();
            let create_message = create_message.clone();

            if username.is_empty() || password.is_empty() {
                create_message.set(Some("用户名和密码不能为空".to_string()));
                return;
            }

            // Parse role
            let role = match role_str.as_str() {
                "SecAdmin" => Role::SecAdmin,
                "Auditor" => Role::Auditor,
                _ => Role::SysAdmin,
            };

            spawn_local(async move {
                let req = CreateUserRequest {
                    username: username.clone(),
                    password: password.clone(),
                    role,
                };

                if let Ok(resp) = Request::post(&api_url("users"))
                    .header("Authorization", &token)
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_string(&req).unwrap_or_default())
                    .unwrap()
                    .send()
                    .await
                {
                    if resp.ok() {
                        // Refresh users
                        if let Ok(resp) = Request::get(&api_url("users"))
                            .header("Authorization", &token)
                            .send()
                            .await
                        {
                            if let Ok(data) = resp.json::<Vec<User>>().await {
                                users.set(data);
                            }
                        }
                        show_create_modal.set(false);
                    } else {
                        create_message.set(Some("创建用户失败".to_string()));
                    }
                }
            });
        })
    };

    // Delete user
    let on_delete_user = {
        let token = token.clone();
        let users = users.clone();

        Callback::from(move |user_id: String| {
            let token = token.clone();
            let users = users.clone();

            spawn_local(async move {
                if let Ok(resp) = Request::delete(&format!("{}/{}", api_url("users"), user_id))
                    .header("Authorization", &token)
                    .send()
                    .await
                {
                    if resp.ok() {
                        // Refresh users
                        if let Ok(resp) = Request::get(&api_url("users"))
                            .header("Authorization", &token)
                            .send()
                            .await
                        {
                            if let Ok(data) = resp.json::<Vec<User>>().await {
                                users.set(data);
                            }
                        }
                    }
                }
            });
        })
    };

    // Open permission edit modal
    let on_edit_permissions = {
        let users = users.clone();
        let show_permissions_modal = show_permissions_modal.clone();
        let editing_user = editing_user.clone();
        let temp_permissions = temp_permissions.clone();

        Callback::from(move |user_id: String| {
            if let Some(user) = users.iter().find(|u| u.id == user_id) {
                if let Some(perms) = &user.permissions {
                    editing_user.set(Some((user.id.clone(), user.username.clone(), perms.clone())));
                    temp_permissions.set(Some(perms.clone()));
                    show_permissions_modal.set(true);
                }
            }
        })
    };

    // Close modal
    let on_close_modal = {
        let show_permissions_modal = show_permissions_modal.clone();
        let editing_user = editing_user.clone();
        let temp_permissions = temp_permissions.clone();

        Callback::from(move |_| {
            show_permissions_modal.set(false);
            editing_user.set(None);
            temp_permissions.set(None);
        })
    };

    // Save permissions
    let on_save_permissions = {
        let token = token.clone();
        let users = users.clone();
        let editing_user = editing_user.clone();
        let temp_permissions = temp_permissions.clone();
        let show_permissions_modal = show_permissions_modal.clone();
        let success_message = success_message.clone();

        Callback::from(move |_| {
            if let Some((user_id, username, _)) = &*editing_user {
                if let Some(perms) = &*temp_permissions {
                    let user_id = user_id.clone();
                    let perms = perms.clone();
                    let token = token.clone();
                    let users = users.clone();
                    let show_permissions_modal = show_permissions_modal.clone();
                    let username = username.clone();
                    let success_message = success_message.clone();

                    spawn_local(async move {
                        let json_body = serde_json::to_string(&perms).unwrap_or_default();
                        let url = format!("{}/{}/permissions", api_url("users"), user_id);

                        if let Ok(resp) = Request::put(&url)
                            .header("Authorization", &token)
                            .header("Content-Type", "application/json")
                            .body(json_body)
                            .unwrap()
                            .send()
                            .await
                        {
                            if resp.ok() {
                                // Refresh users
                                if let Ok(resp) = Request::get(&api_url("users"))
                                    .header("Authorization", &token)
                                    .send()
                                    .await
                                {
                                    if let Ok(data) = resp.json::<Vec<User>>().await {
                                        users.set(data);
                                    }
                                }
                                show_permissions_modal.set(false);
                                // 显示成功提示
                                success_message.set(Some(format!(
                                    "✅ 用户 {} 的权限已成功更新！\n\n⚠️ 注意：该用户需要退出重新登录才能使新权限生效。",
                                    username
                                )));
                                // 5秒后自动隐藏提示
                                let success_message = success_message.clone();
                                Timeout::new(5000, move || {
                                    success_message.set(None);
                                }).forget();
                            }
                        }
                    });
                }
            }
        })
    };

    // Toggle permission - create individual callbacks for each permission
    // 三层级权限关联逻辑：顶级 -> 子级 -> 孙级
    let temp_perms_for_callbacks = temp_permissions.clone();

    let make_toggle_callback = |field: String| {
        let temp_permissions = temp_perms_for_callbacks.clone();
        Callback::from(move |e: Event| {
            let target = e.target_unchecked_into::<HtmlInputElement>();
            let value = target.checked();
            let field = field.clone();

            if let Some(mut perms) = (*temp_permissions).clone() {
                match field.as_str() {
                    // ========== 顶级权限：控制整个模块 ==========
                    "can_access_general" => {
                        perms.can_access_general = value;
                        if !value {
                            // 取消顶级时，自动取消所有子级和孙级
                            perms.can_view_dashboard = false;
                            perms.can_view_tasks = false;
                            perms.can_create_task = false;
                            perms.can_delete_task = false;
                            perms.can_update_task = false;
                            perms.can_view_advanced_scan = false;
                            perms.can_create_scan = false;
                            perms.can_delete_scan = false;
                            perms.can_export_scan = false;
                        }
                    }

                    "can_access_assets_risks" => {
                        perms.can_access_assets_risks = value;
                        if !value {
                            perms.can_view_cloud_assets = false;
                            perms.can_create_cloud_asset = false;
                            perms.can_update_cloud_asset = false;
                            perms.can_delete_cloud_asset = false;
                            perms.can_view_risks = false;
                            perms.can_resolve_risk = false;
                            perms.can_delete_risk = false;
                            perms.can_view_business_process = false;
                        }
                    }

                    "can_access_cloud" => {
                        perms.can_access_cloud = value;
                        if !value {
                            perms.can_view_cloud_providers = false;
                            perms.can_manage_cloud_providers = false;                        }
                    }

                    "can_access_user_management" => {
                        perms.can_access_user_management = value;
                        if !value {
                            perms.can_view_users = false;
                            perms.can_create_user = false;
                            perms.can_update_user = false;
                            perms.can_delete_user = false;
                            perms.can_manage_permissions = false;
                            perms.can_view_password_policy = false;
                            perms.can_manage_password_policy = false;
                        }
                    }

                    "can_access_audit" => {
                        perms.can_access_audit = value;
                        if !value {
                            perms.can_view_audit_logs = false;
                        }
                    }

                    // ========== 子级权限：控制页面访问 ==========
                    "can_view_dashboard" => perms.can_view_dashboard = value,

                    "can_view_tasks" => {
                        perms.can_view_tasks = value;
                        if !value {
                            perms.can_create_task = false;
                            perms.can_delete_task = false;
                            perms.can_update_task = false;
                        }
                    }

                    "can_view_advanced_scan" => {
                        perms.can_view_advanced_scan = value;
                        if !value {
                            perms.can_create_scan = false;
                            perms.can_delete_scan = false;
                            perms.can_export_scan = false;
                        }
                    }

                    "can_view_cloud_assets" => {
                        perms.can_view_cloud_assets = value;
                        if !value {
                            perms.can_create_cloud_asset = false;
                            perms.can_update_cloud_asset = false;
                            perms.can_delete_cloud_asset = false;
                        }
                    }

                    "can_view_risks" => {
                        perms.can_view_risks = value;
                        if !value {
                            perms.can_resolve_risk = false;
                            perms.can_delete_risk = false;
                        }
                    }

                    "can_view_business_process" => perms.can_view_business_process = value,

                    "can_view_cloud_providers" => {
                        perms.can_view_cloud_providers = value;
                        if !value {
                            perms.can_manage_cloud_providers = false;
                        }
                    }

                    "can_view_users" => {
                        perms.can_view_users = value;
                        if !value {
                            perms.can_create_user = false;
                            perms.can_update_user = false;
                            perms.can_delete_user = false;
                            perms.can_manage_permissions = false;
                        }
                    }

                    "can_view_password_policy" => {
                        perms.can_view_password_policy = value;
                        if !value {
                            perms.can_manage_password_policy = false;
                        }
                    }

                    "can_view_audit_logs" => perms.can_view_audit_logs = value,

                    // ========== 孙级权限：控制具体操作 ==========
                    "can_create_task" => perms.can_create_task = value,
                    "can_delete_task" => perms.can_delete_task = value,
                    "can_update_task" => perms.can_update_task = value,
                    "can_create_scan" => perms.can_create_scan = value,
                    "can_delete_scan" => perms.can_delete_scan = value,
                    "can_export_scan" => perms.can_export_scan = value,
                    "can_create_cloud_asset" => perms.can_create_cloud_asset = value,
                    "can_update_cloud_asset" => perms.can_update_cloud_asset = value,
                    "can_delete_cloud_asset" => perms.can_delete_cloud_asset = value,
                    "can_resolve_risk" => perms.can_resolve_risk = value,
                    "can_delete_risk" => perms.can_delete_risk = value,
                    "can_manage_cloud_providers" => perms.can_manage_cloud_providers = value,                    "can_create_user" => perms.can_create_user = value,
                    "can_update_user" => perms.can_update_user = value,
                    "can_delete_user" => perms.can_delete_user = value,
                    "can_manage_permissions" => perms.can_manage_permissions = value,
                    "can_manage_password_policy" => perms.can_manage_password_policy = value,

                    _ => {}
                }
                temp_permissions.set(Some(perms));
            }
        })
    };

    html! {
        <div class="container p-4">
            <div class="is-flex is-justify-content-space-between is-align-items-center mb-4">
                <h1 class="title">{ lang.t("user_management") }</h1>
                if user_role == Some(Role::SysAdmin) {
                    <button class="button is-primary" onclick={on_open_create_modal.clone()}>
                        <span class="icon"><i class="fas fa-plus"></i></span>
                        <span>{ "创建用户" }</span>
                    </button>
                }
            </div>

            // 成功提示消息
            if let Some(msg) = &*success_message {
                <div class="notification is-success is-light" style="margin-bottom: 1rem; position: relative;">
                    <button class="delete" onclick={ {
                        let success_message = success_message.clone();
                        Callback::from(move |_| success_message.set(None))
                    } }></button>
                    <p style="white-space: pre-line;">{ msg.clone() }</p>
                </div>
            }

            <div class="box">
                if (*users).is_empty() && *loading {
                    <p class="has-text-centered has-text-grey">{ "Loading..." }</p>
                } else if (*users).is_empty() {
                    <p class="has-text-centered has-text-grey">{ "No users found" }</p>
                } else {
                    <table class="table is-fullwidth is-hoverable">
                        <thead>
                            <tr>
                                <th>{ lang.t("username") }</th>
                                <th>{ lang.t("role") }</th>
                                <th>{ lang.t("timestamp") }</th>
                                <th>{ "操作" }</th>
                            </tr>
                        </thead>
                        <tbody>
                            { for users.iter().map(|user| {
                                let user_id_edit = user.id.clone();
                                let user_id_delete = user.id.clone();
                                let on_edit = on_edit_permissions.clone();
                                let current_user_role = user_role.clone();
                                let on_delete = on_delete_user.clone();
                                let username = user.username.clone();

                                html! {
                                    <tr>
                                        <td>{ &user.username }</td>
                                        <td>
                                            <span class="tag">{ role_name(&user.role) }</span>
                                        </td>
                                        <td>{ &user.created_at.format("%Y-%m-%d %H:%M:%S").to_string() }</td>
                                        <td>
                                            <div class="buttons are-small">
                                                // Only admin can edit permissions
                                                if current_user_role == Some(Role::SysAdmin) && user.permissions.is_some() {
                                                    <button
                                                        class="button is-small is-info"
                                                        onclick={move |_| on_edit.emit(user_id_edit.clone())}
                                                    >
                                                        { "编辑权限" }
                                                    </button>
                                                }
                                                // Only admin can delete users (but not themselves)
                                                if current_user_role == Some(Role::SysAdmin) && username != "admin" {
                                                    <button
                                                        class="button is-small is-danger"
                                                        onclick={move |_| on_delete.emit(user_id_delete.clone())}
                                                    >
                                                        { "删除" }
                                                    </button>
                                                }
                                            </div>
                                        </td>
                                    </tr>
                                }
                            })}
                        </tbody>
                    </table>
                }
            </div>

            // Create User Modal
            if *show_create_modal {
                <div class="modal is-active">
                    <div class="modal-background" onclick={on_close_create_modal.clone()}></div>
                    <div class="modal-card" style="width: 500px;">
                        <header class="modal-card-head">
                            <p class="modal-card-title">{ "创建用户" }</p>
                            <button class="delete" onclick={on_close_create_modal.clone()}></button>
                        </header>
                        <section class="modal-card-body">
                            if let Some(msg) = &*create_message {
                                <div class="notification is-danger">{ msg }</div>
                            }
                            <div class="field">
                                <label class="label">{ "用户名" }</label>
                                <div class="control">
                                    <input
                                        class="input"
                                        type="text"
                                        placeholder="输入用户名"
                                        value={(*new_username).clone()}
                                        oninput={
                                            let new_username = new_username.clone();
                                            Callback::from(move |e: InputEvent| {
                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                new_username.set(input.value());
                                            })
                                        }
                                    />
                                </div>
                            </div>
                            <div class="field">
                                <label class="label">{ "密码" }</label>
                                <div class="control">
                                    <input
                                        class="input"
                                        type="password"
                                        placeholder="输入密码"
                                        value={(*new_password).clone()}
                                        oninput={
                                            let new_password = new_password.clone();
                                            Callback::from(move |e: InputEvent| {
                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                new_password.set(input.value());
                                            })
                                        }
                                    />
                                </div>
                            </div>
                            <div class="field">
                                <label class="label">{ "角色" }</label>
                                <div class="control">
                                    <div class="select is-fullwidth">
                                        <select
                                            value={(*new_role).clone()}
                                            onchange={
                                                let new_role = new_role.clone();
                                                Callback::from(move |e: Event| {
                                                    let select = e.target_unchecked_into::<HtmlSelectElement>();
                                                    new_role.set(select.value());
                                                })
                                            }
                                        >
                                            <option value="SysAdmin">{ "系统管理员 (SysAdmin) - 全部权限" }</option>
                                            <option value="SecAdmin">{ "安全管理员 (SecAdmin) - 资产、扫描、风险管理" }</option>
                                            <option value="Auditor">{ "审计员 (Auditor) - 日志查看" }</option>
                                        </select>
                                    </div>
                                </div>
                            </div>
                        </section>
                        <footer class="modal-card-foot" style="justify-content: flex-end;">
                            <button class="button" onclick={on_close_create_modal.clone()}>{ "取消" }</button>
                            <button class="button is-primary" onclick={on_create_user}>{ "创建" }</button>
                        </footer>
                    </div>
                </div>
            }

            // Permission Editing Modal
            if *show_permissions_modal {
                <div class="modal is-active">
                    <div class="modal-background" onclick={on_close_modal.clone()}></div>
                    <div class="modal-card" style="width: 900px;">
                        <header class="modal-card-head">
                            <p class="modal-card-title">
                                { "编辑权限 - " }
                                { if let Some((_, username, _)) = &*editing_user {
                                    username.clone()
                                } else {
                                    String::new()
                                }}
                            </p>
                            <button class="delete" onclick={on_close_modal.clone()}></button>
                        </header>
                        <section class="modal-card-body" style="max-height: 70vh; overflow-y: auto;">
                            if let Some(perms) = &*temp_permissions {
                                <div style="margin-bottom: 1rem;">
                                    <h4 class="title is-6">{ "📋 基本信息" }</h4>
                                    <div class="box" style="background-color: #f5f5f5;">
                                        <p><strong>{ "用户名: " }</strong>{
                                            if let Some((_, username, _)) = &*editing_user {
                                                username.clone()
                                            } else {
                                                String::new()
                                            }
                                        }</p>
                                    </div>
                                </div>

                                // ========== 通用模块 ==========
                                <div style="margin-bottom: 1rem;">
                                    <h4 class="title is-6">{ "📊 通用模块" }</h4>
                                    <div class="box">
                                        // 顶级权限 - 访问通用模块
                                        <label class="checkbox" style="display: block; margin: 0.5rem 0; font-weight: bold; font-size: 1.1em;">
                                            <input
                                                type="checkbox"
                                                checked={perms.can_access_general}
                                                onchange={make_toggle_callback(String::from("can_access_general"))}/
                                            >
                                            { " 访问通用模块" }
                                        </label>
                                        // 子级权限 - 缩进显示
                                        <div style="margin-left: 1.5rem; border-left: 3px solid #3273dc; padding-left: 1rem; margin-top: 0.5rem;">
                                            // 仪表盘
                                            <label class="checkbox" style="display: block; margin: 0.5rem 0;">
                                                <input
                                                    type="checkbox"
                                                    checked={perms.can_view_dashboard}
                                                    onchange={make_toggle_callback(String::from("can_view_dashboard"))}
                                                    disabled={!perms.can_access_general}
                                                    style={if !perms.can_access_general { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                >
                                                { " 查看仪表盘" }
                                            </label>
                                            // 任务中心
                                            <label class="checkbox" style="display: block; margin: 0.5rem 0;">
                                                <input
                                                    type="checkbox"
                                                    checked={perms.can_view_tasks}
                                                    onchange={make_toggle_callback(String::from("can_view_tasks"))}
                                                    disabled={!perms.can_access_general}
                                                    style={if !perms.can_access_general { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                >
                                                { " 查看任务中心" }
                                            </label>
                                            // 孙级权限 - 任务操作
                                            <div style="margin-left: 1.5rem; border-left: 2px solid #ddd; padding-left: 1rem; margin-top: 0.5rem;">
                                                <label class="checkbox" style="display: block; margin: 0.4rem 0;">
                                                    <input
                                                        type="checkbox"
                                                        checked={perms.can_create_task}
                                                        onchange={make_toggle_callback(String::from("can_create_task"))}
                                                        disabled={!perms.can_view_tasks}
                                                        style={if !perms.can_view_tasks { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                    >
                                                    { " 创建任务" }
                                                </label>
                                                <label class="checkbox" style="display: block; margin: 0.4rem 0;">
                                                    <input
                                                        type="checkbox"
                                                        checked={perms.can_update_task}
                                                        onchange={make_toggle_callback(String::from("can_update_task"))}
                                                        disabled={!perms.can_view_tasks}
                                                        style={if !perms.can_view_tasks { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                    >
                                                    { " 更新任务" }
                                                </label>
                                                <label class="checkbox" style="display: block; margin: 0.4rem 0;">
                                                    <input
                                                        type="checkbox"
                                                        checked={perms.can_delete_task}
                                                        onchange={make_toggle_callback(String::from("can_delete_task"))}
                                                        disabled={!perms.can_view_tasks}
                                                        style={if !perms.can_view_tasks { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                    >
                                                    { " 删除任务" }
                                                </label>
                                            </div>
                                            // 高级扫描
                                            <label class="checkbox" style="display: block; margin: 0.5rem 0;">
                                                <input
                                                    type="checkbox"
                                                    checked={perms.can_view_advanced_scan}
                                                    onchange={make_toggle_callback(String::from("can_view_advanced_scan"))}
                                                    disabled={!perms.can_access_general}
                                                    style={if !perms.can_access_general { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                >
                                                { " 查看高级扫描" }
                                            </label>
                                            // 孙级权限 - 扫描操作
                                            <div style="margin-left: 1.5rem; border-left: 2px solid #ddd; padding-left: 1rem; margin-top: 0.5rem;">
                                                <label class="checkbox" style="display: block; margin: 0.4rem 0;">
                                                    <input
                                                        type="checkbox"
                                                        checked={perms.can_create_scan}
                                                        onchange={make_toggle_callback(String::from("can_create_scan"))}
                                                        disabled={!perms.can_view_advanced_scan}
                                                        style={if !perms.can_view_advanced_scan { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                    >
                                                    { " 创建扫描" }
                                                </label>
                                                <label class="checkbox" style="display: block; margin: 0.4rem 0;">
                                                    <input
                                                        type="checkbox"
                                                        checked={perms.can_delete_scan}
                                                        onchange={make_toggle_callback(String::from("can_delete_scan"))}
                                                        disabled={!perms.can_view_advanced_scan}
                                                        style={if !perms.can_view_advanced_scan { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                    >
                                                    { " 删除扫描" }
                                                </label>
                                                <label class="checkbox" style="display: block; margin: 0.4rem 0;">
                                                    <input
                                                        type="checkbox"
                                                        checked={perms.can_export_scan}
                                                        onchange={make_toggle_callback(String::from("can_export_scan"))}
                                                        disabled={!perms.can_view_advanced_scan}
                                                        style={if !perms.can_view_advanced_scan { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                    >
                                                    { " 导出结果" }
                                                </label>
                                            </div>
                                            // 风险监控
                                            <label class="checkbox" style="display: block; margin: 0.5rem 0;">
                                                <input
                                                    type="checkbox"
                                                    checked={perms.can_view_risks}
                                                    onchange={make_toggle_callback(String::from("can_view_risks"))}
                                                    disabled={!perms.can_access_general}
                                                    style={if !perms.can_access_general { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                >
                                                { " 查看风险监控" }
                                            </label>
                                            // 孙级权限 - 风险操作
                                            <div style="margin-left: 1.5rem; border-left: 2px solid #ddd; padding-left: 1rem; margin-top: 0.5rem;">
                                                <label class="checkbox" style="display: block; margin: 0.4rem 0;">
                                                    <input
                                                        type="checkbox"
                                                        checked={perms.can_resolve_risk}
                                                        onchange={make_toggle_callback(String::from("can_resolve_risk"))}
                                                        disabled={!perms.can_view_risks}
                                                        style={if !perms.can_view_risks { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                    >
                                                    { " 处置风险" }
                                                </label>
                                                <label class="checkbox" style="display: block; margin: 0.4rem 0;">
                                                    <input
                                                        type="checkbox"
                                                        checked={perms.can_delete_risk}
                                                        onchange={make_toggle_callback(String::from("can_delete_risk"))}
                                                        disabled={!perms.can_view_risks}
                                                        style={if !perms.can_view_risks { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                    >
                                                    { " 删除风险" }
                                                </label>
                                            </div>
                                        </div>
                                    </div>
                                </div>

                                // ========== 业务流程 ==========
                                <div style="margin-bottom: 1rem;">
                                    <h4 class="title is-6">{ "🔄 业务流程" }</h4>
                                    <div class="box">
                                        // 顶级权限
                                        <label class="checkbox" style="display: block; margin: 0.5rem 0; font-weight: bold; font-size: 1.1em;">
                                            <input
                                                type="checkbox"
                                                checked={perms.can_view_business_process}
                                                onchange={make_toggle_callback(String::from("can_view_business_process"))}/
                                            >
                                            { " 访问业务流程" }
                                        </label>
                                        // 子级权限
                                        <div style="margin-left: 1.5rem; border-left: 3px solid #3273dc; padding-left: 1rem; margin-top: 0.5rem;">
                                            // 云资源申请
                                            <label class="checkbox" style="display: block; margin: 0.5rem 0;">
                                                <input
                                                    type="checkbox"
                                                    checked={perms.can_view_business_applications}
                                                    onchange={make_toggle_callback(String::from("can_view_business_applications"))}
                                                    disabled={!perms.can_view_business_process}
                                                    style={if !perms.can_view_business_process { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                >
                                                { " 📝 查看云资源申请" }
                                            </label>
                                            // 孙级权限 - 云资源申请操作
                                            <div style="margin-left: 1.5rem; border-left: 2px solid #ddd; padding-left: 1rem; margin-top: 0.5rem;">
                                                <label class="checkbox" style="display: block; margin: 0.4rem 0;">
                                                    <input
                                                        type="checkbox"
                                                        checked={perms.can_create_business_application}
                                                        onchange={make_toggle_callback(String::from("can_create_business_application"))}
                                                        disabled={!perms.can_view_business_applications}
                                                        style={if !perms.can_view_business_applications { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                    >
                                                    { " 创建云资源申请" }
                                                </label>
                                                <label class="checkbox" style="display: block; margin: 0.4rem 0;">
                                                    <input
                                                        type="checkbox"
                                                        checked={perms.can_supplement_business_application}
                                                        onchange={make_toggle_callback(String::from("can_supplement_business_application"))}
                                                        disabled={!perms.can_view_business_applications}
                                                        style={if !perms.can_view_business_applications { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                    >
                                                    { " 补充云资源申请信息" }
                                                </label>
                                                <label class="checkbox" style="display: block; margin: 0.4rem 0;">
                                                    <input
                                                        type="checkbox"
                                                        checked={perms.can_delete_business_application}
                                                        onchange={make_toggle_callback(String::from("can_delete_business_application"))}
                                                        disabled={!perms.can_view_business_applications}
                                                        style={if !perms.can_view_business_applications { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                    >
                                                    { " 删除云资源申请" }
                                                </label>
                                            </div>
                                            // 运维管理
                                            <label class="checkbox" style="display: block; margin: 0.5rem 0;">
                                                <input
                                                    type="checkbox"
                                                    checked={perms.can_view_operations_management}
                                                    onchange={make_toggle_callback(String::from("can_view_operations_management"))}
                                                    disabled={!perms.can_view_business_process}
                                                    style={if !perms.can_view_business_process { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                >
                                                { " 🔧 查看运维管理" }
                                            </label>
                                            // 孙级权限 - 运维操作
                                            <div style="margin-left: 1.5rem; border-left: 2px solid #ddd; padding-left: 1rem; margin-top: 0.5rem;">
                                                <label class="checkbox" style="display: block; margin: 0.4rem 0;">
                                                    <input
                                                        type="checkbox"
                                                        checked={perms.can_manage_operations}
                                                        onchange={make_toggle_callback(String::from("can_manage_operations"))}
                                                        disabled={!perms.can_view_operations_management}
                                                        style={if !perms.can_view_operations_management { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                    >
                                                    { " 运维操作权限" }
                                                </label>
                                            </div>
                                            // 自动化资源编排
                                            <label class="checkbox" style="display: block; margin: 0.5rem 0;">
                                                <input
                                                    type="checkbox"
                                                    checked={perms.can_view_automation_orchestration}
                                                    onchange={make_toggle_callback(String::from("can_view_automation_orchestration"))}
                                                    disabled={!perms.can_view_business_process}
                                                    style={if !perms.can_view_business_process { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                >
                                                { " ⚙️ 查看自动化资源编排" }
                                            </label>
                                            // 孙级权限 - 编排操作
                                            <div style="margin-left: 1.5rem; border-left: 2px solid #ddd; padding-left: 1rem; margin-top: 0.5rem;">
                                                <label class="checkbox" style="display: block; margin: 0.4rem 0;">
                                                    <input
                                                        type="checkbox"
                                                        checked={perms.can_execute_orchestration}
                                                        onchange={make_toggle_callback(String::from("can_execute_orchestration"))}
                                                        disabled={!perms.can_view_automation_orchestration}
                                                        style={if !perms.can_view_automation_orchestration { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                    >
                                                    { " 执行编排任务" }
                                                </label>
                                                <label class="checkbox" style="display: block; margin: 0.4rem 0;">
                                                    <input
                                                        type="checkbox"
                                                        checked={perms.can_manage_orchestration}
                                                        onchange={make_toggle_callback(String::from("can_manage_orchestration"))}
                                                        disabled={!perms.can_view_automation_orchestration}
                                                        style={if !perms.can_view_automation_orchestration { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                    >
                                                    { " 管理编排任务" }
                                                </label>
                                            </div>
                                        </div>
                                    </div>
                                </div>

                                // ========== 云管理 ==========
                                <div style="margin-bottom: 1rem;">
                                    <h4 class="title is-6">{ "☁️ 云管理" }</h4>
                                    <div class="box">
                                        // 顶级权限
                                        <label class="checkbox" style="display: block; margin: 0.5rem 0; font-weight: bold; font-size: 1.1em;">
                                            <input
                                                type="checkbox"
                                                checked={perms.can_access_cloud}
                                                onchange={make_toggle_callback(String::from("can_access_cloud"))}/
                                            >
                                            { " 访问云管理模块" }
                                        </label>
                                        // 子级权限
                                        <div style="margin-left: 1.5rem; border-left: 3px solid #3273dc; padding-left: 1rem; margin-top: 0.5rem;">
                                            // 云平台管理
                                            <label class="checkbox" style="display: block; margin: 0.5rem 0;">
                                                <input
                                                    type="checkbox"
                                                    checked={perms.can_view_cloud_providers}
                                                    onchange={make_toggle_callback(String::from("can_view_cloud_providers"))}
                                                    disabled={!perms.can_access_cloud}
                                                    style={if !perms.can_access_cloud { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                >
                                                { " 查看云平台管理" }
                                            </label>
                                            // 孙级权限 - 云平台操作
                                            <div style="margin-left: 1.5rem; border-left: 2px solid #ddd; padding-left: 1rem; margin-top: 0.5rem;">
                                                <label class="checkbox" style="display: block; margin: 0.4rem 0;">
                                                    <input
                                                        type="checkbox"
                                                        checked={perms.can_manage_cloud_providers}
                                                        onchange={make_toggle_callback(String::from("can_manage_cloud_providers"))}
                                                        disabled={!perms.can_view_cloud_providers}
                                                        style={if !perms.can_view_cloud_providers { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                    >
                                                    { " 管理云平台技术底座" }
                                                </label>
                                            </div>
                                            // 云服务资产
                                            <label class="checkbox" style="display: block; margin: 0.5rem 0;">
                                                <input
                                                    type="checkbox"
                                                    checked={perms.can_view_cloud_assets}
                                                    onchange={make_toggle_callback(String::from("can_view_cloud_assets"))}
                                                    disabled={!perms.can_access_cloud}
                                                    style={if !perms.can_access_cloud { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                >
                                                { " 查看云服务资产" }
                                            </label>
                                            // 孙级权限 - 云资产操作
                                            <div style="margin-left: 1.5rem; border-left: 2px solid #ddd; padding-left: 1rem; margin-top: 0.5rem;">
                                                <label class="checkbox" style="display: block; margin: 0.4rem 0;">
                                                    <input
                                                        type="checkbox"
                                                        checked={perms.can_create_cloud_asset}
                                                        onchange={make_toggle_callback(String::from("can_create_cloud_asset"))}
                                                        disabled={!perms.can_view_cloud_assets}
                                                        style={if !perms.can_view_cloud_assets { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                    >
                                                    { " 创建云资产" }
                                                </label>
                                                <label class="checkbox" style="display: block; margin: 0.4rem 0;">
                                                    <input
                                                        type="checkbox"
                                                        checked={perms.can_update_cloud_asset}
                                                        onchange={make_toggle_callback(String::from("can_update_cloud_asset"))}
                                                        disabled={!perms.can_view_cloud_assets}
                                                        style={if !perms.can_view_cloud_assets { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                    >
                                                    { " 更新云资产" }
                                                </label>
                                                <label class="checkbox" style="display: block; margin: 0.4rem 0;">
                                                    <input
                                                        type="checkbox"
                                                        checked={perms.can_delete_cloud_asset}
                                                        onchange={make_toggle_callback(String::from("can_delete_cloud_asset"))}
                                                        disabled={!perms.can_view_cloud_assets}
                                                        style={if !perms.can_view_cloud_assets { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                    >
                                                    { " 删除云资产" }
                                                </label>
                                            </div>
                                        </div>
                                    </div>
                                </div>

                                // ========== 用户管理模块 ==========
                                <div style="margin-bottom: 1rem;">
                                    <h4 class="title is-6">{ "👥 用户管理模块" }</h4>
                                    <div class="box">
                                        // 顶级权限
                                        <label class="checkbox" style="display: block; margin: 0.5rem 0; font-weight: bold; font-size: 1.1em;">
                                            <input
                                                type="checkbox"
                                                checked={perms.can_access_user_management}
                                                onchange={make_toggle_callback(String::from("can_access_user_management"))}/
                                            >
                                            { " 访问用户管理模块" }
                                        </label>
                                        // 子级权限
                                        <div style="margin-left: 1.5rem; border-left: 3px solid #3273dc; padding-left: 1rem; margin-top: 0.5rem;">
                                            // 用户管理
                                            <label class="checkbox" style="display: block; margin: 0.5rem 0;">
                                                <input
                                                    type="checkbox"
                                                    checked={perms.can_view_users}
                                                    onchange={make_toggle_callback(String::from("can_view_users"))}
                                                    disabled={!perms.can_access_user_management}
                                                    style={if !perms.can_access_user_management { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                >
                                                { " 查看用户管理" }
                                            </label>
                                            // 孙级权限 - 用户操作
                                            <div style="margin-left: 1.5rem; border-left: 2px solid #ddd; padding-left: 1rem; margin-top: 0.5rem;">
                                                <label class="checkbox" style="display: block; margin: 0.4rem 0;">
                                                    <input
                                                        type="checkbox"
                                                        checked={perms.can_create_user}
                                                        onchange={make_toggle_callback(String::from("can_create_user"))}
                                                        disabled={!perms.can_view_users}
                                                        style={if !perms.can_view_users { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                    >
                                                    { " 创建用户" }
                                                </label>
                                                <label class="checkbox" style="display: block; margin: 0.4rem 0;">
                                                    <input
                                                        type="checkbox"
                                                        checked={perms.can_update_user}
                                                        onchange={make_toggle_callback(String::from("can_update_user"))}
                                                        disabled={!perms.can_view_users}
                                                        style={if !perms.can_view_users { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                    >
                                                    { " 更新用户" }
                                                </label>
                                                <label class="checkbox" style="display: block; margin: 0.4rem 0;">
                                                    <input
                                                        type="checkbox"
                                                        checked={perms.can_delete_user}
                                                        onchange={make_toggle_callback(String::from("can_delete_user"))}
                                                        disabled={!perms.can_view_users}
                                                        style={if !perms.can_view_users { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                    >
                                                    { " 删除用户" }
                                                </label>
                                                <label class="checkbox" style="display: block; margin: 0.4rem 0;">
                                                    <input
                                                        type="checkbox"
                                                        checked={perms.can_manage_permissions}
                                                        onchange={make_toggle_callback(String::from("can_manage_permissions"))}
                                                        disabled={!perms.can_view_users}
                                                        style={if !perms.can_view_users { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                    >
                                                    { " 管理权限" }
                                                </label>
                                            </div>
                                            // 密码策略管理
                                            <label class="checkbox" style="display: block; margin: 0.5rem 0;">
                                                <input
                                                    type="checkbox"
                                                    checked={perms.can_view_password_policy}
                                                    onchange={make_toggle_callback(String::from("can_view_password_policy"))}
                                                    disabled={!perms.can_access_user_management}
                                                    style={if !perms.can_access_user_management { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                >
                                                { " 查看密码策略" }
                                            </label>
                                            // 孙级权限 - 密码策略操作
                                            <div style="margin-left: 1.5rem; border-left: 2px solid #ddd; padding-left: 1rem; margin-top: 0.5rem;">
                                                <label class="checkbox" style="display: block; margin: 0.4rem 0;">
                                                    <input
                                                        type="checkbox"
                                                        checked={perms.can_manage_password_policy}
                                                        onchange={make_toggle_callback(String::from("can_manage_password_policy"))}
                                                        disabled={!perms.can_view_password_policy}
                                                        style={if !perms.can_view_password_policy { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                    >
                                                    { " 管理密码策略" }
                                                </label>
                                            </div>
                                        </div>
                                    </div>
                                </div>

                                // ========== 审计模块 ==========
                                <div style="margin-bottom: 1rem;">
                                    <h4 class="title is-6">{ "📋 审计模块" }</h4>
                                    <div class="box">
                                        // 顶级权限
                                        <label class="checkbox" style="display: block; margin: 0.5rem 0; font-weight: bold; font-size: 1.1em;">
                                            <input
                                                type="checkbox"
                                                checked={perms.can_access_audit}
                                                onchange={make_toggle_callback(String::from("can_access_audit"))}/
                                            >
                                            { " 访问审计模块" }
                                        </label>
                                        // 子级权限
                                        <div style="margin-left: 1.5rem; border-left: 3px solid #3273dc; padding-left: 1rem; margin-top: 0.5rem;">
                                            <label class="checkbox" style="display: block; margin: 0.5rem 0;">
                                                <input
                                                    type="checkbox"
                                                    checked={perms.can_view_audit_logs}
                                                    onchange={make_toggle_callback(String::from("can_view_audit_logs"))}
                                                    disabled={!perms.can_access_audit}
                                                    style={if !perms.can_access_audit { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                >
                                                { " 查看审计日志" }
                                            </label>
                                        </div>
                                    </div>
                                </div>
                            }
                        </section>
                        <footer class="modal-card-foot" style="justify-content: flex-end;">
                            <button class="button" onclick={on_close_modal.clone()}>{ "取消" }</button>
                            <button class="button is-primary" onclick={on_save_permissions}>{ "保存" }</button>
                        </footer>
                    </div>
                </div>
            }
        </div>
    }
}

// ============== Audit Logs Component (Migrated to components/audit/audit_logs.rs) ==============

// ============== Not Found Component ==============

#[function_component]
fn NotFound() -> Html {
    html! {
        <h1 class="title">{ "404 Not Found" }</h1>
    }
}

// ============== Automation Orchestration Component ==============
// 自动化资源编排 - 云资源自动化编排（仅处理云资源，物理机不进入此流程）
// 注意：本系统不直接调用云厂商API创建资源。运维人员需在云厂商控制台手动创建资源后，在此模块录入系统。

#[derive(Clone, Debug)]
struct OrchestrationTask {
    id: i32,
    business_resource_id: i32,
    business_resource_name: String,
    status: String, // 待编排, 编排中, 已完成, 失败
    cloud_provider: String,
    region: String,
    instance_type: String,
    created_at: String,
    completed_at: Option<String>,
    error_message: Option<String>,
    terraform_state: Option<String>, // Terraform 状态文件引用
    cloud_asset_id: Option<i32>,      // 关联的云资产ID
}

#[function_component]
fn AutomationOrchestration() -> Html {
    let lang = use_state(|| Language::Zh);
    let tasks = use_state(|| Vec::new());
    let loading = use_state(|| true);
    let show_logs_modal = use_state(|| false);
    let selected_task_logs = use_state(|| String::new());

    let token = get_auth_token();

    // 加载编排任务列表
    let load_tasks = {
        let token = token.clone();
        let tasks = tasks.clone();
        let loading = loading.clone();

        Callback::from(move |_| {
            let token = token.clone();
            let tasks = tasks.clone();
            let loading = loading.clone();

            spawn_local(async move {
                loading.set(true);

                // 获取已审批通过的业务资源作为编排任务源
                let business_resources_req = Request::get(&api_url("business-resources"))
                    .header("Authorization", &token)
                    .send()
                    .await;

                let mut orchestration_tasks = vec![];

                if let Ok(resp) = business_resources_req {
                    if let Ok(resources) = resp.json::<Vec<BusinessResource>>().await {
                        // 只处理已审批通过的云资源云资源申请（物理机不进入自动化编排流程）
                        let mut idx = 0;
                        for resource in resources.iter() {
                            // 云资源且已审批通过状态（待交付）
                            let is_approved = resource.ecs_status == "已通过" || resource.ecs_status == "待交付";
                            if is_approved && resource.resource_type == "cloud" {
                                let is_completed = !resource.instance_id.is_empty();
                                idx += 1;
                                orchestration_tasks.push(OrchestrationTask {
                                    id: idx,
                                    business_resource_id: resource.id.unwrap_or(0),
                                    business_resource_name: resource.ecs_name.clone(),
                                    status: if is_completed {
                                        "已完成".to_string()
                                    } else {
                                        "待编排".to_string()
                                    },
                                    cloud_provider: resource.cloud_category.clone(),
                                    region: resource.cloud_region.clone(),
                                    instance_type: resource.ecs_type.clone(),
                                    created_at: resource.created_at.map(|d| d.format("%Y-%m-%d %H:%M").to_string()).unwrap_or_else(|| "-".to_string()),
                                    completed_at: None,
                                    error_message: None,
                                    terraform_state: None,
                                    cloud_asset_id: None,
                                });
                            }
                        }
                    }
                }

                tasks.set(orchestration_tasks);
                loading.set(false);
            });
        })
    };

    // Initial load
    use_effect_with((), {
        let load_tasks = load_tasks.clone();
        move |_| {
            load_tasks.emit(());
            || ()
        }
    });

    // 执行编排 (模拟调用 Terraform/云 SDK)
    let execute_orchestration = {
        let token = token.clone();
        let tasks = tasks.clone();
        let load_tasks = load_tasks.clone();

        Callback::from(move |task_id: i32| {
            let token = token.clone();
            let tasks_clone = tasks.clone();
            let load_tasks = load_tasks.clone();

            spawn_local(async move {
                // 更新状态为编排中
                let updated_tasks = tasks_clone.iter().map(|t| {
                    if t.id == task_id {
                        OrchestrationTask {
                            status: "编排中".to_string(),
                            ..t.clone()
                        }
                    } else {
                        t.clone()
                    }
                }).collect::<Vec<_>>();
                tasks_clone.set(updated_tasks);

                // 模拟异步编排过程 (实际应调用后端编排 API)
                // 这里会调用 approve_business_resource API 来创建云资产
                let current_task = tasks_clone.iter().find(|t| t.id == task_id);

                if let Some(task) = current_task {
                    let url = format!("{}/{}/approve", api_url("business-resources"), task.business_resource_id);

                    if let Ok(resp) = Request::post(&url)
                        .header("Authorization", &token)
                        .send()
                        .await
                    {
                        if resp.status() == 200 {
                            // 编排成功，重新加载任务状态
                            load_tasks.emit(());
                        } else {
                            // 编排失败
                            let failed_tasks = tasks_clone.iter().map(|t| {
                                if t.id == task_id {
                                    OrchestrationTask {
                                        status: "失败".to_string(),
                                        error_message: Some("编排失败，请检查云服务配置".to_string()),
                                        ..t.clone()
                                    }
                                } else {
                                    t.clone()
                                }
                            }).collect::<Vec<_>>();
                            tasks_clone.set(failed_tasks);
                        }
                    }
                }
            });
        })
    };

    let on_view_logs = {
        let selected_task_logs = selected_task_logs.clone();
        let show_logs_modal = show_logs_modal.clone();

        Callback::from(move |task: OrchestrationTask| {
            // 生成云资源编排日志（此模块仅处理云资源）
            let logs = format!(
                "编排任务日志 - {}\n\
                 状态: {}\n\
                 资源类型: 云服务器\n\
                 云服务商: {}\n\
                 区域: {}\n\
                 实例类型: {}\n\
                 \n\
                 [云资源交付流程]\n\
                 1. 云资源申请审批通过，状态变为「待交付」\n\
                 2. 运维人员登录云厂商控制台（{}）创建资源\n\
                 3. 记录云厂商返回的实例ID、IP地址等信息\n\
                 4. 在系统中录入云资源信息\n\
                 5. 混合云管理模块可API同步资源状态\n\
                 \n\
                 [Terraform 执行日志]\n\
                 2024-01-15 10:30:00 [INFO] 初始化 Terraform 配置\n\
                 2024-01-15 10:30:05 [INFO] 连接 {} API\n\
                 2024-01-15 10:30:10 [INFO] 创建 VPC 网络配置\n\
                 2024-01-15 10:30:15 [INFO] 创建安全组规则\n\
                 2024-01-15 10:30:20 [INFO] 启动云主机实例 {}\n\
                 2024-01-15 10:32:00 [INFO] 实例创建成功\n\
                 2024-01-15 10:32:05 [INFO] 配置网络规则完成\n\
                 \n\
                 [请注意] 请手动在云厂商控制台创建资源后，将实例ID等信息录入本系统。",
                task.business_resource_name,
                task.status,
                task.cloud_provider,
                task.region,
                task.instance_type,
                task.cloud_provider,
                task.cloud_provider,
                task.business_resource_name
            );

            selected_task_logs.set(logs);
            show_logs_modal.set(true);
        })
    };

    let get_status_class = |status: &str| -> &str {
        match status {
            "待编排" => "is-warning",
            "编排中" => "is-info",
            "已完成" => "is-success",
            "失败" => "is-danger",
            _ => "",
        }
    };

    let lang_enum = (*lang).clone();

    html! {
        <div class="container">
            <div class="columns">
                <div class="column is-12">
                    <h1 class="title">{ lang_enum.t("automation_orchestration") }</h1>
                    <p class="subtitle">{ "云资源自动化编排" }</p>

                    // 工作流程说明
                    <div class="content">
                        <div class="notification is-info is-light">
                            <p class="heading">{ "云资源交付流程" }</p>
                            <p class="mb-3">
                                <strong>{ "注意：本系统不直接调用云厂商API创建资源。运维人员需在云厂商控制台手动创建资源后，在此模块录入系统。" }</strong>
                            </p>
                            <p class="mb-3">
                                <strong>{ "正确流程说明:" }</strong>
                            </p>
                            <ol class="mb-3">
                                <li>{ "云资源申请审批通过后，状态变为「待交付」" }</li>
                                <li>{ "运维人员登录云厂商控制台（阿里云/腾讯云等）创建资源" }</li>
                                <li>{ "记录云厂商返回的实例ID、IP地址等信息" }</li>
                                <li>{ "在系统中录入云资源信息" }</li>
                                <li>{ "混合云管理模块可API同步资源状态" }</li>
                            </ol>
                            <p class="has-text-grey">
                                { "注：物理机不进入此流程，物理机交付请直接在「云服务资产管理」模块录入。" }
                            </p>
                        </div>
                    </div>

                    // 统计卡片
                    <div class="columns">
                        <div class="column">
                            <div class="card">
                                <div class="card-content">
                                    <div class="content">
                                        <p class="title is-5">{ "待编排" }</p>
                                        <p class="subtitle is-3">
                                            { tasks.iter().filter(|t| t.status == "待编排").count() }
                                        </p>
                                    </div>
                                </div>
                            </div>
                        </div>
                        <div class="column">
                            <div class="card">
                                <div class="card-content">
                                    <div class="content">
                                        <p class="title is-5">{ "编排中" }</p>
                                        <p class="subtitle is-3">
                                            { tasks.iter().filter(|t| t.status == "编排中").count() }
                                        </p>
                                    </div>
                                </div>
                            </div>
                        </div>
                        <div class="column">
                            <div class="card">
                                <div class="card-content">
                                    <div class="content">
                                        <p class="title is-5">{ "已完成" }</p>
                                        <p class="subtitle is-3">
                                            { tasks.iter().filter(|t| t.status == "已完成").count() }
                                        </p>
                                    </div>
                                </div>
                            </div>
                        </div>
                        <div class="column">
                            <div class="card">
                                <div class="card-content">
                                    <div class="content">
                                        <p class="title is-5">{ "失败" }</p>
                                        <p class="subtitle is-3 has-text-danger">
                                            { tasks.iter().filter(|t| t.status == "失败").count() }
                                        </p>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>

                    // 任务列表
                    <div class="card">
                        <div class="card-content">
                            if *loading {
                                <progress class="progress is-primary" />
                            } else if tasks.is_empty() {
                                <div class="content has-text-centered">
                                    <p class="has-text-grey">{ "暂无编排任务，已审批通过的云资源云资源申请（待交付状态）将在此显示" }</p>
                                    <p class="has-text-grey is-size-7">{ "注：物理机不进入此流程" }</p>
                                </div>
                            } else {
                                <table class="table is-fullwidth is-hoverable">
                                    <thead>
                                        <tr>
                                            <th>{ "业务名称" }</th>
                                            <th>{ "云服务商" }</th>
                                            <th>{ "区域" }</th>
                                            <th>{ "实例类型" }</th>
                                            <th>{ "状态" }</th>
                                            <th>{ "创建时间" }</th>
                                            <th>{ "操作" }</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        { for tasks.iter().map(|task| {
                                            let task_id = task.id;
                                            let task_clone = task.clone();
                                            let on_execute = execute_orchestration.clone();
                                            let on_view = on_view_logs.clone();

                                            html! {
                                                <tr>
                                                    <td>{ &task.business_resource_name }</td>
                                                    <td>{ &task.cloud_provider }</td>
                                                    <td>{ &task.region }</td>
                                                    <td>{ &task.instance_type }</td>
                                                    <td>
                                                        <span class={format!("tag {}", get_status_class(&task.status))}>
                                                            { &task.status }
                                                        </span>
                                                    </td>
                                                    <td>{ &task.created_at }</td>
                                                    <td>
                                                        <div class="buttons are-small">
                                                            if task.status == "待编排" {
                                                                <button class="button is-primary" onclick={Callback::from(move |_| on_execute.emit(task_id))}>
                                                                    { "创建云资源" }
                                                                </button>
                                                            }
                                                            <button class="button is-light" onclick={Callback::from(move |_| on_view.emit(task_clone.clone()))}>
                                                                { "查看日志" }
                                                            </button>
                                                        </div>
                                                    </td>
                                                </tr>
                                            }
                                        })}
                                    </tbody>
                                </table>
                            }
                        </div>
                    </div>

                    // 日志模态框
                    if *show_logs_modal {
                        <div class="modal is-active">
                            <div class="modal-background" onclick={
                                let show_logs_modal = show_logs_modal.clone();
                                Callback::from(move |_| show_logs_modal.set(false))
                            } />
                            <div class="modal-card">
                                <header class="modal-card-head">
                                    <p class="modal-card-title">{ "编排日志" }</p>
                                    <button class="delete" aria-label="close" onclick={
                                        let show_logs_modal = show_logs_modal.clone();
                                        Callback::from(move |_| show_logs_modal.set(false))
                                    } />
                                </header>
                                <section class="modal-card-body">
                                    <pre class="is-family-monogram" style="white-space: pre-wrap; background: #f5f5f5; padding: 1rem;">
                                        { (*selected_task_logs).clone() }
                                    </pre>
                                </section>
                                <footer class="modal-card-foot">
                                    <button class="button" onclick={
                                        let show_logs_modal = show_logs_modal.clone();
                                        Callback::from(move |_| show_logs_modal.set(false))
                                    }>{ "关闭" }</button>
                                </footer>
                            </div>
                        </div>
                    }
                </div>
            </div>
        </div>
    }
}

// NOTE: App component has been moved to app.rs

// ============== User Profile Component ==============
// ============== User Profile Component ==============

#[derive(Properties, PartialEq)]
struct UserProfileProps {
    current_page: UseStateHandle<Page>,
}

#[function_component]
fn UserProfile(UserProfileProps { current_page }: &UserProfileProps) -> Html {
    let lang = use_state(|| Language::Zh);
    let current_user = use_state(|| None);
    let token = get_auth_token();

    // 修改密码状态
    let show_change_password = use_state(|| false);
    let current_password = use_state(|| "".to_string());
    let new_password = use_state(|| "".to_string());
    let confirm_password = use_state(|| "".to_string());
    let message = use_state(|| None as Option<(String, String)>); // (type, message)

    // 加载当前用户信息
    {
        let current_user = current_user.clone();

        use_effect_with((), move |_| {
            spawn_local(async move {
                if let Some(auth_user) = get_auth_user() {
                    current_user.set(Some(auth_user));
                }
            });
            || ()
        });
    }

    let on_logout = {
        let current_page = current_page.clone();
        Callback::from(move |_| {
            if let Some(window) = web_sys::window() {
                let storage = window.local_storage().unwrap().unwrap();
                storage.delete("auth_token").unwrap();
                storage.delete("auth_user").unwrap();
            }
            current_page.set(Page::Login);
        })
    };

    let on_change_password = {
        let show_change_password = show_change_password.clone();
        let message = message.clone();
        Callback::from(move |_| {
            show_change_password.set(true);
            message.set(None);
        })
    };

    let on_close_modal = {
        let show_change_password = show_change_password.clone();
        let message = message.clone();
        Callback::from(move |_| {
            show_change_password.set(false);
            message.set(None);
        })
    };

    let on_submit_password = {
        let token = token.clone();
        let current_pwd = current_password.clone();
        let new_pwd = new_password.clone();
        let confirm_pwd = confirm_password.clone();
        let show_change_password = show_change_password.clone();
        let msg = message.clone();
        let lang = lang.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            // 验证密码
            if *new_pwd != *confirm_pwd {
                msg.set(Some(("is-danger".to_string(), lang.t("password_mismatch"))));
                return;
            }

            if new_pwd.len() < 6 {
                msg.set(Some(("is-danger".to_string(), "密码长度至少6位".to_string())));
                return;
            }

            let token = token.clone();
            let current_pwd = current_pwd.clone();
            let new_pwd = new_pwd.clone();
            let show_change_password = show_change_password.clone();
            let msg = msg.clone();
            let lang = lang.clone();

            spawn_local(async move {
                match Request::post(&api_url("users/change-password"))
                    .header("Authorization", &token)
                    .header("Content-Type", "application/json")
                    .body(
                        serde_json::to_string(&(serde_json::json!({
                            "current_password": current_pwd.as_str(),
                            "new_password": new_pwd.as_str(),
                        }))).unwrap_or_default()
                    )
                    .unwrap()
                    .send()
                    .await
                {
                    Ok(resp) if resp.ok() => {
                        msg.set(Some(("is-success".to_string(), lang.t("password_updated"))));
                        gloo_timers::callback::Timeout::new(1500, move || {
                            show_change_password.set(false);
                        }).forget();
                    }
                    Ok(resp) => {
                        if let Ok(text) = resp.text().await {
                            msg.set(Some(("is-danger".to_string(), text)));
                        }
                    }
                    Err(e) => {
                        msg.set(Some(("is-danger".to_string(), format!("请求失败: {:?}", e))));
                    }
                }
            });
        })
    };

    match current_user.as_ref() {
        None => html! {
            <div class="section">
                <div class="container">
                    <div class="has-text-centered">
                        <progress class="progress is-small is-info" max="100">{"30%"}</progress>
                        <p class="mt-4 has-text-grey">{"加载中..."}</p>
                    </div>
                </div>
            </div>
        },
        Some(user) => {
            let (role_display, role_color) = match &user.role {
                Role::SysAdmin => ("系统管理员", "is-primary"),
                Role::SecAdmin => ("安全管理员", "is-success"),
                Role::Auditor => ("审计员", "is-warning"),
                Role::Custom(name) => (name.as_str(), "is-info"),
            };

            let total_permissions = user.permissions.as_ref().map(|p| {
                let mut count = 0;
                if p.can_create_scan { count += 1; }
                if p.can_delete_scan { count += 1; }
                if p.can_export_scan { count += 1; }
                if p.can_view_cloud_assets { count += 1; }
                if p.can_create_cloud_asset { count += 1; }
                if p.can_update_cloud_asset { count += 1; }
                if p.can_delete_cloud_asset { count += 1; }
                if p.can_view_cloud_providers { count += 1; }
                if p.can_manage_cloud_providers { count += 1; }                if p.can_view_risks { count += 1; }
                if p.can_resolve_risk { count += 1; }
                if p.can_delete_risk { count += 1; }
                if p.can_view_users { count += 1; }
                if p.can_create_user { count += 1; }
                if p.can_update_user { count += 1; }
                if p.can_delete_user { count += 1; }
                if p.can_manage_permissions { count += 1; }
                if p.can_view_audit_logs { count += 1; }
                count
            }).unwrap_or(0);

            html! {
                <div class="section" style="background-color: #f5f7fa; min-height: 100vh;">
                    <div class="container">
                        // 页面标题
                        <div class="level mb-5">
                            <div class="level-left">
                                <h1 class="title is-3">
                                    <span class="icon mr-2"><i class="fas fa-user-circle"></i></span>
                                    { lang.t("user_profile") }
                                </h1>
                            </div>
                            <div class="level-right">
                                <button class="button is-danger is-outlined" onclick={on_logout}>
                                    <span class="icon"><i class="fas fa-sign-out-alt"></i></span>
                                    <span>{ lang.t("logout") }</span>
                                </button>
                            </div>
                        </div>

                        <div class="columns">
                            // 左侧：个人信息卡片
                            <div class="column is-4">
                                // 用户信息卡片
                                <div class="card mb-4">
                                    <div class="card-content">
                                        <div class="has-text-centered">
                                            <div class="image is-128x128 is-inline-block mb-3">
                                                <img class="is-rounded" 
                                                     src="https://bulma.io/images/placeholders/128x128.png" 
                                                     alt="User Avatar"
                                                     style="border: 4px solid #3273dc; box-shadow: 0 4px 6px rgba(0,0,0,0.1);" />
                                            </div>
                                            <h2 class="title is-4 mb-1">{ &user.username }</h2>
                                            <span class={classes!("tag", "is-medium", role_color)}>
                                                { role_display }
                                            </span>
                                        </div>

                                        <hr class="my-4" />

                                        <div class="content">
                                            <table class="table is-fullwidth is-borderless">
                                                <tbody>
                                                    <tr>
                                                        <td class="has-text-grey-light">{ "用户 ID" }</td>
                                                        <td class="has-text-right">
                                                            <code class="is-size-7">{ &user.id[..8] }{"..."}</code>
                                                        </td>
                                                    </tr>
                                                    <tr>
                                                        <td class="has-text-grey">{ "📅 " }{ lang.t("created_at") }</td>
                                                        <td class="has-text-right">
                                                            { format!("{}", user.created_at.format("%Y-%m-%d")) }
                                                        </td>
                                                    </tr>
                                                    <tr>
                                                        <td class="has-text-grey">{ "🔑 拥有权限" }</td>
                                                        <td class="has-text-right">
                                                            <span class="tag is-info">{ format!("{} 项", total_permissions) }</span>
                                                        </td>
                                                    </tr>
                                                </tbody>
                                            </table>
                                        </div>

                                        <div class="buttons is-centered">
                                            <button class="button is-info is-fullwidth" onclick={on_change_password}>
                                                <span class="icon"><i class="fas fa-key"></i></span>
                                                <span>{ lang.t("change_password") }</span>
                                            </button>
                                        </div>
                                    </div>
                                </div>

                                // 账户安全提示
                                <div class="card">
                                    <div class="card-content">
                                        <p class="title is-6 mb-2">
                                            <span class="icon has-text-info"><i class="fas fa-shield-alt"></i></span>
                                            { "安全提示" }
                                        </p>
                                        <div class="content is-small">
                                            <ul>
                                                <li>{"定期修改密码以保护账户安全"}</li>
                                                <li>{"不要与他人分享账户信息"}</li>
                                                <li>{"退出时记得点击退出登录"}</li>
                                            </ul>
                                        </div>
                                    </div>
                                </div>
                            </div>

                            // 右侧：权限信息
                            <div class="column is-8">
                                if let Some(permissions) = &user.permissions {
                                    <div class="columns is-multiline">
                                        // 扫描权限
                                        <div class="column is-6">
                                            <div class="card">
                                                <div class="card-header">
                                                    <p class="card-header-title">
                                                        <span class="icon mr-2"><i class="fas fa-search has-text-primary"></i></span>
                                                        {"扫描权限"}
                                                    </p>
                                                    <span class="card-header-icon">
                                                        <span class="tag is-light">{ 
                                                            format!("{}/3", 
                                                                [permissions.can_create_scan, permissions.can_delete_scan, permissions.can_export_scan]
                                                                .iter().filter(|&&x| x).count())
                                                            }</span>
                                                    </span>
                                                </div>
                                                <div class="card-content">
                                                    <div class="tags are-small">
                                                        { for [permissions.can_create_scan, permissions.can_delete_scan, permissions.can_export_scan]
                                                            .iter()
                                                            .zip(["创建扫描", "删除扫描", "导出结果"].iter())
                                                            .map(|(&enabled, name)| {
                                                                if enabled {
                                                                    html! { <span class="tag is-success is-light">{"✓ "}{ name }</span> }
                                                                } else {
                                                                    html! { <span class="tag is-danger is-light is-light">{"✗ "}{ name }</span> }
                                                                }
                                                            })
                                                        }
                                                    </div>
                                                </div>
                                            </div>
                                        </div>

                                        // 资产权限
                                        <div class="column is-6">
                                            <div class="card">
                                                <div class="card-header">
                                                    <p class="card-header-title">
                                                        <span class="icon mr-2"><i class="fas fa-box has-text-info"></i></span>
                                                        {"资产权限"}
                                                    </p>
                                                    <span class="card-header-icon">
                                                        <span class="tag is-light">{
                                                            format!("{}/4",
                                                                [permissions.can_view_cloud_assets, permissions.can_create_cloud_asset, permissions.can_update_cloud_asset, permissions.can_delete_cloud_asset]
                                                                .iter().filter(|&&x| x).count())
                                                            }</span>
                                                    </span>
                                                </div>
                                                <div class="card-content">
                                                    <div class="tags are-small">
                                                        { for [permissions.can_view_cloud_assets, permissions.can_create_cloud_asset, permissions.can_update_cloud_asset, permissions.can_delete_cloud_asset]
                                                            .iter()
                                                            .zip(["查看资产", "创建资产", "更新资产", "删除资产"].iter())
                                                            .map(|(&enabled, name)| {
                                                                if enabled {
                                                                    html! { <span class="tag is-success is-light">{"✓ "}{ name }</span> }
                                                                } else {
                                                                    html! { <span class="tag is-danger is-light">{"✗ "}{ name }</span> }
                                                                }
                                                            })
                                                        }
                                                    </div>
                                                </div>
                                            </div>
                                        </div>

                                        // 云资产权限
                                        <div class="column is-6">
                                            <div class="card">
                                                <div class="card-header">
                                                    <p class="card-header-title">
                                                        <span class="icon mr-2"><i class="fas fa-cloud has-text-link"></i></span>
                                                        {"云区对接权限"}
                                                    </p>
                                                    <span class="card-header-icon">
                                                        <span class="tag is-light">{
                                                            format!("{}/2",
                                                                [permissions.can_view_cloud_providers, permissions.can_manage_cloud_providers]
                                                                .iter().filter(|&&x| x).count())
                                                            }</span>
                                                    </span>
                                                </div>
                                                <div class="card-content">
                                                    <div class="tags are-small">
                                                        { for [permissions.can_view_cloud_providers, permissions.can_manage_cloud_providers]
                                                            .iter()
                                                            .zip(["查看云区对接", "管理云区对接"].iter())
                                                            .map(|(&enabled, name)| {
                                                                if enabled {
                                                                    html! { <span class="tag is-success is-light">{"✓ "}{ name }</span> }
                                                                } else {
                                                                    html! { <span class="tag is-danger is-light">{"✗ "}{ name }</span> }
                                                                }
                                                            })
                                                        }
                                                    </div>
                                                </div>
                                            </div>
                                        </div>

                                        // 混合云管理权限
                                        // 风险权限
                                        <div class="column is-6">
                                            <div class="card">
                                                <div class="card-header">
                                                    <p class="card-header-title">
                                                        <span class="icon mr-2"><i class="fas fa-exclamation-triangle has-text-warning"></i></span>
                                                        {"风险权限"}
                                                    </p>
                                                    <span class="card-header-icon">
                                                        <span class="tag is-light">{ 
                                                            format!("{}/3", 
                                                                [permissions.can_view_risks, permissions.can_resolve_risk, permissions.can_delete_risk]
                                                                .iter().filter(|&&x| x).count())
                                                            }</span>
                                                    </span>
                                                </div>
                                                <div class="card-content">
                                                    <div class="tags are-small">
                                                        { for [permissions.can_view_risks, permissions.can_resolve_risk, permissions.can_delete_risk]
                                                            .iter()
                                                            .zip(["查看风险", "处置风险", "删除风险"].iter())
                                                            .map(|(&enabled, name)| {
                                                                if enabled {
                                                                    html! { <span class="tag is-success is-light">{"✓ "}{ name }</span> }
                                                                } else {
                                                                    html! { <span class="tag is-danger is-light">{"✗ "}{ name }</span> }
                                                                }
                                                            })
                                                        }
                                                    </div>
                                                </div>
                                            </div>
                                        </div>

                                        // 用户管理权限
                                        <div class="column is-6">
                                            <div class="card">
                                                <div class="card-header">
                                                    <p class="card-header-title">
                                                        <span class="icon mr-2"><i class="fas fa-users-cog has-text-primary"></i></span>
                                                        {"用户管理权限"}
                                                    </p>
                                                    <span class="card-header-icon">
                                                        <span class="tag is-light">{ 
                                                            format!("{}/5", 
                                                                [permissions.can_view_users, permissions.can_create_user, permissions.can_update_user, permissions.can_delete_user, permissions.can_manage_permissions]
                                                                .iter().filter(|&&x| x).count())
                                                            }</span>
                                                    </span>
                                                </div>
                                                <div class="card-content">
                                                    <div class="tags are-small">
                                                        { for [permissions.can_view_users, permissions.can_create_user, permissions.can_update_user, permissions.can_delete_user, permissions.can_manage_permissions]
                                                            .iter()
                                                            .zip(["查看用户", "创建用户", "更新用户", "删除用户", "管理权限"].iter())
                                                            .map(|(&enabled, name)| {
                                                                if enabled {
                                                                    html! { <span class="tag is-success is-light">{"✓ "}{ name }</span> }
                                                                } else {
                                                                    html! { <span class="tag is-danger is-light">{"✗ "}{ name }</span> }
                                                                }
                                                            })
                                                        }
                                                    </div>
                                                </div>
                                            </div>
                                        </div>

                                        // 审计权限
                                        <div class="column is-6">
                                            <div class="card">
                                                <div class="card-header">
                                                    <p class="card-header-title">
                                                        <span class="icon mr-2"><i class="fas fa-clipboard-list has-text-grey"></i></span>
                                                        {"审计权限"}
                                                    </p>
                                                    <span class="card-header-icon">
                                                        <span class="tag is-light">{
                                                            format!("{}/1",
                                                                [permissions.can_view_audit_logs]
                                                                .iter().filter(|&&x| x).count())
                                                        }</span>
                                                    </span>
                                                </div>
                                                <div class="card-content">
                                                    <div class="tags are-small">
                                                        { for [permissions.can_view_audit_logs]
                                                            .iter()
                                                            .zip(["查看审计日志"].iter())
                                                            .map(|(&enabled, name)| {
                                                                if enabled {
                                                                    html! { <span class="tag is-success is-light">{"✓ "}{ name }</span> }
                                                                } else {
                                                                    html! { <span class="tag is-danger is-light">{"✗ "}{ name }</span> }
                                                                }
                                                            })
                                                        }
                                                    </div>
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                } else {
                                    <div class="notification is-warning is-light">
                                        <p class="has-text-centered">
                                            <span class="icon is-medium"><i class="fas fa-exclamation-circle"></i></span>
                                            <span class="ml-2">{"暂无权限信息配置"}</span>
                                        </p>
                                    </div>
                                }
                            </div>
                        </div>

                        // 修改密码对话框
                        if *show_change_password {
                            <div class="modal is-active">
                                <div class="modal-background" onclick={on_close_modal.clone()}></div>
                                <div class="modal-card" style="max-width: 450px;">
                                    <header class="modal-card-head">
                                        <p class="modal-card-title">
                                            <span class="icon mr-2"><i class="fas fa-key"></i></span>
                                            { lang.t("change_password") }
                                        </p>
                                        <button class="delete" aria-label="close" onclick={on_close_modal.clone()}></button>
                                    </header>
                                    <section class="modal-card-body">
                                        if let Some((msg_type, msg_text)) = message.as_ref() {
                                            <div class={classes!("notification", msg_type)}>
                                                <button class="delete" onclick={let message = message.clone(); move |_| message.set(None)}></button>
                                                { msg_text }
                                            </div>
                                        }

                                        <form onsubmit={on_submit_password}>
                                            <div class="field">
                                                <label class="label">{ lang.t("current_password") }</label>
                                                <div class="control has-icons-left">
                                                    <input
                                                        type="password"
                                                        class="input"
                                                        placeholder="请输入当前密码"
                                                        value={(*current_password).clone()}
                                                        onchange={let current_password = current_password.clone(); move |e: Event| {
                                                            let input = e.target_unchecked_into::<HtmlInputElement>();
                                                            current_password.set(input.value());
                                                        }}
                                                        required=true
                                                    />
                                                    <span class="icon is-small is-left">
                                                        <i class="fas fa-lock"></i>
                                                    </span>
                                                </div>
                                            </div>

                                            <div class="field">
                                                <label class="label">{ lang.t("new_password") }</label>
                                                <div class="control has-icons-left">
                                                    <input
                                                        type="password"
                                                        class="input"
                                                        placeholder="请输入新密码（至少6位）"
                                                        value={(*new_password).clone()}
                                                        onchange={let new_password = new_password.clone(); move |e: Event| {
                                                            let input = e.target_unchecked_into::<HtmlInputElement>();
                                                            new_password.set(input.value());
                                                        }}
                                                        required=true
                                                        minlength="6"
                                                    />
                                                    <span class="icon is-small is-left">
                                                        <i class="fas fa-key"></i>
                                                    </span>
                                                </div>
                                                <p class="help">{"密码长度至少 6 位字符"}</p>
                                            </div>

                                            <div class="field">
                                                <label class="label">{ lang.t("confirm_password") }</label>
                                                <div class="control has-icons-left">
                                                    <input
                                                        type="password"
                                                        class="input"
                                                        placeholder="请再次输入新密码"
                                                        value={(*confirm_password).clone()}
                                                        onchange={let confirm_password = confirm_password.clone(); move |e: Event| {
                                                            let input = e.target_unchecked_into::<HtmlInputElement>();
                                                            confirm_password.set(input.value());
                                                        }}
                                                        required=true
                                                        minlength="6"
                                                    />
                                                    <span class="icon is-small is-left">
                                                        <i class="fas fa-check-circle"></i>
                                                    </span>
                                                </div>
                                            </div>

                                            <div class="field mt-5">
                                                <div class="control">
                                                    <button type="submit" class="button is-primary is-fullwidth">
                                                        <span class="icon"><i class="fas fa-save"></i></span>
                                                        <span>{ lang.t("save") }</span>
                                                    </button>
                                                </div>
                                            </div>
                                        </form>
                                    </section>
                                    <footer class="modal-card-foot">
                                        <button class="button" onclick={on_close_modal.clone()}>{ lang.t("cancel") }</button>
                                    </footer>
                                </div>
                            </div>
                        }
                    </div>
                </div>
            }
        }
    }
}

// ============== Permission Management Component ==============

#[function_component]
fn PermissionManagement() -> Html {
    let lang = use_state(|| Language::Zh);
    let users = use_state(|| Vec::new());
    let loading = use_state(|| true);
    let show_edit_modal = use_state(|| false);
    let editing_user = use_state(|| None as Option<(String, String, Permissions)>);
    let temp_permissions = use_state(|| None as Option<Permissions>);
    let success_message = use_state(|| None as Option<String>);

    let token = get_auth_token();
    let user_role = get_user_role();

    // Fetch users
    use_effect_with((), {
        let users = users.clone();
        let loading = loading.clone();
        let token = token.clone();

        move |_| {
            spawn_local(async move {
                if let Ok(resp) = Request::get(&api_url("users"))
                    .header("Authorization", &token)
                    .send()
                    .await
                {
                    if let Ok(data) = resp.json::<Vec<User>>().await {
                        users.set(data);
                    }
                }
                loading.set(false);
            });
            || ()
        }
    });

    // Open edit modal
    let on_edit = {
        let users = users.clone();
        let show_edit_modal = show_edit_modal.clone();
        let editing_user = editing_user.clone();
        let temp_permissions = temp_permissions.clone();

        Callback::from(move |user_id: String| {
            if let Some(user) = users.iter().find(|u| u.id == user_id) {
                if let Some(perms) = &user.permissions {
                    editing_user.set(Some((user.id.clone(), user.username.clone(), perms.clone())));
                    temp_permissions.set(Some(perms.clone()));
                    show_edit_modal.set(true);
                }
            }
        })
    };

    // Close modal
    let on_close_modal = {
        let show_edit_modal = show_edit_modal.clone();
        Callback::from(move |_| {
            show_edit_modal.set(false);
        })
    };

    // Save permissions
    let on_save = {
        let token = token.clone();
        let editing_user = editing_user.clone();
        let temp_permissions = temp_permissions.clone();
        let show_edit_modal = show_edit_modal.clone();
        let success_message = success_message.clone();

        Callback::from(move |_| {
            if let Some((user_id, _, _)) = &*editing_user {
                if let Some(perms) = &*temp_permissions {
                    let token = token.clone();
                    let user_id = user_id.clone();
                    let perms = perms.clone();
                    let show_edit_modal = show_edit_modal.clone();
                    let success_message = success_message.clone();

                    spawn_local(async move {
                        if let Ok(resp) = Request::put(&format!("{}/users/{}/permissions", api_url(""), user_id))
                            .header("Authorization", &token)
                            .header("Content-Type", "application/json")
                            .body(serde_json::to_string(&perms).unwrap_or_default())
                            .unwrap()
                            .send()
                            .await
                        {
                            if resp.ok() {
                                success_message.set(Some("权限更新成功".to_string()));
                                show_edit_modal.set(false);
                            } else {
                                success_message.set(Some("权限更新失败".to_string()));
                            }
                        }
                    });
                }
            }
        })
    };

    // Toggle permission
    let on_toggle_permission = {
        let temp_permissions = temp_permissions.clone();
        Callback::from(move |(perm_key, value): (String, bool)| {
            if let Some(mut perms) = (*temp_permissions).clone() {
                match perm_key.as_str() {
                    // 顶级权限
                    "can_access_general" => perms.can_access_general = value,
                    "can_view_business_process" => perms.can_view_business_process = value,
                    "can_access_cloud" => perms.can_access_cloud = value,
                    "can_access_user_management" => perms.can_access_user_management = value,
                    "can_access_audit" => perms.can_access_audit = value,
                    // 子级权限 - 通用模块
                    "can_view_dashboard" => perms.can_view_dashboard = value,
                    "can_view_tasks" => perms.can_view_tasks = value,
                    "can_view_advanced_scan" => perms.can_view_advanced_scan = value,
                    "can_view_risks" => perms.can_view_risks = value,
                    // 子级权限 - 业务流程
                    "can_view_business_applications" => perms.can_view_business_applications = value,
                    "can_create_business_application" => perms.can_create_business_application = value,
                    "can_approve_business_application" => perms.can_approve_business_application = value,
                    "can_supplement_business_application" => perms.can_supplement_business_application = value,
                    "can_delete_business_application" => perms.can_delete_business_application = value,
                    "can_view_operations_management" => perms.can_view_operations_management = value,
                    "can_manage_operations" => perms.can_manage_operations = value,
                    "can_view_automation_orchestration" => perms.can_view_automation_orchestration = value,
                    "can_execute_orchestration" => perms.can_execute_orchestration = value,
                    "can_manage_orchestration" => perms.can_manage_orchestration = value,
                    // 子级权限 - 云管理
                    "can_view_cloud_providers" => perms.can_view_cloud_providers = value,
                    "can_view_cloud_assets" => perms.can_view_cloud_assets = value,
                    // 子级权限 - 用户管理
                    "can_view_users" => perms.can_view_users = value,
                    "can_view_password_policy" => perms.can_view_password_policy = value,
                    "can_manage_permissions" => perms.can_manage_permissions = value,
                    // 子级权限 - 审计日志
                    "can_view_audit_logs" => perms.can_view_audit_logs = value,
                    _ => {}
                }
                temp_permissions.set(Some(perms));
            }
        })
    };

    html! {
        <div class="container p-4">
            <div class="is-flex is-justify-content-space-between is-align-items-center mb-4">
                <h1 class="title">{ lang.t("permission_management") }</h1>
            </div>

            if let Some(msg) = &*success_message {
                <div class="notification is-success is-light" style="margin-bottom: 1rem; position: relative;">
                    <button class="delete" onclick={
                        let success_message = success_message.clone();
                        Callback::from(move |_| success_message.set(None))
                    }></button>
                    <p>{ msg.clone() }</p>
                </div>
            }

            <div class="box">
                if *loading {
                    <div class="has-text-centered">
                        <span class="icon is-large">
                            <i class="fas fa-spinner fa-spin fa-2x"></i>
                        </span>
                        <p>{"加载中..."}</p>
                    </div>
                } else {
                    <table class="table is-fullwidth is-hoverable">
                        <thead>
                            <tr>
                                <th>{"用户名"}</th>
                                <th>{"角色"}</th>
                                <th>{"通用"}</th>
                                <th>{"业务流程"}</th>
                                <th>{"云管理"}</th>
                                <th>{"用户管理"}</th>
                                <th>{"审计日志"}</th>
                                <th>{"操作"}</th>
                            </tr>
                        </thead>
                        <tbody>
                            { for users.iter().map(|user| {
                                let role_display = match &user.role {
                                    Role::SysAdmin => "SysAdmin",
                                    Role::SecAdmin => "SecAdmin",
                                    Role::Auditor => "Auditor",
                                    Role::Custom(name) => name,
                                };
                                let perms = user.permissions.as_ref();
                                let on_edit = on_edit.clone();
                                let user_id = user.id.clone();

                                html! {
                                    <tr key={user_id.clone()}>
                                        <td>{ &user.username }</td>
                                        <td><span class="tag is-info">{ role_display }</span></td>
                                        <td>
                                            { perms.map(|p| {
                                                if p.can_access_general || p.can_view_tasks || p.can_view_advanced_scan || p.can_view_risks {
                                                    html! { <span class="icon has-text-success"><i class="fas fa-check"></i></span> }
                                                } else {
                                                    html! { <span class="icon has-text-danger"><i class="fas fa-times"></i></span> }
                                                }
                                            }).unwrap_or_else(|| html! { <span class="icon"><i class="fas fa-minus"></i></span> }) }
                                        </td>
                                        <td>
                                            { perms.map(|p| {
                                                if p.can_view_business_process {
                                                    html! { <span class="icon has-text-success"><i class="fas fa-check"></i></span> }
                                                } else {
                                                    html! { <span class="icon has-text-danger"><i class="fas fa-times"></i></span> }
                                                }
                                            }).unwrap_or_else(|| html! { <span class="icon"><i class="fas fa-minus"></i></span> }) }
                                        </td>
                                        <td>
                                            { perms.map(|p| {
                                                if p.can_access_cloud {
                                                    html! { <span class="icon has-text-success"><i class="fas fa-check"></i></span> }
                                                } else {
                                                    html! { <span class="icon has-text-danger"><i class="fas fa-times"></i></span> }
                                                }
                                            }).unwrap_or_else(|| html! { <span class="icon"><i class="fas fa-minus"></i></span> }) }
                                        </td>
                                        <td>
                                            { perms.map(|p| {
                                                if p.can_access_user_management {
                                                    html! { <span class="icon has-text-success"><i class="fas fa-check"></i></span> }
                                                } else {
                                                    html! { <span class="icon has-text-danger"><i class="fas fa-times"></i></span> }
                                                }
                                            }).unwrap_or_else(|| html! { <span class="icon"><i class="fas fa-minus"></i></span> }) }
                                        </td>
                                        <td>
                                            { perms.map(|p| {
                                                if p.can_access_audit {
                                                    html! { <span class="icon has-text-success"><i class="fas fa-check"></i></span> }
                                                } else {
                                                    html! { <span class="icon has-text-danger"><i class="fas fa-times"></i></span> }
                                                }
                                            }).unwrap_or_else(|| html! { <span class="icon"><i class="fas fa-minus"></i></span> }) }
                                        </td>
                                        <td>
                                            if user.role != Role::SysAdmin {
                                                <button class="button is-small is-info" onclick={
                                                    let on_edit = on_edit.clone();
                                                    move |_| on_edit.emit(user_id.clone())
                                                }>
                                                    <span class="icon"><i class="fas fa-edit"></i></span>
                                                    <span>{"编辑"}</span>
                                                </button>
                                            }
                                        </td>
                                    </tr>
                                }
                            })}
                        </tbody>
                    </table>
                }
            </div>

            // Edit Modal
            if *show_edit_modal {
                if let Some((user_id, username, perms)) = &*editing_user {
                    <div class="modal is-active">
                        <div class="modal-background" onclick={on_close_modal.clone()}></div>
                        <div class="modal-card" style="width: 900px;">
                            <header class="modal-card-head">
                                <p class="modal-card-title">{ format!("编辑用户权限: {}", username) }</p>
                                <button class="delete" onclick={on_close_modal.clone()}></button>
                            </header>
                            <section class="modal-card-body" style="max-height: 70vh; overflow-y: auto;">
                                if let Some(current_perms) = &*temp_permissions {
                                    <div class="content">
                                        // ========== 通用模块 ==========
                                        <div class="box has-background-light">
                                            <h4 class="title is-6 mb-3">{"📋 通用模块"}</h4>
                                            <div class="field" style="margin-bottom: 0.5rem;">
                                                <label class="checkbox">
                                                    <input type="checkbox"
                                                        checked={current_perms.can_access_general}
                                                        onchange={let on_toggle = on_toggle_permission.clone(); move |e: Event| {
                                                            let input = e.target_unchecked_into::<HtmlInputElement>();
                                                            on_toggle.emit(("can_access_general".to_string(), input.checked()))
                                                        }}
                                                    />
                                                    <strong>{ " 访问通用模块" }</strong>
                                                </label>
                                                <p class="help is-size-7 ml-5">{"控制是否可以访问通用模块下的所有功能"}</p>
                                            </div>
                                            <div class="ml-5">
                                                <div class="field" style="margin-bottom: 0.5rem;">
                                                    <label class="checkbox">
                                                        <input type="checkbox"
                                                            checked={current_perms.can_view_dashboard}
                                                            onchange={let on_toggle = on_toggle_permission.clone(); move |e: Event| {
                                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                                on_toggle.emit(("can_view_dashboard".to_string(), input.checked()))
                                                            }}
                                                        />
                                                        { " 仪表盘查看" }
                                                    </label>
                                                </div>
                                                <div class="field" style="margin-bottom: 0.5rem;">
                                                    <label class="checkbox">
                                                        <input type="checkbox"
                                                            checked={current_perms.can_view_tasks}
                                                            onchange={let on_toggle = on_toggle_permission.clone(); move |e: Event| {
                                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                                on_toggle.emit(("can_view_tasks".to_string(), input.checked()))
                                                            }}
                                                        />
                                                        { " 任务中心查看" }
                                                    </label>
                                                </div>
                                                <div class="field" style="margin-bottom: 0.5rem;">
                                                    <label class="checkbox">
                                                        <input type="checkbox"
                                                            checked={current_perms.can_view_advanced_scan}
                                                            onchange={let on_toggle = on_toggle_permission.clone(); move |e: Event| {
                                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                                on_toggle.emit(("can_view_advanced_scan".to_string(), input.checked()))
                                                            }}
                                                        />
                                                        { " 高级扫描查看" }
                                                    </label>
                                                </div>
                                                <div class="field" style="margin-bottom: 0.5rem;">
                                                    <label class="checkbox">
                                                        <input type="checkbox"
                                                            checked={current_perms.can_view_risks}
                                                            onchange={let on_toggle = on_toggle_permission.clone(); move |e: Event| {
                                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                                on_toggle.emit(("can_view_risks".to_string(), input.checked()))
                                                            }}
                                                        />
                                                        { " 风险监控查看" }
                                                    </label>
                                                </div>
                                            </div>
                                        </div>

                                        // ========== 业务流程 ==========
                                        <div class="box has-background-light">
                                            <h4 class="title is-6 mb-3">{"🔄 业务流程"}</h4>
                                            <div class="field" style="margin-bottom: 0.5rem;">
                                                <label class="checkbox">
                                                    <input type="checkbox"
                                                        checked={current_perms.can_view_business_process}
                                                        onchange={let on_toggle = on_toggle_permission.clone(); move |e: Event| {
                                                            let input = e.target_unchecked_into::<HtmlInputElement>();
                                                            on_toggle.emit(("can_view_business_process".to_string(), input.checked()))
                                                        }}
                                                    />
                                                    <strong>{ " 访问业务流程" }</strong>
                                                </label>
                                                <p class="help is-size-7 ml-5">{"控制是否可以访问业务流程管理功能"}</p>
                                            </div>
                                            <div class="ml-5">
                                                <div class="field" style="margin-bottom: 0.5rem;">
                                                    <label class="checkbox">
                                                        <input type="checkbox"
                                                            checked={current_perms.can_view_business_applications}
                                                            onchange={let on_toggle = on_toggle_permission.clone(); move |e: Event| {
                                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                                on_toggle.emit(("can_view_business_applications".to_string(), input.checked()))
                                                            }}
                                                        />
                                                        { " 📝 云资源申请查看" }
                                                    </label>
                                                </div>
                                                <div class="field" style="margin-bottom: 0.5rem;">
                                                    <label class="checkbox">
                                                        <input type="checkbox"
                                                            checked={current_perms.can_create_business_application}
                                                            onchange={let on_toggle = on_toggle_permission.clone(); move |e: Event| {
                                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                                on_toggle.emit(("can_create_business_application".to_string(), input.checked()))
                                                            }}
                                                        />
                                                        { " 创建云资源申请" }
                                                    </label>
                                                </div>
                                                <div class="field" style="margin-bottom: 0.5rem;">
                                                    <label class="checkbox">
                                                        <input type="checkbox"
                                                            checked={current_perms.can_supplement_business_application}
                                                            onchange={let on_toggle = on_toggle_permission.clone(); move |e: Event| {
                                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                                on_toggle.emit(("can_supplement_business_application".to_string(), input.checked()))
                                                            }}
                                                        />
                                                        { " 补充云资源申请信息" }
                                                    </label>
                                                </div>
                                                <div class="field" style="margin-bottom: 0.5rem;">
                                                    <label class="checkbox">
                                                        <input type="checkbox"
                                                            checked={current_perms.can_delete_business_application}
                                                            onchange={let on_toggle = on_toggle_permission.clone(); move |e: Event| {
                                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                                on_toggle.emit(("can_delete_business_application".to_string(), input.checked()))
                                                            }}
                                                        />
                                                        { " 删除云资源申请" }
                                                    </label>
                                                </div>
                                                <div class="field" style="margin-bottom: 0.5rem;">
                                                    <label class="checkbox">
                                                        <input type="checkbox"
                                                            checked={current_perms.can_view_operations_management}
                                                            onchange={let on_toggle = on_toggle_permission.clone(); move |e: Event| {
                                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                                on_toggle.emit(("can_view_operations_management".to_string(), input.checked()))
                                                            }}
                                                        />
                                                        { " 🔧 运维管理查看" }
                                                    </label>
                                                </div>
                                                <div class="field" style="margin-bottom: 0.5rem;">
                                                    <label class="checkbox">
                                                        <input type="checkbox"
                                                            checked={current_perms.can_manage_operations}
                                                            onchange={let on_toggle = on_toggle_permission.clone(); move |e: Event| {
                                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                                on_toggle.emit(("can_manage_operations".to_string(), input.checked()))
                                                            }}
                                                        />
                                                        { " 运维操作权限" }
                                                    </label>
                                                </div>
                                                <div class="field" style="margin-bottom: 0.5rem;">
                                                    <label class="checkbox">
                                                        <input type="checkbox"
                                                            checked={current_perms.can_view_automation_orchestration}
                                                            onchange={let on_toggle = on_toggle_permission.clone(); move |e: Event| {
                                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                                on_toggle.emit(("can_view_automation_orchestration".to_string(), input.checked()))
                                                            }}
                                                        />
                                                        { " ⚙️ 自动化资源编排查看" }
                                                    </label>
                                                </div>
                                                <div class="field" style="margin-bottom: 0.5rem;">
                                                    <label class="checkbox">
                                                        <input type="checkbox"
                                                            checked={current_perms.can_execute_orchestration}
                                                            onchange={let on_toggle = on_toggle_permission.clone(); move |e: Event| {
                                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                                on_toggle.emit(("can_execute_orchestration".to_string(), input.checked()))
                                                            }}
                                                        />
                                                        { " 执行编排任务" }
                                                    </label>
                                                </div>
                                                <div class="field" style="margin-bottom: 0.5rem;">
                                                    <label class="checkbox">
                                                        <input type="checkbox"
                                                            checked={current_perms.can_manage_orchestration}
                                                            onchange={let on_toggle = on_toggle_permission.clone(); move |e: Event| {
                                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                                on_toggle.emit(("can_manage_orchestration".to_string(), input.checked()))
                                                            }}
                                                        />
                                                        { " 管理编排任务" }
                                                    </label>
                                                </div>
                                            </div>
                                        </div>

                                        // ========== 云管理 ==========
                                        <div class="box has-background-light">
                                            <h4 class="title is-6 mb-3">{"☁️ 云管理"}</h4>
                                            <div class="field" style="margin-bottom: 0.5rem;">
                                                <label class="checkbox">
                                                    <input type="checkbox"
                                                        checked={current_perms.can_access_cloud}
                                                        onchange={let on_toggle = on_toggle_permission.clone(); move |e: Event| {
                                                            let input = e.target_unchecked_into::<HtmlInputElement>();
                                                            on_toggle.emit(("can_access_cloud".to_string(), input.checked()))
                                                        }}
                                                    />
                                                    <strong>{ " 访问云管理模块" }</strong>
                                                </label>
                                                <p class="help is-size-7 ml-5">{"控制是否可以访问云管理模块下的所有功能"}</p>
                                            </div>
                                            <div class="ml-5">
                                                <div class="field" style="margin-bottom: 0.5rem;">
                                                    <label class="checkbox">
                                                        <input type="checkbox"
                                                            checked={current_perms.can_view_cloud_providers}
                                                            onchange={let on_toggle = on_toggle_permission.clone(); move |e: Event| {
                                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                                on_toggle.emit(("can_view_cloud_providers".to_string(), input.checked()))
                                                            }}
                                                        />
                                                        { " 云服务商管理查看" }
                                                    </label>
                                                </div>
                                                <div class="field" style="margin-bottom: 0.5rem;">
                                                    <label class="checkbox">
                                                        <input type="checkbox"
                                                            checked={current_perms.can_view_cloud_assets}
                                                            onchange={let on_toggle = on_toggle_permission.clone(); move |e: Event| {
                                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                                on_toggle.emit(("can_view_cloud_assets".to_string(), input.checked()))
                                                            }}
                                                        />
                                                        { " 云服务资产查看" }
                                                    </label>
                                                </div>
                                            </div>
                                        </div>

                                        // ========== 用户管理 ==========
                                        <div class="box has-background-light">
                                            <h4 class="title is-6 mb-3">{"👥 用户管理"}</h4>
                                            <div class="field" style="margin-bottom: 0.5rem;">
                                                <label class="checkbox">
                                                    <input type="checkbox"
                                                        checked={current_perms.can_access_user_management}
                                                        onchange={let on_toggle = on_toggle_permission.clone(); move |e: Event| {
                                                            let input = e.target_unchecked_into::<HtmlInputElement>();
                                                            on_toggle.emit(("can_access_user_management".to_string(), input.checked()))
                                                        }}
                                                    />
                                                    <strong>{ " 访问用户管理模块" }</strong>
                                                </label>
                                                <p class="help is-size-7 ml-5">{"控制是否可以访问用户管理模块下的所有功能"}</p>
                                            </div>
                                            <div class="ml-5">
                                                <div class="field" style="margin-bottom: 0.5rem;">
                                                    <label class="checkbox">
                                                        <input type="checkbox"
                                                            checked={current_perms.can_view_users}
                                                            onchange={let on_toggle = on_toggle_permission.clone(); move |e: Event| {
                                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                                on_toggle.emit(("can_view_users".to_string(), input.checked()))
                                                            }}
                                                        />
                                                        { " 用户管理查看" }
                                                    </label>
                                                </div>
                                                <div class="field" style="margin-bottom: 0.5rem;">
                                                    <label class="checkbox">
                                                        <input type="checkbox"
                                                            checked={current_perms.can_view_password_policy}
                                                            onchange={let on_toggle = on_toggle_permission.clone(); move |e: Event| {
                                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                                on_toggle.emit(("can_view_password_policy".to_string(), input.checked()))
                                                            }}
                                                        />
                                                        { " 密码策略查看" }
                                                    </label>
                                                </div>
                                                <div class="field" style="margin-bottom: 0.5rem;">
                                                    <label class="checkbox">
                                                        <input type="checkbox"
                                                            checked={current_perms.can_manage_permissions}
                                                            onchange={let on_toggle = on_toggle_permission.clone(); move |e: Event| {
                                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                                on_toggle.emit(("can_manage_permissions".to_string(), input.checked()))
                                                            }}
                                                        />
                                                        { " 权限管理（可编辑其他用户权限）" }
                                                    </label>
                                                </div>
                                            </div>
                                        </div>

                                        // ========== 审计日志 ==========
                                        <div class="box has-background-light">
                                            <h4 class="title is-6 mb-3">{"📜 审计日志"}</h4>
                                            <div class="field" style="margin-bottom: 0.5rem;">
                                                <label class="checkbox">
                                                    <input type="checkbox"
                                                        checked={current_perms.can_access_audit}
                                                        onchange={let on_toggle = on_toggle_permission.clone(); move |e: Event| {
                                                            let input = e.target_unchecked_into::<HtmlInputElement>();
                                                            on_toggle.emit(("can_access_audit".to_string(), input.checked()))
                                                        }}
                                                    />
                                                    <strong>{ " 访问审计日志" }</strong>
                                                </label>
                                                <p class="help is-size-7 ml-5">{"控制是否可以访问审计日志功能"}</p>
                                            </div>
                                            <div class="ml-5">
                                                <div class="field" style="margin-bottom: 0.5rem;">
                                                    <label class="checkbox">
                                                        <input type="checkbox"
                                                            checked={current_perms.can_view_audit_logs}
                                                            onchange={let on_toggle = on_toggle_permission.clone(); move |e: Event| {
                                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                                on_toggle.emit(("can_view_audit_logs".to_string(), input.checked()))
                                                            }}
                                                        />
                                                        { " 审计日志查看" }
                                                    </label>
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                }
                            </section>
                            <footer class="modal-card-foot">
                                <button class="button is-success" onclick={on_save}>{ "保存" }</button>
                                <button class="button" onclick={on_close_modal}>{ "取消" }</button>
                            </footer>
                        </div>
                    </div>
                }
            }
        </div>
    }
}

// ============== Password Policy Management Component ==============

#[function_component]
fn PasswordPolicyManagement() -> Html {
    let lang = use_state(|| Language::Zh);
    let policy = use_state(|| None as Option<PasswordPolicy>);
    let loading = use_state(|| true);
    let message = use_state(|| None as Option<(String, String)>); // (type, message)
    let token = get_auth_token();

    // Form state
    let min_length = use_state(|| 6u32);
    let require_uppercase = use_state(|| false);
    let require_lowercase = use_state(|| false);
    let require_number = use_state(|| false);
    let require_special = use_state(|| false);
    let max_age_days = use_state(|| Some(90u32));  // 默认90天
    let prevent_reuse = use_state(|| 3u32);
    let min_strength = use_state(|| "weak".to_string());
    let max_login_attempts = use_state(|| Some(5u32));
    let lockout_duration_minutes = use_state(|| 30u32);

    // Load current policy
    {
        let policy = policy.clone();
        let loading = loading.clone();
        let token = token.clone();
        let min_length = min_length.clone();
        let require_uppercase = require_uppercase.clone();
        let require_lowercase = require_lowercase.clone();
        let require_number = require_number.clone();
        let require_special = require_special.clone();
        let max_age_days = max_age_days.clone();
        let prevent_reuse = prevent_reuse.clone();
        let min_strength = min_strength.clone();
        let max_login_attempts = max_login_attempts.clone();
        let lockout_duration_minutes = lockout_duration_minutes.clone();
        let msg = message.clone();
        let lang_clone = lang.clone();

        use_effect_with((), move |_| {
            spawn_local(async move {
                match Request::get(&api_url("password-policy"))
                    .header("Authorization", &token)
                    .send()
                    .await
                {
                    Ok(response) => {
                        if response.ok() {
                            if let Ok(p) = response.json::<PasswordPolicy>().await {
                                min_length.set(p.min_length);
                                require_uppercase.set(p.require_uppercase);
                                require_lowercase.set(p.require_lowercase);
                                require_number.set(p.require_number);
                                require_special.set(p.require_special);
                                max_age_days.set(p.max_age_days);
                                prevent_reuse.set(p.prevent_reuse);
                                min_strength.set(p.min_strength.clone());
                                max_login_attempts.set(p.max_login_attempts);
                                lockout_duration_minutes.set(p.lockout_duration_minutes);
                                policy.set(Some(p));
                            }
                        } else {
                            msg.set(Some(("is-danger".to_string(), lang_clone.t("policy_error"))));
                        }
                    }
                    Err(_) => {
                        msg.set(Some(("is-danger".to_string(), lang_clone.t("policy_error"))));
                    }
                }
                loading.set(false);
            });
            || ()
        });
    }

    let on_save = {
        let token = token.clone();
        let policy_state = policy.clone();
        let min_length = min_length.clone();
        let require_uppercase = require_uppercase.clone();
        let require_lowercase = require_lowercase.clone();
        let require_number = require_number.clone();
        let require_special = require_special.clone();
        let max_age_days = max_age_days.clone();
        let prevent_reuse = prevent_reuse.clone();
        let min_strength = min_strength.clone();
        let max_login_attempts = max_login_attempts.clone();
        let lockout_duration_minutes = lockout_duration_minutes.clone();
        let msg = message.clone();
        let lang = lang.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            let new_policy = PasswordPolicy {
                min_length: *min_length,
                require_uppercase: *require_uppercase,
                require_lowercase: *require_lowercase,
                require_number: *require_number,
                require_special: *require_special,
                max_age_days: *max_age_days,
                prevent_reuse: *prevent_reuse,
                min_strength: (*min_strength).clone(),
                max_login_attempts: *max_login_attempts,
                lockout_duration_minutes: *lockout_duration_minutes,
            };

            let token = token.clone();
            let policy_state = policy_state.clone();
            let msg = msg.clone();
            let lang = lang.clone();

            spawn_local(async move {
                match Request::put(&api_url("password-policy"))
                    .header("Authorization", &token)
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_string(&new_policy).unwrap_or_default())
                    .unwrap()
                    .send()
                    .await
                {
                    Ok(response) => {
                        if response.ok() {
                            if let Ok(p) = response.json::<PasswordPolicy>().await {
                                policy_state.set(Some(p));
                                msg.set(Some(("is-success".to_string(), lang.t("policy_saved"))));
                            }
                        } else {
                            msg.set(Some(("is-danger".to_string(), lang.t("policy_error"))));
                        }
                    }
                    Err(_) => {
                        msg.set(Some(("is-danger".to_string(), lang.t("policy_error"))));
                    }
                }
            });
        })
    };

    let on_reset = {
        let policy = policy.clone();
        let min_length = min_length.clone();
        let require_uppercase = require_uppercase.clone();
        let require_lowercase = require_lowercase.clone();
        let require_number = require_number.clone();
        let require_special = require_special.clone();
        let max_age_days = max_age_days.clone();
        let prevent_reuse = prevent_reuse.clone();
        let min_strength = min_strength.clone();
        let max_login_attempts = max_login_attempts.clone();
        let lockout_duration_minutes = lockout_duration_minutes.clone();

        Callback::from(move |_| {
            if let Some(p) = (*policy).clone() {
                min_length.set(p.min_length);
                require_uppercase.set(p.require_uppercase);
                require_lowercase.set(p.require_lowercase);
                require_number.set(p.require_number);
                require_special.set(p.require_special);
                max_age_days.set(p.max_age_days);
                prevent_reuse.set(p.prevent_reuse);
                min_strength.set(p.min_strength);
                max_login_attempts.set(p.max_login_attempts);
                lockout_duration_minutes.set(p.lockout_duration_minutes);
            }
        })
    };

    html! {
        <div class="container p-4">
            <h1 class="title">{ lang.t("password_policy_management") }</h1>

            if *loading {
                <div class="notification is-info">
                    { "Loading..." }
                </div>
            } else {
                if let Some((msg_type, msg_text)) = message.as_ref() {
                    <div class={classes!("notification", msg_type)}>
                        <button class="delete" onclick={let message = message.clone(); move |_| message.set(None)}></button>
                        { msg_text }
                    </div>
                }

                <div class="box">
                    <form onsubmit={on_save}>
                        <div class="columns">
                            // Left column - basic settings
                            <div class="column is-6">
                                <div class="field">
                                    <label class="label">{ lang.t("min_length") }</label>
                                    <div class="control">
                                        <input
                                            type="number"
                                            class="input"
                                            min="1"
                                            max="128"
                                            value={(*min_length).to_string()}
                                            onchange={let min_length = min_length.clone(); move |e: Event| {
                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                if let Ok(val) = input.value().parse::<u32>() {
                                                    min_length.set(val);
                                                }
                                            }}
                                        />
                                    </div>
                                    <p class="help">{"密码的最小长度（1-128位）"}</p>
                                </div>

                                <div class="field">
                                    <label class="label">{ lang.t("max_age_days") }</label>
                                    <div class="control">
                                        <input
                                            type="number"
                                            class="input"
                                            min="0"
                                            placeholder="0表示不限制"
                                            value={max_age_days.map(|d| d.to_string()).unwrap_or_else(|| "0".to_string())}
                                            onchange={let max_age_days = max_age_days.clone(); move |e: Event| {
                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                let val = input.value();
                                                if let Ok(d) = val.parse::<u32>() {
                                                    // 0 表示不限制，设置为 None
                                                    if d == 0 {
                                                        max_age_days.set(None);
                                                    } else {
                                                        max_age_days.set(Some(d));
                                                    }
                                                }
                                            }}
                                        />
                                    </div>
                                    <p class="help">{"密码最大有效期（天），输入 0 表示不限制"}</p>
                                </div>

                                <div class="field">
                                    <label class="label">{ lang.t("prevent_reuse") }</label>
                                    <div class="control">
                                        <input
                                            type="number"
                                            class="input"
                                            min="0"
                                            max="50"
                                            value={(*prevent_reuse).to_string()}
                                            onchange={let prevent_reuse = prevent_reuse.clone(); move |e: Event| {
                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                if let Ok(val) = input.value().parse::<u32>() {
                                                    prevent_reuse.set(val);
                                                }
                                            }}
                                        />
                                    </div>
                                    <p class="help">{"防止用户重用最近 N 次的旧密码"}</p>
                                </div>

                                <div class="field">
                                    <label class="label">{ lang.t("min_strength") }</label>
                                    <div class="control has-icons-left">
                                        <div class="select is-fullwidth">
                                            <select
                                                value={(*min_strength).clone()}
                                                onchange={let min_strength = min_strength.clone(); move |e: Event| {
                                                    let select = e.target_unchecked_into::<HtmlSelectElement>();
                                                    min_strength.set(select.value());
                                                }}
                                            >
                                                <option value="weak">{ lang.t("weak") }</option>
                                                <option value="medium">{ lang.t("medium") }</option>
                                                <option value="strong">{ lang.t("strong") }</option>
                                            </select>
                                        </div>
                                        <span class="icon is-small is-left">
                                            <i class="fas fa-shield-alt"></i>
                                        </span>
                                    </div>
                                    <p class="help">{"密码最低强度要求"}</p>
                                    <div class="box has-background-info-light has-text-info mt-2 p-3">
                                        <p class="is-size-7 mb-1"><strong>{"密码强度说明："}</strong></p>
                                        <ul class="is-size-7 ml-4 mt-1">
                                            <li>{"🔴 弱：长度 < 8 位，或字符类型单一"}</li>
                                            <li>{"🟡 中：长度 ≥ 8 位，包含 2-3 种字符类型（大小写、数字、特殊字符）"}</li>
                                            <li>{"🟢 强：长度 ≥ 12 位，包含所有字符类型（大小写、数字、特殊字符）"}</li>
                                        </ul>
                                    </div>
                                </div>

                                <hr class="has-background-grey-light mt-4 mb-4" />

                                <h4 class="title is-6 has-text-primary">{ lang.t("account_lockout") }</h4>

                                <div class="field">
                                    <label class="label">{ lang.t("max_login_attempts") }</label>
                                    <div class="control has-icons-left">
                                        <input
                                            type="number"
                                            class="input"
                                            min="0"
                                            placeholder="留空表示不限制"
                                            value={max_login_attempts.map(|n| n.to_string()).unwrap_or_default()}
                                            onchange={let max_login_attempts = max_login_attempts.clone(); move |e: Event| {
                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                let val = input.value();
                                                if val.is_empty() {
                                                    max_login_attempts.set(None);
                                                } else if let Ok(n) = val.parse::<u32>() {
                                                    max_login_attempts.set(Some(n));
                                                }
                                            }}
                                        />
                                        <span class="icon is-small is-left">
                                            <i class="fas fa-user-lock"></i>
                                        </span>
                                    </div>
                                    <p class="help">{format!("登录失败多少次后锁定账户，0 或留空表示{}", lang.t("no_limit"))}</p>
                                </div>

                                <div class="field">
                                    <label class="label">{ lang.t("lockout_duration_minutes") }</label>
                                    <div class="control has-icons-left">
                                        <input
                                            type="number"
                                            class="input"
                                            min="1"
                                            max="1440"
                                            value={(*lockout_duration_minutes).to_string()}
                                            onchange={let lockout_duration_minutes = lockout_duration_minutes.clone(); move |e: Event| {
                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                if let Ok(val) = input.value().parse::<u32>() {
                                                    lockout_duration_minutes.set(val.max(1));
                                                }
                                            }}
                                        />
                                        <span class="icon is-small is-left">
                                            <i class="fas fa-clock"></i>
                                        </span>
                                    </div>
                                    <p class="help">{"账户锁定后多久自动解锁（分钟）"}</p>
                                </div>
                            </div>

                            // Right column - character requirements
                            <div class="column is-6">
                                <div class="field">
                                    <label class="checkbox">
                                        <input
                                            type="checkbox"
                                            checked={*require_uppercase}
                                            onchange={let require_uppercase = require_uppercase.clone(); move |e: Event| {
                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                require_uppercase.set(input.checked());
                                            }}
                                        />
                                        <span class="ml-2">{ lang.t("require_uppercase") }</span>
                                    </label>
                                    <p class="help ml-6">{"密码必须包含至少一个大写字母（A-Z）"}</p>
                                </div>

                                <div class="field">
                                    <label class="checkbox">
                                        <input
                                            type="checkbox"
                                            checked={*require_lowercase}
                                            onchange={let require_lowercase = require_lowercase.clone(); move |e: Event| {
                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                require_lowercase.set(input.checked());
                                            }}
                                        />
                                        <span class="ml-2">{ lang.t("require_lowercase") }</span>
                                    </label>
                                    <p class="help ml-6">{"密码必须包含至少一个小写字母（a-z）"}</p>
                                </div>

                                <div class="field">
                                    <label class="checkbox">
                                        <input
                                            type="checkbox"
                                            checked={*require_number}
                                            onchange={let require_number = require_number.clone(); move |e: Event| {
                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                require_number.set(input.checked());
                                            }}
                                        />
                                        <span class="ml-2">{ lang.t("require_number") }</span>
                                    </label>
                                    <p class="help ml-6">{"密码必须包含至少一个数字（0-9）"}</p>
                                </div>

                                <div class="field">
                                    <label class="checkbox">
                                        <input
                                            type="checkbox"
                                            checked={*require_special}
                                            onchange={let require_special = require_special.clone(); move |e: Event| {
                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                require_special.set(input.checked());
                                            }}
                                        />
                                        <span class="ml-2">{ lang.t("require_special") }</span>
                                    </label>
                                    <p class="help ml-6">{"密码必须包含至少一个特殊字符（如 !@#$%^&*）"}</p>
                                </div>

                                // Policy preview card
                                <div class="box has-background-light mt-5">
                                    <h4 class="title is-6">{"密码要求预览"}</h4>
                                    <ul class="is-size-7">
                                        <li>{format!("• 最小长度：{} 位", *min_length)}</li>
                                        <li>{format!("• 强度要求：{}", match min_strength.as_str() {
                                            "weak" => "弱",
                                            "medium" => "中",
                                            "strong" => "强",
                                            _ => "未知"
                                        })}</li>
                                        <li>{format!("• 大写字母：{}", if *require_uppercase { "✓ 必需" } else { "✗ 不必需" })}</li>
                                        <li>{format!("• 小写字母：{}", if *require_lowercase { "✓ 必需" } else { "✗ 不必需" })}</li>
                                        <li>{format!("• 数字：{}", if *require_number { "✓ 必需" } else { "✗ 不必需" })}</li>
                                        <li>{format!("• 特殊字符：{}", if *require_special { "✓ 必需" } else { "✗ 不必需" })}</li>
                                        <li>{format!("• 有效期：{}", max_age_days.map(|d| format!("{} 天", d)).unwrap_or("不限制".to_string()))}</li>
                                        <li>{format!("• 防重用：最近 {} 次", *prevent_reuse)}</li>
                                        <li>{format!("• 账户锁定：{}", max_login_attempts.map(|n| format!("{} 次失败后锁定", n)).unwrap_or("不限制".to_string()))}</li>
                                        <li>{format!("• 锁定时长：{} 分钟", *lockout_duration_minutes)}</li>
                                    </ul>
                                </div>
                            </div>
                        </div>

                        <hr />

                        <div class="field is-grouped">
                            <div class="control">
                                <button type="submit" class="button is-primary">
                                    <span class="icon"><i class="fas fa-save"></i></span>
                                    <span>{ lang.t("save_policy") }</span>
                                </button>
                            </div>
                            <div class="control">
                                <button type="button" class="button" onclick={on_reset}>
                                    <span class="icon"><i class="fas fa-undo"></i></span>
                                    <span>{ lang.t("cancel") }</span>
                                </button>
                            </div>
                        </div>
                    </form>
                </div>
                }
            </div>
        }
    }
