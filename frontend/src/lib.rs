//! RustSet Frontend - Yew 0.21 Version
//! Security Asset Management Platform

use gloo_net::http::Request;
use shared::{
    Asset, NetworkZone, Task, TaskStatus,
    Risk, ZoneConfig, User, Role, LoginRequest, LoginResponse, AuditLog,
    // Multi-Cloud types
    CloudAsset, CloudAssetStats, CloudProvider, VMStatus, BillingMode,
};
use wasm_bindgen_futures::spawn_local;
use web_sys::{InputEvent, Event};
use yew::prelude::*;
use gloo_timers::callback::Timeout;

// ============== Page State ==============

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Page {
    Login,
    Dashboard,
    TaskCenter,
    AssetCenter,
    ZoneManagement,
    RiskCenter,
    UserManagement,
    AuditLogs,
    CloudManagement, // 混合云管理
}

// ============== Auth State ==============

#[derive(Clone, Debug, PartialEq)]
pub struct AuthState {
    pub token: Option<String>,
    pub user: Option<User>,
}

// ============== Language ==============

#[derive(Clone, PartialEq, Eq)]
pub enum Language {
    Zh,
    En,
}

impl Language {
    fn t(&self, key: &str) -> String {
        match (self, key) {
            (Language::Zh, "login") => "登录".to_string(),
            (Language::En, "login") => "Login".to_string(),
            (Language::Zh, "username") => "用户名".to_string(),
            (Language::En, "username") => "Username".to_string(),
            (Language::Zh, "password") => "密码".to_string(),
            (Language::En, "password") => "Password".to_string(),
            (Language::Zh, "logout") => "退出登录".to_string(),
            (Language::En, "logout") => "Logout".to_string(),
            (Language::Zh, "user_management") => "👥 用户管理".to_string(),
            (Language::En, "user_management") => "👥 User Management".to_string(),
            (Language::Zh, "audit_logs") => "📜 审计日志".to_string(),
            (Language::En, "audit_logs") => "📜 Audit Logs".to_string(),
            (Language::Zh, "role") => "角色".to_string(),
            (Language::En, "role") => "Role".to_string(),
            (Language::Zh, "create_user") => "创建用户".to_string(),
            (Language::En, "create_user") => "Create User".to_string(),
            (Language::Zh, "general") => "通用".to_string(),
            (Language::En, "general") => "General".to_string(),
            (Language::Zh, "dashboard") => "📊 仪表盘".to_string(),
            (Language::En, "dashboard") => "📊 Dashboard".to_string(),
            (Language::Zh, "task_center") => "🚀 任务中心".to_string(),
            (Language::En, "task_center") => "🚀 Task Center".to_string(),
            (Language::Zh, "assets_risks") => "资产与风险".to_string(),
            (Language::En, "assets_risks") => "Assets & Risks".to_string(),
            (Language::Zh, "asset_management") => "📂 资产管理".to_string(),
            (Language::En, "asset_management") => "📂 Asset Management".to_string(),
            (Language::Zh, "risk_monitoring") => "⚠️ 风险监控".to_string(),
            (Language::En, "risk_monitoring") => "⚠️ Risk Monitoring".to_string(),
            (Language::Zh, "verify") => "验证".to_string(),
            (Language::En, "verify") => "Verify".to_string(),
            (Language::Zh, "verified") => "已验证".to_string(),
            (Language::En, "verified") => "Verified".to_string(),
            (Language::Zh, "ignored") => "已忽略".to_string(),
            (Language::En, "ignored") => "Ignored".to_string(),
            (Language::Zh, "resolved") => "已解决".to_string(),
            (Language::En, "resolved") => "Resolved".to_string(),
            (Language::Zh, "all") => "全部".to_string(),
            (Language::En, "all") => "All".to_string(),
            (Language::Zh, "open") => "待处理".to_string(),
            (Language::En, "open") => "Open".to_string(),
            (Language::Zh, "total_assets") => "资产总数".to_string(),
            (Language::En, "total_assets") => "Total Assets".to_string(),
            (Language::Zh, "total_ports") => "端口总数".to_string(),
            (Language::En, "total_ports") => "Total Ports".to_string(),
            (Language::Zh, "unbound_ports") => "未绑定端口".to_string(),
            (Language::En, "unbound_ports") => "Unbound Ports".to_string(),
            (Language::Zh, "active_tasks") => "活跃任务".to_string(),
            (Language::En, "active_tasks") => "Active Tasks".to_string(),
            (Language::Zh, "system_status") => "系统状态".to_string(),
            (Language::En, "system_status") => "System Status".to_string(),
            (Language::Zh, "system_running_msg") => "系统运行正常".to_string(),
            (Language::En, "system_running_msg") => "System is running normally".to_string(),
            (Language::Zh, "create_new_task") => "创建新任务".to_string(),
            (Language::En, "create_new_task") => "Create New Task".to_string(),
            (Language::Zh, "task_name") => "任务名称".to_string(),
            (Language::En, "task_name") => "Task Name".to_string(),
            (Language::Zh, "target_ip_domain") => "目标 (IP/域名)".to_string(),
            (Language::En, "target_ip_domain") => "Target (IP/Domain)".to_string(),
            (Language::Zh, "start_task") => "开始任务".to_string(),
            (Language::En, "start_task") => "Start Task".to_string(),
            (Language::Zh, "task_list") => "任务列表".to_string(),
            (Language::En, "task_list") => "Task List".to_string(),
            (Language::Zh, "name") => "名称".to_string(),
            (Language::En, "name") => "Name".to_string(),
            (Language::Zh, "target") => "目标".to_string(),
            (Language::En, "target") => "Target".to_string(),
            (Language::Zh, "owner") => "责任人".to_string(),
            (Language::En, "owner") => "Owner".to_string(),
            (Language::Zh, "weight") => "权重".to_string(),
            (Language::En, "weight") => "Weight".to_string(),
            (Language::Zh, "labels") => "标签".to_string(),
            (Language::En, "labels") => "Labels".to_string(),
            (Language::Zh, "status") => "状态".to_string(),
            (Language::En, "status") => "Status".to_string(),
            (Language::Zh, "assets_found") => "发现资产".to_string(),
            (Language::En, "assets_found") => "Assets Found".to_string(),
            (Language::Zh, "risks_found") => "发现风险".to_string(),
            (Language::En, "risks_found") => "Risks Found".to_string(),
            (Language::Zh, "add_asset") => "添加资产".to_string(),
            (Language::En, "add_asset") => "Add Asset".to_string(),
            (Language::Zh, "ip") => "IP地址".to_string(),
            (Language::En, "ip") => "IP".to_string(),
            (Language::Zh, "zone") => "区域".to_string(),
            (Language::En, "zone") => "Zone".to_string(),
            (Language::Zh, "add") => "添加".to_string(),
            (Language::En, "add") => "Add".to_string(),
            (Language::Zh, "save") => "保存".to_string(),
            (Language::En, "save") => "Save".to_string(),
            (Language::Zh, "cancel") => "取消".to_string(),
            (Language::En, "cancel") => "Cancel".to_string(),
            (Language::Zh, "severity") => "严重程度".to_string(),
            (Language::En, "severity") => "Severity".to_string(),
            (Language::Zh, "asset") => "资产".to_string(),
            (Language::En, "asset") => "Asset".to_string(),
            (Language::Zh, "port") => "端口".to_string(),
            (Language::En, "port") => "Port".to_string(),
            (Language::Zh, "description") => "描述".to_string(),
            (Language::En, "description") => "Description".to_string(),
            (Language::Zh, "action") => "操作".to_string(),
            (Language::En, "action") => "Action".to_string(),
            (Language::Zh, "resolve") => "处置".to_string(),
            (Language::En, "resolve") => "Resolve".to_string(),
            (Language::Zh, "no_risks") => "暂无风险".to_string(),
            (Language::En, "no_risks") => "No risks detected yet".to_string(),
            (Language::Zh, "port_policy") => "端口扫描类型".to_string(),
            (Language::En, "port_policy") => "Port Policy".to_string(),
            (Language::Zh, "domain_brute") => "域名爆破".to_string(),
            (Language::En, "domain_brute") => "Domain Brute".to_string(),
            (Language::Zh, "service_detection") => "服务识别".to_string(),
            (Language::En, "service_detection") => "Service Detection".to_string(),
            (Language::Zh, "os_detection") => "操作系统识别".to_string(),
            (Language::En, "os_detection") => "OS Detection".to_string(),
            (Language::Zh, "site_identify") => "站点指纹识别".to_string(),
            (Language::En, "site_identify") => "Web Fingerprint".to_string(),
            (Language::Zh, "zone_management") => "🌐 区域管理".to_string(),
            (Language::En, "zone_management") => "🌐 Zone Management".to_string(),
            (Language::Zh, "cidr") => "CIDR".to_string(),
            (Language::En, "cidr") => "CIDR".to_string(),
            (Language::Zh, "priority") => "优先级".to_string(),
            (Language::En, "priority") => "Priority".to_string(),
            (Language::Zh, "add_zone") => "添加区域".to_string(),
            (Language::En, "add_zone") => "Add Zone".to_string(),
            (Language::Zh, "delete") => "删除".to_string(),
            (Language::En, "delete") => "Delete".to_string(),
            (Language::Zh, "edit") => "编辑".to_string(),
            (Language::En, "edit") => "Edit".to_string(),
            (Language::Zh, "edit_asset") => "编辑资产".to_string(),
            (Language::En, "edit_asset") => "Edit Asset".to_string(),
            (Language::Zh, "edit_task") => "编辑任务".to_string(),
            (Language::En, "edit_task") => "Edit Task".to_string(),
            (Language::Zh, "edit_zone") => "编辑区域".to_string(),
            (Language::En, "edit_zone") => "Edit Zone".to_string(),
            (Language::Zh, "ports") => "端口".to_string(),
            (Language::En, "ports") => "Ports".to_string(),
            (Language::Zh, "service") => "服务".to_string(),
            (Language::En, "service") => "Service".to_string(),
            (Language::Zh, "add_port") => "添加端口".to_string(),
            (Language::En, "add_port") => "Add Port".to_string(),
            (Language::Zh, "save_port") => "保存端口".to_string(),
            (Language::En, "save_port") => "Save Port".to_string(),
            (Language::Zh, "timestamp") => "时间戳".to_string(),
            (Language::En, "timestamp") => "Timestamp".to_string(),
            (Language::Zh, "details") => "详情".to_string(),
            (Language::En, "details") => "Details".to_string(),
            (Language::Zh, "login_failed") => "登录失败".to_string(),
            (Language::En, "login_failed") => "Login Failed".to_string(),
            (Language::Zh, "access_denied") => "访问拒绝".to_string(),
            (Language::En, "access_denied") => "Access Denied".to_string(),

            // Multi-Cloud Management
            (Language::Zh, "cloud_management") => "☁️ 混合云管理".to_string(),
            (Language::En, "cloud_management") => "☁️ Multi-Cloud".to_string(),
            (Language::Zh, "asset_name") => "资产名称".to_string(),
            (Language::En, "asset_name") => "Asset Name".to_string(),
            (Language::Zh, "spec") => "规格".to_string(),
            (Language::En, "spec") => "Spec".to_string(),
            (Language::Zh, "system_disk") => "系统盘".to_string(),
            (Language::En, "system_disk") => "System Disk".to_string(),
            (Language::Zh, "cloud_region") => "云区".to_string(),
            (Language::En, "cloud_region") => "Cloud Region".to_string(),
            (Language::Zh, "ip_address") => "IP地址".to_string(),
            (Language::En, "ip_address") => "IP Address".to_string(),
            (Language::Zh, "billing_mode") => "计费模式".to_string(),
            (Language::En, "billing_mode") => "Billing Mode".to_string(),
            (Language::Zh, "expire_time") => "到期时间".to_string(),
            (Language::En, "expire_time") => "Expire Time".to_string(),
            (Language::Zh, "vm_status") => "状态".to_string(),
            (Language::En, "vm_status") => "Status".to_string(),
            (Language::Zh, "os") => "操作系统".to_string(),
            (Language::En, "os") => "OS".to_string(),
            (Language::Zh, "department") => "部门".to_string(),
            (Language::En, "department") => "Department".to_string(),
            (Language::Zh, "project") => "项目".to_string(),
            (Language::En, "project") => "Project".to_string(),
            (Language::Zh, "cloud_owner") => "负责人".to_string(),
            (Language::En, "cloud_owner") => "Owner".to_string(),
            (Language::Zh, "vm_created_at") => "创建时间".to_string(),
            (Language::En, "vm_created_at") => "Created".to_string(),
            (Language::Zh, "image_id") => "镜像ID".to_string(),
            (Language::En, "image_id") => "Image ID".to_string(),
            (Language::Zh, "disk_total") => "云盘总量".to_string(),
            (Language::En, "disk_total") => "Disk Total".to_string(),
            (Language::Zh, "disk_count") => "云盘数量".to_string(),
            (Language::En, "disk_count") => "Disks".to_string(),
            (Language::Zh, "has_snapshot") => "快照".to_string(),
            (Language::En, "has_snapshot") => "Snapshot".to_string(),
            (Language::Zh, "public_ip") => "公网IP".to_string(),
            (Language::En, "public_ip") => "Public IP".to_string(),
            (Language::Zh, "private_ip") => "内网IP".to_string(),
            (Language::En, "private_ip") => "Private IP".to_string(),
            (Language::Zh, "cpu_memory") => "CPU/内存".to_string(),
            (Language::En, "cpu_memory") => "CPU/Memory".to_string(),
            (Language::Zh, "running") => "运行中".to_string(),
            (Language::En, "running") => "Running".to_string(),
            (Language::Zh, "stopped") => "已停止".to_string(),
            (Language::En, "stopped") => "Stopped".to_string(),
            (Language::Zh, "pay_as_you_go") => "按量付费".to_string(),
            (Language::En, "pay_as_you_go") => "Pay-As-You-Go".to_string(),
            (Language::Zh, "subscription") => "包年包月".to_string(),
            (Language::En, "subscription") => "Subscription".to_string(),
            (Language::Zh, "yes") => "是".to_string(),
            (Language::En, "yes") => "Yes".to_string(),
            (Language::Zh, "no") => "否".to_string(),
            (Language::En, "no") => "No".to_string(),
            (Language::Zh, "no_cloud_assets") => "暂无云资产".to_string(),
            (Language::En, "no_cloud_assets") => "No cloud assets yet".to_string(),
            (Language::Zh, "add_cloud_asset") => "添加云资产".to_string(),
            (Language::En, "add_cloud_asset") => "Add Cloud Asset".to_string(),
            (Language::Zh, "sync_cloud_assets") => "同步云资产".to_string(),
            (Language::En, "sync_cloud_assets") => "Sync Assets".to_string(),
            (Language::Zh, "expiring_soon") => "即将到期".to_string(),
            (Language::En, "expiring_soon") => "Expiring Soon".to_string(),

            _ => key.to_string(),
        }
    }
}

// ============== Context Providers ==============

// We'll use a simpler approach with callback props for Yew

// Helper function to get token from localStorage
fn get_auth_token() -> String {
    match web_sys::window() {
        Some(window) => match window.local_storage() {
            Ok(Some(storage)) => match storage.get_item("auth_token") {
                Ok(token) => token.unwrap_or_default(),
                _ => String::new(),
            },
            _ => String::new(),
        },
        _ => String::new(),
    }
}

// Helper function to get user from localStorage
fn get_auth_user() -> Option<User> {
    match web_sys::window() {
        Some(window) => match window.local_storage() {
            Ok(Some(storage)) => match storage.get_item("auth_user") {
                Ok(Some(user_str)) => serde_json::from_str::<LoginResponse>(&user_str).ok().map(|r| r.user),
                _ => None,
            },
            _ => None,
        },
        _ => None,
    }
}

// Helper function to set auth in localStorage
fn set_auth(token: &str, user_str: &str) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.set_item("auth_token", token);
            let _ = storage.set_item("auth_user", user_str);
        }
    }
}

// Helper function to clear auth from localStorage
fn clear_auth() {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.remove_item("auth_token");
            let _ = storage.remove_item("auth_user");
        }
    }
}

// ============== Login Component ==============

#[derive(Properties, PartialEq)]
pub struct LoginProps {
    pub current_page: UseStateHandle<Page>,
}

#[function_component]
fn Login(props: &LoginProps) -> Html {
    let current_page = props.current_page.clone();
    let username = use_state(|| String::new());
    let password = use_state(|| String::new());
    let error_msg = use_state(|| None::<String>);
    let loading = use_state(|| false);
    let lang = use_state(|| Language::Zh);

    // We need a way to manage auth state - in Yew we'll use a callback
    let on_submit = {
        let username = username.clone();
        let password = password.clone();
        let error_msg = error_msg.clone();
        let loading = loading.clone();
        let current_page = current_page.clone();

        Callback::from(move |_| {
            let username_val = (*username).clone();
            let password_val = (*password).clone();
            let error_msg = error_msg.clone();
            let loading = loading.clone();
            let current_page = current_page.clone();

            spawn_local(async move {
                loading.set(true);
                error_msg.set(None);

                let req = LoginRequest {
                    username: username_val,
                    password: password_val,
                };

                let json = serde_json::to_string(&req).unwrap();
                let http_req = Request::post("http://localhost:3003/api/login")
                    .header("Content-Type", "application/json")
                    .body(json)
                    .unwrap();
                let resp = http_req.send().await;

                match resp {
                    Ok(response) if response.ok() => {
                        let text = response.text().await.unwrap_or_default();
                        gloo_console::log!("Login response text:", &text);
                        if let Ok(login_resp) = serde_json::from_str::<LoginResponse>(&text) {
                            // Store token in localStorage for simplicity
                            set_auth(&login_resp.token, &text);
                            current_page.set(Page::Dashboard);
                        } else {
                            error_msg.set(Some("JSON Parse Error".to_string()));
                            loading.set(false);
                        }
                    }
                    Ok(response) => {
                        let status = response.status();
                        gloo_console::error!("Login failed with status:", status as i32);
                        error_msg.set(Some(format!("Login failed: {}", status)));
                        loading.set(false);
                    }
                    Err(e) => {
                        gloo_console::error!("Network Error:", e.to_string());
                        error_msg.set(Some(format!("Network Error: {}", e)));
                        loading.set(false);
                    }
                }
            });
        })
    };

    let on_username_input = {
        let username = username.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            username.set(input.value());
        })
    };

    let on_password_input = {
        let password = password.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            password.set(input.value());
        })
    };

    let toggle_lang = {
        let lang = lang.clone();
        Callback::from(move |_| {
            lang.set(if *lang == Language::Zh {
                Language::En
            } else {
                Language::Zh
            });
        })
    };

    let loading_class = if *loading { " is-loading" } else { "" };

    html! {
        <section class="hero is-fullheight is-light">
            <div class="hero-body">
                <div class="container">
                    <div class="columns is-centered">
                        <div class="column is-5-tablet is-4-desktop is-3-widescreen">
                            <div class="has-text-centered mb-5">
                                <h1 class="title is-2">{ "RustSet" }</h1>
                                <button class="button is-small is-white" onclick={toggle_lang}>
                                    { if *lang == Language::Zh { "中文" } else { "English" } }
                                </button>
                            </div>
                            <div class="box">
                                <h3 class="title has-text-centered">{ lang.t("login") }</h3>
                                <div>
                                    <div class="field">
                                        <label class="label">{ lang.t("username") }</label>
                                        <div class="control">
                                            <input
                                                class="input"
                                                type="text"
                                                placeholder="e.g. admin"
                                                value={(*username).clone()}
                                                oninput={on_username_input}
                                            />
                                        </div>
                                    </div>
                                    <div class="field">
                                        <label class="label">{ lang.t("password") }</label>
                                        <div class="control">
                                            <input
                                                class="input"
                                                type="password"
                                                placeholder="*******"
                                                value={(*password).clone()}
                                                oninput={on_password_input}
                                            />
                                        </div>
                                    </div>
                                    if let Some(msg) = (*error_msg).as_ref() {
                                        <div class="notification is-danger is-light">
                                            { msg.clone() }
                                        </div>
                                    }
                                    <div class="field">
                                        <button
                                            class={format!("button is-primary is-fullwidth{}", loading_class)}
                                            onclick={on_submit}
                                            disabled={*loading}
                                        >
                                            { lang.t("login") }
                                        </button>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </section>
    }
}

// ============== Sidebar Component ==============

#[derive(Properties, PartialEq)]
pub struct SidebarProps {
    pub current_page: UseStateHandle<Page>,
}

#[function_component]
fn Sidebar(props: &SidebarProps) -> Html {
    let current_page = props.current_page.clone();
    let lang = use_state(|| Language::Zh);

    // Get user from localStorage
    let user_role = get_auth_user().map(|u| u.role);

    let toggle_lang = {
        let lang = lang.clone();
        Callback::from(move |_| {
            lang.set(if *lang == Language::Zh {
                Language::En
            } else {
                Language::Zh
            });
        })
    };

    let on_logout = {
        let current_page = current_page.clone();
        Callback::from(move |_| {
            clear_auth();
            current_page.set(Page::Login);
        })
    };

    let navigate = |page: Page| -> Callback<MouseEvent> {
        let current_page = current_page.clone();
        Callback::from(move |_| current_page.set(page))
    };

    html! {
        <aside class="menu p-4" style="height: 100vh; background-color: #f5f5f5;">
            <div class="level is-mobile mb-4">
                <div class="level-left">
                    <h1 class="title is-4">{ "RustSet" }</h1>
                </div>
                <div class="level-right">
                    <button class="button is-small is-white" onclick={toggle_lang}>
                        { if *lang == Language::Zh { "中文" } else { "English" } }
                    </button>
                </div>
            </div>
            <div class="level is-mobile">
                <div class="level-left">
                    <p class="menu-label">{ lang.t("general") }</p>
                </div>
            </div>
            <ul class="menu-list">
                <li><a onclick={navigate(Page::Dashboard)}>{ lang.t("dashboard") }</a></li>
                if user_role == Some(Role::SecAdmin) || user_role == Some(Role::Auditor) {
                    <li><a onclick={navigate(Page::TaskCenter)}>{ lang.t("task_center") }</a></li>
                }
            </ul>
            if user_role == Some(Role::SecAdmin) {
                <p class="menu-label">{ lang.t("assets_risks") }</p>
                <ul class="menu-list">
                    <li><a onclick={navigate(Page::AssetCenter)}>{ lang.t("asset_management") }</a></li>
                    <li><a onclick={navigate(Page::ZoneManagement)}>{ lang.t("zone_management") }</a></li>
                    <li><a onclick={navigate(Page::RiskCenter)}>{ lang.t("risk_monitoring") }</a></li>
                </ul>
            }
            // 混合云管理 - 所有角色都可访问
            <p class="menu-label">{ "Cloud" }</p>
            <ul class="menu-list">
                <li><a onclick={navigate(Page::CloudManagement)}>{ lang.t("cloud_management") }</a></li>
            </ul>
            if user_role == Some(Role::SysAdmin) {
                <p class="menu-label">{ lang.t("user_management") }</p>
                <ul class="menu-list">
                    <li><a onclick={navigate(Page::UserManagement)}>{ lang.t("user_management") }</a></li>
                </ul>
            }
            if user_role == Some(Role::Auditor) {
                <p class="menu-label">{ lang.t("audit_logs") }</p>
                <ul class="menu-list">
                    <li><a onclick={navigate(Page::AuditLogs)}>{ lang.t("audit_logs") }</a></li>
                </ul>
            }
            <p class="menu-label">{ "Account" }</p>
            <ul class="menu-list">
                <li><a onclick={on_logout}>{ lang.t("logout") }</a></li>
            </ul>
        </aside>
    }
}

// ============== Dashboard Component ==============

#[function_component]
fn Dashboard() -> Html {
    let lang = use_state(|| Language::Zh);
    let stats = use_state(|| (0, 0, 0, 0));
    let loading = use_state(|| true);

    // Get token from localStorage
    let token = get_auth_token();

    // Use effect to fetch data only once on mount
    use_effect_with((), {
        let stats = stats.clone();
        let loading = loading.clone();
        let token = token.clone();

        move |_| {
            spawn_local(async move {
                loading.set(true);

                let assets_req = Request::get("http://localhost:3003/api/assets")
                    .header("Authorization", &token)
                    .send()
                    .await;
                let tasks_req = Request::get("http://localhost:3003/api/tasks")
                    .header("Authorization", &token)
                    .send()
                    .await;

                let mut asset_count = 0;
                let mut port_count = 0;
                let mut unbound_count = 0;
                let mut task_count = 0;

                if let Ok(resp) = assets_req {
                    if let Ok(assets) = resp.json::<Vec<Asset>>().await {
                        asset_count = assets.len();
                        port_count = assets.iter().map(|a| a.ports.len()).sum();
                        unbound_count = assets.iter().map(|a| a.ports.iter().filter(|p| !p.is_bound).count()).sum();
                    }
                }

                if let Ok(resp) = tasks_req {
                    if let Ok(tasks) = resp.json::<Vec<Task>>().await {
                        task_count = tasks.len();
                    }
                }

                stats.set((asset_count, port_count, unbound_count, task_count));
                loading.set(false);
            });
            || ()
        }
    });

    let (total_assets, total_ports, unbound_ports, total_tasks) = *stats;

    html! {
        <div class="container p-4">
            <h1 class="title">{ lang.t("dashboard") }</h1>
            <div class="columns is-multiline">
                <div class="column is-3">
                    <div class="box has-background-info-light">
                        <div class="heading">{ lang.t("total_assets") }</div>
                        <div class="title">{ total_assets }</div>
                    </div>
                </div>
                <div class="column is-3">
                    <div class="box has-background-primary-light">
                        <div class="heading">{ lang.t("total_ports") }</div>
                        <div class="title">{ total_ports }</div>
                    </div>
                </div>
                <div class="column is-3">
                    <div class="box has-background-danger-light">
                        <div class="heading">{ lang.t("unbound_ports") }</div>
                        <div class="title has-text-danger">{ unbound_ports }</div>
                    </div>
                </div>
                <div class="column is-3">
                    <div class="box has-background-warning-light">
                        <div class="heading">{ lang.t("active_tasks") }</div>
                        <div class="title">{ total_tasks }</div>
                    </div>
                </div>
            </div>
            <div class="box">
                <h2 class="subtitle">{ lang.t("system_status") }</h2>
                <p>{ lang.t("system_running_msg") }</p>
            </div>
        </div>
    }
}

// ============== Task Center Component ==============

#[function_component]
fn TaskCenter() -> Html {
    let lang = use_state(|| Language::Zh);
    let tasks = use_state(|| Vec::new());
    let loading = use_state(|| true);

    // Get token from localStorage
    let token = get_auth_token();

    // Use effect to fetch data only once on mount
    use_effect_with((), {
        let tasks = tasks.clone();
        let loading = loading.clone();
        let token = token.clone();

        move |_| {
            spawn_local(async move {
                if let Ok(resp) = Request::get("http://localhost:3003/api/tasks").header("Authorization", &token).send().await {
                    if let Ok(data) = resp.json::<Vec<Task>>().await {
                        tasks.set(data);
                    }
                }
                loading.set(false);
            });
            || ()
        }
    });

    let status_class = |status: &TaskStatus| -> &'static str {
        match status {
            TaskStatus::Pending => "is-warning",
            TaskStatus::Running => "is-info",
            TaskStatus::Completed => "is-success",
            TaskStatus::Failed => "is-danger",
        }
    };

    let status_to_lowercase = |status: &TaskStatus| -> String {
        format!("{:?}", status).to_lowercase()
    };

    html! {
        <div class="container p-4">
            <h1 class="title">{ lang.t("task_center") }</h1>
            <div class="box">
                if (*tasks).is_empty() && *loading {
                    <p>{ "Loading..." }</p>
                } else if (*tasks).is_empty() {
                    <p>{ "No tasks yet" }</p>
                } else {
                    <table class="table is-fullwidth is-hoverable">
                        <thead>
                            <tr>
                                <th>{ lang.t("name") }</th>
                                <th>{ lang.t("target") }</th>
                                <th>{ lang.t("status") }</th>
                                <th>{ lang.t("assets_found") }</th>
                                <th>{ lang.t("risks_found") }</th>
                            </tr>
                        </thead>
                        <tbody>
                            { for tasks.iter().map(|task| {
                                let status_str = format!("{:?}", task.status);
                                html! {
                                    <tr>
                                        <td>{ &task.name }</td>
                                        <td><code>{ &task.target }</code></td>
                                        <td>
                                            <span class={classes!(status_class(&task.status), format!("status-{}", status_to_lowercase(&task.status)))}>
                                                { status_str }
                                            </span>
                                        </td>
                                        <td>{ task.found_assets }</td>
                                        <td>{ task.found_risks }</td>
                                    </tr>
                                }
                            })}
                        </tbody>
                    </table>
                }
            </div>
        </div>
    }
}

// ============== Risk Center Component ==============

#[function_component]
fn RiskCenter() -> Html {
    let lang = use_state(|| Language::Zh);
    let risks = use_state(|| Vec::new());
    let loading = use_state(|| true);

    // Get token from localStorage
    let token = get_auth_token();

    // Use effect to fetch data only once on mount
    use_effect_with((), {
        let risks = risks.clone();
        let loading = loading.clone();
        let token = token.clone();

        move |_| {
            spawn_local(async move {
                if let Ok(resp) = Request::get("http://localhost:3003/api/risks").header("Authorization", &token).send().await {
                    if let Ok(data) = resp.json::<Vec<Risk>>().await {
                        risks.set(data);
                    }
                }
                loading.set(false);
            });
            || ()
        }
    });

    let severity_class = |sev: &str| -> &'static str {
        match sev {
            "Critical" => "is-danger",
            "High" => "is-warning",
            "Medium" => "is-info",
            _ => "is-light",
        }
    };

    html! {
        <div class="container p-4">
            <h1 class="title">{ lang.t("risk_monitoring") }</h1>
            if (*risks).is_empty() && *loading {
                <div class="box has-text-centered">
                    <p class="has-text-grey">{ "Loading..." }</p>
                </div>
            } else if (*risks).is_empty() {
                <div class="box has-text-centered">
                    <p class="has-text-grey">{ lang.t("no_risks") }</p>
                </div>
            } else {
                <div class="box">
                    <table class="table is-fullwidth is-hoverable">
                        <thead>
                            <tr>
                                <th>{ lang.t("severity") }</th>
                                <th>{ lang.t("asset") }</th>
                                <th>{ lang.t("port") }</th>
                                <th>{ lang.t("description") }</th>
                                <th>{ lang.t("status") }</th>
                            </tr>
                        </thead>
                        <tbody>
                            { for risks.iter().map(|risk| {
                                let status_str = format!("{:?}", risk.status);
                                html! {
                                    <tr>
                                        <td>
                                            <span class={classes!("tag", severity_class(&risk.severity))}>
                                                { &risk.severity }
                                            </span>
                                        </td>
                                        <td>{ &risk.asset_ip }</td>
                                        <td>{ risk.port }</td>
                                        <td>{ &risk.description }</td>
                                        <td>{ status_str }</td>
                                    </tr>
                                }
                            })}
                        </tbody>
                    </table>
                </div>
            }
        </div>
    }
}

// ============== Asset Center Component ==============

#[function_component]
fn AssetCenter() -> Html {
    let lang = use_state(|| Language::Zh);
    let assets = use_state(|| Vec::new());
    let loading = use_state(|| true);

    // Get token from localStorage
    let token = get_auth_token();

    // Use effect to fetch data only once on mount
    use_effect_with((), {
        let assets = assets.clone();
        let loading = loading.clone();
        let token = token.clone();

        move |_| {
            spawn_local(async move {
                if let Ok(resp) = Request::get("http://localhost:3003/api/assets").header("Authorization", &token).send().await {
                    if let Ok(data) = resp.json::<Vec<Asset>>().await {
                        assets.set(data);
                    }
                }
                loading.set(false);
            });
            || ()
        }
    });

    let zone_name = |zone: &NetworkZone| -> String {
        match zone {
            NetworkZone::Internet => "Internet".to_string(),
            NetworkZone::DMZ => "DMZ".to_string(),
            NetworkZone::Intranet => "Intranet".to_string(),
            NetworkZone::Custom(s) => s.clone(),
        }
    };

    let zone_class = |zone: &NetworkZone| -> &'static str {
        match zone {
            NetworkZone::Internet => "is-info",
            NetworkZone::DMZ => "is-warning",
            NetworkZone::Intranet => "is-success",
            NetworkZone::Custom(_) => "is-light",
        }
    };

    html! {
        <div class="container p-4">
            <h1 class="title">{ lang.t("asset_management") }</h1>
            <div class="box">
                if (*assets).is_empty() && *loading {
                    <p class="has-text-centered has-text-grey">{ "Loading..." }</p>
                } else if (*assets).is_empty() {
                    <p class="has-text-centered has-text-grey">{ "No assets yet" }</p>
                } else {
                    <div class="table-container">
                        <table class="table is-fullwidth is-hoverable is-striped">
                            <thead>
                                <tr>
                                    <th>{ lang.t("name") }</th>
                                    <th>{ lang.t("ip") }</th>
                                    <th>{ lang.t("zone") }</th>
                                    <th>{ lang.t("owner") }</th>
                                    <th>{ lang.t("weight") }</th>
                                    <th>{ lang.t("ports") }</th>
                                </tr>
                            </thead>
                            <tbody>
                                { for assets.iter().map(|asset| {
                                    let zone = asset.zone.clone();
                                    html! {
                                        <tr>
                                            <td>{ &asset.name }</td>
                                            <td><code>{ &asset.ip }</code></td>
                                            <td>
                                                <span class={classes!("tag", zone_class(&zone))}>
                                                    { zone_name(&zone) }
                                                </span>
                                            </td>
                                            <td>{ asset.owner.clone().unwrap_or_default() }</td>
                                            <td>{ asset.weight }</td>
                                            <td>
                                                <div class="tags are-small">
                                                    { for asset.ports.iter().take(3).map(|port| {
                                                        html! {
                                                            <span class={if port.is_bound { "tag is-primary" } else { "tag is-light" }}>
                                                                { format!("{} ({})", port.port, port.service.clone().unwrap_or_default()) }
                                                            </span>
                                                        }
                                                    })}
                                                    if asset.ports.len() > 3 {
                                                        <span class="tag">{ format!("+{}", asset.ports.len() - 3) }</span>
                                                    }
                                                </div>
                                            </td>
                                        </tr>
                                    }
                                })}
                            </tbody>
                        </table>
                    </div>
                }
            </div>
        </div>
    }
}

// ============== Zone Management Component ==============

#[function_component]
fn ZoneManagement() -> Html {
    let lang = use_state(|| Language::Zh);
    let zones = use_state(|| Vec::new());
    let loading = use_state(|| true);

    // Get token from localStorage
    let token = get_auth_token();

    // Use effect to fetch data only once on mount
    use_effect_with((), {
        let zones = zones.clone();
        let loading = loading.clone();
        let token = token.clone();

        move |_| {
            spawn_local(async move {
                if let Ok(resp) = Request::get("http://localhost:3003/api/zones").header("Authorization", &token).send().await {
                    if let Ok(data) = resp.json::<Vec<ZoneConfig>>().await {
                        zones.set(data);
                    }
                }
                loading.set(false);
            });
            || ()
        }
    });

    html! {
        <div class="container p-4">
            <h1 class="title">{ lang.t("zone_management") }</h1>
            <div class="box">
                if (*zones).is_empty() && *loading {
                    <p class="has-text-centered has-text-grey">{ "Loading..." }</p>
                } else if (*zones).is_empty() {
                    <p class="has-text-centered has-text-grey">{ "No zones configured" }</p>
                } else {
                    <table class="table is-fullwidth is-hoverable">
                        <thead>
                            <tr>
                                <th>{ lang.t("zone") }</th>
                                <th>{ lang.t("cidr") }</th>
                                <th>{ lang.t("priority") }</th>
                            </tr>
                        </thead>
                        <tbody>
                            { for zones.iter().map(|zone| {
                                html! {
                                    <tr>
                                        <td>{ &zone.name }</td>
                                        <td><code>{ &zone.cidr }</code></td>
                                        <td>{ zone.priority }</td>
                                    </tr>
                                }
                            })}
                        </tbody>
                    </table>
                }
            </div>
        </div>
    }
}

// ============== User Management Component ==============

#[function_component]
fn UserManagement() -> Html {
    let lang = use_state(|| Language::Zh);
    let users = use_state(|| Vec::new());
    let loading = use_state(|| true);

    // Get token from localStorage
    let token = get_auth_token();

    // Use effect to fetch data only once on mount
    use_effect_with((), {
        let users = users.clone();
        let loading = loading.clone();
        let token = token.clone();

        move |_| {
            spawn_local(async move {
                if let Ok(resp) = Request::get("http://localhost:3003/api/users").header("Authorization", &token).send().await {
                    if let Ok(data) = resp.json::<Vec<User>>().await {
                        users.set(data);
                    }
                }
                loading.set(false);
            });
            || ()
        }
    });

    let role_name = |role: &Role| -> &'static str {
        match role {
            Role::SysAdmin => "SysAdmin",
            Role::SecAdmin => "SecAdmin",
            Role::Auditor => "Auditor",
        }
    };

    html! {
        <div class="container p-4">
            <h1 class="title">{ lang.t("user_management") }</h1>
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
                            </tr>
                        </thead>
                        <tbody>
                            { for users.iter().map(|user| {
                                html! {
                                    <tr>
                                        <td>{ &user.username }</td>
                                        <td>
                                            <span class="tag">{ role_name(&user.role) }</span>
                                        </td>
                                        <td>{ &user.created_at.format("%Y-%m-%d %H:%M:%S").to_string() }</td>
                                    </tr>
                                }
                            })}
                        </tbody>
                    </table>
                }
            </div>
        </div>
    }
}

// ============== Audit Logs Component ==============

#[function_component]
fn AuditLogs() -> Html {
    let lang = use_state(|| Language::Zh);
    let logs = use_state(|| Vec::new());
    let loading = use_state(|| true);

    // Get token from localStorage
    let token = get_auth_token();

    // Use effect to fetch data only once on mount
    use_effect_with((), {
        let logs = logs.clone();
        let loading = loading.clone();
        let token = token.clone();

        move |_| {
            spawn_local(async move {
                if let Ok(resp) = Request::get("http://localhost:3003/api/logs").header("Authorization", &token).send().await {
                    if let Ok(data) = resp.json::<Vec<AuditLog>>().await {
                        logs.set(data);
                    }
                }
                loading.set(false);
            });
            || ()
        }
    });

    html! {
        <div class="container p-4">
            <h1 class="title">{ lang.t("audit_logs") }</h1>
            <div class="box">
                if (*logs).is_empty() && *loading {
                    <p class="has-text-centered has-text-grey">{ "Loading..." }</p>
                } else if (*logs).is_empty() {
                    <p class="has-text-centered has-text-grey">{ "No audit logs yet" }</p>
                } else {
                    <table class="table is-fullwidth is-hoverable">
                        <thead>
                            <tr>
                                <th>{ lang.t("timestamp") }</th>
                                <th>{ lang.t("username") }</th>
                                <th>{ lang.t("action") }</th>
                                <th>{ lang.t("details") }</th>
                            </tr>
                        </thead>
                        <tbody>
                            { for logs.iter().map(|log| {
                                html! {
                                    <tr>
                                        <td>{ log.timestamp.format("%Y-%m-%d %H:%M:%S").to_string() }</td>
                                        <td>{ &log.username }</td>
                                        <td>{ &log.action }</td>
                                        <td>{ &log.details }</td>
                                    </tr>
                                }
                            })}
                        </tbody>
                    </table>
                }
            </div>
        </div>
    }
}

// ============== Not Found Component ==============

#[function_component]
fn NotFound() -> Html {
    html! {
        <h1 class="title">{ "404 Not Found" }</h1>
    }
}

// ============== Cloud Management Component ==============

#[function_component]
fn CloudManagement() -> Html {
    let lang = use_state(|| Language::Zh);
    let assets = use_state(|| Vec::new());
    let stats = use_state(|| None as Option<CloudAssetStats>);
    let loading = use_state(|| true);

    // Edit modal state
    let editing_asset = use_state(|| None as Option<CloudAsset>);
    let show_edit_modal = use_state(|| false);
    let edit_message = use_state(|| None as Option<String>);

    // Filters
    let filter_provider = use_state(|| String::new());
    let filter_status = use_state(|| String::new());
    let filter_search = use_state(|| String::new());

    // Get token from localStorage
    let token = get_auth_token();

    // Use effect to fetch data only once on mount
    use_effect_with((), {
        let assets = assets.clone();
        let stats = stats.clone();
        let loading = loading.clone();
        let token = token.clone();

        move |_| {
            spawn_local(async move {
                // Fetch cloud assets
                if let Ok(resp) = Request::get("http://localhost:3003/api/cloud-assets")
                    .header("Authorization", &token)
                    .send()
                    .await
                {
                    if let Ok(data) = resp.json::<Vec<CloudAsset>>().await {
                        assets.set(data);
                    }
                }

                // Fetch stats
                if let Ok(resp) = Request::get("http://localhost:3003/api/cloud-assets/stats")
                    .header("Authorization", &token)
                    .send()
                    .await
                {
                    if let Ok(data) = resp.json::<CloudAssetStats>().await {
                        stats.set(Some(data));
                    }
                }

                loading.set(false);
            });
            || ()
        }
    });

    // Helper functions for display
    let provider_name = |provider: &CloudProvider| -> String {
        match provider {
            CloudProvider::Aliyun => "阿里云".to_string(),
            CloudProvider::Tencent => "腾讯云".to_string(),
            CloudProvider::Huawei => "华为云".to_string(),
            CloudProvider::Aws => "AWS".to_string(),
            CloudProvider::Azure => "Azure".to_string(),
            CloudProvider::Gcp => "GCP".to_string(),
            CloudProvider::Baidu => "百度云".to_string(),
            CloudProvider::Custom(s) => s.clone(),
        }
    };

    let status_text = |status: &VMStatus| -> &'static str {
        match status {
            VMStatus::Running => "运行中",
            VMStatus::Stopped => "已停止",
            VMStatus::Starting => "启动中",
            VMStatus::Stopping => "停止中",
            VMStatus::Rebooting => "重启中",
            VMStatus::Deleted => "已释放",
            VMStatus::Error => "异常",
        }
    };

    let billing_text = |mode: &BillingMode| -> &'static str {
        match mode {
            BillingMode::PayAsYouGo => "按量付费",
            BillingMode::Subscription => "包年包月",
            BillingMode::Spot => "抢占式",
        }
    };

    // Helper to check provider match with string filter
    let provider_matches = |provider: &CloudProvider, filter: &str| -> bool {
        match (provider, filter) {
            (CloudProvider::Aliyun, "aliyun") => true,
            (CloudProvider::Tencent, "tencent") => true,
            (CloudProvider::Huawei, "huawei") => true,
            (CloudProvider::Aws, "aws") => true,
            (CloudProvider::Azure, "azure") => true,
            (CloudProvider::Gcp, "gcp") => true,
            (CloudProvider::Baidu, "baidu") => true,
            _ => false,
        }
    };

    // Helper to check status match with string filter
    let status_matches = |status: &VMStatus, filter: &str| -> bool {
        match (status, filter) {
            (VMStatus::Running, "running") => true,
            (VMStatus::Stopped, "stopped") => true,
            _ => false,
        }
    };

    let format_datetime = |dt: &chrono::DateTime<chrono::Utc>| -> String {
        dt.format("%Y-%m-%d %H:%M").to_string()
    };

    let format_expire = |dt: &Option<chrono::DateTime<chrono::Utc>>| -> String {
        match dt {
            Some(d) => {
                let now = chrono::Utc::now();
                let duration = *d - now;
                let days = duration.num_days();
                if days <= 7 {
                    format!("{} ({}天)", d.format("%Y-%m-%d"), days)
                } else {
                    d.format("%Y-%m-%d").to_string()
                }
            }
            None => "永久".to_string(),
        }
    };

    // Filter assets
    let filtered_assets = (*assets).iter().filter(|asset| {
        let provider_match = filter_provider.is_empty()
            || provider_matches(&asset.cloud_region.provider, &filter_provider);
        let status_match = filter_status.is_empty()
            || status_matches(&asset.status, &filter_status);
        let search_match = filter_search.is_empty()
            || asset.asset_name.contains(&*filter_search)
            || asset.instance_id.contains(&*filter_search);
        provider_match && status_match && search_match
    }).cloned().collect::<Vec<_>>();

    let on_provider_change = {
        let filter_provider = filter_provider.clone();
        Callback::from(move |e: Event| {
            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
            filter_provider.set(select.value());
        })
    };

    let on_status_change = {
        let filter_status = filter_status.clone();
        Callback::from(move |e: Event| {
            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
            filter_status.set(select.value());
        })
    };

    let on_search_input = {
        let filter_search = filter_search.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            filter_search.set(input.value());
        })
    };

    let on_refresh = {
        let assets = assets.clone();
        let stats = stats.clone();
        let loading = loading.clone();
        let token = token.clone();

        Callback::from(move |_| {
            let assets = assets.clone();
            let stats = stats.clone();
            let loading = loading.clone();
            let token = token.clone();

            spawn_local(async move {
                loading.set(true);

                if let Ok(resp) = Request::get("http://localhost:3003/api/cloud-assets")
                    .header("Authorization", &token)
                    .send()
                    .await
                {
                    if let Ok(data) = resp.json::<Vec<CloudAsset>>().await {
                        assets.set(data);
                    }
                }

                if let Ok(resp) = Request::get("http://localhost:3003/api/cloud-assets/stats")
                    .header("Authorization", &token)
                    .send()
                    .await
                {
                    if let Ok(data) = resp.json::<CloudAssetStats>().await {
                        stats.set(Some(data));
                    }
                }

                loading.set(false);
            });
        })
    };

    // Edit handlers
    let on_edit_click = {
        let editing_asset = editing_asset.clone();
        let show_edit_modal = show_edit_modal.clone();

        Callback::from(move |asset: CloudAsset| {
            editing_asset.set(Some(asset));
            show_edit_modal.set(true);
        })
    };

    let on_close_modal = {
        let show_edit_modal = show_edit_modal.clone();
        let editing_asset = editing_asset.clone();
        let edit_message = edit_message.clone();

        Callback::from(move |_| {
            show_edit_modal.set(false);
            editing_asset.set(None);
            edit_message.set(None);
        })
    };

    let on_save_asset = {
        let assets = assets.clone();
        let editing_asset = editing_asset.clone();
        let show_edit_modal = show_edit_modal.clone();
        let edit_message = edit_message.clone();
        let token = token.clone();

        Callback::from(move |_: Event| {
            if let Some(asset) = (*editing_asset).clone() {
                let assets = assets.clone();
                let token = token.clone();
                let edit_message = edit_message.clone();
                let show_edit_modal = show_edit_modal.clone();
                let editing_asset = editing_asset.clone();
                let asset_id = asset.id.unwrap_or(0);

                spawn_local(async move {
                    let asset_json = match serde_json::to_string(&asset) {
                        Ok(json) => json,
                        Err(e) => {
                            edit_message.set(Some(format!("序列化失败: {}", e)));
                            return;
                        }
                    };

                    if let Ok(resp) = Request::put(&format!("http://localhost:3003/api/cloud-assets/{}", asset_id))
                        .header("Authorization", &token)
                        .header("Content-Type", "application/json")
                        .body(asset_json)
                        .unwrap()
                        .send()
                        .await
                    {
                        if resp.ok() {
                            edit_message.set(Some("保存成功".to_string()));
                            // Refresh assets
                            if let Ok(resp) = Request::get("http://localhost:3003/api/cloud-assets")
                                .header("Authorization", &token)
                                .send()
                                .await
                            {
                                if let Ok(data) = resp.json::<Vec<CloudAsset>>().await {
                                    assets.set(data);
                                }
                            }
                            // Close modal after a short delay
                            spawn_local(async move {
                                Timeout::new(1000, move || {
                                    show_edit_modal.set(false);
                                    editing_asset.set(None);
                                }).forget();
                            });
                        } else {
                            edit_message.set(Some("保存失败".to_string()));
                        }
                    }
                });
            }
        })
    };

    html! {
        <div class="container p-4">
            <h1 class="title">{ lang.t("cloud_management") }</h1>

            // Statistics Cards
            if let Some(ref s) = *stats {
                <div class="columns is-multiline mb-4">
                    <div class="column is-2">
                        <div class="box has-background-info-light is-paddingless">
                            <div class="p-3">
                                <div class="is-size-7 has-text-grey">{"资产总数"}</div>
                                <div class="title is-5">{ s.total_count }</div>
                            </div>
                        </div>
                    </div>
                    <div class="column is-2">
                        <div class="box has-background-success-light is-paddingless">
                            <div class="p-3">
                                <div class="is-size-7 has-text-grey">{"运行中"}</div>
                                <div class="title is-5">{ s.running_count }</div>
                            </div>
                        </div>
                    </div>
                    <div class="column is-2">
                        <div class="box has-background-warning-light is-paddingless">
                            <div class="p-3">
                                <div class="is-size-7 has-text-grey">{"CPU核心"}</div>
                                <div class="title is-5">{ s.total_cpu_cores }</div>
                            </div>
                        </div>
                    </div>
                    <div class="column is-2">
                        <div class="box has-background-primary-light is-paddingless">
                            <div class="p-3">
                                <div class="is-size-7 has-text-grey">{"内存(GB)"}</div>
                                <div class="title is-5">{ s.total_memory_gb }</div>
                            </div>
                        </div>
                    </div>
                    <div class="column is-2">
                        <div class="box has-background-danger-light is-paddingless">
                            <div class="p-3">
                                <div class="is-size-7 has-text-grey">{"即将到期"}</div>
                                <div class="title is-5 has-text-danger">{ s.expiring_soon_count }</div>
                            </div>
                        </div>
                    </div>
                </div>
            }

            // Filters
            <div class="box mb-4">
                <div class="columns">
                    <div class="column is-2">
                        <label class="label is-small">{"云厂商"}</label>
                        <div class="select is-small is-fullwidth">
                            <select onchange={on_provider_change}>
                                <option value="">{"全部"}</option>
                                <option value="aliyun">{"阿里云"}</option>
                                <option value="tencent">{"腾讯云"}</option>
                                <option value="huawei">{"华为云"}</option>
                                <option value="aws">{"AWS"}</option>
                                <option value="azure">{"Azure"}</option>
                                <option value="gcp">{"GCP"}</option>
                                <option value="baidu">{"百度云"}</option>
                            </select>
                        </div>
                    </div>
                    <div class="column is-2">
                        <label class="label is-small">{"状态"}</label>
                        <div class="select is-small is-fullwidth">
                            <select onchange={on_status_change}>
                                <option value="">{"全部"}</option>
                                <option value="running">{"运行中"}</option>
                                <option value="stopped">{"已停止"}</option>
                            </select>
                        </div>
                    </div>
                    <div class="column is-4">
                        <label class="label is-small">{"搜索"}</label>
                        <input
                            class="input is-small"
                            type="text"
                            placeholder="资产名称或实例ID"
                            value={(*filter_search).clone()}
                            oninput={on_search_input}
                        />
                    </div>
                    <div class="column is-2">
                        <label class="label is-small">{"操作"}</label>
                        <button class="button is-small is-primary is-fullwidth" onclick={on_refresh}>
                            <span class="icon"><i class="fas fa-sync"></i></span>
                            <span>{"刷新"}</span>
                        </button>
                    </div>
                </div>
            </div>

            // Loading state
            if *loading {
                <div class="has-text-centered py-6">
                    <progress class="progress is-primary is-small" value="100">{ "Loading..." }</progress>
                </div>
            } else if filtered_assets.is_empty() {
                <div class="box has-text-centered py-6">
                    <p class="has-text-grey">{"暂无云资产数据"}</p>
                </div>
            } else {
                // Cloud Assets Table - Excel-like with resizable columns
                <div class="box is-paddingless" style="overflow: hidden;">
                    <div class="excel-table-wrapper" style="overflow-x: auto; position: relative; border: 1px solid #ddd; border-radius: 4px;">
                        <table class="excel-table" style="border-collapse: collapse; width: 100%; min-width: 3200px; font-size: 14px;">
                            <thead>
                                <tr style="background: #f5f5f5;">
                                    <th style="border: 1px solid #ddd; padding: 16px 10px; min-width: 180px; position: relative; text-align: left; font-weight: 600; color: #333;">
                                        {"\u{1f4c1} 资产名称"}
                                        <span class="resizer" style="position: absolute; right: 0; top: 0; height: 100%; width: 5px; cursor: col-resize; user-select: none;"></span>
                                    </th>
                                    <th style="border: 1px solid #ddd; padding: 16px 10px; min-width: 140px; position: relative; text-align: left; font-weight: 600; color: #333;">
                                        {"\u{1f4ca} 规格"}
                                        <span class="resizer" style="position: absolute; right: 0; top: 0; height: 100%; width: 5px; cursor: col-resize; user-select: none;"></span>
                                    </th>
                                    <th style="border: 1px solid #ddd; padding: 16px 10px; min-width: 80px; position: relative; text-align: left; font-weight: 600; color: #333;">
                                        {"CPU"}
                                        <span class="resizer" style="position: absolute; right: 0; top: 0; height: 100%; width: 5px; cursor: col-resize; user-select: none;"></span>
                                    </th>
                                    <th style="border: 1px solid #ddd; padding: 16px 10px; min-width: 80px; position: relative; text-align: left; font-weight: 600; color: #333;">
                                        {"内存"}
                                        <span class="resizer" style="position: absolute; right: 0; top: 0; height: 100%; width: 5px; cursor: col-resize; user-select: none;"></span>
                                    </th>
                                    <th style="border: 1px solid #ddd; padding: 16px 10px; min-width: 140px; position: relative; text-align: left; font-weight: 600; color: #333;">
                                        {"系统盘"}
                                        <span class="resizer" style="position: absolute; right: 0; top: 0; height: 100%; width: 5px; cursor: col-resize; user-select: none;"></span>
                                    </th>
                                    <th style="border: 1px solid #ddd; padding: 16px 10px; min-width: 100px; position: relative; text-align: left; font-weight: 600; color: #333;">
                                        {"\u{2601}️ 云厂商"}
                                        <span class="resizer" style="position: absolute; right: 0; top: 0; height: 100%; width: 5px; cursor: col-resize; user-select: none;"></span>
                                    </th>
                                    <th style="border: 1px solid #ddd; padding: 16px 10px; min-width: 140px; position: relative; text-align: left; font-weight: 600; color: #333;">
                                        {"\u{1f30e} 云区"}
                                        <span class="resizer" style="position: absolute; right: 0; top: 0; height: 100%; width: 5px; cursor: col-resize; user-select: none;"></span>
                                    </th>
                                    <th style="border: 1px solid #ddd; padding: 16px 10px; min-width: 140px; position: relative; text-align: left; font-weight: 600; color: #333;">
                                        {"公网IP"}
                                        <span class="resizer" style="position: absolute; right: 0; top: 0; height: 100%; width: 5px; cursor: col-resize; user-select: none;"></span>
                                    </th>
                                    <th style="border: 1px solid #ddd; padding: 16px 10px; min-width: 140px; position: relative; text-align: left; font-weight: 600; color: #333;">
                                        {"内网IP"}
                                        <span class="resizer" style="position: absolute; right: 0; top: 0; height: 100%; width: 5px; cursor: col-resize; user-select: none;"></span>
                                    </th>
                                    <th style="border: 1px solid #ddd; padding: 16px 10px; min-width: 100px; position: relative; text-align: left; font-weight: 600; color: #333;">
                                        {"计费模式"}
                                        <span class="resizer" style="position: absolute; right: 0; top: 0; height: 100%; width: 5px; cursor: col-resize; user-select: none;"></span>
                                    </th>
                                    <th style="border: 1px solid #ddd; padding: 16px 10px; min-width: 140px; position: relative; text-align: left; font-weight: 600; color: #333;">
                                        {"到期时间"}
                                        <span class="resizer" style="position: absolute; right: 0; top: 0; height: 100%; width: 5px; cursor: col-resize; user-select: none;"></span>
                                    </th>
                                    <th style="border: 1px solid #ddd; padding: 16px 10px; min-width: 90px; position: relative; text-align: left; font-weight: 600; color: #333;">
                                        {"状态"}
                                        <span class="resizer" style="position: absolute; right: 0; top: 0; height: 100%; width: 5px; cursor: col-resize; user-select: none;"></span>
                                    </th>
                                    <th style="border: 1px solid #ddd; padding: 16px 10px; min-width: 160px; position: relative; text-align: left; font-weight: 600; color: #333;">
                                        {"操作系统"}
                                        <span class="resizer" style="position: absolute; right: 0; top: 0; height: 100%; width: 5px; cursor: col-resize; user-select: none;"></span>
                                    </th>
                                    <th style="border: 1px solid #ddd; padding: 16px 10px; min-width: 160px; position: relative; text-align: left; font-weight: 600; color: #333;">
                                        {"镜像ID"}
                                        <span class="resizer" style="position: absolute; right: 0; top: 0; height: 100%; width: 5px; cursor: col-resize; user-select: none;"></span>
                                    </th>
                                    <th style="border: 1px solid #ddd; padding: 16px 10px; min-width: 120px; position: relative; text-align: left; font-weight: 600; color: #333;">
                                        {"部门"}
                                        <span class="resizer" style="position: absolute; right: 0; top: 0; height: 100%; width: 5px; cursor: col-resize; user-select: none;"></span>
                                    </th>
                                    <th style="border: 1px solid #ddd; padding: 16px 10px; min-width: 120px; position: relative; text-align: left; font-weight: 600; color: #333;">
                                        {"项目"}
                                        <span class="resizer" style="position: absolute; right: 0; top: 0; height: 100%; width: 5px; cursor: col-resize; user-select: none;"></span>
                                    </th>
                                    <th style="border: 1px solid #ddd; padding: 16px 10px; min-width: 100px; position: relative; text-align: left; font-weight: 600; color: #333;">
                                        {"负责人"}
                                        <span class="resizer" style="position: absolute; right: 0; top: 0; height: 100%; width: 5px; cursor: col-resize; user-select: none;"></span>
                                    </th>
                                    <th style="border: 1px solid #ddd; padding: 16px 10px; min-width: 140px; position: relative; text-align: left; font-weight: 600; color: #333;">
                                        {"创建时间"}
                                        <span class="resizer" style="position: absolute; right: 0; top: 0; height: 100%; width: 5px; cursor: col-resize; user-select: none;"></span>
                                    </th>
                                    <th style="border: 1px solid #ddd; padding: 16px 10px; min-width: 80px; position: relative; text-align: left; font-weight: 600; color: #333;">
                                        {"云盘数"}
                                        <span class="resizer" style="position: absolute; right: 0; top: 0; height: 100%; width: 5px; cursor: col-resize; user-select: none;"></span>
                                    </th>
                                    <th style="border: 1px solid #ddd; padding: 16px 10px; min-width: 100px; position: relative; text-align: left; font-weight: 600; color: #333;">
                                        {"云盘总量"}
                                        <span class="resizer" style="position: absolute; right: 0; top: 0; height: 100%; width: 5px; cursor: col-resize; user-select: none;"></span>
                                    </th>
                                    <th style="border: 1px solid #ddd; padding: 16px 10px; min-width: 80px; position: relative; text-align: left; font-weight: 600; color: #333;">
                                        {"快照"}
                                        <span class="resizer" style="position: absolute; right: 0; top: 0; height: 100%; width: 5px; cursor: col-resize; user-select: none;"></span>
                                    </th>
                                    <th style="border: 1px solid #ddd; padding: 16px 10px; min-width: 140px; position: sticky; right: 0; background: #f5f5f5; border-left: 2px solid #ccc; z-index: 10; text-align: left; font-weight: 600; color: #333;">
                                        {"\u{2699}️ 操作"}
                                    </th>
                                </tr>
                            </thead>
                            <tbody>
                                { for filtered_assets.iter().enumerate().map(|(idx, asset)| {
                                    let provider = provider_name(&asset.cloud_region.provider);
                                    let region = &asset.cloud_region.region_name;
                                    let status_display = status_text(&asset.status);
                                    let is_running = asset.status == VMStatus::Running;
                                    let is_stopped = asset.status == VMStatus::Stopped;
                                    let asset_for_edit = asset.clone();
                                    let asset_for_ssh = asset.clone();
                                    let asset_for_console = asset.clone();
                                    let row_bg = if idx % 2 == 0 { "#ffffff" } else { "#fafafa" };

                                    html! {
                                        <tr class="excel-row" style={format!("background: {}; transition: background-color 0.15s ease;", row_bg)}>
                                            <td style="border: 1px solid #ddd; padding: 16px 10px;">
                                                <div style="font-weight: 500; color: #2c3e50; margin-bottom: 2px;">{ &asset.asset_name }</div>
                                                <div style="font-size: 11px; color: #95a5a6;">{ &asset.instance_id }</div>
                                            </td>
                                            <td style="border: 1px solid #ddd; padding: 16px 10px; font-family: monospace; font-size: 12px;">{ &asset.spec.instance_type }</td>
                                            <td style="border: 1px solid #ddd; padding: 16px 10px;">{ format!("{}核", asset.spec.cpu_cores) }</td>
                                            <td style="border: 1px solid #ddd; padding: 16px 10px;">{ format!("{}GB", asset.spec.memory_gb) }</td>
                                            <td style="border: 1px solid #ddd; padding: 16px 10px; font-size: 12px;">{ format!("{}GB {}", asset.system_disk.size_gb, asset.system_disk.disk_type) }</td>
                                            <td style="border: 1px solid #ddd; padding: 16px 10px;">{ provider }</td>
                                            <td style="border: 1px solid #ddd; padding: 16px 10px; font-size: 12px;">{ region }</td>
                                            <td style="border: 1px solid #ddd; padding: 16px 10px;">
                                                { if let Some(ref pub_ip) = asset.public_ip {
                                                    html! { <span style="color: #3273dc; font-family: monospace; font-size: 12px;">{ pub_ip }</span> }
                                                } else {
                                                    html! { <span style="color: #bdc3c7;">{"-"}</span> }
                                                }}
                                            </td>
                                            <td style="border: 1px solid #ddd; padding: 16px 10px; font-family: monospace; font-size: 12px;">{ &asset.private_ip }</td>
                                            <td style="border: 1px solid #ddd; padding: 16px 10px;">{ billing_text(&asset.billing_mode) }</td>
                                            <td style="border: 1px solid #ddd; padding: 16px 10px; font-size: 12px;">{ format_expire(&asset.expire_time) }</td>
                                            <td style="border: 1px solid #ddd; padding: 16px 10px;">
                                                <span style={format!("background: {}; color: white; padding: 3px 8px; border-radius: 3px; font-size: 11px; font-weight: 500;",
                                                    if is_running { "#27ae60" } else if is_stopped { "#e74c3c" } else { "#f39c12" })}>
                                                    { status_display }
                                                </span>
                                            </td>
                                            <td style="border: 1px solid #ddd; padding: 16px 10px; font-size: 12px; max-width: 160px; overflow: hidden; text-overflow: ellipsis;">{ &asset.os_name }</td>
                                            <td style="border: 1px solid #ddd; padding: 16px 10px; font-family: monospace; font-size: 11px; max-width: 160px; overflow: hidden; text-overflow: ellipsis;">{ &asset.image_id }</td>
                                            <td style="border: 1px solid #ddd; padding: 16px 10px;">{ &asset.department.name }</td>
                                            <td style="border: 1px solid #ddd; padding: 16px 10px;">{ &asset.project.name }</td>
                                            <td style="border: 1px solid #ddd; padding: 16px 10px;">{ &asset.owner.name }</td>
                                            <td style="border: 1px solid #ddd; padding: 16px 10px; font-size: 12px;">{ format_datetime(&asset.created_at) }</td>
                                            <td style="border: 1px solid #ddd; padding: 16px 10px; text-align: center;">{ asset.cloud_disk_count }</td>
                                            <td style="border: 1px solid #ddd; padding: 16px 10px;">{ format!("{}GB", asset.cloud_disk_total_size_gb) }</td>
                                            <td style="border: 1px solid #ddd; padding: 16px 10px; text-align: center;">
                                                { if asset.snapshot_info.has_snapshot {
                                                    html! { <span style="color: #27ae60; font-weight: bold;">{"✓"}</span> }
                                                } else {
                                                    html! { <span style="color: #bdc3c7;">{"-"}</span> }
                                                }}
                                                { if asset.snapshot_info.has_snapshot {
                                                    html! { <span style="font-size: 11px; color: #7f8c8d; margin-left: 4px;">{ format!("({})", asset.snapshot_info.snapshot_count) }</span> }
                                                } else {
                                                    html! {}
                                                }}
                                            </td>
                                            <td style="border: 1px solid #ddd; padding: 6px; position: sticky; right: 0; background: {row_bg}; border-left: 2px solid #ccc; z-index: 10;">
                                                <div class="select is-small is-fullwidth" style="margin-bottom: 0;">
                                                    <select style="border: 1px solid #ddd; border-radius: 4px; padding: 4px 8px; font-size: 12px; cursor: pointer;" onchange={ {
                                                        let on_edit_click = on_edit_click.clone();
                                                        let asset_for_console = asset_for_console.clone();
                                                        let asset_for_ssh = asset_for_ssh.clone();
                                                        Callback::from(move |e: Event| {
                                                            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
                                                            match select.value().as_str() {
                                                                "view" => {
                                                                    let on_edit_click = on_edit_click.clone();
                                                                    on_edit_click.emit(asset_for_edit.clone());
                                                                }
                                                                "console" => {
                                                                    if let Some(window) = web_sys::window() {
                                                                        let _ = window.alert_with_message("控制台链接功能：点击后跳转到对应云厂商控制台");
                                                                    }
                                                                }
                                                                "ssh" => {
                                                                    if let Some(window) = web_sys::window() {
                                                                        let _ = window.alert_with_message(&format!("SSH命令: ssh root@{}", &asset_for_ssh.private_ip));
                                                                    }
                                                                }
                                                                "reboot" => {
                                                                    if let Some(window) = web_sys::window() {
                                                                        let _ = window.alert_with_message(&format!("重启实例: {}", &asset_for_console.asset_name));
                                                                    }
                                                                }
                                                                "start" => {
                                                                    if let Some(window) = web_sys::window() {
                                                                        let _ = window.alert_with_message(&format!("启动实例: {}", &asset_for_console.asset_name));
                                                                    }
                                                                }
                                                                "stop" => {
                                                                    if let Some(window) = web_sys::window() {
                                                                        let _ = window.alert_with_message(&format!("停止实例: {}", &asset_for_console.asset_name));
                                                                    }
                                                                }
                                                                "delete" => {
                                                                    if let Some(window) = web_sys::window() {
                                                                        let _ = window.alert_with_message(&format!("释放实例: {}", &asset_for_console.asset_name));
                                                                    }
                                                                }
                                                                _ => {}
                                                            }
                                                            // Reset select to default
                                                            select.set_value("");
                                                        })
                                                    } }>
                                                        <option value="">{"\u{1f4cb} 操作..."}</option>
                                                        <option value="view">{"\u{1f440} 查看详情"}</option>
                                                        <option value="console">{"\u{1f5a5} 云控制台"}</option>
                                                        <option value="ssh">{"\u{1f4bb} SSH连接"}</option>
                                                        <option value="reboot">{"\u{1f504} 重启实例"}</option>
                                                        <option value="start">{"\u{25b6} 启动实例"}</option>
                                                        <option value="stop">{"\u{23f9} 停止实例"}</option>
                                                        <option value="delete" style="color: #e74c3c;">{"\u{1f5d1} 释放实例"}</option>
                                                    </select>
                                                </div>
                                            </td>
                                        </tr>
                                    }
                                }) }
                            </tbody>
                        </table>
                    </div>
                </div>

                // Summary info
                <div class="box">
                    <p class="is-size-7">
                        { format!("共 {} 个云资产", filtered_assets.len()) }
                    </p>
                </div>
            }

            // Edit Modal
            if *show_edit_modal {
                if let Some(ref asset) = *editing_asset {
                    <div class="modal is-active">
                        <div class="modal-background" onclick={on_close_modal.clone()}></div>
                        <div class="modal-card" style="width: 800px;">
                            <header class="modal-card-head">
                                <p class="modal-card-title">{ format!("编辑: {}", asset.asset_name) }</p>
                                <button class="delete" aria-label="close" onclick={on_close_modal.clone()}></button>
                            </header>
                            <section class="modal-card-body" style="max-height: 60vh; overflow-y: auto;">
                                // Display message if any
                                if let Some(ref msg) = *edit_message {
                                    <div class={classes!("notification", if msg.contains("成功") { "is-success" } else { "is-danger" })}>
                                        { msg }
                                    </div>
                                }

                                <div class="columns is-multiline">
                                    // Asset Name (readonly - from cloud)
                                    <div class="column is-6">
                                        <label class="label is-small">{"资产名称"}</label>
                                        <input class="input is-small" type="text" value={asset.asset_name.clone()} readonly=true />
                                    </div>

                                    // Instance ID (readonly)
                                    <div class="column is-6">
                                        <label class="label is-small">{"实例ID"}</label>
                                        <input class="input is-small" type="text" value={asset.instance_id.clone()} readonly=true />
                                    </div>

                                    // Department (editable - for internal management)
                                    <div class="column is-6">
                                        <label class="label is-small">{"部门"}</label>
                                        <input class="input is-small" type="text" value={asset.department.name.clone()} readonly=true />
                                        <p class="help is-size-7">{"部门信息同步自组织架构"}</p>
                                    </div>

                                    // Project (editable)
                                    <div class="column is-6">
                                        <label class="label is-small">{"项目"}</label>
                                        <input class="input is-small" type="text" value={asset.project.name.clone()} readonly=true />
                                    </div>

                                    // Owner Name
                                    <div class="column is-6">
                                        <label class="label is-small">{"负责人"}</label>
                                        <input class="input is-small" type="text" value={asset.owner.name.clone()} readonly=true />
                                    </div>

                                    // Owner Email
                                    <div class="column is-6">
                                        <label class="label is-small">{"负责人邮箱"}</label>
                                        <input class="input is-small" type="text" value={asset.owner.email.clone()} readonly=true />
                                    </div>

                                    // Cloud Provider (readonly)
                                    <div class="column is-6">
                                        <label class="label is-small">{"云厂商"}</label>
                                        <input class="input is-small" type="text" value={provider_name(&asset.cloud_region.provider)} readonly=true />
                                    </div>

                                    // Region (readonly)
                                    <div class="column is-6">
                                        <label class="label is-small">{"云区"}</label>
                                        <input class="input is-small" type="text" value={asset.cloud_region.region_name.clone()} readonly=true />
                                    </div>

                                    // Instance Type
                                    <div class="column is-6">
                                        <label class="label is-small">{"实例规格"}</label>
                                        <input class="input is-small" type="text" value={asset.spec.instance_type.clone()} readonly=true />
                                    </div>

                                    // Status (readonly - from cloud)
                                    <div class="column is-6">
                                        <label class="label is-small">{"状态"}</label>
                                        <input class="input is-small" type="text" value={status_text(&asset.status)} readonly=true />
                                    </div>

                                    // CPU
                                    <div class="column is-4">
                                        <label class="label is-small">{"CPU核心"}</label>
                                        <input class="input is-small" type="text" value={format!("{}", asset.spec.cpu_cores)} readonly=true />
                                    </div>

                                    // Memory
                                    <div class="column is-4">
                                        <label class="label is-small">{"内存(GB)"}</label>
                                        <input class="input is-small" type="text" value={format!("{}", asset.spec.memory_gb)} readonly=true />
                                    </div>

                                    // System Disk
                                    <div class="column is-4">
                                        <label class="label is-small">{"系统盘(GB)"}</label>
                                        <input class="input is-small" type="text" value={format!("{}", asset.system_disk.size_gb)} readonly=true />
                                    </div>

                                    // IPs (readonly)
                                    <div class="column is-6">
                                        <label class="label is-small">{"公网IP"}</label>
                                        <input class="input is-small" type="text" value={asset.public_ip.clone().unwrap_or_else(|| "无".to_string())} readonly=true />
                                    </div>

                                    <div class="column is-6">
                                        <label class="label is-small">{"内网IP"}</label>
                                        <input class="input is-small" type="text" value={asset.private_ip.clone()} readonly=true />
                                    </div>

                                    // Billing Mode
                                    <div class="column is-6">
                                        <label class="label is-small">{"计费模式"}</label>
                                        <input class="input is-small" type="text" value={billing_text(&asset.billing_mode)} readonly=true />
                                    </div>

                                    // Expire Time
                                    <div class="column is-6">
                                        <label class="label is-small">{"到期时间"}</label>
                                        <input class="input is-small" type="text" value={format_expire(&asset.expire_time)} readonly=true />
                                    </div>

                                    // OS Info
                                    <div class="column is-6">
                                        <label class="label is-small">{"操作系统"}</label>
                                        <input class="input is-small" type="text" value={asset.os_name.clone()} readonly=true />
                                    </div>

                                    // Image ID
                                    <div class="column is-6">
                                        <label class="label is-small">{"镜像ID"}</label>
                                        <input class="input is-small" type="text" value={asset.image_id.clone()} readonly=true />
                                    </div>

                                    // Created Time
                                    <div class="column is-6">
                                        <label class="label is-small">{"创建时间"}</label>
                                        <input class="input is-small" type="text" value={format_datetime(&asset.created_at)} readonly=true />
                                    </div>

                                    // Cloud Disks
                                    <div class="column is-6">
                                        <label class="label is-small">{"云盘"}</label>
                                        <input class="input is-small" type="text" value={format!("{}个 / {}GB", asset.cloud_disk_count, asset.cloud_disk_total_size_gb)} readonly=true />
                                    </div>

                                    // Snapshots
                                    <div class="column is-12">
                                        <label class="label is-small">{"快照信息"}</label>
                                        <div class="box is-small">
                                            { if asset.snapshot_info.has_snapshot {
                                                html! {
                                                    <>
                                                        <p>{ format!("快照数量: {}", asset.snapshot_info.snapshot_count) }</p>
                                                        <p>{ format!("快照总量: {}GB", asset.snapshot_info.total_snapshot_size_gb) }</p>
                                                        {
                                                            if let Some(ref snap_time) = asset.snapshot_info.latest_snapshot_time {
                                                                html! { <p>{ format!("最新快照: {}", format_datetime(snap_time)) }</p> }
                                                            } else {
                                                                html! {}
                                                            }
                                                        }
                                                    </>
                                                }
                                            } else {
                                                html! { <p class="has-text-grey">{"无快照"}</p> }
                                            }}
                                        </div>
                                    </div>

                                    // Tags
                                    <div class="column is-12">
                                        <label class="label is-small">{"标签"}</label>
                                        <div class="tags">
                                            { for asset.tags.iter().map(|tag| {
                                                html! { <span class="tag is-light">{ tag }</span> }
                                            }) }
                                        </div>
                                    </div>
                                </div>

                                <div class="notification is-info is-light">
                                    <p class="is-size-7">{"注意：云资产的基础信息（如规格、IP、状态等）由云平台同步，只读。组织信息（部门、项目、负责人）可通过云平台控制台修改。"}</p>
                                </div>
                            </section>
                            <footer class="modal-card-foot" style="justify-content: flex-end;">
                                <button class="button" onclick={on_close_modal.clone()}>{"关闭"}</button>
                            </footer>
                        </div>
                    </div>
                }
            }
        </div>
    }
}

// ============== Main App Component ==============

#[function_component]
pub fn App() -> Html {
    let current_page = use_state(|| {
        // Check if user is logged in
        if !get_auth_token().is_empty() {
            Page::Dashboard
        } else {
            Page::Login
        }
    });

    let page = *current_page;

    html! {
        <div>
            { match page {
                Page::Login => html! {
                    <Login current_page={current_page.clone()} />
                },
                _ => html! {
                    <div class="columns is-gapless">
                        <div class="column is-2">
                            <Sidebar current_page={current_page.clone()} />
                        </div>
                        <div class="column">
                            { match page {
                                Page::Dashboard => html! { <Dashboard /> },
                                Page::TaskCenter => html! { <TaskCenter /> },
                                Page::AssetCenter => html! { <AssetCenter /> },
                                Page::ZoneManagement => html! { <ZoneManagement /> },
                                Page::RiskCenter => html! { <RiskCenter /> },
                                Page::UserManagement => html! { <UserManagement /> },
                                Page::AuditLogs => html! { <AuditLogs /> },
                                Page::CloudManagement => html! { <CloudManagement /> },
                                Page::Login => html! { <Login current_page={current_page.clone()} /> },
                            }}
                        </div>
                    </div>
                }
            }}
        </div>
    }
}
