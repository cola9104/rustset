//! Advanced Scanning component

use yew::prelude::*;
use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use web_sys::{Event, InputEvent, HtmlInputElement, HtmlTextAreaElement, HtmlSelectElement, Url};
use gloo_timers::callback::Timeout;
use js_sys::Array;
use shared::{TaskStatus, ScanStrategy, ScanEngine, AdvancedScanTask, CreateAdvancedScanRequest};
use crate::{api_url, Language, get_auth_token, get_user_permissions};

#[function_component]
pub fn AdvancedScanning() -> Html {
    let lang = use_state(|| Language::Zh);
    let tasks = use_state(|| Vec::new());
    let loading = use_state(|| true);
    let show_create_modal = use_state(|| false);

    let scan_name = use_state(|| "".to_string());
    let scan_targets = use_state(|| "".to_string());
    let scan_strategy = use_state(|| ScanStrategy::Standard);
    let scan_engine = use_state(|| ScanEngine::Hybrid);

    let token = get_auth_token();
    let user_permissions = get_user_permissions();

    let load_tasks = {
        let tasks = tasks.clone();
        let token = token.clone();

        Callback::from(move |_| {
            let tasks = tasks.clone();
            let token = token.clone();

            spawn_local(async move {
                if let Ok(resp) = Request::get(&api_url("scan/advanced/tasks"))
                    .header("Authorization", &token)
                    .send()
                    .await
                {
                    if let Ok(data) = resp.json::<Vec<AdvancedScanTask>>().await {
                        tasks.set(data);
                    }
                }
            });
        })
    };

    {
        let loading = loading.clone();
        let load_tasks = load_tasks.clone();

        use_effect_with((), move |_| {
            load_tasks.emit(());
            loading.set(false);
            || ()
        });
    }

    {
        let tasks = tasks.clone();
        let load_tasks = load_tasks.clone();

        use_effect_with((), move |_| {
            let has_running = tasks.iter().any(|t| matches!(t.status, TaskStatus::Running));

            if has_running {
                let load_tasks = load_tasks.clone();
                Timeout::new(2000, move || {
                    load_tasks.emit(());
                }).forget();
            }

            || ()
        });
    }

    let on_create_scan = {
        let scan_name = scan_name.clone();
        let scan_targets = scan_targets.clone();
        let scan_strategy = scan_strategy.clone();
        let scan_engine = scan_engine.clone();
        let show_create_modal = show_create_modal.clone();
        let token = token.clone();
        let load_tasks = load_tasks.clone();

        Callback::from(move |_| {
            let name = (*scan_name).clone();
            let targets: Vec<String> = (*scan_targets).clone()
                .split('\n')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            let strategy = (*scan_strategy).clone();
            let engine = (*scan_engine).clone();
            let token = token.clone();
            let load_tasks = load_tasks.clone();

            spawn_local(async move {
                let request = CreateAdvancedScanRequest {
                    name: name.clone(),
                    targets: targets.clone(),
                    strategy,
                    engine,
                    concurrency: Some(1000),
                    timeout_ms: Some(5000),
                    service_detection: Some(true),
                    os_detection: Some(false),
                    web_fingerprint: Some(true),
                    cloud_tag_sync: Some(true),
                };

                let json = serde_json::to_string(&request).unwrap();
                let http_req = Request::post(&api_url("scan/advanced"))
                    .header("Authorization", &token)
                    .header("Content-Type", "application/json")
                    .body(json)
                    .unwrap();
                let resp = http_req.send().await;

                match resp {
                    Ok(response) if response.ok() => {
                        load_tasks.emit(());
                    }
                    Ok(response) => {
                        let status = response.status();
                        gloo_console::error!("Create scan failed with status:", status as i32);
                    }
                    Err(e) => {
                        gloo_console::error!("Network Error:", e.to_string());
                    }
                }
            });

            show_create_modal.set(false);
        })
    };

    let on_open_modal = {
        let show_create_modal = show_create_modal.clone();
        Callback::from(move |_| show_create_modal.set(true))
    };

    let on_close_modal = {
        let show_create_modal = show_create_modal.clone();
        Callback::from(move |_| show_create_modal.set(false))
    };

    let on_scan_name_input = {
        let scan_name = scan_name.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(input) = e.target_dyn_into::<HtmlInputElement>() {
                scan_name.set(input.value());
            }
        })
    };

    let on_scan_targets_input = {
        let scan_targets = scan_targets.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(input) = e.target_dyn_into::<HtmlTextAreaElement>() {
                scan_targets.set(input.value());
            }
        })
    };

    let status_class = |status: &TaskStatus| -> &'static str {
        match status {
            TaskStatus::Pending => "is-warning",
            TaskStatus::Running => "is-info",
            TaskStatus::Completed => "is-success",
            TaskStatus::Failed => "is-danger",
        }
    };

    let on_strategy_change = {
        let scan_strategy = scan_strategy.clone();
        Callback::from(move |e: Event| {
            if let Some(select) = e.target_dyn_into::<HtmlSelectElement>() {
                let value = select.value();
                let strategy = match value.as_str() {
                    "Quick" => ScanStrategy::Quick,
                    "Standard" => ScanStrategy::Standard,
                    "Full" => ScanStrategy::Full,
                    "Cloud" => ScanStrategy::Cloud,
                    _ => ScanStrategy::Standard,
                };
                scan_strategy.set(strategy);
            }
        })
    };

    let on_engine_change = {
        let scan_engine = scan_engine.clone();
        Callback::from(move |e: Event| {
            if let Some(select) = e.target_dyn_into::<HtmlSelectElement>() {
                let value = select.value();
                let engine = match value.as_str() {
                    "BasicTcp" => ScanEngine::BasicTcp,
                    "RustScan" => ScanEngine::RustScan,
                    "Nmap" => ScanEngine::Nmap,
                    "Hybrid" => ScanEngine::Hybrid,
                    _ => ScanEngine::Hybrid,
                };
                scan_engine.set(engine);
            }
        })
    };

    let on_export_results = {
        let token = token.clone();

        Callback::from(move |task_id: String| {
            let token = token.clone();
            let task_id_clone = task_id.clone();

            spawn_local(async move {
                if let Ok(resp) = Request::get(&format!(
                    "{}/{}/export",
                    api_url("scan/advanced/tasks"),
                    task_id_clone
                ))
                .header("Authorization", &token)
                .send()
                .await
                {
                    if let Ok(results) = resp.json::<Vec<shared::QuickScanResult>>().await {
                        let mut csv = String::from("IP,Status,Open Ports,Services\n");
                        for result in &results {
                            let status = if result.is_alive { "Alive" } else { "Down" };
                            let ports: Vec<String> = result.open_ports.iter()
                                .map(|p| format!("{}", p.port))
                                .collect();
                            let services: Vec<String> = result.open_ports.iter()
                                .filter_map(|p| p.service.as_ref())
                                .cloned()
                                .collect();

                            csv.push_str(&format!(
                                "{},{},{},{}\n",
                                result.ip,
                                status,
                                ports.join(";"),
                                services.join(";")
                            ));
                        }

                        if let Some(window) = web_sys::window() {
                            let blob_array = Array::new();
                            let bytes = js_sys::Uint8Array::from(csv.as_bytes());
                            blob_array.push(&bytes);

                            let js_blob = web_sys::Blob::new_with_u8_array_sequence_and_options(
                                &blob_array.into(),
                                web_sys::BlobPropertyBag::new().type_("text/csv")
                            ).unwrap();

                            let url = Url::create_object_url_with_blob(&js_blob).unwrap();

                            if let Some(document) = window.document() {
                                let a = document.create_element("a").unwrap();
                                a.set_attribute("href", &url).unwrap();
                                a.set_attribute("download", "scan_results.csv").unwrap();
                                let event = web_sys::Event::new("click").unwrap();
                                a.dispatch_event(&event).unwrap();
                            }
                        }
                    }
                }
            });
        })
    };

    html! {
        <div class="container p-4">
            <div class="level">
                <div class="level-left">
                    <h1 class="title">{ lang.t("advanced_scanning") }</h1>
                </div>
                <div class="level-right">
                    if user_permissions.as_ref().map(|p| p.can_create_scan).unwrap_or(false) {
                        <button class="button is-primary" onclick={on_open_modal}>
                            { lang.t("create_scan") }
                        </button>
                    }
                </div>
            </div>

            if *show_create_modal {
                <div class="modal is-active">
                    <div class="modal-background" onclick={on_close_modal.clone()}></div>
                    <div class="modal-card">
                        <header class="modal-card-head">
                            <p class="modal-card-title">{ lang.t("create_scan") }</p>
                            <button class="delete" aria-label="close" onclick={on_close_modal.clone()}></button>
                        </header>
                        <section class="modal-card-body">
                            <div class="field">
                                <label class="label">{ lang.t("scan_name") }</label>
                                <div class="control">
                                    <input
                                        class="input"
                                        type="text"
                                        value={(*scan_name).clone()}
                                        oninput={on_scan_name_input}
                                    />
                                </div>
                            </div>

                            <div class="field">
                                <label class="label">{ lang.t("targets") }</label>
                                <div class="control">
                                    <textarea
                                        class="textarea"
                                        rows="5"
                                        placeholder="192.168.1.1&#10;192.168.1.100-192.168.1.200&#10;10.0.0.0/24"
                                        value={(*scan_targets).clone()}
                                        oninput={on_scan_targets_input}
                                    ></textarea>
                                    <p class="help">{"支持单个 IP、IP 范围或 CIDR，每行一个"}</p>
                                </div>
                            </div>

                            <div class="field">
                                <label class="label">{ lang.t("strategy") }</label>
                                <div class="control">
                                    <div class="select is-fullwidth">
                                        <select onchange={on_strategy_change}>
                                            <option value="Quick" selected={*scan_strategy == ScanStrategy::Quick}>{"快速扫描 (TOP 100)"}</option>
                                            <option value="Standard" selected={*scan_strategy == ScanStrategy::Standard}>{"标准扫描 (TOP 1000)"}</option>
                                            <option value="Full" selected={*scan_strategy == ScanStrategy::Full}>{"全端口扫描 (1-65535)"}</option>
                                            <option value="Cloud" selected={*scan_strategy == ScanStrategy::Cloud}>{"云平台优化扫描"}</option>
                                        </select>
                                    </div>
                                </div>
                            </div>

                            <div class="field">
                                <label class="label">{ lang.t("engine") }</label>
                                <div class="control">
                                    <div class="select is-fullwidth">
                                        <select onchange={on_engine_change}>
                                            <option value="Hybrid" selected={*scan_engine == ScanEngine::Hybrid}>{"混合模式 (推荐)"}</option>
                                            <option value="RustScan" selected={*scan_engine == ScanEngine::RustScan}>{"RustScan (极速)"}</option>
                                            <option value="Nmap" selected={*scan_engine == ScanEngine::Nmap}>{"Nmap (深度)"}</option>
                                            <option value="BasicTcp" selected={*scan_engine == ScanEngine::BasicTcp}>{"基础 TCP (兼容)"}</option>
                                        </select>
                                    </div>
                                </div>
                            </div>
                        </section>
                        <footer class="modal-card-foot">
                            <button class="button is-success" onclick={on_create_scan}>
                                { lang.t("start_scan") }
                            </button>
                            <button class="button" onclick={on_close_modal.clone()}>{"取消"}</button>
                        </footer>
                    </div>
                </div>
            }

            <div class="box mt-4">
                if (*tasks).is_empty() && *loading {
                    <p>{"Loading..." }</p>
                } else if (*tasks).is_empty() {
                    <div class="has-text-centered">
                        <p class="has-text-grey">{"暂无扫描任务"}</p>
                    </div>
                } else {
                    <table class="table is-fullwidth is-hoverable">
                        <thead>
                            <tr>
                                <th>{ lang.t("scan_name") }</th>
                                <th>{"目标数量"}</th>
                                <th>{"策略"}</th>
                                <th>{"引擎"}</th>
                                <th>{ lang.t("scan_progress") }</th>
                                <th>{"状态"}</th>
                                <th>{"操作"}</th>
                            </tr>
                        </thead>
                        <tbody>
                            { for tasks.iter().map(|task| {
                                let progress_percent = (task.progress * 100.0) as i32;
                                let status_str = format!("{:?}", task.status);
                                let task_id = task.id.clone();
                                html! {
                                    <tr>
                                        <td>{ &task.name }</td>
                                        <td>{ task.total_count }</td>
                                        <td>{ format!("{:?}", task.config.strategy) }</td>
                                        <td>{ format!("{:?}", task.config.engine) }</td>
                                        <td>
                                            <progress
                                                value={format!("{}", task.progress)}
                                                max="1"
                                                class={if task.progress >= 1.0 { "progress is-success" } else { "progress is-primary" }}
                                            >
                                                { format!("{}%", progress_percent) }
                                            </progress>
                                        </td>
                                        <td>
                                            <span class={format!("tag {}", status_class(&task.status))}>
                                                { status_str }
                                            </span>
                                        </td>
                                        <td>
                                            if task.status == TaskStatus::Completed {
                                                if user_permissions.as_ref().map(|p| p.can_export_scan).unwrap_or(false) {
                                                    <button
                                                        class="button is-small is-info is-light"
                                                        onclick={on_export_results.reform(move |_| task_id.clone())}
                                                    >{"导出结果"}</button>
                                                }
                                            }
                                        </td>
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
