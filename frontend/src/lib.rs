use gloo_net::http::Request;
use shared::{Asset, NetworkZone, PortBindingRequest, Task, TaskStatus, CreateTaskRequest, Risk, RiskStatus, ZoneConfig};
use wasm_bindgen::prelude::*;
use yew::prelude::*;
use yew_router::prelude::*;
use web_sys::HtmlInputElement;

#[derive(Clone, PartialEq)]
pub enum Language {
    Zh,
    En,
}

impl Language {
    fn t(&self, key: &str) -> String {
        match (self, key) {
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

            _ => key.to_string(),
        }
    }
}

#[derive(Clone, Routable, PartialEq)]
enum Route {
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
    #[not_found]
    #[at("/404")]
    NotFound,
}

#[function_component(Sidebar)]
fn sidebar() -> Html {
    let lang = use_context::<UseStateHandle<Language>>().expect("Language context missing");
    let route = use_route::<Route>().unwrap_or(Route::Dashboard);

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
                <li><Link<Route> to={Route::TaskCenter} classes={is_active(Route::TaskCenter)}>{lang.t("task_center")}</Link<Route>></li>
            </ul>
            <p class="menu-label">{lang.t("assets_risks")}</p>
            <ul class="menu-list">
                <li><Link<Route> to={Route::AssetCenter} classes={is_active(Route::AssetCenter)}>{lang.t("asset_management")}</Link<Route>></li>
                <li><Link<Route> to={Route::ZoneManagement} classes={is_active(Route::ZoneManagement)}>{lang.t("zone_management")}</Link<Route>></li>
                <li><Link<Route> to={Route::RiskCenter} classes={is_active(Route::RiskCenter)}>{lang.t("risk_monitoring")}</Link<Route>></li>
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
                let assets_req = Request::get("http://localhost:3003/api/assets").send().await;
                let tasks_req = Request::get("http://localhost:3003/api/tasks").send().await;
                
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

    // Fetch tasks
    {
        let tasks = tasks.clone();
        use_effect_with((), move |_| {
            let tasks = tasks.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(resp) = Request::get("http://localhost:3003/api/tasks").send().await {
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

            wasm_bindgen_futures::spawn_local(async move {
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
                    let url = format!("http://localhost:3003/api/tasks/{}", id);
                    Request::put(&url).json(&req).unwrap().send().await
                } else {
                    Request::post("http://localhost:3003/api/tasks").json(&req).unwrap().send().await
                };
                
                // Refresh
                if let Ok(resp) = Request::get("http://localhost:3003/api/tasks").send().await {
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
        Callback::from(move |id: String| {
            let tasks = tasks.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let url = format!("http://localhost:3003/api/tasks/{}", id);
                let _ = Request::delete(&url).send().await;
                
                if let Ok(resp) = Request::get("http://localhost:3003/api/tasks").send().await {
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
                    <button class="button is-primary" onclick={open_modal}>
                        {lang.t("create_new_task")}
                    </button>
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
                if let Ok(resp) = Request::get("http://localhost:3003/api/risks").send().await {
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
                 let url = format!("http://localhost:3003/api/risks/{}/resolve", risk_id);
                 let _ = Request::post(&url).send().await;
                 
                 // Refresh
                 if let Ok(resp) = Request::get("http://localhost:3003/api/risks").send().await {
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
    let assets = use_state(|| vec![]);
    let is_add_modal_active = use_state(|| false);
    let form_id = use_state(|| None::<i32>); // For Edit
    let form_name = use_state(|| "".to_string());
    let form_ip = use_state(|| "".to_string());
    
    // Binding Modal State
    let active_port = use_state(|| None::<(String, u16)>);
    let bind_owner = use_state(|| "".to_string());
    let bind_system = use_state(|| "".to_string());
    let bind_middleware = use_state(|| "".to_string());

    // Fetch Assets
    {
        let assets = assets.clone();
        use_effect_with((), move |_| {
            let assets = assets.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(resp) = Request::get("http://localhost:3003/api/assets").send().await {
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
        
        Callback::from(move |_| {
            form_id.set(None);
            form_name.set("".to_string());
            form_ip.set("".to_string());
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
        let is_add_modal_active = is_add_modal_active.clone();
        
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            let assets = assets.clone();
            let id_val = *form_id;
            let name_val = (*name).clone();
            let ip_val = (*ip).clone();
            let is_add_modal_active = is_add_modal_active.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let asset_data = Asset {
                    id: id_val,
                    name: name_val,
                    ip: ip_val,
                    zone: NetworkZone::Internet, // Will be auto-assigned/re-calculated by backend
                    ports: vec![], // Ports are preserved in backend update, ignored in create if empty
                    last_scanned: None,
                };

                let _ = if let Some(id) = id_val {
                    let url = format!("http://localhost:3003/api/assets/{}", id);
                    Request::put(&url)
                        .json(&asset_data)
                        .unwrap()
                        .send()
                        .await
                } else {
                    Request::post("http://localhost:3003/api/assets")
                        .json(&asset_data)
                        .unwrap()
                        .send()
                        .await
                };
                
                if let Ok(resp) = Request::get("http://localhost:3003/api/assets").send().await {
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
        Callback::from(move |id: i32| {
            let assets = assets.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let url = format!("http://localhost:3003/api/assets/{}", id);
                let _ = Request::delete(&url).send().await;
                
                if let Ok(resp) = Request::get("http://localhost:3003/api/assets").send().await {
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
        let is_add_modal_active = is_add_modal_active.clone();

        Callback::from(move |asset: Asset| {
            form_id.set(asset.id);
            form_name.set(asset.name);
            form_ip.set(asset.ip);
            is_add_modal_active.set(true);
        })
    };

    let on_bind_submit = {
        let assets = assets.clone();
        let active_port = active_port.clone();
        let bind_owner = bind_owner.clone();
        let bind_system = bind_system.clone();
        let bind_middleware = bind_middleware.clone();

        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            if let Some((ip, port)) = (*active_port).clone() {
                let assets = assets.clone();
                let active_port = active_port.clone();
                let owner = (*bind_owner).clone();
                let system_name = (*bind_system).clone();
                let middleware = (*bind_middleware).clone();
                let ip = ip.clone();

                wasm_bindgen_futures::spawn_local(async move {
                    let req = PortBindingRequest { owner, system_name, middleware };
                    let url = format!("http://localhost:3003/api/assets/{}/ports/{}/bind", ip, port);
                    let _ = Request::post(&url).json(&req).unwrap().send().await;
                    
                    if let Ok(resp) = Request::get("http://localhost:3003/api/assets").send().await {
                        if let Ok(data) = resp.json::<Vec<Asset>>().await {
                            assets.set(data);
                        }
                    }
                    active_port.set(None);
                });
            }
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
                                        <div class="tags">
                                            {
                                                for asset.ports.iter().map(|port| {
                                                    let color = if port.is_bound { "is-success" } else { "is-danger" };
                                                    let active_port = active_port.clone();
                                                    let bind_owner = bind_owner.clone();
                                                    let bind_system = bind_system.clone();
                                                    let bind_middleware = bind_middleware.clone();
                                                    let asset_ip = asset.ip.clone();
                                                    let port_num = port.port;
                                                    let p_owner = port.owner.clone().unwrap_or_default();
                                                    let p_sys = port.system_name.clone().unwrap_or_default();
                                                    let p_mid = port.middleware.clone().unwrap_or_default();

                                                    let on_port_click = Callback::from(move |_| {
                                                        bind_owner.set(p_owner.clone());
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
                                <label class="label">{lang.t("owner")}</label>
                                <input class="input" type="text" value={(*bind_owner).clone()}
                                    oninput={Callback::from(move |e: InputEvent| {
                                        let input: HtmlInputElement = e.target_unchecked_into();
                                        bind_owner.set(input.value());
                                    })}
                                />
                            </div>
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
                if let Ok(resp) = Request::get("http://localhost:3003/api/zones").send().await {
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
                    Request::post("http://localhost:3003/api/zones")
                        .json(&zone_data)
                        .unwrap()
                        .send()
                        .await
                } else {
                    let url = format!("http://localhost:3003/api/zones/{}", id_val);
                    Request::put(&url)
                        .json(&zone_data)
                        .unwrap()
                        .send()
                        .await
                };
                
                if let Ok(resp) = Request::get("http://localhost:3003/api/zones").send().await {
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
                let url = format!("http://localhost:3003/api/zones/{}", id);
                let resp = Request::delete(&url).send().await;
                
                match resp {
                    Ok(response) => {
                        if response.ok() {
                            if let Ok(resp) = Request::get("http://localhost:3003/api/zones").send().await {
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

fn switch(routes: Route) -> Html {
    match routes {
        Route::Dashboard => html! { <Dashboard /> },
        Route::TaskCenter => html! { <TaskCenter /> },
        Route::AssetCenter => html! { <AssetCenter /> },
        Route::ZoneManagement => html! { <ZoneManagement /> },
        Route::RiskCenter => html! { <RiskCenter /> },
        Route::NotFound => html! { <h1 class="title">{"404 Not Found"}</h1> },
    }
}

#[function_component(App)]
pub fn app() -> Html {
    let language = use_state(|| Language::Zh);

    html! {
        <ContextProvider<UseStateHandle<Language>> context={language}>
            <BrowserRouter>
                <div class="columns is-gapless">
                    <div class="column is-2">
                        <Sidebar />
                    </div>
                    <div class="column">
                        <Switch<Route> render={switch} />
                    </div>
                </div>
            </BrowserRouter>
        </ContextProvider<UseStateHandle<Language>>>
    }
}

#[wasm_bindgen(start)]
pub fn run_app() {
    yew::Renderer::<App>::new().render();
}
