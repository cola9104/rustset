use gloo_net::http::Request;
use shared::{
    Asset, NetworkZone, PortInfo, PortBindingRequest, Task, TaskStatus, CreateTaskRequest, 
    Risk, RiskStatus, ZoneConfig, User, Role, LoginRequest, LoginResponse, AuditLog, CreateUserRequest
};
use wasm_bindgen::prelude::*;
use yew::prelude::*;
use yew_router::prelude::*;
use web_sys::HtmlInputElement;
use wasm_bindgen_futures::spawn_local;

// Auth Context
#[derive(Clone, Debug, PartialEq)]
pub struct AuthState {
    pub token: Option<String>,
    pub user: Option<User>,
}

pub type AuthContext = UseStateHandle<AuthState>;

#[derive(Clone, PartialEq)]
pub enum Language {
    Zh,
    En,
}

impl Language {
    fn t(&self, key: &str) -> String {
        match (self, key) {
            // Auth & Users
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

            (Language::Zh, "timestamp") => "时间戳".to_string(),
            (Language::En, "timestamp") => "Timestamp".to_string(),
            (Language::Zh, "details") => "详情".to_string(),
            (Language::En, "details") => "Details".to_string(),
            (Language::Zh, "login_failed") => "登录失败".to_string(),
            (Language::En, "login_failed") => "Login Failed".to_string(),
            (Language::Zh, "access_denied") => "访问拒绝".to_string(),
            (Language::En, "access_denied") => "Access Denied".to_string(),
            // General & Sidebar
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

            // Dashboard
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
            (Language::Zh, "system_running_msg") => "系统运行正常。后端已连接至端口 3003。".to_string(),
            (Language::En, "system_running_msg") => "System is running normally. Backend connected on port 3003.".to_string(),

            // Task Center
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
            (Language::Zh, "status") => "状态".to_string(),
            (Language::En, "status") => "Status".to_string(),
            (Language::Zh, "assets_found") => "发现资产".to_string(),
            (Language::En, "assets_found") => "Assets Found".to_string(),
            (Language::Zh, "risks_found") => "发现风险".to_string(),
            (Language::En, "risks_found") => "Risks Found".to_string(),

            // Asset Center
            (Language::Zh, "add_asset") => "添加资产".to_string(),
            (Language::En, "add_asset") => "Add Asset".to_string(),
            (Language::Zh, "ip") => "IP地址".to_string(),
            (Language::En, "ip") => "IP".to_string(),
            (Language::Zh, "zone") => "区域".to_string(),
            (Language::En, "zone") => "Zone".to_string(),
            (Language::Zh, "add") => "添加".to_string(),
            (Language::En, "add") => "Add".to_string(),
            (Language::Zh, "owner") => "负责人".to_string(),
            (Language::En, "owner") => "Owner".to_string(),
            (Language::Zh, "system") => "系统名称".to_string(),
            (Language::En, "system") => "System".to_string(),
            (Language::Zh, "middleware") => "中间件".to_string(),
            (Language::En, "middleware") => "Middleware".to_string(),
            (Language::Zh, "contact_person") => "联系人".to_string(),
            (Language::En, "contact_person") => "Contact Person".to_string(),
            (Language::Zh, "contact_phone") => "联系电话".to_string(),
            (Language::En, "contact_phone") => "Contact Phone".to_string(),
            (Language::Zh, "bind") => "绑定".to_string(),
            (Language::En, "bind") => "Bind".to_string(),
            (Language::Zh, "save") => "保存".to_string(),
            (Language::En, "save") => "Save".to_string(),
            (Language::Zh, "cancel") => "取消".to_string(),
            (Language::En, "cancel") => "Cancel".to_string(),

            // Risk Center
            (Language::Zh, "severity") => "严重程度".to_string(),
            (Language::En, "severity") => "Severity".to_string(),
            (Language::Zh, "asset") => "资产".to_string(),
            (Language::En, "asset") => "Asset".to_string(),
            (Language::Zh, "port") => "端口".to_string(),
            (Language::En, "port") => "Port".to_string(),
            (Language::Zh, "description") => "描述".to_string(),
            (Language::En, "description") => "Description".to_string(),
            (Language::Zh, "solution") => "解决方案".to_string(),
            (Language::En, "solution") => "Solution".to_string(),
            (Language::Zh, "action") => "操作".to_string(),
            (Language::En, "action") => "Action".to_string(),
            (Language::Zh, "resolve") => "处置".to_string(),
            (Language::En, "resolve") => "Resolve".to_string(),
            (Language::Zh, "no_risks") => "暂无风险。干得好！".to_string(),
            (Language::En, "no_risks") => "No risks detected yet. Good job!".to_string(),

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

            // Zone Management
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
            (Language::Zh, "confirm_delete") => "确认删除".to_string(),
            (Language::En, "confirm_delete") => "Confirm Delete".to_string(),
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
            (Language::Zh, "port_exists") => "端口已存在".to_string(),
            (Language::En, "port_exists") => "Port already exists".to_string(),


            _ => key.to_string(),
        }
    }
}

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/login")]
    Login,
    #[at("/")]
    Dashboard,
    #[at("/tasks")]
    TaskCenter,
    #[at("/assets")]
    AssetCenter,
    #[at("/zones")]
    ZoneManagement,
    #[at("/risks")]
    RiskCenter,
    #[at("/users")]
    UserManagement,
    #[at("/logs")]
    AuditLogs,
    #[not_found]
    #[at("/404")]
    NotFound,
}

#[function_component(Login)]
fn login() -> Html {
    gloo_console::log!("Login Component Loaded - Version 2.0 (Fixed JSON)");
    let lang = use_context::<UseStateHandle<Language>>().expect("Language context missing");
    let auth = use_context::<AuthContext>().expect("Auth context missing");
    let navigator = use_navigator().unwrap();

    let username = use_state(|| "".to_string());
    let password = use_state(|| "".to_string());
    let error_msg = use_state(|| None::<String>);

    let on_submit = {
        let username = username.clone();
        let password = password.clone();
        let auth = auth.clone();
        let navigator = navigator.clone();
        let error_msg = error_msg.clone();
        let lang = lang.clone();

        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            let username = username.clone();
            let password = password.clone();
            let auth = auth.clone();
            let navigator = navigator.clone();
            let error_msg = error_msg.clone();
            let lang = lang.clone();

            spawn_local(async move {
                let req = LoginRequest {
                    username: (*username).clone(),
                    password: (*password).clone(),
                };

                let resp = Request::post("/api/login")
                    .json(&req)
                    .unwrap()
                    .send()
                    .await;

                match resp {
                    Ok(response) => {
                        if response.ok() {
                            let text_result = response.text().await;
                            match text_result {
                                Ok(text) => {
                                    gloo_console::log!("Login response text:", &text);
                                    match serde_json::from_str::<LoginResponse>(&text) {
                                        Ok(login_resp) => {
                                            auth.set(AuthState {
                                                token: Some(login_resp.token),
                                                user: Some(login_resp.user),
                                            });
                                            navigator.push(&Route::Dashboard);
                                        },
                                        Err(e) => {
                                            gloo_console::error!("JSON Parse Error:", e.to_string());
                                            error_msg.set(Some(format!("JSON Parse Error: {}", e)));
                                        }
                                    }
                                },
                                Err(e) => {
                                    gloo_console::error!("Failed to read response text:", e.to_string());
                                    error_msg.set(Some("Failed to read response".to_string()));
                                }
                            }
                        } else {
                            let status = response.status();
                            let status_text = response.status_text();
                            gloo_console::error!("Login failed with status:", status, &status_text);
                            error_msg.set(Some(format!("Login failed: {} {}", status, status_text)));
                        }
                    },
                    Err(e) => {
                        gloo_console::error!("Network Error:", e.to_string());
                        error_msg.set(Some(format!("Network Error: {}", e)));
                    }
                }
            });
        })
    };

    html! {
        <section class="hero is-fullheight is-light">
            <div class="hero-body">
                <div class="container">
                    <div class="columns is-centered">
                        <div class="column is-5-tablet is-4-desktop is-3-widescreen">
                            <div class="box">
                                <h3 class="title has-text-centered">{lang.t("login")}</h3>
                                <form>
                                    <div class="field">
                                        <label class="label">{lang.t("username")}</label>
                                        <div class="control">
                                            <input class="input" type="text" placeholder="e.g. admin" 
                                                value={(*username).clone()}
                                                oninput={Callback::from(move |e: InputEvent| {
                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                    username.set(input.value());
                                                })}
                                            />
                                        </div>
                                    </div>

                                    <div class="field">
                                        <label class="label">{lang.t("password")}</label>
                                        <div class="control">
                                            <input class="input" type="password" placeholder="*******" 
                                                value={(*password).clone()}
                                                oninput={Callback::from(move |e: InputEvent| {
                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                    password.set(input.value());
                                                })}
                                            />
                                        </div>
                                    </div>

                                    if let Some(msg) = (*error_msg).clone() {
                                        <div class="notification is-danger is-light">
                                            {msg}
                                        </div>
                                    }

                                    <div class="field">
                                        <button class="button is-primary is-fullwidth" onclick={on_submit}>
                                            {lang.t("login")}
                                        </button>
                                    </div>
                                </form>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </section>
    }
}

#[function_component(Sidebar)]
fn sidebar() -> Html {
    let lang = use_context::<UseStateHandle<Language>>().expect("Language context missing");
    let auth = use_context::<AuthContext>().expect("Auth context missing");
    let route = use_route::<Route>().unwrap_or(Route::Dashboard);
    let navigator = use_navigator().unwrap();

    let user_role = auth.user.as_ref().map(|u| u.role.clone());

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
        let auth = auth.clone();
        let navigator = navigator.clone();
        Callback::from(move |_| {
            auth.set(AuthState { token: None, user: None });
            navigator.push(&Route::Login);
        })
    };

    let is_active = |r: Route| -> Classes {
        if route == r { classes!("is-active") } else { classes!("") }
    };

    html! {
        <aside class="menu p-4" style="height: 100vh; background-color: #f5f5f5;">
            <div class="level is-mobile">
                <div class="level-left">
                     <p class="menu-label">{lang.t("general")}</p>
                </div>
                <div class="level-right">
                    <button class="button is-small is-white" onclick={toggle_lang}>
                        {if *lang == Language::Zh { "En" } else { "中" }}
                    </button>
                </div>
            </div>
            <ul class="menu-list">
                <li><Link<Route> to={Route::Dashboard} classes={is_active(Route::Dashboard)}>{lang.t("dashboard")}</Link<Route>></li>
                
                if user_role == Some(Role::SecAdmin) || user_role == Some(Role::Auditor) {
                    <li><Link<Route> to={Route::TaskCenter} classes={is_active(Route::TaskCenter)}>{lang.t("task_center")}</Link<Route>></li>
                }
            </ul>
            
            if user_role == Some(Role::SecAdmin) {
                <p class="menu-label">{lang.t("assets_risks")}</p>
                <ul class="menu-list">
                    <li><Link<Route> to={Route::AssetCenter} classes={is_active(Route::AssetCenter)}>{lang.t("asset_management")}</Link<Route>></li>
                    <li><Link<Route> to={Route::ZoneManagement} classes={is_active(Route::ZoneManagement)}>{lang.t("zone_management")}</Link<Route>></li>
                    <li><Link<Route> to={Route::RiskCenter} classes={is_active(Route::RiskCenter)}>{lang.t("risk_monitoring")}</Link<Route>></li>
                </ul>
            }

            if user_role == Some(Role::SysAdmin) {
                <p class="menu-label">{lang.t("user_management")}</p>
                <ul class="menu-list">
                    <li><Link<Route> to={Route::UserManagement} classes={is_active(Route::UserManagement)}>{lang.t("user_management")}</Link<Route>></li>
                </ul>
            }

            if user_role == Some(Role::Auditor) {
                <p class="menu-label">{lang.t("audit_logs")}</p>
                <ul class="menu-list">
                    <li><Link<Route> to={Route::AuditLogs} classes={is_active(Route::AuditLogs)}>{lang.t("audit_logs")}</Link<Route>></li>
                </ul>
            }

            <p class="menu-label">{"Account"}</p>
            <ul class="menu-list">
                 <li><a onclick={on_logout}>{lang.t("logout")}</a></li>
            </ul>
        </aside>
    }
}

#[function_component(Dashboard)]
fn dashboard() -> Html {
    let lang = use_context::<UseStateHandle<Language>>().expect("Language context missing");
    let stats = use_state(|| (0, 0, 0, 0)); // (assets, ports, unbound, tasks)

    {
        let stats = stats.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let assets_req = Request::get("/api/assets").send().await;
                let tasks_req = Request::get("/api/tasks").send().await;
                
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
            });
            || ()
        });
    }

    let (total_assets, total_ports, unbound_ports, total_tasks) = *stats;

    html! {
        <div class="container p-4">
            <h1 class="title">{lang.t("dashboard")}</h1>
            <div class="columns is-multiline">
                <div class="column is-3">
                    <div class="box has-background-info-light">
                        <div class="heading">{lang.t("total_assets")}</div>
                        <div class="title">{total_assets}</div>
                    </div>
                </div>
                <div class="column is-3">
                    <div class="box has-background-primary-light">
                        <div class="heading">{lang.t("total_ports")}</div>
                        <div class="title">{total_ports}</div>
                    </div>
                </div>
                <div class="column is-3">
                    <div class="box has-background-danger-light">
                        <div class="heading">{lang.t("unbound_ports")}</div>
                        <div class="title has-text-danger">{unbound_ports}</div>
                    </div>
                </div>
                <div class="column is-3">
                    <div class="box has-background-warning-light">
                        <div class="heading">{lang.t("active_tasks")}</div>
                        <div class="title">{total_tasks}</div>
                    </div>
                </div>
            </div>
            
            <div class="box">
                <h2 class="subtitle">{lang.t("system_status")}</h2>
                <p>{lang.t("system_running_msg")}</p>
            </div>
        </div>
    }
}

#[function_component(TaskCenter)]
fn task_center() -> Html {
    let lang = use_context::<UseStateHandle<Language>>().expect("Language context missing");
    let auth = use_context::<AuthContext>().expect("Auth context missing");
    let tasks = use_state(|| vec![]);
    let is_modal_active = use_state(|| false);
    let form_id = use_state(|| None::<String>);
    let form_name = use_state(|| "".to_string());
    let form_target = use_state(|| "".to_string());
    let form_policy = use_state(|| "TOP1000".to_string());
    let form_domain_brute = use_state(|| false);
    let form_service_detection = use_state(|| false);
    let form_os_detection = use_state(|| false);
    let form_site_identify = use_state(|| false);

    let token = auth.token.clone().unwrap_or_default();
    let user_role = auth.user.as_ref().map(|u| u.role.clone());
    let is_sec_admin = user_role == Some(Role::SecAdmin);

    // Fetch tasks
    {
        let tasks = tasks.clone();
        let token = token.clone();
        use_effect_with((), move |_| {
            let tasks = tasks.clone();
            spawn_local(async move {
                let req = Request::get("/api/tasks").header("Authorization", &token);
                if let Ok(resp) = req.send().await {
                    if let Ok(data) = resp.json::<Vec<Task>>().await {
                        tasks.set(data);
                    }
                }
            });
            || ()
        });
    }

    let open_modal = {
        let is_modal_active = is_modal_active.clone();
        let form_id = form_id.clone();
        let form_name = form_name.clone();
        let form_target = form_target.clone();
        let form_policy = form_policy.clone();
        let form_domain_brute = form_domain_brute.clone();
        let form_service_detection = form_service_detection.clone();
        let form_os_detection = form_os_detection.clone();
        let form_site_identify = form_site_identify.clone();

        Callback::from(move |_| {
            form_id.set(None);
            form_name.set("".to_string());
            form_target.set("".to_string());
            form_policy.set("TOP1000".to_string());
            form_domain_brute.set(false);
            form_service_detection.set(false);
            form_os_detection.set(false);
            form_site_identify.set(false);
            is_modal_active.set(true)
        })
    };

    let close_modal = {
        let is_modal_active = is_modal_active.clone();
        Callback::from(move |_| is_modal_active.set(false))
    };

    let on_submit_task = {
        let tasks = tasks.clone();
        let form_id = form_id.clone();
        let name = form_name.clone();
        let target = form_target.clone();
        let policy = form_policy.clone();
        let domain_brute = form_domain_brute.clone();
        let service_detection = form_service_detection.clone();
        let os_detection = form_os_detection.clone();
        let site_identify = form_site_identify.clone();
        let is_modal_active = is_modal_active.clone();
        let token = token.clone();
        
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            let tasks = tasks.clone();
            let id_val = (*form_id).clone();
            let name_val = (*name).clone();
            let target_val = (*target).clone();
            let policy_val = (*policy).clone();
            let domain_brute_val = *domain_brute;
            let service_detection_val = *service_detection;
            let os_detection_val = *os_detection;
            let site_identify_val = *site_identify;
            let is_modal_active = is_modal_active.clone();
            let token = token.clone();

            spawn_local(async move {
                let req = CreateTaskRequest {
                    name: name_val,
                    target: target_val,
                    port_policy: policy_val,
                    domain_brute: domain_brute_val,
                    service_detection: service_detection_val,
                    os_detection: os_detection_val,
                    site_identify: site_identify_val,
                };

                let _ = if let Some(id) = id_val {
                    let url = format!("/api/tasks/{}", id);
                    Request::put(&url).header("Authorization", &token).json(&req).unwrap().send().await
                } else {
                    Request::post("/api/tasks").header("Authorization", &token).json(&req).unwrap().send().await
                };
                
                // Refresh
                if let Ok(resp) = Request::get("/api/tasks").header("Authorization", &token).send().await {
                    if let Ok(data) = resp.json::<Vec<Task>>().await {
                        tasks.set(data);
                    }
                }
                is_modal_active.set(false);
            });
        })
    };

    let on_delete_task = {
        let tasks = tasks.clone();
        let token = token.clone();
        Callback::from(move |id: String| {
            let tasks = tasks.clone();
            let token = token.clone();
            spawn_local(async move {
                let url = format!("/api/tasks/{}", id);
                let _ = Request::delete(&url).header("Authorization", &token).send().await;
                
                if let Ok(resp) = Request::get("/api/tasks").header("Authorization", &token).send().await {
                    if let Ok(data) = resp.json::<Vec<Task>>().await {
                        tasks.set(data);
                    }
                }
            });
        })
    };

    let on_edit_task = {
        let form_id = form_id.clone();
        let form_name = form_name.clone();
        let form_target = form_target.clone();
        let form_policy = form_policy.clone();
        let form_domain_brute = form_domain_brute.clone();
        let form_service_detection = form_service_detection.clone();
        let form_os_detection = form_os_detection.clone();
        let form_site_identify = form_site_identify.clone();
        let is_modal_active = is_modal_active.clone();

        Callback::from(move |task: Task| {
            form_id.set(Some(task.id));
            form_name.set(task.name);
            form_target.set(task.target);
            form_policy.set(task.port_policy);
            form_domain_brute.set(task.domain_brute);
            form_service_detection.set(task.service_detection);
            form_os_detection.set(task.os_detection);
            form_site_identify.set(task.site_identify);
            is_modal_active.set(true);
        })
    };

    html! {
        <div class="container p-4">
            <div class="level">
                <div class="level-left">
                    <h1 class="title">{lang.t("task_center")}</h1>
                </div>
                <div class="level-right">
                    if is_sec_admin {
                        <button class="button is-primary" onclick={open_modal}>
                            {lang.t("create_new_task")}
                        </button>
                    }
                </div>
            </div>
            
            <div class="box">
                <h2 class="subtitle">{lang.t("task_list")}</h2>
                <table class="table is-fullwidth is-striped">
                    <thead>
                        <tr>
                            <th>{lang.t("name")}</th>
                            <th>{lang.t("target")}</th>
                            <th>{lang.t("status")}</th>
                            <th>{lang.t("assets_found")}</th>
                            <th>{lang.t("risks_found")}</th>
                            <th>{lang.t("action")}</th>
                        </tr>
                    </thead>
                    <tbody>
                        {
                            for tasks.iter().map(|task| {
                                let status_color = match task.status {
                                    TaskStatus::Completed => "is-success",
                                    TaskStatus::Running => "is-warning",
                                    TaskStatus::Failed => "is-danger",
                                    TaskStatus::Pending => "is-light",
                                };
                                let on_delete = on_delete_task.clone();
                                let on_edit = on_edit_task.clone();
                                let id = task.id.clone();
                                let t_clone = task.clone();
                                let on_click_delete = Callback::from(move |_| on_delete.emit(id.clone()));
                                let on_click_edit = Callback::from(move |_| on_edit.emit(t_clone.clone()));

                                html! {
                                    <tr>
                                        <td>{&task.name}</td>
                                        <td>{&task.target}</td>
                                        <td><span class={classes!("tag", status_color)}>{format!("{:?}", task.status)}</span></td>
                                        <td>{task.found_assets}</td>
                                        <td>{task.found_risks}</td>
                                        <td>
                                            if is_sec_admin {
                                                <div class="buttons">
                                                    <button class="button is-small is-info" onclick={on_click_edit}>
                                                        {lang.t("edit")}
                                                    </button>
                                                    <button class="button is-small is-danger" onclick={on_click_delete}>
                                                        {lang.t("delete")}
                                                    </button>
                                                </div>
                                            }
                                        </td>
                                    </tr>
                                }
                            })
                        }
                    </tbody>
                </table>
            </div>

            if *is_modal_active {
                <div class="modal is-active">
                    <div class="modal-background" onclick={close_modal.clone()}></div>
                    <div class="modal-card">
                        <header class="modal-card-head">
                            <p class="modal-card-title">
                                {if (*form_id).is_some() { lang.t("edit_task") } else { lang.t("create_new_task") }}
                            </p>
                            <button class="delete" onclick={close_modal.clone()}></button>
                        </header>
                        <section class="modal-card-body">
                            <form>
                                <div class="field">
                                    <label class="label">{lang.t("task_name")}</label>
                                    <div class="control">
                                        <input class="input" type="text" placeholder="e.g. Monthly Scan" 
                                            value={(*form_name).clone()}
                                            oninput={Callback::from(move |e: InputEvent| {
                                                let input: HtmlInputElement = e.target_unchecked_into();
                                                form_name.set(input.value());
                                            })}
                                        />
                                    </div>
                                </div>
                                <div class="field">
                                    <label class="label">{lang.t("target_ip_domain")}</label>
                                    <div class="control">
                                        <input class="input" type="text" placeholder="e.g. 192.168.1.100" 
                                            value={(*form_target).clone()}
                                            oninput={Callback::from(move |e: InputEvent| {
                                                let input: HtmlInputElement = e.target_unchecked_into();
                                                form_target.set(input.value());
                                            })}
                                        />
                                    </div>
                                </div>

                                <div class="field">
                                    <label class="label">{lang.t("port_policy")}</label>
                                    <div class="control">
                                        <div class="select is-fullwidth">
                                            <select value={(*form_policy).clone()} onchange={Callback::from(move |e: Event| {
                                                let input: HtmlInputElement = e.target_unchecked_into();
                                                form_policy.set(input.value());
                                            })}>
                                                <option value="TOP1000">{"TOP 1000"}</option>
                                                <option value="TOP100">{"TOP 100"}</option>
                                                <option value="ALL">{"ALL PORTS"}</option>
                                                <option value="TEST">{"TEST (Few Ports)"}</option>
                                            </select>
                                        </div>
                                    </div>
                                </div>

                                <div class="field">
                                    <label class="checkbox">
                                        <input type="checkbox" checked={*form_domain_brute} onclick={Callback::from(move |_| form_domain_brute.set(!*form_domain_brute))} />
                                        {" "}{lang.t("domain_brute")}
                                    </label>
                                </div>
                                <div class="field">
                                    <label class="checkbox">
                                        <input type="checkbox" checked={*form_service_detection} onclick={Callback::from(move |_| form_service_detection.set(!*form_service_detection))} />
                                        {" "}{lang.t("service_detection")}
                                    </label>
                                </div>
                                <div class="field">
                                    <label class="checkbox">
                                        <input type="checkbox" checked={*form_os_detection} onclick={Callback::from(move |_| form_os_detection.set(!*form_os_detection))} />
                                        {" "}{lang.t("os_detection")}
                                    </label>
                                </div>
                                <div class="field">
                                    <label class="checkbox">
                                        <input type="checkbox" checked={*form_site_identify} onclick={Callback::from(move |_| form_site_identify.set(!*form_site_identify))} />
                                        {" "}{lang.t("site_identify")}
                                    </label>
                                </div>
                            </form>
                        </section>
                        <footer class="modal-card-foot">
                            <button class="button is-primary" onclick={on_submit_task}>
                                {if (*form_id).is_some() { lang.t("save") } else { lang.t("start_task") }}
                            </button>
                            <button class="button" onclick={close_modal}>
                                {lang.t("cancel")}
                            </button>
                        </footer>
                    </div>
                </div>
            }
        </div>
    }
}

#[function_component(RiskCenter)]
fn risk_center() -> Html {
    let lang = use_context::<UseStateHandle<Language>>().expect("Language context missing");
    let risks = use_state(|| vec![]);

    {
        let risks = risks.clone();
        use_effect_with((), move |_| {
            let risks = risks.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(resp) = Request::get("/api/risks").send().await {
                    if let Ok(data) = resp.json::<Vec<Risk>>().await {
                        risks.set(data);
                    }
                }
            });
            || ()
        });
    }

    let on_resolve = {
        let risks = risks.clone();
        Callback::from(move |risk_id: String| {
             let risks = risks.clone();
             wasm_bindgen_futures::spawn_local(async move {
                 let url = format!("/api/risks/{}/resolve", risk_id);
                 let _ = Request::post(&url).send().await;
                 
                 // Refresh
                 if let Ok(resp) = Request::get("/api/risks").send().await {
                    if let Ok(data) = resp.json::<Vec<Risk>>().await {
                        risks.set(data);
                    }
                 }
             });
        })
    };

    html! {
        <div class="container p-4">
            <h1 class="title">{lang.t("risk_monitoring")}</h1>
            <div class="box">
                 {
                    if risks.is_empty() {
                        html! { <p class="has-text-grey">{lang.t("no_risks")}</p> }
                    } else {
                        html! {
                            <table class="table is-fullwidth is-hoverable">
                                <thead>
                                    <tr>
                                        <th>{lang.t("severity")}</th>
                                        <th>{lang.t("asset")}</th>
                                        <th>{lang.t("port")}</th>
                                        <th>{lang.t("description")}</th>
                                        <th>{lang.t("status")}</th>
                                        <th>{lang.t("solution")}</th>
                                        <th>{lang.t("action")}</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {
                                        for risks.iter().map(|risk| {
                                            let sev_color = match risk.severity.as_str() {
                                                "Critical" => "is-danger is-dark",
                                                "High" => "is-danger",
                                                "Medium" => "is-warning",
                                                _ => "is-info",
                                            };
                                            
                                            let on_resolve = on_resolve.clone();
                                            let id = risk.id.clone();
                                            let on_click_resolve = Callback::from(move |_| on_resolve.emit(id.clone()));

                                            html! {
                                                <tr>
                                                    <td><span class={classes!("tag", sev_color)}>{&risk.severity}</span></td>
                                                    <td>{&risk.asset_ip}</td>
                                                    <td>{risk.port}</td>
                                                    <td>{&risk.description}</td>
                                                    <td>{format!("{:?}", risk.status)}</td>
                                                    <td>{risk.solution.clone().unwrap_or_default()}</td>
                                                    <td>
                                                        if risk.status == RiskStatus::Open {
                                                            <button class="button is-small is-success" onclick={on_click_resolve}>{lang.t("resolve")}</button>
                                                        }
                                                    </td>
                                                </tr>
                                            }
                                        })
                                    }
                                </tbody>
                            </table>
                        }
                    }
                }
            </div>
        </div>
    }
}

#[function_component(AssetCenter)]
fn asset_center() -> Html {
    let lang = use_context::<UseStateHandle<Language>>().expect("Language context missing");
    let auth = use_context::<AuthContext>().expect("Auth context missing");
    let assets = use_state(|| vec![]);
    let is_add_modal_active = use_state(|| false);
    
    // Asset Form State
    let form_id = use_state(|| None::<i32>);
    let form_name = use_state(|| "".to_string());
    let form_ip = use_state(|| "".to_string());
    let form_contact_person = use_state(|| "".to_string());
    let form_contact_phone = use_state(|| "".to_string());
    let form_ports = use_state(|| vec![]); // Changed from String to Vec<PortInfo>
    
    // New Port Form State
    let port_form_port = use_state(|| "".to_string());
    let port_form_service = use_state(|| "".to_string());
    let port_form_system = use_state(|| "".to_string());
    let port_form_middleware = use_state(|| "".to_string());
    let editing_port_num = use_state(|| None::<u16>); // Track which port is being edited

    // Bind Modal State
    let active_port = use_state(|| None::<(String, u16)>);
    let bind_system = use_state(|| "".to_string());
    let bind_middleware = use_state(|| "".to_string());

    let token = auth.token.clone().unwrap_or_default();

    // Fetch Assets
    {
        let assets = assets.clone();
        let token = token.clone();
        use_effect_with((), move |_| {
            let assets = assets.clone();
            spawn_local(async move {
                let req = Request::get("/api/assets").header("Authorization", &token);
                if let Ok(resp) = req.send().await {
                    if let Ok(data) = resp.json::<Vec<Asset>>().await {
                        assets.set(data);
                    }
                }
            });
            || ()
        });
    }

    let open_add_modal = {
        let is_add_modal_active = is_add_modal_active.clone();
        let form_id = form_id.clone();
        let form_name = form_name.clone();
        let form_ip = form_ip.clone();
        let form_contact_person = form_contact_person.clone();
        let form_contact_phone = form_contact_phone.clone();
        let form_ports = form_ports.clone();
        
        // Reset port form
        let pf_port = port_form_port.clone();
        let pf_service = port_form_service.clone();
        let pf_system = port_form_system.clone();
        let pf_middleware = port_form_middleware.clone();
        let editing_port_num = editing_port_num.clone();

        Callback::from(move |_| {
            form_id.set(None);
            form_name.set("".to_string());
            form_ip.set("".to_string());
            form_contact_person.set("".to_string());
            form_contact_phone.set("".to_string());
            form_ports.set(vec![]);
            
            pf_port.set("".to_string());
            pf_service.set("".to_string());
            pf_system.set("".to_string());
            pf_middleware.set("".to_string());
            editing_port_num.set(None);

            is_add_modal_active.set(true)
        })
    };

    let close_add_modal = {
        let is_add_modal_active = is_add_modal_active.clone();
        Callback::from(move |_| is_add_modal_active.set(false))
    };

    let on_submit_asset = {
        let assets = assets.clone();
        let form_id = form_id.clone();
        let name = form_name.clone();
        let ip = form_ip.clone();
        let contact_person = form_contact_person.clone();
        let contact_phone = form_contact_phone.clone();
        let form_ports = form_ports.clone();
        let is_add_modal_active = is_add_modal_active.clone();
        let token = token.clone();
        
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            let assets = assets.clone();
            let id_val = *form_id;
            let name_val = (*name).clone();
            let ip_val = (*ip).clone();
            let contact_person_val = (*contact_person).clone();
            let contact_phone_val = (*contact_phone).clone();
            let form_ports_val = (*form_ports).clone();
            let is_add_modal_active = is_add_modal_active.clone();
            let token = token.clone();

            spawn_local(async move {
                let asset_data = Asset {
                    id: id_val,
                    name: name_val,
                    ip: ip_val,
                    zone: NetworkZone::Internet, // Will be auto-assigned/re-calculated by backend
                    ports: form_ports_val,
                    last_scanned: None,
                    contact_person: if contact_person_val.is_empty() { None } else { Some(contact_person_val) },
                    contact_phone: if contact_phone_val.is_empty() { None } else { Some(contact_phone_val) },
                    created_by: None, // Backend handles this
                    updated_by: None, // Backend handles this
                };

                let _ = if let Some(id) = id_val {
                    let url = format!("/api/assets/{}", id);
                    Request::put(&url)
                        .header("Authorization", &token)
                        .json(&asset_data)
                        .unwrap()
                        .send()
                        .await
                } else {
                    Request::post("/api/assets")
                        .header("Authorization", &token)
                        .json(&asset_data)
                        .unwrap()
                        .send()
                        .await
                };
                
                if let Ok(resp) = Request::get("/api/assets").header("Authorization", &token).send().await {
                    if let Ok(data) = resp.json::<Vec<Asset>>().await {
                        assets.set(data);
                    }
                }
                is_add_modal_active.set(false);
            });
        })
    };

    let on_delete_asset = {
        let assets = assets.clone();
        let token = token.clone();
        Callback::from(move |id: i32| {
            let assets = assets.clone();
            let token = token.clone();
            spawn_local(async move {
                let url = format!("/api/assets/{}", id);
                let _ = Request::delete(&url).header("Authorization", &token).send().await;
                
                if let Ok(resp) = Request::get("/api/assets").header("Authorization", &token).send().await {
                    if let Ok(data) = resp.json::<Vec<Asset>>().await {
                        assets.set(data);
                    }
                }
            });
        })
    };

    let on_edit_asset = {
        let form_id = form_id.clone();
        let form_name = form_name.clone();
        let form_ip = form_ip.clone();
        let form_contact_person = form_contact_person.clone();
        let form_contact_phone = form_contact_phone.clone();
        let form_ports = form_ports.clone();
        let is_add_modal_active = is_add_modal_active.clone();

        Callback::from(move |asset: Asset| {
            form_id.set(asset.id);
            form_name.set(asset.name);
            form_ip.set(asset.ip);
            form_contact_person.set(asset.contact_person.unwrap_or_default());
            form_contact_phone.set(asset.contact_phone.unwrap_or_default());
            form_ports.set(asset.ports);
            is_add_modal_active.set(true);
        })
    };

    let on_add_port_click = {
        let form_id = form_id.clone();
        let form_ports = form_ports.clone();
        let pf_port = port_form_port.clone();
        let pf_service = port_form_service.clone();
        let pf_system = port_form_system.clone();
        let pf_middleware = port_form_middleware.clone();
        let editing_port_num = editing_port_num.clone();
        
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            if let Ok(port_num) = (*pf_port).parse::<u16>() {
                let new_port = PortInfo {
                    port: port_num,
                    is_open: true,
                    service: if pf_service.is_empty() { None } else { Some((*pf_service).clone()) },
                    is_bound: true, // Assuming manual entry implies bound/confirmed
                    system_name: if pf_system.is_empty() { None } else { Some((*pf_system).clone()) },
                    middleware: if pf_middleware.is_empty() { None } else { Some((*pf_middleware).clone()) },
                    created_by: None,
                    updated_by: None,
                };

                let form_id = form_id.clone();
                let form_ports = form_ports.clone();
                let new_port_clone = new_port.clone();
                let editing_port = *editing_port_num;

                if let Some(id) = *form_id {
                    // Edit Mode: Call API
                    wasm_bindgen_futures::spawn_local(async move {
                        if let Some(old_port) = editing_port {
                             // Update existing port
                             let url = format!("/api/assets/{}/ports/{}", id, old_port);
                             if let Ok(req) = Request::put(&url).json(&new_port_clone) {
                                 if let Ok(resp) = req.send().await {
                                     if resp.ok() {
                                         let mut ports = (*form_ports).clone();
                                         if let Some(idx) = ports.iter().position(|p| p.port == old_port) {
                                             ports[idx] = new_port_clone;
                                             // If port number changed, we might need to resort or handle uniqueness
                                             ports.sort_by(|a, b| a.port.cmp(&b.port));
                                             form_ports.set(ports);
                                         }
                                     }
                                 }
                             }
                        } else {
                            // Add new port
                            let url = format!("/api/assets/{}/ports", id);
                            if let Ok(req) = Request::post(&url).json(&new_port_clone) {
                                if let Ok(resp) = req.send().await {
                                    if resp.ok() {
                                        // Add to local list
                                        let mut ports = (*form_ports).clone();
                                        ports.push(new_port_clone);
                                        ports.sort_by(|a, b| a.port.cmp(&b.port));
                                        form_ports.set(ports);
                                    }
                                }
                            }
                        }
                    });
                } else {
                    // Add Mode: Local only
                    let mut ports = (*form_ports).clone();
                    if let Some(old_port) = editing_port {
                        // Update
                        if let Some(idx) = ports.iter().position(|p| p.port == old_port) {
                             ports[idx] = new_port;
                             ports.sort_by(|a, b| a.port.cmp(&b.port));
                             form_ports.set(ports);
                        }
                    } else {
                        // Add
                        // Check if exists
                        if !ports.iter().any(|p| p.port == port_num) {
                            ports.push(new_port);
                            ports.sort_by(|a, b| a.port.cmp(&b.port));
                            form_ports.set(ports);
                        }
                    }
                }
                
                // Clear inputs
                pf_port.set("".to_string());
                pf_service.set("".to_string());
                pf_system.set("".to_string());
                pf_middleware.set("".to_string());
                editing_port_num.set(None);
            }
        })
    };

    let on_edit_port_click = {
         let form_ports = form_ports.clone();
         let pf_port = port_form_port.clone();
         let pf_service = port_form_service.clone();
         let pf_system = port_form_system.clone();
         let pf_middleware = port_form_middleware.clone();
         let editing_port_num = editing_port_num.clone();

         Callback::from(move |port_num: u16| {
             if let Some(port) = form_ports.iter().find(|p| p.port == port_num) {
                 pf_port.set(port.port.to_string());
                 pf_service.set(port.service.clone().unwrap_or_default());
                 pf_system.set(port.system_name.clone().unwrap_or_default());
                 pf_middleware.set(port.middleware.clone().unwrap_or_default());
                 editing_port_num.set(Some(port_num));
             }
         })
    };

    let on_delete_port_click = {
        let form_id = form_id.clone();
        let form_ports = form_ports.clone();
        
        Callback::from(move |port_num: u16| {
             let form_id = form_id.clone();
             let form_ports = form_ports.clone();
             
             if let Some(id) = *form_id {
                 // Edit Mode: Call API
                 wasm_bindgen_futures::spawn_local(async move {
                     let url = format!("/api/assets/{}/ports/{}", id, port_num);
                     if let Ok(resp) = Request::delete(&url).send().await {
                         if resp.ok() {
                             let mut ports = (*form_ports).clone();
                             ports.retain(|p| p.port != port_num);
                             form_ports.set(ports);
                         }
                     }
                 });
             } else {
                 // Add Mode: Local only
                 let mut ports = (*form_ports).clone();
                 ports.retain(|p| p.port != port_num);
                 form_ports.set(ports);
             }
        })
    };

    let on_bind_submit = {
        let assets = assets.clone();
        let active_port = active_port.clone();
        let bind_system = bind_system.clone();
        let bind_middleware = bind_middleware.clone();

        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            if let Some((ip, port)) = (*active_port).clone() {
                let assets = assets.clone();
                let active_port = active_port.clone();
                let system_name = (*bind_system).clone();
                let middleware = (*bind_middleware).clone();
                let ip = ip.clone();

                wasm_bindgen_futures::spawn_local(async move {
                    let req = PortBindingRequest { system_name, middleware };
                    let url = format!("/api/assets/{}/ports/{}/bind", ip, port);
                    let _ = Request::post(&url).json(&req).unwrap().send().await;
                    
                    if let Ok(resp) = Request::get("/api/assets").send().await {
                        if let Ok(data) = resp.json::<Vec<Asset>>().await {
                            assets.set(data);
                        }
                    }
                    active_port.set(None);
                });
            }
        })
    };

    // Port Form Input Handlers
    let on_input_port_num = {
        let port_form_port = port_form_port.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            port_form_port.set(input.value());
        })
    };
    let on_input_port_service = {
        let port_form_service = port_form_service.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            port_form_service.set(input.value());
        })
    };
    let on_input_port_system = {
        let port_form_system = port_form_system.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            port_form_system.set(input.value());
        })
    };
    let on_input_port_middleware = {
        let port_form_middleware = port_form_middleware.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            port_form_middleware.set(input.value());
        })
    };

    let close_bind_modal = {
        let active_port = active_port.clone();
        Callback::from(move |_| active_port.set(None))
    };

    html! {
        <div class="container p-4">
            <div class="level">
                <div class="level-left">
                    <h1 class="title">{lang.t("asset_management")}</h1>
                </div>
                <div class="level-right">
                    <button class="button is-primary" onclick={open_add_modal}>
                        {lang.t("add_asset")}
                    </button>
                </div>
            </div>

            <div class="box">
                {
                    for assets.iter().map(|asset| {
                        let on_delete = on_delete_asset.clone();
                        let on_edit = on_edit_asset.clone();
                        let a_clone = asset.clone();
                        let id = asset.id;
                        
                        let on_click_delete = Callback::from(move |e: MouseEvent| {
                            e.prevent_default(); // Prevent card collapse if any
                            if let Some(id) = id {
                                on_delete.emit(id);
                            }
                        });
                        
                        let on_click_edit = Callback::from(move |e: MouseEvent| {
                            e.prevent_default();
                            on_edit.emit(a_clone.clone());
                        });

                        html! {
                            <div class="card mb-4">
                                <header class="card-header">
                                    <p class="card-header-title">
                                        {&asset.name} <span class="tag is-light ml-2">{&asset.ip}</span>
                                    </p>
                                    <div class="card-header-icon" aria-label="more options">
                                        <div class="buttons">
                                            <button class="button is-small is-info" onclick={on_click_edit}>
                                                {lang.t("edit")}
                                            </button>
                                            <button class="button is-small is-danger" onclick={on_click_delete}>
                                                {lang.t("delete")}
                                            </button>
                                        </div>
                                    </div>
                                </header>
                                <div class="card-content">
                                    <div class="content">
                                        <div class="tags has-addons mb-2">
                                            <span class="tag is-dark">{lang.t("zone")}</span>
                                            <span class="tag is-info">{format!("{:?}", asset.zone)}</span>
                                        </div>
                                        {
                                            if asset.contact_person.is_some() || asset.contact_phone.is_some() {
                                                html! {
                                                    <div class="tags has-addons mb-2">
                                                        if let Some(person) = &asset.contact_person {
                                                            <span class="tag is-dark">{lang.t("contact_person")}</span>
                                                            <span class="tag is-info">{person}</span>
                                                        }
                                                        if let Some(phone) = &asset.contact_phone {
                                                            <span class={classes!("tag", "is-dark", if asset.contact_person.is_some() { "ml-2" } else { "" })}>{lang.t("contact_phone")}</span>
                                                            <span class="tag is-info">{phone}</span>
                                                        }
                                                    </div>
                                                }
                                            } else {
                                                html! {}
                                            }
                                        }
                                        <div class="tags">
                                            {
                                                for asset.ports.iter().map(|port| {
                                                    let color = if port.is_bound { "is-success" } else { "is-danger" };
                                                    let active_port = active_port.clone();
                                                    let bind_system = bind_system.clone();
                                                    let bind_middleware = bind_middleware.clone();
                                                    let asset_ip = asset.ip.clone();
                                                    let port_num = port.port;
                                                    let p_sys = port.system_name.clone().unwrap_or_default();
                                                    let p_mid = port.middleware.clone().unwrap_or_default();

                                                    let on_port_click = Callback::from(move |_| {
                                                        bind_system.set(p_sys.clone());
                                                        bind_middleware.set(p_mid.clone());
                                                        active_port.set(Some((asset_ip.clone(), port_num)));
                                                    });

                                                    html! {
                                                        <span class={classes!("tag", "is-medium", color, "is-clickable")} onclick={on_port_click}>
                                                            {format!("{}/{}", port.port, port.service.clone().unwrap_or_default())}
                                                            if !port.is_bound { <span class="ml-1">{"⚠️"}</span> }
                                                        </span>
                                                    }
                                                })
                                            }
                                        </div>
                                    </div>
                                </div>
                            </div>
                        }
                    })
                }
            </div>

            if *is_add_modal_active {
                <div class="modal is-active">
                    <div class="modal-background" onclick={close_add_modal.clone()}></div>
                    <div class="modal-card">
                        <header class="modal-card-head">
                            <p class="modal-card-title">
                                {if (*form_id).is_some() { lang.t("edit_asset") } else { lang.t("add_asset") }}
                            </p>
                            <button class="delete" onclick={close_add_modal.clone()}></button>
                        </header>
                        <section class="modal-card-body">
                            <form>
                                <div class="field">
                                    <label class="label">{lang.t("name")}</label>
                                    <div class="control">
                                        <input class="input" type="text" value={(*form_name).clone()}
                                            oninput={Callback::from(move |e: InputEvent| {
                                                let input: HtmlInputElement = e.target_unchecked_into();
                                                form_name.set(input.value());
                                            })}
                                        />
                                    </div>
                                </div>
                                <div class="field">
                                    <label class="label">{lang.t("ip")}</label>
                                    <div class="control">
                                        <input class="input" type="text" value={(*form_ip).clone()}
                                            oninput={Callback::from(move |e: InputEvent| {
                                                let input: HtmlInputElement = e.target_unchecked_into();
                                                form_ip.set(input.value());
                                            })}
                                        />
                                    </div>
                                </div>
                                <div class="field">
                                    <label class="label">{lang.t("contact_person")}</label>
                                    <div class="control">
                                        <input class="input" type="text" value={(*form_contact_person).clone()}
                                            oninput={Callback::from(move |e: InputEvent| {
                                                let input: HtmlInputElement = e.target_unchecked_into();
                                                form_contact_person.set(input.value());
                                            })}
                                        />
                                    </div>
                                </div>
                                <div class="field">
                                    <label class="label">{lang.t("contact_phone")}</label>
                                    <div class="control">
                                        <input class="input" type="text" value={(*form_contact_phone).clone()}
                                            oninput={Callback::from(move |e: InputEvent| {
                                                let input: HtmlInputElement = e.target_unchecked_into();
                                                form_contact_phone.set(input.value());
                                            })}
                                        />
                                    </div>
                                </div>

                                <div class="field mt-5">
                                    <label class="label">{lang.t("ports")}</label>
                                    
                                    // Port List Table
                                    <table class="table is-fullwidth is-striped">
                                        <thead>
                                            <tr>
                                                <th>{lang.t("ports")}</th>
                                                <th>{lang.t("service")}</th>
                                                <th>{lang.t("system")}</th>
                                                <th>{lang.t("middleware")}</th>
                                                <th>{lang.t("action")}</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {
                                                for form_ports.iter().map(|port| {
                                                    let port_num = port.port;
                                                    let on_delete_port_click = on_delete_port_click.clone();
                                                    let on_edit_port_click = on_edit_port_click.clone();
                                                    
                                                    let on_delete = Callback::from(move |e: MouseEvent| {
                                                        e.prevent_default();
                                                        on_delete_port_click.emit(port_num);
                                                    });
                                                    
                                                    let on_edit = Callback::from(move |e: MouseEvent| {
                                                        e.prevent_default();
                                                        on_edit_port_click.emit(port_num);
                                                    });
                                                    
                                                    html! {
                                                        <tr>
                                                            <td>{port.port}</td>
                                                            <td>{port.service.clone().unwrap_or_default()}</td>
                                                            <td>{port.system_name.clone().unwrap_or_default()}</td>
                                                            <td>{port.middleware.clone().unwrap_or_default()}</td>
                                                            <td>
                                                                <button class="button is-small is-info mr-2" onclick={on_edit}>
                                                                    {lang.t("edit")}
                                                                </button>
                                                                <button class="button is-small is-danger" onclick={on_delete}>
                                                                    {lang.t("delete")}
                                                                </button>
                                                            </td>
                                                        </tr>
                                                    }
                                                })
                                            }
                                        </tbody>
                                    </table>

                                    // Add Port Form
                                    <div class="box has-background-light">
                                        <h4 class="subtitle is-6">{lang.t("add_port")}</h4>
                                        <div class="columns is-multiline">
                                            <div class="column is-2">
                                                <input class="input is-small" type="number" placeholder="Port" value={(*port_form_port).clone()}
                                                    oninput={on_input_port_num}
                                                />
                                            </div>
                                            <div class="column is-2">
                                                <input class="input is-small" type="text" placeholder={lang.t("service")} value={(*port_form_service).clone()}
                                                    oninput={on_input_port_service}
                                                />
                                            </div>
                                            <div class="column is-2">
                                                <input class="input is-small" type="text" placeholder={lang.t("system")} value={(*port_form_system).clone()}
                                                    oninput={on_input_port_system}
                                                />
                                            </div>
                                            <div class="column is-2">
                                                <input class="input is-small" type="text" placeholder={lang.t("middleware")} value={(*port_form_middleware).clone()}
                                                    oninput={on_input_port_middleware}
                                                />
                                            </div>
                                            <div class="column is-2">
                                                <button class="button is-small is-success" onclick={on_add_port_click}>
                                                    {
                                                        if (*editing_port_num).is_some() {
                                                            lang.t("save_port")
                                                        } else {
                                                            lang.t("add_port")
                                                        }
                                                    }
                                                </button>
                                                {
                                                    if (*editing_port_num).is_some() {
                                                        let pf_port = port_form_port.clone();
                                                        let pf_service = port_form_service.clone();
                                                        let pf_system = port_form_system.clone();
                                                        let pf_middleware = port_form_middleware.clone();
                                                        let editing_port_num = editing_port_num.clone();
                                                        let on_cancel = Callback::from(move |e: MouseEvent| {
                                                            e.prevent_default();
                                                            pf_port.set("".to_string());
                                                            pf_service.set("".to_string());
                                                            pf_system.set("".to_string());
                                                            pf_middleware.set("".to_string());
                                                            editing_port_num.set(None);
                                                        });
                                                        html! {
                                                            <button class="button is-small is-warning ml-2" onclick={on_cancel}>
                                                                {lang.t("cancel")}
                                                            </button>
                                                        }
                                                    } else {
                                                        html! {}
                                                    }
                                                }
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </form>
                        </section>
                        <footer class="modal-card-foot">
                            <button class="button is-link" onclick={on_submit_asset}>{lang.t("save")}</button>
                            <button class="button" onclick={close_add_modal}>{lang.t("cancel")}</button>
                        </footer>
                    </div>
                </div>
            }

            if let Some((ip, port)) = (*active_port).clone() {
                <div class="modal is-active">
                    <div class="modal-background" onclick={close_bind_modal.clone()}></div>
                    <div class="modal-card">
                        <header class="modal-card-head">
                            <p class="modal-card-title">{format!("{} {}:{}", lang.t("bind"), ip, port)}</p>
                            <button class="delete" onclick={close_bind_modal.clone()}></button>
                        </header>
                        <section class="modal-card-body">
                            <div class="field">
                                <label class="label">{lang.t("system")}</label>
                                <input class="input" type="text" value={(*bind_system).clone()}
                                    oninput={Callback::from(move |e: InputEvent| {
                                        let input: HtmlInputElement = e.target_unchecked_into();
                                        bind_system.set(input.value());
                                    })}
                                />
                            </div>
                            <div class="field">
                                <label class="label">{lang.t("middleware")}</label>
                                <input class="input" type="text" value={(*bind_middleware).clone()}
                                    oninput={Callback::from(move |e: InputEvent| {
                                        let input: HtmlInputElement = e.target_unchecked_into();
                                        bind_middleware.set(input.value());
                                    })}
                                />
                            </div>
                        </section>
                        <footer class="modal-card-foot">
                            <button class="button is-success" onclick={on_bind_submit}>{lang.t("save")}</button>
                            <button class="button" onclick={close_bind_modal}>{lang.t("cancel")}</button>
                        </footer>
                    </div>
                </div>
            }
        </div>
    }
}

#[function_component(UserManagement)]
fn user_management() -> Html {
    let lang = use_context::<UseStateHandle<Language>>().expect("Language context missing");
    let auth = use_context::<AuthContext>().expect("Auth context missing");
    let users = use_state(|| vec![]);
    let is_modal_active = use_state(|| false);
    let form_username = use_state(|| "".to_string());
    let form_password = use_state(|| "".to_string());
    let form_role = use_state(|| Role::Auditor);

    let token = auth.token.clone().unwrap_or_default();

    // Fetch Users
    {
        let users = users.clone();
        let token = token.clone();
        use_effect_with((), move |_| {
            let users = users.clone();
            spawn_local(async move {
                let req = Request::get("/api/users")
                    .header("Authorization", &token);
                if let Ok(resp) = req.send().await {
                    if let Ok(data) = resp.json::<Vec<User>>().await {
                        users.set(data);
                    }
                }
            });
            || ()
        });
    }

    let open_modal = {
        let is_modal_active = is_modal_active.clone();
        let form_username = form_username.clone();
        let form_password = form_password.clone();
        
        Callback::from(move |_| {
            form_username.set("".to_string());
            form_password.set("".to_string());
            is_modal_active.set(true)
        })
    };

    let close_modal = {
        let is_modal_active = is_modal_active.clone();
        Callback::from(move |_| is_modal_active.set(false))
    };

    let on_submit_user = {
        let users = users.clone();
        let username = form_username.clone();
        let password = form_password.clone();
        let role = form_role.clone();
        let is_modal_active = is_modal_active.clone();
        let token = token.clone();
        
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            let users = users.clone();
            let username_val = (*username).clone();
            let password_val = (*password).clone();
            let role_val = (*role).clone();
            let is_modal_active = is_modal_active.clone();
            let token = token.clone();

            spawn_local(async move {
                let req_body = CreateUserRequest {
                    username: username_val,
                    password: password_val,
                    role: role_val,
                };

                let _ = Request::post("/api/users")
                    .header("Authorization", &token)
                    .json(&req_body)
                    .unwrap()
                    .send()
                    .await;
                
                // Refresh
                if let Ok(resp) = Request::get("/api/users")
                    .header("Authorization", &token)
                    .send()
                    .await 
                {
                    if let Ok(data) = resp.json::<Vec<User>>().await {
                        users.set(data);
                    }
                }
                is_modal_active.set(false);
            });
        })
    };

    let on_delete_user = {
        let users = users.clone();
        let token = token.clone();
        Callback::from(move |id: String| {
            let users = users.clone();
            let token = token.clone();
            spawn_local(async move {
                let url = format!("/api/users/{}", id);
                let _ = Request::delete(&url)
                    .header("Authorization", &token)
                    .send()
                    .await;
                
                if let Ok(resp) = Request::get("/api/users")
                    .header("Authorization", &token)
                    .send()
                    .await 
                {
                    if let Ok(data) = resp.json::<Vec<User>>().await {
                        users.set(data);
                    }
                }
            });
        })
    };

    html! {
        <div class="container p-4">
            <div class="level">
                <div class="level-left">
                    <h1 class="title">{lang.t("user_management")}</h1>
                </div>
                <div class="level-right">
                    <button class="button is-primary" onclick={open_modal}>
                        {lang.t("create_user")}
                    </button>
                </div>
            </div>
            
            <div class="box">
                <table class="table is-fullwidth is-striped">
                    <thead>
                        <tr>
                            <th>{lang.t("username")}</th>
                            <th>{lang.t("role")}</th>
                            <th>{lang.t("action")}</th>
                        </tr>
                    </thead>
                    <tbody>
                        {
                            for users.iter().map(|user| {
                                let on_delete = on_delete_user.clone();
                                let id = user.id.clone();
                                let on_click_delete = Callback::from(move |_| on_delete.emit(id.clone()));

                                html! {
                                    <tr>
                                        <td>{&user.username}</td>
                                        <td>{format!("{:?}", user.role)}</td>
                                        <td>
                                            if user.username != "admin" {
                                                <button class="button is-small is-danger" onclick={on_click_delete}>
                                                    {lang.t("delete")}
                                                </button>
                                            }
                                        </td>
                                    </tr>
                                }
                            })
                        }
                    </tbody>
                </table>
            </div>

            if *is_modal_active {
                <div class="modal is-active">
                    <div class="modal-background" onclick={close_modal.clone()}></div>
                    <div class="modal-card">
                        <header class="modal-card-head">
                            <p class="modal-card-title">{lang.t("create_user")}</p>
                            <button class="delete" onclick={close_modal.clone()}></button>
                        </header>
                        <section class="modal-card-body">
                            <form>
                                <div class="field">
                                    <label class="label">{lang.t("username")}</label>
                                    <div class="control">
                                        <input class="input" type="text" 
                                            value={(*form_username).clone()}
                                            oninput={Callback::from(move |e: InputEvent| {
                                                let input: HtmlInputElement = e.target_unchecked_into();
                                                form_username.set(input.value());
                                            })}
                                        />
                                    </div>
                                </div>
                                <div class="field">
                                    <label class="label">{lang.t("password")}</label>
                                    <div class="control">
                                        <input class="input" type="password" 
                                            value={(*form_password).clone()}
                                            oninput={Callback::from(move |e: InputEvent| {
                                                let input: HtmlInputElement = e.target_unchecked_into();
                                                form_password.set(input.value());
                                            })}
                                        />
                                    </div>
                                </div>
                                <div class="field">
                                    <label class="label">{lang.t("role")}</label>
                                    <div class="control">
                                        <div class="select is-fullwidth">
                                            <select value={format!("{:?}", *form_role)} onchange={Callback::from(move |e: Event| {
                                                let input: HtmlInputElement = e.target_unchecked_into();
                                                let r = match input.value().as_str() {
                                                    "SysAdmin" => Role::SysAdmin,
                                                    "SecAdmin" => Role::SecAdmin,
                                                    _ => Role::Auditor,
                                                };
                                                form_role.set(r);
                                            })}>
                                                <option value="SysAdmin">{"SysAdmin"}</option>
                                                <option value="SecAdmin">{"SecAdmin"}</option>
                                                <option value="Auditor">{"Auditor"}</option>
                                            </select>
                                        </div>
                                    </div>
                                </div>
                            </form>
                        </section>
                        <footer class="modal-card-foot">
                            <button class="button is-primary" onclick={on_submit_user}>{lang.t("save")}</button>
                            <button class="button" onclick={close_modal}>{lang.t("cancel")}</button>
                        </footer>
                    </div>
                </div>
            }
        </div>
    }
}

#[function_component(AuditLogs)]
fn audit_logs() -> Html {
    let lang = use_context::<UseStateHandle<Language>>().expect("Language context missing");
    let auth = use_context::<AuthContext>().expect("Auth context missing");
    let logs = use_state(|| vec![]);

    let token = auth.token.clone().unwrap_or_default();

    // Fetch Logs
    {
        let logs = logs.clone();
        let token = token.clone();
        use_effect_with((), move |_| {
            let logs = logs.clone();
            spawn_local(async move {
                let req = Request::get("/api/logs")
                    .header("Authorization", &token);
                if let Ok(resp) = req.send().await {
                    if let Ok(data) = resp.json::<Vec<AuditLog>>().await {
                        logs.set(data);
                    }
                }
            });
            || ()
        });
    }

    html! {
        <div class="container p-4">
            <h1 class="title">{lang.t("audit_logs")}</h1>
            <div class="box">
                <table class="table is-fullwidth is-striped">
                    <thead>
                        <tr>
                            <th>{lang.t("timestamp")}</th>
                            <th>{lang.t("username")}</th>
                            <th>{lang.t("action")}</th>
                            <th>{lang.t("target")}</th>
                            <th>{lang.t("details")}</th>
                        </tr>
                    </thead>
                    <tbody>
                        {
                            for logs.iter().rev().map(|log| {
                                html! {
                                    <tr>
                                        <td>{log.timestamp.format("%Y-%m-%d %H:%M:%S").to_string()}</td>
                                        <td>{&log.username}</td>
                                        <td><span class="tag is-info">{&log.action}</span></td>
                                        <td>{&log.target}</td>
                                        <td>{&log.details}</td>
                                    </tr>
                                }
                            })
                        }
                    </tbody>
                </table>
            </div>
        </div>
    }
}

#[function_component(ZoneManagement)]
fn zone_management() -> Html {
    let lang = use_context::<UseStateHandle<Language>>().expect("Language context missing");
    let zones = use_state(|| vec![]);
    let is_modal_active = use_state(|| false);
    let form_id = use_state(|| "".to_string()); // For Edit
    let form_name = use_state(|| "".to_string());
    let form_cidr = use_state(|| "".to_string());
    let form_priority = use_state(|| "10".to_string());

    // Fetch Zones
    {
        let zones = zones.clone();
        use_effect_with((), move |_| {
            let zones = zones.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(resp) = Request::get("/api/zones").send().await {
                    if let Ok(data) = resp.json::<Vec<ZoneConfig>>().await {
                        zones.set(data);
                    }
                }
            });
            || ()
        });
    }

    let open_modal = {
        let is_modal_active = is_modal_active.clone();
        let form_id = form_id.clone();
        let form_name = form_name.clone();
        let form_cidr = form_cidr.clone();
        let form_priority = form_priority.clone();
        
        Callback::from(move |_| {
            form_id.set("".to_string());
            form_name.set("".to_string());
            form_cidr.set("".to_string());
            form_priority.set("10".to_string());
            is_modal_active.set(true)
        })
    };

    let close_modal = {
        let is_modal_active = is_modal_active.clone();
        Callback::from(move |_| is_modal_active.set(false))
    };

    let on_submit_zone = {
        let zones = zones.clone();
        let id = form_id.clone();
        let name = form_name.clone();
        let cidr = form_cidr.clone();
        let priority = form_priority.clone();
        let is_modal_active = is_modal_active.clone();
        
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            let zones = zones.clone();
            let id_val = (*id).clone();
            let name_val = (*name).clone();
            let cidr_val = (*cidr).clone();
            let priority_val = (*priority).parse::<i32>().unwrap_or(10);
            let is_modal_active = is_modal_active.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let zone_data = ZoneConfig {
                    id: id_val.clone(),
                    name: name_val,
                    cidr: cidr_val,
                    priority: priority_val,
                };

                let _ = if id_val.is_empty() {
                    Request::post("/api/zones")
                        .json(&zone_data)
                        .unwrap()
                        .send()
                        .await
                } else {
                    let url = format!("/api/zones/{}", id_val);
                    Request::put(&url)
                        .json(&zone_data)
                        .unwrap()
                        .send()
                        .await
                };
                
                if let Ok(resp) = Request::get("/api/zones").send().await {
                    if let Ok(data) = resp.json::<Vec<ZoneConfig>>().await {
                        zones.set(data);
                    }
                }
                is_modal_active.set(false);
            });
        })
    };

    let on_delete_zone = {
        let zones = zones.clone();
        Callback::from(move |id: String| {
            let zones = zones.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let url = format!("/api/zones/{}", id);
                let resp = Request::delete(&url).send().await;
                
                match resp {
                    Ok(response) => {
                        if response.ok() {
                            if let Ok(resp) = Request::get("/api/zones").send().await {
                                if let Ok(data) = resp.json::<Vec<ZoneConfig>>().await {
                                    zones.set(data);
                                }
                            }
                        } else {
                            // Extract error message from response (the Json string "...")
                            let text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                            // Since the backend returns Json<String>, the text will be quoted, e.g., "Cannot delete..."
                            // We can strip quotes if we want, but showing as is is okay for now.
                            gloo_dialogs::alert(&format!("Failed to delete zone: {}", text));
                        }
                    },
                    Err(e) => {
                         gloo_dialogs::alert(&format!("Network error: {}", e));
                    }
                }
            });
        })
    };

    let on_edit_zone = {
        let form_id = form_id.clone();
        let form_name = form_name.clone();
        let form_cidr = form_cidr.clone();
        let form_priority = form_priority.clone();
        let is_modal_active = is_modal_active.clone();

        Callback::from(move |zone: ZoneConfig| {
            form_id.set(zone.id);
            form_name.set(zone.name);
            form_cidr.set(zone.cidr);
            form_priority.set(zone.priority.to_string());
            is_modal_active.set(true);
        })
    };

    html! {
        <div class="container p-4">
            <div class="level">
                <div class="level-left">
                    <h1 class="title">{lang.t("zone_management")}</h1>
                </div>
                <div class="level-right">
                    <button class="button is-primary" onclick={open_modal}>
                        {lang.t("add_zone")}
                    </button>
                </div>
            </div>

            <div class="box">
                <table class="table is-fullwidth is-striped">
                    <thead>
                        <tr>
                            <th>{lang.t("name")}</th>
                            <th>{lang.t("cidr")}</th>
                            <th>{lang.t("priority")}</th>
                            <th>{lang.t("action")}</th>
                        </tr>
                    </thead>
                    <tbody>
                        {
                            for zones.iter().map(|zone| {
                                let on_delete = on_delete_zone.clone();
                                let on_edit = on_edit_zone.clone();
                                let id = zone.id.clone();
                                let z_clone = zone.clone();
                                let on_click_delete = Callback::from(move |_| on_delete.emit(id.clone()));
                                let on_click_edit = Callback::from(move |_| on_edit.emit(z_clone.clone()));

                                html! {
                                    <tr>
                                        <td>{&zone.name}</td>
                                        <td>{&zone.cidr}</td>
                                        <td>{zone.priority}</td>
                                        <td>
                                            <div class="buttons">
                                                <button class="button is-small is-info" onclick={on_click_edit}>
                                                    {lang.t("edit")}
                                                </button>
                                                <button class="button is-small is-danger" onclick={on_click_delete}>
                                                    {lang.t("delete")}
                                                </button>
                                            </div>
                                        </td>
                                    </tr>
                                }
                            })
                        }
                    </tbody>
                </table>
            </div>

            if *is_modal_active {
                <div class="modal is-active">
                    <div class="modal-background" onclick={close_modal.clone()}></div>
                    <div class="modal-card">
                        <header class="modal-card-head">
                            <p class="modal-card-title">
                                {if (*form_id).is_empty() { lang.t("add_zone") } else { lang.t("edit_zone") }}
                            </p>
                            <button class="delete" onclick={close_modal.clone()}></button>
                        </header>
                        <section class="modal-card-body">
                            <form>
                                <div class="field">
                                    <label class="label">{lang.t("name")}</label>
                                    <div class="control">
                                        <input class="input" type="text" placeholder="e.g. Office, Server Farm" value={(*form_name).clone()}
                                            oninput={Callback::from(move |e: InputEvent| {
                                                let input: HtmlInputElement = e.target_unchecked_into();
                                                form_name.set(input.value());
                                            })}
                                        />
                                    </div>
                                </div>
                                <div class="field">
                                    <label class="label">{lang.t("cidr")}</label>
                                    <div class="control">
                                        <input class="input" type="text" placeholder="e.g. 192.168.1.0/24" value={(*form_cidr).clone()}
                                            oninput={Callback::from(move |e: InputEvent| {
                                                let input: HtmlInputElement = e.target_unchecked_into();
                                                form_cidr.set(input.value());
                                            })}
                                        />
                                    </div>
                                </div>
                                <div class="field">
                                    <label class="label">{lang.t("priority")}</label>
                                    <div class="control">
                                        <input class="input" type="number" value={(*form_priority).clone()}
                                            oninput={Callback::from(move |e: InputEvent| {
                                                let input: HtmlInputElement = e.target_unchecked_into();
                                                form_priority.set(input.value());
                                            })}
                                        />
                                    </div>
                                </div>
                            </form>
                        </section>
                        <footer class="modal-card-foot">
                            <button class="button is-primary" onclick={on_submit_zone}>{lang.t("save")}</button>
                            <button class="button" onclick={close_modal}>{lang.t("cancel")}</button>
                        </footer>
                    </div>
                </div>
            }
        </div>
    }
}

fn switch(routes: Route, auth: &AuthState) -> Html {
    let is_logged_in = auth.token.is_some();
    let user_role = auth.user.as_ref().map(|u| u.role.clone());

    match routes {
        Route::Login => {
            if is_logged_in {
                html! { <Redirect<Route> to={Route::Dashboard}/> }
            } else {
                html! { <Login /> }
            }
        },
        Route::Dashboard => {
            if !is_logged_in {
                html! { <Redirect<Route> to={Route::Login}/> }
            } else {
                html! { <Dashboard /> }
            }
        },
        Route::TaskCenter => {
            if !is_logged_in {
                html! { <Redirect<Route> to={Route::Login}/> }
            } else if user_role != Some(Role::SecAdmin) && user_role != Some(Role::Auditor) {
                 // Auditor can view tasks too? Or maybe just SecAdmin. Let's say SecAdmin and Auditor (read-only for Auditor potentially)
                 // For now, let's stick to the requirement: SecAdmin manages assets/tasks.
                 // If strict separation:
                 if user_role == Some(Role::SecAdmin) {
                     html! { <TaskCenter /> }
                 } else {
                     html! { <h1 class="title">{"Access Denied: SecAdmin Only"}</h1> }
                 }
            } else {
                 html! { <TaskCenter /> }
            }
        },
        Route::AssetCenter => {
            if !is_logged_in {
                html! { <Redirect<Route> to={Route::Login}/> }
            } else if user_role == Some(Role::SecAdmin) {
                html! { <AssetCenter /> }
            } else {
                html! { <h1 class="title">{"Access Denied: SecAdmin Only"}</h1> }
            }
        },
        Route::ZoneManagement => {
            if !is_logged_in {
                html! { <Redirect<Route> to={Route::Login}/> }
            } else if user_role == Some(Role::SecAdmin) {
                 html! { <ZoneManagement /> }
            } else {
                 html! { <h1 class="title">{"Access Denied: SecAdmin Only"}</h1> }
            }
        },
        Route::RiskCenter => {
             if !is_logged_in {
                html! { <Redirect<Route> to={Route::Login}/> }
            } else if user_role == Some(Role::SecAdmin) {
                 html! { <RiskCenter /> }
            } else {
                 html! { <h1 class="title">{"Access Denied: SecAdmin Only"}</h1> }
            }
        },
        Route::UserManagement => {
            if !is_logged_in {
                html! { <Redirect<Route> to={Route::Login}/> }
            } else if user_role == Some(Role::SysAdmin) {
                html! { <UserManagement /> }
            } else {
                html! { <h1 class="title">{"Access Denied: SysAdmin Only"}</h1> }
            }
        },
        Route::AuditLogs => {
            if !is_logged_in {
                html! { <Redirect<Route> to={Route::Login}/> }
            } else if user_role == Some(Role::Auditor) {
                html! { <AuditLogs /> }
            } else {
                html! { <h1 class="title">{"Access Denied: Auditor Only"}</h1> }
            }
        },
        Route::NotFound => html! { <h1 class="title">{"404 Not Found"}</h1> },
    }
}

#[function_component(App)]
pub fn app() -> Html {
    let language = use_state(|| Language::Zh);
    let auth = use_state(|| AuthState { token: None, user: None });

    let render_route = {
        let auth_state = (*auth).clone();
        Callback::from(move |route: Route| switch(route, &auth_state))
    };

    html! {
        <ContextProvider<UseStateHandle<Language>> context={language}>
            <ContextProvider<AuthContext> context={auth.clone()}>
                <BrowserRouter>
                    {
                        if auth.token.is_some() {
                            html! {
                                <div class="columns is-gapless">
                                    <div class="column is-2">
                                        <Sidebar />
                                    </div>
                                    <div class="column">
                                        <Switch<Route> render={render_route} />
                                    </div>
                                </div>
                            }
                        } else {
                            html! {
                                <Switch<Route> render={render_route} />
                            }
                        }
                    }
                </BrowserRouter>
            </ContextProvider<AuthContext>>
        </ContextProvider<UseStateHandle<Language>>>
    }
}

#[wasm_bindgen(start)]
pub fn run_app() {
    yew::Renderer::<App>::new().render();
}
