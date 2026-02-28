//! Operations Management component
//!
//! 运维管理 - 确认信息后更新交付状态

use yew::prelude::*;
use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use web_sys::{Event, InputEvent, HtmlInputElement, HtmlSelectElement};
use shared::BusinessResource;
use crate::{api_url, Language, get_auth_token, get_user_permissions};

#[derive(Clone, PartialEq)]
enum DeliveryFilter {
    All,
    Pending,      // 待交付
    InProgress,   // 交付中
    Delivered,    // 已交付 (用于查看历史)
}

#[function_component]
pub fn OperationsManagement() -> Html {
    let lang = use_state(|| Language::Zh);
    let resources = use_state(|| Vec::new());
    let loading = use_state(|| true);
    let filter = use_state(|| DeliveryFilter::All);
    let show_detail_modal = use_state(|| None::<i32>);
    let search_keyword = use_state(|| "".to_string());

    let token = get_auth_token();
    let user_permissions = get_user_permissions();

    // 加载待运维处理的资源（已批准但未交付的）
    let load_resources = {
        let resources = resources.clone();
        let loading = loading.clone();
        let token = token.clone();

        Callback::from(move |_| {
            let resources = resources.clone();
            let loading = loading.clone();
            let token = token.clone();

            spawn_local(async move {
                loading.set(true);

                // 获取所有业务资源，然后在前端筛选
                if let Ok(resp) = Request::get(&api_url("business-resources"))
                    .header("Authorization", &token)
                    .send()
                    .await
                {
                    if let Ok(data) = resp.json::<Vec<BusinessResource>>().await {
                        // 筛选：已批准的申请
                        let filtered: Vec<BusinessResource> = data.into_iter()
                            .filter(|r| {
                                r.application_status.as_ref()
                                    .map(|s| s == "已批准")
                                    .unwrap_or(false)
                            })
                            .collect();
                        resources.set(filtered);
                    }
                }

                loading.set(false);
            });
        })
    };

    {
        let load_resources = load_resources.clone();

        use_effect_with((), move |_| {
            load_resources.emit(());
            || ()
        });
    }

    // 更新交付状态
    let update_delivery_status = {
        let resources = resources.clone();
        let load_resources = load_resources.clone();
        let token = token.clone();

        Callback::from(move |(resource_id, new_status): (i32, String)| {
            let resources = resources.clone();
            let load_resources = load_resources.clone();
            let token = token.clone();

            spawn_local(async move {
                // 获取当前用户名
                let username = web_sys::window()
                    .and_then(|w| w.local_storage().ok().flatten())
                    .and_then(|storage| storage.get_item("username").ok().flatten())
                    .unwrap_or_else(|| "unknown".to_string());

                let update_data = serde_json::json!({
                    "delivery_status": new_status.clone(),
                    "delivery_confirmed_by": username,
                });

                let json = serde_json::to_string(&update_data).unwrap();
                let url = format!("{}/{}", api_url("business-resources"), resource_id);

                if let Ok(resp) = Request::patch(&url)
                    .header("Authorization", &token)
                    .header("Content-Type", "application/json")
                    .body(json)
                    .unwrap()
                    .send()
                    .await
                {
                    if resp.ok() {
                        load_resources.emit(());
                    }
                }
            });
        })
    };

    let on_search_input = {
        let search_keyword = search_keyword.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(input) = e.target_dyn_into::<HtmlInputElement>() {
                search_keyword.set(input.value());
            }
        })
    };

    let on_filter_change = {
        let filter = filter.clone();
        Callback::from(move |e: Event| {
            if let Some(select) = e.target_dyn_into::<HtmlSelectElement>() {
                let value = select.value();
                let new_filter = match value.as_str() {
                    "pending" => DeliveryFilter::Pending,
                    "in_progress" => DeliveryFilter::InProgress,
                    "delivered" => DeliveryFilter::Delivered,
                    _ => DeliveryFilter::All,
                };
                filter.set(new_filter);
            }
        })
    };

    // 筛选资源
    let filtered_resources: Vec<BusinessResource> = (*resources).iter()
        .filter(|r| {
            // 关键词筛选
            if !search_keyword.is_empty() {
                let keyword = search_keyword.to_lowercase();
                let matches = r.ecs_name.to_lowercase().contains(&keyword)
                    || r.customer_name.to_lowercase().contains(&keyword)
                    || r.ip_address.to_lowercase().contains(&keyword)
                    || r.instance_id.to_lowercase().contains(&keyword);
                if !matches {
                    return false;
                }
            }

            // 交付状态筛选
            match *filter {
                DeliveryFilter::All => true,
                DeliveryFilter::Pending => {
                    r.delivery_status.as_ref()
                        .map(|s| s == "待交付")
                        .unwrap_or(true)
                }
                DeliveryFilter::InProgress => {
                    r.delivery_status.as_ref()
                        .map(|s| s == "交付中")
                        .unwrap_or(false)
                }
                DeliveryFilter::Delivered => {
                    r.delivery_status.as_ref()
                        .map(|s| s == "已交付")
                        .unwrap_or(false)
                }
            }
        })
        .cloned()
        .collect();

    let status_class = |status: &Option<String>| -> &'static str {
        match status.as_ref().map(|s| s.as_str()) {
            Some("待交付") => "is-warning",
            Some("交付中") => "is-info",
            Some("已交付") => "is-success",
            _ => "is-light",
        }
    };

    let resource_type_label = |rt: &str| -> &'static str {
        match rt {
            "cloud" => "云资源",
            "physical" => "物理机",
            _ => "未知",
        }
    };

    html! {
        <div class="container p-4">
            <div class="level">
                <div class="level-left">
                    <h1 class="title">{ "运维管理" }</h1>
                </div>
                <div class="level-right">
                    <span class="tag is-info is-large">
                        { format!("待处理: {} 项", filtered_resources.iter().filter(|r| r.delivery_status.as_ref() != Some(&"已交付".to_string())).count()) }
                    </span>
                </div>
            </div>

            <div class="box">
                <div class="level">
                    <div class="level-left">
                        <div class="field has-addons">
                            <div class="control has-icons-left">
                                <input
                                    class="input"
                                    type="text"
                                    placeholder="搜索名称、客户、IP..."
                                    value={(*search_keyword).clone()}
                                    oninput={on_search_input}
                                />
                                <span class="icon is-small is-left">
                                    <i class="fas fa-search"></i>
                                </span>
                            </div>
                        </div>
                    </div>
                    <div class="level-right">
                        <div class="field">
                            <div class="control">
                                <div class="select">
                                    <select onchange={on_filter_change}>
                                        <option value="all" selected={*filter == DeliveryFilter::All}>{ "全部" }</option>
                                        <option value="pending" selected={*filter == DeliveryFilter::Pending}>{ "待交付" }</option>
                                        <option value="in_progress" selected={*filter == DeliveryFilter::InProgress}>{ "交付中" }</option>
                                        <option value="delivered" selected={*filter == DeliveryFilter::Delivered}>{ "已交付" }</option>
                                    </select>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                if *loading {
                    <div class="has-text-centered p-4">
                        <span class="button is-loading">{ "加载中" }</span>
                    </div>
                } else if filtered_resources.is_empty() {
                    <div class="has-text-centered has-text-grey p-6">
                        <p>{ "暂无需要处理的资源" }</p>
                    </div>
                } else {
                    <div class="table-container">
                        <table class="table is-fullwidth is-hoverable is-striped">
                            <thead>
                                <tr>
                                    <th>{ "类型" }</th>
                                    <th>{ "资源名称" }</th>
                                    <th>{ "客户名称" }</th>
                                    <th>{ "IP地址" }</th>
                                    <th>{ "配置" }</th>
                                    <th>{ "云平台" }</th>
                                    <th>{ "交付状态" }</th>
                                    <th>{ "操作" }</th>
                                </tr>
                            </thead>
                            <tbody>
                                { for filtered_resources.iter().map(|resource| {
                                    let resource_id = resource.id.unwrap_or(0);
                                    let resource_clone = resource.clone();
                                    let resource_clone2 = resource.clone();
                                    let resource_clone3 = resource.clone();

                                    let on_start_delivery = {
                                        let update_delivery_status = update_delivery_status.clone();
                                        let resource_id = resource_id;
                                        Callback::from(move |_| {
                                            update_delivery_status.emit((resource_id, "交付中".to_string()));
                                        })
                                    };

                                    let on_confirm_delivery = {
                                        let update_delivery_status = update_delivery_status.clone();
                                        let resource_id = resource_id;
                                        Callback::from(move |_| {
                                            update_delivery_status.emit((resource_id, "已交付".to_string()));
                                        })
                                    };

                                    let on_show_detail = {
                                        let show_detail_modal = show_detail_modal.clone();
                                        let resource_id = resource_id;
                                        Callback::from(move |_| {
                                            show_detail_modal.set(Some(resource_id));
                                        })
                                    };

                                    html! {
                                        <tr>
                                            <td>
                                                <span class="tag is-small">
                                                    { resource_type_label(&resource.resource_type) }
                                                </span>
                                            </td>
                                            <td>
                                                <strong>{ &resource.ecs_name }</strong>
                                                <br/>
                                                <small class="has-text-grey">{ &resource.instance_id }</small>
                                            </td>
                                            <td>{ &resource.customer_name }</td>
                                            <td>
                                                <code>{ &resource.ip_address }</code>
                                            </td>
                                            <td>
                                                <small>
                                                    { format!("{}核 {}GB {}GB系统盘",
                                                        resource.cpu_cores,
                                                        resource.memory_gb,
                                                        resource.system_disk_size_gb
                                                    ) }
                                                </small>
                                            </td>
                                            <td>
                                                { &resource.cloud_category }
                                                { if let Some(ref zone) = resource.zone_name {
                                                    html! { <><br/><small class="has-text-grey">{ zone }</small></> }
                                                } else {
                                                    html! {}
                                                }}
                                            </td>
                                            <td>
                                                <span class={format!("tag {}", status_class(&resource.delivery_status))}>
                                                    { resource.delivery_status.as_ref().unwrap_or(&"待交付".to_string()) }
                                                </span>
                                            </td>
                                            <td>
                                                <div class="buttons are-small">
                                                    <button
                                                        class="button is-info is-light"
                                                        onclick={on_show_detail}
                                                    >
                                                        <span class="icon"><i class="fas fa-eye"></i></span>
                                                        <span>{ "详情" }</span>
                                                    </button>
                                                    {
                                                        match resource.delivery_status.as_ref().map(|s| s.as_str()) {
                                                            Some("待交付") | None => html! {
                                                                <button
                                                                    class="button is-warning"
                                                                    onclick={on_start_delivery}
                                                                >
                                                                    <span class="icon"><i class="fas fa-play"></i></span>
                                                                    <span>{ "开始交付" }</span>
                                                                </button>
                                                            },
                                                            Some("交付中") => html! {
                                                                <button
                                                                    class="button is-success"
                                                                    onclick={on_confirm_delivery}
                                                                >
                                                                    <span class="icon"><i class="fas fa-check"></i></span>
                                                                    <span>{ "确认交付" }</span>
                                                                </button>
                                                            },
                                                            Some("已交付") => html! {
                                                                <span class="tag is-success is-light">
                                                                    <span class="icon"><i class="fas fa-check-circle"></i></span>
                                                                    <span>{ "已完成" }</span>
                                                                </span>
                                                            },
                                                            _ => html! {}
                                                        }
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

            // 详情模态框
            if let Some(resource_id) = *show_detail_modal {
                if let Some(resource) = resources.iter().find(|r| r.id == Some(resource_id)) {
                    <div class="modal is-active">
                        <div class="modal-background" onclick={{
                            let show_detail_modal = show_detail_modal.clone();
                            Callback::from(move |_| show_detail_modal.set(None))
                        }}></div>
                        <div class="modal-card">
                            <header class="modal-card-head">
                                <p class="modal-card-title">{ "资源详情" }</p>
                                <button class="delete" onclick={{
                                    let show_detail_modal = show_detail_modal.clone();
                                    Callback::from(move |_| show_detail_modal.set(None))
                                }}></button>
                            </header>
                            <section class="modal-card-body">
                                <div class="content">
                                    <h5>{ "基本信息" }</h5>
                                    <table class="table is-fullwidth is-narrow">
                                        <tbody>
                                            <tr><th>{ "资源类型" }</th><td>{ resource_type_label(&resource.resource_type) }</td></tr>
                                            <tr><th>{ "资源名称" }</th><td>{ &resource.ecs_name }</td></tr>
                                            <tr><th>{ "实例ID" }</th><td>{ &resource.instance_id }</td></tr>
                                            <tr><th>{ "状态" }</th><td>{ &resource.ecs_status }</td></tr>
                                            <tr><th>{ "IP地址" }</th><td><code>{ &resource.ip_address }</code></td></tr>
                                        </tbody>
                                    </table>

                                    <h5>{ "配置信息" }</h5>
                                    <table class="table is-fullwidth is-narrow">
                                        <tbody>
                                            <tr><th>{ "CPU" }</th><td>{ format!("{}核", resource.cpu_cores) }</td></tr>
                                            <tr><th>{ "内存" }</th><td>{ format!("{}GB", resource.memory_gb) }</td></tr>
                                            <tr><th>{ "系统盘" }</th><td>{ format!("{} {}GB", &resource.system_disk, resource.system_disk_size_gb) }</td></tr>
                                            <tr><th>{ "操作系统" }</th><td>{ &resource.ecs_os }</td></tr>
                                            <tr><th>{ "机型" }</th><td>{ &resource.ecs_type }</td></tr>
                                        </tbody>
                                    </table>

                                    <h5>{ "业务信息" }</h5>
                                    <table class="table is-fullwidth is-narrow">
                                        <tbody>
                                            <tr><th>{ "客户名称" }</th><td>{ &resource.customer_name }</td></tr>
                                            <tr><th>{ "应用名称" }</th><td>{ resource.application_name.as_ref().unwrap_or(&"-".to_string()) }</td></tr>
                                            <tr><th>{ "合同名称" }</th><td>{ resource.contract_name.as_ref().unwrap_or(&"-".to_string()) }</td></tr>
                                            <tr><th>{ "云类别" }</th><td>{ &resource.cloud_category }</td></tr>
                                            <tr><th>{ "区域" }</th><td>{ resource.zone_name.as_ref().unwrap_or(&"-".to_string()) }</td></tr>
                                        </tbody>
                                    </table>

                                    <h5>{ "登录信息" }</h5>
                                    <table class="table is-fullwidth is-narrow">
                                        <tbody>
                                            <tr><th>{ "登录方式" }</th><td>{ resource.ecs_login_method.as_ref().unwrap_or(&"-".to_string()) }</td></tr>
                                            <tr><th>{ "登录用户" }</th><td>{ resource.ecs_login_username.as_ref().unwrap_or(&"-".to_string()) }</td></tr>
                                            <tr><th>{ "堡垒机地址" }</th><td>{ resource.bastion_address.as_ref().unwrap_or(&"-".to_string()) }</td></tr>
                                            <tr><th>{ "堡垒机账号" }</th><td>{ resource.bastion_admin_account.as_ref().unwrap_or(&"-".to_string()) }</td></tr>
                                        </tbody>
                                    </table>

                                    if resource.resource_type == "physical" {
                                        if let Some(ref pm_info) = resource.physical_machine_info {
                                            <h5>{ "物理机信息" }</h5>
                                            <table class="table is-fullwidth is-narrow">
                                                <tbody>
                                                    <tr><th>{ "序列号" }</th><td>{ pm_info.serial_number.as_ref().unwrap_or(&"-".to_string()) }</td></tr>
                                                    <tr><th>{ "机架位置" }</th><td>{ pm_info.rack_location.as_ref().unwrap_or(&"-".to_string()) }</td></tr>
                                                    <tr><th>{ "硬件型号" }</th><td>{ pm_info.hardware_model.as_ref().unwrap_or(&"-".to_string()) }</td></tr>
                                                    <tr><th>{ "IPMI地址" }</th><td>{ pm_info.ipmi_address.as_ref().unwrap_or(&"-".to_string()) }</td></tr>
                                                </tbody>
                                            </table>
                                        }
                                    }

                                    <h5>{ "交付状态" }</h5>
                                    <div class="tags has-addons">
                                        <span class="tag">{ "申请状态" }</span>
                                        <span class="tag is-success">
                                            { resource.application_status.as_ref().unwrap_or(&"-".to_string()) }
                                        </span>
                                    </div>
                                    <div class="tags has-addons">
                                        <span class="tag">{ "交付状态" }</span>
                                        <span class={format!("tag {}", status_class(&resource.delivery_status))}>
                                            { resource.delivery_status.as_ref().unwrap_or(&"待交付".to_string()) }
                                        </span>
                                    </div>
                                    if let Some(ref confirmed_at) = resource.delivery_confirmed_at {
                                        <p class="has-text-grey is-size-7">
                                            { format!("确认时间: {}", confirmed_at.format("%Y-%m-%d %H:%M")) }
                                        </p>
                                    }
                                    if let Some(ref confirmed_by) = resource.delivery_confirmed_by {
                                        <p class="has-text-grey is-size-7">
                                            { format!("确认人: {}", confirmed_by) }
                                        </p>
                                    }
                                </div>
                            </section>
                            <footer class="modal-card-foot">
                                {
                                    match resource.delivery_status.as_ref().map(|s| s.as_str()) {
                                        Some("待交付") | None => {
                                            let on_start = {
                                                let update_delivery_status = update_delivery_status.clone();
                                                let show_detail_modal = show_detail_modal.clone();
                                                Callback::from(move |_| {
                                                    update_delivery_status.emit((resource_id, "交付中".to_string()));
                                                    show_detail_modal.set(None);
                                                })
                                            };
                                            html! {
                                                <button class="button is-warning" onclick={on_start}>
                                                    <span class="icon"><i class="fas fa-play"></i></span>
                                                    <span>{ "开始交付" }</span>
                                                </button>
                                            }
                                        },
                                        Some("交付中") => {
                                            let on_confirm = {
                                                let update_delivery_status = update_delivery_status.clone();
                                                let show_detail_modal = show_detail_modal.clone();
                                                Callback::from(move |_| {
                                                    update_delivery_status.emit((resource_id, "已交付".to_string()));
                                                    show_detail_modal.set(None);
                                                })
                                            };
                                            html! {
                                                <button class="button is-success" onclick={on_confirm}>
                                                    <span class="icon"><i class="fas fa-check"></i></span>
                                                    <span>{ "确认交付" }</span>
                                                </button>
                                            }
                                        },
                                        _ => html! {}
                                    }
                                }
                                <button class="button" onclick={{
                                    let show_detail_modal = show_detail_modal.clone();
                                    Callback::from(move |_| show_detail_modal.set(None))
                                }}>{ "关闭" }</button>
                            </footer>
                        </div>
                    </div>
                }
            }
        </div>
    }
}
