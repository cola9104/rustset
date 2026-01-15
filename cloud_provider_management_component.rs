// ============== Cloud Provider Management Component ==============
// 云区对接管理组件
//
// 功能：
// 1. 管理已对接的云平台账户和区域配置
// 2. 测试云平台连接
// 3. 启用/停用配置

use yew::prelude::*;
use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use serde::{Deserialize, Serialize};
use shared::{
    CloudProvider, CloudProviderConfigStatus,
    CreateCloudProviderConfigRequest, UpdateCloudProviderConfigRequest,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CloudProviderConfigDisplay {
    pub id: Option<i32>,
    pub provider: CloudProvider,
    pub region_id: String,
    pub region_name: String,
    pub available_zones: Vec<String>,
    pub account_name: String,
    pub access_key_id: String,
    pub status: CloudProviderConfigStatus,
    pub remarks: Option<String>,
    pub last_test_time: Option<chrono::DateTime<chrono::Utc>>,
    pub last_test_result: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[function_component]
pub fn CloudProviderManagement() -> Html {
    let configs = use_state(|| Vec::new());
    let loading = use_state(|| true);
    let error_message = use_state(|| None as Option<String>);
    let success_message = use_state(|| None as Option<String>);

    // Modal states
    let show_create_modal = use_state(|| false);
    let show_edit_modal = use_state(|| false);
    let editing_config = use_state(|| None as Option<CloudProviderConfigDisplay>);

    // Form states
    let form_provider = use_state(|| CloudProvider::Aliyun);
    let form_region_id = use_state(|| String::new());
    let form_region_name = use_state(|| String::new());
    let form_account_name = use_state(|| String::new());
    let form_access_key_id = use_state(|| String::new());
    let form_access_key_secret = use_state(|| String::new());
    let form_remarks = use_state(|| None as Option<String>);
    let form_status = use_state(|| CloudProviderConfigStatus::Active);

    // Filter states
    let filter_provider = use_state(|| String::new());
    let filter_status = use_state(|| String::new());

    let token = get_auth_token();

    // Fetch configs
    let fetch_configs = {
        let configs = configs.clone();
        let loading = loading.clone();
        let error_message = error_message.clone();
        let token = token.clone();

        Callback::from(move |_| {
            let configs = configs.clone();
            let loading = loading.clone();
            let error_message = error_message.clone();
            let token = token.clone();

            spawn_local(async move {
                loading.set(true);
                error_message.set(None);

                match Request::get(&api_url("cloud-provider-configs"))
                    .header("Authorization", &token)
                    .send()
                    .await
                {
                    Ok(resp) if resp.ok() => {
                        match resp.json::<Vec<serde_json::Value>>().await {
                            Ok(data) => {
                                // Parse the response - the API returns masked data
                                let parsed_configs: Vec<CloudProviderConfigDisplay> = data
                                    .into_iter()
                                    .filter_map(|v| serde_json::from_value(v).ok())
                                    .collect();
                                configs.set(parsed_configs);
                            }
                            Err(e) => {
                                error_message.set(Some(format!("解析响应失败: {}", e)));
                            }
                        }
                    }
                    Ok(resp) => {
                        let status = resp.status();
                        error_message.set(Some(format!("获取配置失败: HTTP {}", status)));
                    }
                    Err(e) => {
                        error_message.set(Some(format!("网络错误: {}", e)));
                    }
                }

                loading.set(false);
            });
        })
    };

    // Initial fetch
    use_effect_with((), {
        let fetch_configs = fetch_configs.clone();
        move |_| {
            fetch_configs.emit(());
            || ()
        }
    });

    // Helper functions
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

    let status_badge = |status: &CloudProviderConfigStatus| -> Html {
        match status {
            CloudProviderConfigStatus::Active => {
                html! { <span class="tag is-success">{ "已启用" }</span> }
            }
            CloudProviderConfigStatus::Inactive => {
                html! { <span class="tag is-light">{ "已停用" }</span> }
            }
            CloudProviderConfigStatus::Testing => {
                html! { <span class="tag is-warning">{ "测试中" }</span> }
            }
            CloudProviderConfigStatus::Error => {
                html! { <span class="tag is-danger">{ "连接错误" }</span> }
            }
        }
    };

    // Filter configs
    let filtered_configs = (*configs).iter().filter(|config| {
        let provider_match = filter_provider.is_empty()
            || config.provider.as_str() == &*filter_provider;
        let status_match = filter_status.is_empty()
            || config.status.as_str() == &*filter_status;
        provider_match && status_match
    }).cloned().collect::<Vec<_>>();

    // Provider filter change handler
    let on_provider_filter_change = {
        let filter_provider = filter_provider.clone();
        Callback::from(move |e: Event| {
            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
            filter_provider.set(select.value());
        })
    };

    // Status filter change handler
    let on_status_filter_change = {
        let filter_status = filter_status.clone();
        Callback::from(move |e: Event| {
            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
            filter_status.set(select.value());
        })
    };

    // Open create modal
    let on_open_create_modal = {
        let show_create_modal = show_create_modal.clone();
        Callback::from(move |_| {
            show_create_modal.set(true);
        })
    };

    // Close create modal
    let on_close_create_modal = {
        let show_create_modal = show_create_modal.clone();
        let form_provider = form_provider.clone();
        let form_region_id = form_region_id.clone();
        let form_region_name = form_region_name.clone();
        let form_account_name = form_account_name.clone();
        let form_access_key_id = form_access_key_id.clone();
        let form_access_key_secret = form_access_key_secret.clone();
        let form_remarks = form_remarks.clone();

        Callback::from(move |_| {
            show_create_modal.set(false);
            // Reset form
            form_provider.set(CloudProvider::Aliyun);
            form_region_id.set(String::new());
            form_region_name.set(String::new());
            form_account_name.set(String::new());
            form_access_key_id.set(String::new());
            form_access_key_secret.set(String::new());
            form_remarks.set(None);
        })
    };

    // Create config
    let on_create_config = {
        let fetch_configs = fetch_configs.clone();
        let show_create_modal = show_create_modal.clone();
        let success_message = success_message.clone();
        let error_message = error_message.clone();
        let token = token.clone();
        let form_provider = form_provider.clone();
        let form_region_id = form_region_id.clone();
        let form_region_name = form_region_name.clone();
        let form_account_name = form_account_name.clone();
        let form_access_key_id = form_access_key_id.clone();
        let form_access_key_secret = form_access_key_secret.clone();
        let form_remarks = form_remarks.clone();
        let on_close_create_modal = on_close_create_modal.clone();

        Callback::from(move |_: Event| {
            let request = CreateCloudProviderConfigRequest {
                provider: (*form_provider).clone(),
                region_id: (*form_region_id).clone(),
                region_name: (*form_region_name).clone(),
                available_zones: vec![],
                account_name: (*form_account_name).clone(),
                access_key_id: (*form_access_key_id).clone(),
                access_key_secret: (*form_access_key_secret).clone(),
                remarks: (*form_remarks).clone(),
            };

            let fetch_configs = fetch_configs.clone();
            let show_create_modal = show_create_modal.clone();
            let success_message = success_message.clone();
            let error_message = error_message.clone();
            let token = token.clone();
            let on_close_create_modal = on_close_create_modal.clone();

            spawn_local(async move {
                match serde_json::to_string(&request) {
                    Ok(body) => {
                        match Request::post(&api_url("cloud-provider-configs"))
                            .header("Authorization", &token)
                            .header("Content-Type", "application/json")
                            .body(body)
                            .unwrap()
                            .send()
                            .await
                        {
                            Ok(resp) if resp.ok() => {
                                success_message.set(Some("云区对接配置创建成功".to_string()));
                                show_create_modal.set(false);
                                fetch_configs.emit(());
                                spawn_local(async move {
                                    Timeout::new(3000, move || {
                                        success_message.set(None);
                                    }).forget();
                                });
                            }
                            Ok(resp) => {
                                let status = resp.status();
                                error_message.set(Some(format!("创建失败: HTTP {}", status)));
                            }
                            Err(e) => {
                                error_message.set(Some(format!("网络错误: {}", e)));
                            }
                        }
                    }
                    Err(e) => {
                        error_message.set(Some(format!("序列化失败: {}", e)));
                    }
                }
            });
            on_close_create_modal.emit(());
        })
    };

    // Test connection
    let on_test_connection = {
        let configs = configs.clone();
        let fetch_configs = fetch_configs.clone();
        let success_message = success_message.clone();
        let error_message = error_message.clone();
        let token = token.clone();

        Callback::from(move |config_id: i32| {
            let configs = configs.clone();
            let fetch_configs = fetch_configs.clone();
            let success_message = success_message.clone();
            let error_message = error_message.clone();
            let token = token.clone();

            spawn_local(async move {
                match Request::post(&format!("{}{}/test", api_url("cloud-provider-configs"), config_id))
                    .header("Authorization", &token)
                    .send()
                    .await
                {
                    Ok(resp) if resp.ok() => {
                        match resp.json::<serde_json::Value>().await {
                            Ok(result) => {
                                if result.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                                    success_message.set(Some(format!(
                                        "连接测试成功: {}",
                                        result.get("message").and_then(|v| v.as_str()).unwrap_or("成功")
                                    )));
                                } else {
                                    error_message.set(Some(format!(
                                        "连接测试失败: {}",
                                        result.get("message").and_then(|v| v.as_str()).unwrap_or("未知错误")
                                    )));
                                }
                                fetch_configs.emit(());
                            }
                            Err(e) => {
                                error_message.set(Some(format!("解析响应失败: {}", e)));
                            }
                        }
                    }
                    Ok(resp) => {
                        let status = resp.status();
                        error_message.set(Some(format!("测试失败: HTTP {}", status)));
                    }
                    Err(e) => {
                        error_message.set(Some(format!("网络错误: {}", e)));
                    }
                }
            });
        })
    };

    // Delete config
    let on_delete_config = {
        let fetch_configs = fetch_configs.clone();
        let success_message = success_message.clone();
        let error_message = error_message.clone();
        let token = token.clone();

        Callback::from(move |config_id: i32| {
            if !web_sys::window().unwrap().confirm("确定要删除这个云区对接配置吗？") {
                return;
            }

            let fetch_configs = fetch_configs.clone();
            let success_message = success_message.clone();
            let error_message = error_message.clone();
            let token = token.clone();

            spawn_local(async move {
                match Request::delete(&format!("{}{}", api_url("cloud-provider-configs"), config_id))
                    .header("Authorization", &token)
                    .send()
                    .await
                {
                    Ok(resp) if resp.ok() => {
                        success_message.set(Some("云区对接配置已删除".to_string()));
                        fetch_configs.emit(());
                        spawn_local(async move {
                            Timeout::new(3000, move || {
                                success_message.set(None);
                            }).forget();
                        });
                    }
                    Ok(resp) => {
                        let status = resp.status();
                        error_message.set(Some(format!("删除失败: HTTP {}", status)));
                    }
                    Err(e) => {
                        error_message.set(Some(format!("网络错误: {}", e)));
                    }
                }
            });
        })
    };

    // Toggle status
    let on_toggle_status = {
        let fetch_configs = fetch_configs.clone();
        let success_message = success_message.clone();
        let error_message = error_message.clone();
        let token = token.clone();

        Callback::from(move |(config_id, new_status): (i32, CloudProviderConfigStatus)| {
            let fetch_configs = fetch_configs.clone();
            let success_message = success_message.clone();
            let error_message = error_message.clone();
            let token = token.clone();

            spawn_local(async move {
                let update_req = UpdateCloudProviderConfigRequest {
                    region_name: None,
                    available_zones: None,
                    account_name: None,
                    access_key_id: None,
                    access_key_secret: None,
                    status: Some(new_status.clone()),
                    remarks: None,
                };

                match serde_json::to_string(&update_req) {
                    Ok(body) => {
                        match Request::put(&format!("{}{}", api_url("cloud-provider-configs"), config_id))
                            .header("Authorization", &token)
                            .header("Content-Type", "application/json")
                            .body(body)
                            .unwrap()
                            .send()
                            .await
                        {
                            Ok(resp) if resp.ok() => {
                                let msg = match new_status {
                                    CloudProviderConfigStatus::Active => "已启用",
                                    CloudProviderConfigStatus::Inactive => "已停用",
                                    _ => "状态已更新",
                                };
                                success_message.set(Some(format!("配置{}", msg)));
                                fetch_configs.emit(());
                                spawn_local(async move {
                                    Timeout::new(3000, move || {
                                        success_message.set(None);
                                    }).forget();
                                });
                            }
                            Ok(resp) => {
                                let status = resp.status();
                                error_message.set(Some(format!("更新失败: HTTP {}", status)));
                            }
                            Err(e) => {
                                error_message.set(Some(format!("网络错误: {}", e)));
                            }
                        }
                    }
                    Err(e) => {
                        error_message.set(Some(format!("序列化失败: {}", e)));
                    }
                }
            });
        })
    };

    html! {
        <div class="cloud-provider-management">
            <div class="level mb-4">
                <div class="level-left">
                    <div class="level-item">
                        <h1 class="title is-4">{ "🌐 云区对接管理" }</h1>
                    </div>
                </div>
                <div class="level-right">
                    <div class="level-item">
                        <button class="button is-primary" onclick={on_open_create_modal}>
                            <span class="icon"><i class="fas fa-plus"></i></span>
                            <span>{ "添加云区配置" }</span>
                        </button>
                    </div>
                </div>
            </div>

            // Notifications
            { if let Some(msg) = (*error_message).clone() {
                html! {
                    <div class="notification is-danger is-light">
                        <button class="delete" onclick={
                            let error_message = error_message.clone();
                            Callback::from(move |_| error_message.set(None))
                        }></button>
                        { msg }
                    </div>
                }
            } else { html! {} } }

            { if let Some(msg) = (*success_message).clone() {
                html! {
                    <div class="notification is-success is-light">
                        <button class="delete" onclick={
                            let success_message = success_message.clone();
                            Callback::from(move |_| success_message.set(None))
                        }></button>
                        { msg }
                    </div>
                }
            } else { html! {} } }

            // Filters
            <div class="box">
                <div class="field is-horizontal">
                    <div class="field-label is-normal">
                        <label class="label">{ "筛选条件" }</label>
                    </div>
                    <div class="field-body">
                        <div class="field">
                            <label class="label">{ "云厂商" }</label>
                            <div class="control">
                                <div class="select is-fullwidth">
                                    <select onchange={on_provider_filter_change.clone()}>
                                        <option value="">{ "全部" }</option>
                                        <option value="aliyun">{ "阿里云" }</option>
                                        <option value="tencent">{ "腾讯云" }</option>
                                        <option value="huawei">{ "华为云" }</option>
                                        <option value="aws">{ "AWS" }</option>
                                    </select>
                                </div>
                            </div>
                        </div>
                        <div class="field">
                            <label class="label">{ "状态" }</label>
                            <div class="control">
                                <div class="select is-fullwidth">
                                    <select onchange={on_status_filter_change.clone()}>
                                        <option value="">{ "全部" }</option>
                                        <option value="active">{ "已启用" }</option>
                                        <option value="inactive">{ "已停用" }</option>
                                        <option value="testing">{ "测试中" }</option>
                                        <option value="error">{ "连接错误" }</option>
                                    </select>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            // Config List
            if *loading {
                html! {
                    <div class="has-text-centered py-6">
                        <span class="button is-loading is-large is-white"></span>
                        <p class="mt-4">{ "加载中..." }</p>
                    </div>
                }
            } else if filtered_configs.is_empty() {
                html! {
                    <div class="box has-text-centered py-6">
                        <p class="has-text-grey">{ "暂无云区对接配置" }</p>
                        <p class="has-text-grey is-size-7 mt-2">
                            { "点击上方「添加云区配置」按钮添加新的云平台对接" }
                        </p>
                    </div>
                }
            } else {
                html! {
                    <div class="columns is-multiline">
                        { for filtered_configs.iter().map(|config| {
                            let config_id = config.id.unwrap_or(0);
                            let on_test_click = on_test_connection.clone();
                            let on_delete_click = on_delete_config.clone();
                            let on_toggle_click = on_toggle_status.clone();
                            let provider = config.provider.clone();
                            let status = config.status.clone();
                            let config_name = format!("{} - {} ({})",
                                provider_name(&provider),
                                config.region_name,
                                config.account_name
                            );

                            let can_enable = status != CloudProviderConfigStatus::Active;
                            let can_disable = status != CloudProviderConfigStatus::Inactive;

                            html! {
                                <div key={format!("{:?}", config_id)} class="column is-6">
                                    <div class="box">
                                        <div class="level">
                                            <div class="level-left">
                                                <div class="level-item">
                                                    <span class="icon is-medium has-text-info">
                                                        <i class="fas fa-cloud fa-lg"></i>
                                                    </span>
                                                    <div class="ml-3">
                                                        <p class="title is-5 mb-1">{ config_name }</p>
                                                        <p class="subtitle is-7 mb-0">
                                                            { format!("Region: {} | Zone: {}",
                                                                config.region_id,
                                                                if config.available_zones.is_empty() {
                                                                    "默认".to_string()
                                                                } else {
                                                                    config.available_zones.join(", ")
                                                                }
                                                            )}
                                                        </p>
                                                    </div>
                                                </div>
                                            </div>
                                            <div class="level-right">
                                                <div class="level-item">
                                                    { status_badge(&status) }
                                                </div>
                                            </div>
                                        </div>

                                        if let Some(result) = &config.last_test_result {
                                            <div class="content is-small mt-3">
                                                <p class={classes![
                                                    "has-text-weight-normal",
                                                    if config.status == CloudProviderConfigStatus::Active {
                                                        "has-text-success"
                                                    } else {
                                                        "has-text-danger"
                                                    }
                                                ]}>
                                                    <span class="icon is-small">
                                                        <i class="fas fa-info-circle"></i>
                                                    </span>
                                                    { result.clone() }
                                                </p>
                                                if let Some(test_time) = config.last_test_time {
                                                    <p class="has-text-grey is-size-7 mt-1">
                                                        { format!("最后测试: {}", test_time.format("%Y-%m-%d %H:%M")) }
                                                    </p>
                                                }
                                            </div>
                                        }

                                        <div class="buttons is-right mt-4">
                                            if can_enable {
                                                <button class="button is-small is-success"
                                                    onclick={on_toggle_click.reform(move |_| (config_id, CloudProviderConfigStatus::Active))}>
                                                    <span class="icon"><i class="fas fa-check"></i></span>
                                                    <span>{ "启用" }</span>
                                                </button>
                                            }
                                            if can_disable {
                                                <button class="button is-small is-light"
                                                    onclick={on_toggle_click.reform(move |_| (config_id, CloudProviderConfigStatus::Inactive))}>
                                                    <span class="icon"><i class="fas fa-pause"></i></span>
                                                    <span>{ "停用" }</span>
                                                </button>
                                            }
                                            <button class="button is-small is-info"
                                                onclick={on_test_click.reform(move |_| config_id)}>
                                                <span class="icon"><i class="fas fa-plug"></i></span>
                                                <span>{ "测试连接" }</span>
                                            </button>
                                            <button class="button is-small is-danger is-outlined"
                                                onclick={on_delete_click.reform(move |_| config_id)}>
                                                <span class="icon"><i class="fas fa-trash"></i></span>
                                                <span>{ "删除" }</span>
                                            </button>
                                        </div>

                                        if let Some(remarks) = &config.remarks {
                                            <div class="content is-small mt-3 pt-3" style="border-top: 1px solid #eee;">
                                                <p class="has-text-grey">{ format!("备注: {}", remarks) }</p>
                                            </div>
                                        }
                                    </div>
                                </div>
                            }
                        }) }
                    </div>
                }
            }

            // Create Modal
            if *show_create_modal {
                <div class="modal is-active">
                    <div class="modal-background" onclick={on_close_create_modal.clone()}></div>
                    <div class="modal-card">
                        <header class="modal-card-head">
                            <p class="modal-card-title">{ "添加云区对接配置" }</p>
                            <button class="delete" onclick={on_close_create_modal.clone()}></button>
                        </header>
                        <section class="modal-card-body">
                            // Form
                            <div class="field">
                                <label class="label">{ "云厂商" }</label>
                                <div class="control">
                                    <div class="select is-fullwidth">
                                        <select onchange={
                                            let form_provider = form_provider.clone();
                                            Callback::from(move |e: Event| {
                                                let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
                                                match select.value().as_str() {
                                                    "aliyun" => form_provider.set(CloudProvider::Aliyun),
                                                    "tencent" => form_provider.set(CloudProvider::Tencent),
                                                    "huawei" => form_provider.set(CloudProvider::Huawei),
                                                    "aws" => form_provider.set(CloudProvider::Aws),
                                                    _ => form_provider.set(CloudProvider::Aliyun),
                                                }
                                            })
                                        }>
                                            <option value="aliyun">{ "阿里云" }</option>
                                            <option value="tencent">{ "腾讯云" }</option>
                                            <option value="huawei">{ "华为云" }</option>
                                            <option value="aws">{ "AWS" }</option>
                                        </select>
                                    </div>
                                </div>
                            </div>

                            <div class="field">
                                <label class="label">{ "区域ID" }</label>
                                <div class="control">
                                    <input type="text" class="input"
                                        placeholder="例如: cn-hangzhou (阿里云) / ap-guangzhou (腾讯云)"
                                        value={(*form_region_id).clone()}
                                        oninput={
                                            let form_region_id = form_region_id.clone();
                                            Callback::from(move |e: InputEvent| {
                                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                form_region_id.set(input.value());
                                            })
                                        }
                                    />
                                </div>
                            </div>

                            <div class="field">
                                <label class="label">{ "区域名称" }</label>
                                <div class="control">
                                    <input type="text" class="input"
                                        placeholder="例如: 华东1(杭州)"
                                        value={(*form_region_name).clone()}
                                        oninput={
                                            let form_region_name = form_region_name.clone();
                                            Callback::from(move |e: InputEvent| {
                                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                form_region_name.set(input.value());
                                            })
                                        }
                                    />
                                </div>
                            </div>

                            <div class="field">
                                <label class="label">{ "账户名称" }</label>
                                <div class="control">
                                    <input type="text" class="input"
                                        placeholder="用于标识不同的云账户"
                                        value={(*form_account_name).clone()}
                                        oninput={
                                            let form_account_name = form_account_name.clone();
                                            Callback::from(move |e: InputEvent| {
                                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                form_account_name.set(input.value());
                                            })
                                        }
                                    />
                                </div>
                            </div>

                            <div class="field">
                                <label class="label">{ "Access Key ID" }</label>
                                <div class="control">
                                    <input type="text" class="input"
                                        placeholder="云平台 Access Key ID"
                                        value={(*form_access_key_id).clone()}
                                        oninput={
                                            let form_access_key_id = form_access_key_id.clone();
                                            Callback::from(move |e: InputEvent| {
                                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                form_access_key_id.set(input.value());
                                            })
                                        }
                                    />
                                </div>
                            </div>

                            <div class="field">
                                <label class="label">{ "Access Key Secret" }</label>
                                <div class="control">
                                    <input type="password" class="input"
                                        placeholder="云平台 Access Key Secret"
                                        value={(*form_access_key_secret).clone()}
                                        oninput={
                                            let form_access_key_secret = form_access_key_secret.clone();
                                            Callback::from(move |e: InputEvent| {
                                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                form_access_key_secret.set(input.value());
                                            })
                                        }
                                    />
                                </div>
                            </div>

                            <div class="field">
                                <label class="label">{ "备注" }</label>
                                <div class="control">
                                    <input type="text" class="input"
                                        placeholder="选填"
                                        value={(*form_remarks).clone().unwrap_or_default()}
                                        oninput={
                                            let form_remarks = form_remarks.clone();
                                            Callback::from(move |e: InputEvent| {
                                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                form_remarks.set(Some(input.value()));
                                            })
                                        }
                                    />
                                </div>
                            </div>

                            <article class="message is-small is-info">
                                <div class="message-body">
                                    <p class="is-size-7">
                                        { "注意：创建后请先进行「测试连接」，确认配置正确后再启用。" }
                                    </p>
                                </div>
                            </article>
                        </section>
                        <footer class="modal-card-foot" style="justify-content: flex-end;">
                            <button class="button" onclick={on_close_create_modal.clone()}>{ "取消" }</button>
                            <button class="button is-primary" onclick={on_create_config}>{ "创建" }</button>
                        </footer>
                    </div>
                </div>
            }

            // Info box
            <div class="box mt-5">
                <p class="heading">{ "🌐 云区对接管理说明" }</p>
                <ul>
                    <li>{ "云区对接配置用于管理已对接的云平台账户和区域信息" }</li>
                    <li>{ "业务申请时只能选择已配置且启用的云区" }</li>
                    <li>{ "创建配置后请先进行「测试连接」，确认凭证正确后再启用" }</li>
                    <li>{ "停用配置不会删除数据，业务申请时将不会显示该配置" }</li>
                </ul>
            </div>
        </div>
    }
}

// Helper functions (need to be imported or defined elsewhere)
fn api_url(path: &str) -> String {
    format!("http://localhost:3003/api/{}", path)
}

fn get_auth_token() -> String {
    web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .and_then(|storage| storage.get("auth_token").ok())
        .flatten()
        .unwrap_or_default()
}

use gloo_timers::callback::Timeout;
