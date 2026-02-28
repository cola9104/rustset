//! Cloud Service Asset Management component
//! Unified management of physical machines and cloud VMs

use yew::prelude::*;
use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use web_sys::{Event, HtmlSelectElement, HtmlInputElement};
use shared::{CloudServiceAsset, CloudServiceAssetStats};
use crate::{api_url, Language, get_auth_token};

#[function_component]
pub fn CloudServiceAssetManagement() -> Html {
    let lang = use_state(|| Language::Zh);
    let assets = use_state(|| Vec::<CloudServiceAsset>::new());
    let stats = use_state(|| None::<CloudServiceAssetStats>);
    let loading = use_state(|| true);

    // Tab filter: all/virtual/physical
    let active_tab = use_state(|| "all".to_string());

    // Other filters
    let filter_status = use_state(|| None::<String>);
    let filter_provider = use_state(|| None::<String>);
    let search_keyword = use_state(|| String::new());

    let token = get_auth_token();

    // Fetch stats
    {
        let stats = stats.clone();
        let token = token.clone();
        use_effect_with((), move |_: &()| {
            spawn_local(async move {
                if let Ok(resp) = Request::get(&api_url("cloud-service-assets/stats"))
                    .header("Authorization", &token)
                    .send()
                    .await
                {
                    if let Ok(data) = resp.json::<CloudServiceAssetStats>().await {
                        stats.set(Some(data));
                    }
                }
            });
            || ()
        });
    }

    // Fetch assets function
    let fetch_assets_fn = {
        let assets = assets.clone();
        let loading = loading.clone();
        let token = token.clone();
        let active_tab = active_tab.clone();
        let filter_status = filter_status.clone();
        let filter_provider = filter_provider.clone();
        let search_keyword = search_keyword.clone();

        move || {
            let assets = assets.clone();
            let loading = loading.clone();
            let token = token.clone();
            let active_tab = active_tab.clone();
            let filter_status = filter_status.clone();
            let filter_provider = filter_provider.clone();
            let search_keyword = search_keyword.clone();

            spawn_local(async move {
                loading.set(true);
                let mut url = api_url("cloud-service-assets");
                let mut has_params = false;

                let asset_type_filter = match (*active_tab).as_str() {
                    "virtual" => Some("virtual".to_string()),
                    "physical" => Some("physical".to_string()),
                    _ => None,
                };

                if let Some(t) = &asset_type_filter {
                    url.push_str(&format!("?asset_type={}", t));
                    has_params = true;
                }
                if let Some(s) = &*filter_status {
                    url.push_str(&format!("{}status={}", if has_params { "&" } else { "?" }, s));
                    has_params = true;
                }
                if let Some(p) = &*filter_provider {
                    url.push_str(&format!("{}cloud_provider={}", if has_params { "&" } else { "?" }, p));
                    has_params = true;
                }
                if !search_keyword.is_empty() {
                    let sk = (*search_keyword).clone();
                    url.push_str(&format!("{}search_keyword={}", if has_params { "&" } else { "?" }, sk));
                }

                if let Ok(resp) = Request::get(&url).header("Authorization", &token).send().await {
                    if let Ok(data) = resp.json::<Vec<CloudServiceAsset>>().await {
                        assets.set(data);
                    }
                }
                loading.set(false);
            });
        }
    };

    // Initial load
    {
        let fetch_assets_fn = fetch_assets_fn.clone();
        use_effect_with((), move |_: &()| {
            fetch_assets_fn();
            || ()
        });
    }

    fn status_class(status: &str) -> &'static str {
        match status {
            "运行中" => "is-success",
            "stopped" | "已停止" => "is-dark",
            "released" | "已释放" => "is-danger",
            s if s.contains("待") => "is-warning",
            _ => "is-info",
        }
    }

    // Tab click handler
    let on_tab_click = {
        let active_tab = active_tab.clone();
        let fetch_assets_fn = fetch_assets_fn.clone();
        Callback::from(move |tab: String| {
            active_tab.set(tab.clone());
            fetch_assets_fn();
        })
    };

    let fetch_assets_onclick = {
        let fetch_assets_fn = fetch_assets_fn.clone();
        Callback::from(move |_| {
            fetch_assets_fn();
        })
    };

    let on_clear_filters = {
        let filter_status = filter_status.clone();
        let filter_provider = filter_provider.clone();
        let search_keyword = search_keyword.clone();
        let fetch_assets_fn = fetch_assets_fn.clone();

        Callback::from(move |_| {
            filter_status.set(None);
            filter_provider.set(None);
            search_keyword.set(String::new());
            fetch_assets_fn();
        })
    };

    html! {
        <div class="container" style="margin-top: 20px;">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px;">
                <div>
                    <h1 class="title">{ "云服务资产管理" }</h1>
                    <p class="subtitle">{ "统一纳管物理机和云虚拟机资产" }</p>
                </div>
                <div class="buttons">
                    <button class="button is-primary">{ "添加资产" }</button>
                    <button class="button is-info is-light">{ "导出" }</button>
                </div>
            </div>

            // Tab filters
            <div class="tabs is-boxed" style="margin-top: 20px;">
                <ul>
                    <li class={if *active_tab == "all" { "is-active" } else { "" }}>
                        <a onclick={
                            let on_tab_click = on_tab_click.clone();
                            Callback::from(move |_| on_tab_click.emit("all".to_string()))
                        }>{ "全部" }</a>
                    </li>
                    <li class={if *active_tab == "virtual" { "is-active" } else { "" }}>
                        <a onclick={
                            let on_tab_click = on_tab_click.clone();
                            Callback::from(move |_| on_tab_click.emit("virtual".to_string()))
                        }>{ "云服务器" }</a>
                    </li>
                    <li class={if *active_tab == "physical" { "is-active" } else { "" }}>
                        <a onclick={
                            let on_tab_click = on_tab_click.clone();
                            Callback::from(move |_| on_tab_click.emit("physical".to_string()))
                        }>{ "物理机" }</a>
                    </li>
                </ul>
            </div>

            // Stats cards
            if let Some(s) = (*stats).clone() {
                <div class="columns" style="margin-top: 20px;">
                    <div class="column">
                        <div class="box">
                            <p class="heading">{ "总资产数" }</p>
                            <p class="title">{ s.total_count }</p>
                        </div>
                    </div>
                    <div class="column">
                        <div class="box">
                            <p class="heading">{ "物理机" }</p>
                            <p class="title has-text-primary">{ s.physical_count }</p>
                        </div>
                    </div>
                    <div class="column">
                        <div class="box">
                            <p class="heading">{ "云虚拟机" }</p>
                            <p class="title has-text-link">{ s.virtual_count }</p>
                        </div>
                    </div>
                    <div class="column">
                        <div class="box">
                            <p class="heading">{ "运行中" }</p>
                            <p class="title has-text-success">{ s.running_count }</p>
                        </div>
                    </div>
                    <div class="column">
                        <div class="box">
                            <p class="heading">{ "总CPU核数" }</p>
                            <p class="title">{ s.total_cpu_cores }</p>
                        </div>
                    </div>
                    <div class="column">
                        <div class="box">
                            <p class="heading">{ "总内存(GB)" }</p>
                            <p class="title">{ s.total_memory_gb }</p>
                        </div>
                    </div>
                </div>
            }

            // Filters
            <div class="box" style="margin-top: 20px;">
                <div class="columns">
                    <div class="column is-2">
                        <label class="label">{ "状态" }</label>
                        <div class="select is-fullwidth">
                            <select
                                onchange={
                                    let filter_status = filter_status.clone();
                                    let fetch_assets_fn = fetch_assets_fn.clone();
                                    Callback::from(move |e: Event| {
                                        let select: HtmlSelectElement = e.target_unchecked_into();
                                        let value = select.value();
                                        filter_status.set(if value.is_empty() { None } else { Some(value) });
                                        fetch_assets_fn();
                                    })
                                }
                            >
                                <option value="" selected={(*filter_status).is_none()}>{ "全部" }</option>
                                <option value="运行中" selected={(*filter_status).as_ref() == Some(&"运行中".to_string())}>{ "运行中" }</option>
                                <option value="已停止" selected={(*filter_status).as_ref() == Some(&"已停止".to_string())}>{ "已停止" }</option>
                            </select>
                        </div>
                    </div>
                    <div class="column is-2">
                        <label class="label">{ "云厂商" }</label>
                        <div class="select is-fullwidth">
                            <select
                                onchange={
                                    let filter_provider = filter_provider.clone();
                                    let fetch_assets_fn = fetch_assets_fn.clone();
                                    Callback::from(move |e: Event| {
                                        let select: HtmlSelectElement = e.target_unchecked_into();
                                        let value = select.value();
                                        filter_provider.set(if value.is_empty() { None } else { Some(value) });
                                        fetch_assets_fn();
                                    })
                                }
                            >
                                <option value="" selected={(*filter_provider).is_none()}>{ "全部" }</option>
                                <option value="阿里云" selected={(*filter_provider).as_ref() == Some(&"阿里云".to_string())}>{ "阿里云" }</option>
                                <option value="腾讯云" selected={(*filter_provider).as_ref() == Some(&"腾讯云".to_string())}>{ "腾讯云" }</option>
                                <option value="华为云" selected={(*filter_provider).as_ref() == Some(&"华为云".to_string())}>{ "华为云" }</option>
                                <option value="AWS" selected={(*filter_provider).as_ref() == Some(&"AWS".to_string())}>{ "AWS" }</option>
                            </select>
                        </div>
                    </div>
                    <div class="column is-6">
                        <label class="label">{ "搜索" }</label>
                        <input class="input"
                            type="text"
                            placeholder="资产名称/实例ID/IP地址"
                            value={(*search_keyword).clone()}
                            onchange={
                                let search_keyword = search_keyword.clone();
                                Callback::from(move |e: Event| {
                                    let input: HtmlInputElement = e.target_unchecked_into();
                                    search_keyword.set(input.value());
                                })
                            }
                        />
                    </div>
                    <div class="column is-2">
                        <label class="label">{ " " }</label>
                        <div class="buttons">
                            <button class="button is-primary" onclick={fetch_assets_onclick.clone()}>
                                { "搜索" }
                            </button>
                            <button class="button" onclick={on_clear_filters.clone()}>
                                { "清除" }
                            </button>
                        </div>
                    </div>
                </div>
            </div>

            // Asset list
            if *loading {
                <div class="section">
                    <progress class="progress is-small is-primary" max="100">{ "30%" }</progress>
                </div>
            } else {
                <div class="table-container" style="margin-top: 20px; overflow-x: auto;">
                    <table class="table is-fullwidth is-hoverable is-striped" style="min-width: max-content;">
                        <thead>
                            <tr>
                                <th>{ "ID" }</th>
                                <th>{ "来源" }</th>
                                <th>{ "类型" }</th>
                                <th>{ "名称" }</th>
                                <th>{ "地区" }</th>
                                <th>{ "运营商/厂家" }</th>
                                <th>{ "实例类型" }</th>
                                <th>{ "CPU" }</th>
                                <th>{ "内存GB" }</th>
                                <th>{ "内网IP" }</th>
                                <th>{ "状态" }</th>
                                <th>{ "操作" }</th>
                            </tr>
                        </thead>
                        <tbody>
                            { for (*assets).iter().map(|asset| {
                                let asset_type_label = match asset.asset_type.as_str() {
                                    "physical" => "物理机",
                                    "virtual" => "云虚拟机",
                                    _ => &asset.asset_type,
                                };
                                let asset_type_class = match asset.asset_type.as_str() {
                                    "physical" => "is-primary",
                                    "virtual" => "is-link",
                                    _ => "is-light",
                                };
                                let source_type_label = match asset.source_type.as_str() {
                                    "business_resource" => "业务受理",
                                    "cloud_asset" => "混合云",
                                    _ => &asset.source_type,
                                };

                                html! {
                                    <tr>
                                        <td>{ &asset.id }</td>
                                        <td><span class={classes!("tag", "is-light")}>{ source_type_label }</span></td>
                                        <td><span class={classes!("tag", asset_type_class)}>{ asset_type_label }</span></td>
                                        <td><strong>{ &asset.name }</strong></td>
                                        <td>{ asset.region.as_ref().unwrap_or(&String::from("-")) }</td>
                                        <td>{ &asset.cloud_zone }</td>
                                        <td>{ &asset.instance_type }</td>
                                        <td>{ asset.cpu_cores }</td>
                                        <td>{ asset.memory_gb }</td>
                                        <td><code>{ &asset.ip_address }</code></td>
                                        <td><span class={classes!("tag", status_class(&asset.status))}>{ &asset.status }</span></td>
                                        <td><button class="button is-small is-info is-light">{ "查看" }</button></td>
                                    </tr>
                                }
                            }).collect::<Vec<_>>() }
                        </tbody>
                    </table>
                    { if (*assets).is_empty() {
                        html! { <div class="has-text-centered" style="padding: 40px;">{ "暂无数据" }</div> }
                    } else { html! {} }}
                </div>
            }
        </div>
    }
}
