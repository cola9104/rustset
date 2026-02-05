// ============== Cloud Platform Management Component (云服务管理) ==============

use crate::{api_url, Language, get_auth_token};
use shared::{CloudPlatform, CloudZone, CreateCloudPlatformRequest, UpdateCloudPlatformRequest};
use wasm_bindgen_futures::spawn_local;
use web_sys::{Event, HtmlInputElement, HtmlTextAreaElement};
use yew::prelude::*;
use gloo_net::http::Request;
use serde_json;

#[function_component]
pub fn CloudPlatformManagement() -> Html {
    let platforms = use_state(|| Vec::new());
    let zones = use_state(|| Vec::new());
    let loading = use_state(|| true);
    let error_message = use_state(|| None as Option<String>);
    let success_message = use_state(|| None as Option<String>);

    let show_create_modal = use_state(|| false);
    let show_edit_modal = use_state(|| false);
    let editing_platform_id = use_state(|| None as Option<i32>);

    let form_zone_id = use_state(|| 0);
    let form_platform_name = use_state(|| String::new());
    let form_platform_code = use_state(|| String::new());
    let form_description = use_state(|| None as Option<String>);

    let token = get_auth_token();

    let fetch_zones = {
        let zones = zones.clone();
        let error_message = error_message.clone();
        let token = token.clone();

        Callback::from(move |_| {
            let zones = zones.clone();
            let error_message = error_message.clone();
            let token = token.clone();

            spawn_local(async move {
                match Request::get(&api_url("providers"))
                    .header("Authorization", &token)
                    .send()
                    .await
                {
                    Ok(resp) if resp.ok() => {
                        match resp.json::<Vec<CloudZone>>().await {
                            Ok(data) => {
                                zones.set(data);
                            }
                            Err(e) => {
                                error_message.set(Some(format!("解析云区列表失败: {}", e)));
                            }
                        }
                    }
                    Ok(resp) => {
                        let status = resp.status();
                        error_message.set(Some(format!("获取运营商/厂家列表失败: HTTP {}", status)));
                    }
                    Err(e) => {
                        error_message.set(Some(format!("网络错误: {}", e)));
                    }
                }
            });
        })
    };

    let fetch_platforms = {
        let platforms = platforms.clone();
        let loading = loading.clone();
        let error_message = error_message.clone();
        let token = token.clone();

        Callback::from(move |_| {
            let platforms = platforms.clone();
            let loading = loading.clone();
            let error_message = error_message.clone();
            let token = token.clone();

            spawn_local(async move {
                loading.set(true);
                error_message.set(None);

                match Request::get(&api_url("cloud-services"))
                    .header("Authorization", &token)
                    .send()
                    .await
                {
                    Ok(resp) if resp.ok() => {
                        match resp.json::<Vec<CloudPlatform>>().await {
                            Ok(data) => {
                                platforms.set(data);
                            }
                            Err(e) => {
                                error_message.set(Some(format!("解析响应失败: {}", e)));
                            }
                        }
                    }
                    Ok(resp) => {
                        let status = resp.status();
                        error_message.set(Some(format!("获取云平台列表失败: HTTP {}", status)));
                    }
                    Err(e) => {
                        error_message.set(Some(format!("网络错误: {}", e)));
                    }
                }

                loading.set(false);
            });
        })
    };

    use_effect_with((), {
        let fetch_zones = fetch_zones.clone();
        let fetch_platforms = fetch_platforms.clone();
        move |_| {
            fetch_zones.emit(());
            fetch_platforms.emit(());
            || ()
        }
    });

    let get_zone_name = {
        let zones = zones.clone();
        move |zone_id: i32| -> String {
            (*zones).iter()
                .find(|z| z.id == Some(zone_id))
                .map(|z| z.zone_name.clone())
                .unwrap_or_else(|| format!("Zone {}", zone_id))
        }
    };

    let on_open_create_modal = Callback::from({
        let show_create_modal = show_create_modal.clone();
        let zones = zones.clone();
        let form_zone_id = form_zone_id.clone();
        move |_| {
            show_create_modal.set(true);
            if !zones.is_empty() && zones[0].id.is_some() {
                form_zone_id.set(zones[0].id.unwrap());
            }
        }
    });

    let on_close_create_modal = {
        let show_create_modal = show_create_modal.clone();
        let form_zone_id = form_zone_id.clone();
        let form_platform_name = form_platform_name.clone();
        let form_platform_code = form_platform_code.clone();
        let form_description = form_description.clone();

        Callback::from(move |_: web_sys::MouseEvent| {
            show_create_modal.set(false);
            form_zone_id.set(0);
            form_platform_name.set(String::new());
            form_platform_code.set(String::new());
            form_description.set(None);
        })
    };

    let on_create_platform = {
        let fetch_platforms = fetch_platforms.clone();
        let show_create_modal = show_create_modal.clone();
        let success_message = success_message.clone();
        let error_message = error_message.clone();
        let token = token.clone();
        let form_zone_id = form_zone_id.clone();
        let form_platform_name = form_platform_name.clone();
        let form_platform_code = form_platform_code.clone();
        let form_description = form_description.clone();
        let on_close_create_modal = on_close_create_modal.clone();

        Callback::from(move |_: web_sys::MouseEvent| {
            let zone_id = (*form_zone_id).clone();
            let platform_name = (*form_platform_name).clone();
            let platform_code = (*form_platform_code).clone();
            let description = (*form_description).clone();

            if zone_id == 0 || platform_name.is_empty() || platform_code.is_empty() {
                error_message.set(Some("请填写必填字段".to_string()));
                return;
            }

            let fetch_platforms = fetch_platforms.clone();
            let show_create_modal = show_create_modal.clone();
            let success_message = success_message.clone();
            let error_message = error_message.clone();
            let token = token.clone();

            spawn_local(async move {
                let request = CreateCloudPlatformRequest {
                    zone_id,
                    platform_name: platform_name.clone(),
                    platform_code: platform_code.clone(),
                    description,
                };

                match Request::post(&api_url("cloud-services"))
                    .header("Authorization", &token)
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_string(&request).unwrap())
                    .unwrap()
                    .send()
                    .await
                {
                    Ok(resp) if resp.ok() => {
                        success_message.set(Some(format!("云平台「{}」创建成功", platform_name)));
                        fetch_platforms.emit(());
                        show_create_modal.set(false);
                    }
                    Ok(resp) => {
                        let status = resp.status();
                        if let Ok(text) = resp.text().await {
                            error_message.set(Some(format!("创建失败 (HTTP {}): {}", status, text)));
                        } else {
                            error_message.set(Some(format!("创建失败: HTTP {}", status)));
                        }
                    }
                    Err(e) => {
                        error_message.set(Some(format!("网络错误: {}", e)));
                    }
                }
            });

            // Reset form after submit
            form_zone_id.set(0);
            form_platform_name.set(String::new());
            form_platform_code.set(String::new());
            form_description.set(None);
        })
    };

    let on_delete_platform = {
        let fetch_platforms = fetch_platforms.clone();
        let success_message = success_message.clone();
        let error_message = error_message.clone();
        let token = token.clone();

        Callback::from(move |id: i32| {
            let fetch_platforms = fetch_platforms.clone();
            let success_message = success_message.clone();
            let error_message = error_message.clone();
            let token = token.clone();

            spawn_local(async move {
                match Request::delete(&format!("{}/{}", api_url("cloud-services"), id))
                    .header("Authorization", &token)
                    .send()
                    .await
                {
                    Ok(resp) if resp.ok() => {
                        success_message.set(Some(format!("云平台 {} 删除成功", id)));
                        fetch_platforms.emit(());
                    }
                    Ok(resp) => {
                        let status = resp.status();
                        if let Ok(text) = resp.text().await {
                            error_message.set(Some(format!("删除失败 (HTTP {}): {}", status, text)));
                        } else {
                            error_message.set(Some(format!("删除失败: HTTP {}", status)));
                        }
                    }
                    Err(e) => {
                        error_message.set(Some(format!("网络错误: {}", e)));
                    }
                }
            });
        })
    };

    let on_open_edit_modal = {
        let show_edit_modal = show_edit_modal.clone();
        let editing_platform_id = editing_platform_id.clone();
        let form_zone_id = form_zone_id.clone();
        let form_platform_name = form_platform_name.clone();
        let form_platform_code = form_platform_code.clone();
        let form_description = form_description.clone();

        Callback::from(move |platform: CloudPlatform| {
            editing_platform_id.set(platform.id);
            form_zone_id.set(platform.zone_id);
            form_platform_name.set(platform.platform_name.clone());
            form_platform_code.set(platform.platform_code.clone());
            form_description.set(platform.description);
            show_edit_modal.set(true);
        })
    };

    let on_close_edit_modal = {
        let show_edit_modal = show_edit_modal.clone();
        Callback::from(move |_: web_sys::MouseEvent| {
            show_edit_modal.set(false);
        })
    };

    let on_update_platform = {
        let fetch_platforms = fetch_platforms.clone();
        let show_edit_modal = show_edit_modal.clone();
        let success_message = success_message.clone();
        let error_message = error_message.clone();
        let token = token.clone();
        let editing_platform_id = editing_platform_id.clone();
        let form_zone_id = form_zone_id.clone();
        let form_platform_name = form_platform_name.clone();
        let form_platform_code = form_platform_code.clone();
        let form_description = form_description.clone();

        Callback::from(move |_: web_sys::MouseEvent| {
            let id = (*editing_platform_id).unwrap();
            let zone_id = (*form_zone_id).clone();
            let platform_name = (*form_platform_name).clone();
            let platform_code = (*form_platform_code).clone();
            let description = (*form_description).clone();

            let fetch_platforms = fetch_platforms.clone();
            let show_edit_modal = show_edit_modal.clone();
            let success_message = success_message.clone();
            let error_message = error_message.clone();
            let token = token.clone();

            spawn_local(async move {
                let request = UpdateCloudPlatformRequest {
                    zone_id: Some(zone_id),
                    platform_name: if platform_name.is_empty() { None } else { Some(platform_name) },
                    platform_code: if platform_code.is_empty() { None } else { Some(platform_code) },
                    description,
                };

                match Request::put(&format!("{}/{}", api_url("cloud-services"), id))
                    .header("Authorization", &token)
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_string(&request).unwrap())
                    .unwrap()
                    .send()
                    .await
                {
                    Ok(resp) if resp.ok() => {
                        success_message.set(Some(format!("云平台 {} 更新成功", id)));
                        fetch_platforms.emit(());
                        show_edit_modal.set(false);
                    }
                    Ok(resp) => {
                        let status = resp.status();
                        if let Ok(text) = resp.text().await {
                            error_message.set(Some(format!("更新失败 (HTTP {}): {}", status, text)));
                        } else {
                            error_message.set(Some(format!("更新失败: HTTP {}", status)));
                        }
                    }
                    Err(e) => {
                        error_message.set(Some(format!("网络错误: {}", e)));
                    }
                }
            });
        })
    };

    let zone_options = (*zones).iter().map(|z| {
        (z.id.unwrap(), z.zone_name.clone())
    }).collect::<Vec<_>>();

    html! {
        <div class="container" style="margin-top: 20px;">
            <h1 class="title">{ "☁️ 云服务管理" }</h1>

            if let Some(ref error) = *error_message {
                <div class="notification is-danger is-light">
                    <button class="delete" onclick={
                        let error_message = error_message.clone();
                        Callback::from(move |_| error_message.set(None))
                    }></button>
                    { error }
                </div>
            }

            if let Some(ref success) = *success_message {
                <div class="notification is-success is-light">
                    <button class="delete" onclick={
                        let success_message = success_message.clone();
                        Callback::from(move |_| success_message.set(None))
                    }></button>
                    { success }
                </div>
            }

            <div class="level">
                <div class="level-left">
                    <button class="button is-primary" onclick={on_open_create_modal}>
                        <span class="icon"><span class="fas fa-plus"></span></span>
                        <span>{ "新建云服务" }</span>
                    </button>
                </div>
            </div>

            if *loading {
                <progress class="progress is-small is-info" max="100">{ "Loading..." }</progress>
            } else {
                <div class="table-container">
                    <table class="table is-fullwidth is-hoverable">
                        <thead>
                            <tr>
                                <th>{ "ID" }</th>
                                <th>{ "所属运营商/厂家" }</th>
                                <th>{ "云服务名称" }</th>
                                <th>{ "云服务代码" }</th>
                                <th>{ "描述" }</th>
                                <th>{ "创建时间" }</th>
                                <th>{ "操作" }</th>
                            </tr>
                        </thead>
                        <tbody>
                            { for platforms.iter().map(|platform| {
                                let platform_clone = platform.clone();
                                let on_edit = on_open_edit_modal.clone();
                                let id = platform.id.unwrap();
                                let on_delete = on_delete_platform.clone();
                                let zone_name = get_zone_name(platform.zone_id);
                                html! {
                                    <tr key={platform.id.unwrap()}>
                                        <td>{ platform.id.unwrap() }</td>
                                        <td><span class="tag is-info">{ zone_name }</span></td>
                                        <td><strong>{ &platform.platform_name }</strong></td>
                                        <td><code>{ &platform.platform_code }</code></td>
                                        <td>{ platform.description.clone().unwrap_or_else(|| "-".to_string()) }</td>
                                        <td>{ platform.created_at.format("%Y-%m-%d %H:%M:%S").to_string() }</td>
                                        <td>
                                            <button class="button is-small is-info" onclick={
                                                let on_edit = on_edit.clone();
                                                Callback::from(move |_| on_edit.emit(platform_clone.clone()))
                                            }>{ "编辑" }</button>
                                            <button class="button is-small is-danger" onclick={
                                                let on_delete = on_delete.clone();
                                                Callback::from(move |_| on_delete.emit(id))
                                            }>{ "删除" }</button>
                                        </td>
                                    </tr>
                                }
                            }) }
                        </tbody>
                    </table>
                </div>
            }

            // Create Modal
            if *show_create_modal {
                <div class="modal is-active">
                    <div class="modal-background" onclick={on_close_create_modal.clone()}></div>
                    <div class="modal-card">
                        <header class="modal-card-head">
                            <p class="modal-card-title">{ "新建云服务" }</p>
                            <button class="delete" onclick={on_close_create_modal.clone()}></button>
                        </header>
                        <section class="modal-card-body">
                            <div class="field">
                                <label class="label">{ "所属运营商/厂家" }</label>
                                <div class="control">
                                    <div class="select is-fullwidth">
                                        <select
                                            value={(*form_zone_id).to_string()}
                                            onchange={
                                                let form_zone_id = form_zone_id.clone();
                                                Callback::from(move |e: Event| {
                                                    let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
                                                    form_zone_id.set(select.value().parse().unwrap_or(0));
                                                })
                                            }
                                        >
                                            <option value="0">{ "请选择运营商/厂家" }</option>
                                            { for zone_options.iter().map(|(id, name)| {
                                                html! {
                                                    <option value={id.to_string()}>{ name }</option>
                                                }
                                            }) }
                                        </select>
                                    </div>
                                </div>
                            </div>

                            <div class="field">
                                <label class="label">{ "云服务名称" }</label>
                                <div class="control">
                                    <input type="text" class="input"
                                        placeholder="例如：公众云、政务云"
                                        value={(*form_platform_name).clone()}
                                        oninput={
                                            let form_platform_name = form_platform_name.clone();
                                            Callback::from(move |e: InputEvent| {
                                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                form_platform_name.set(input.value());
                                            })
                                        }
                                    />
                                </div>
                            </div>

                            <div class="field">
                                <label class="label">{ "云服务代码" }</label>
                                <div class="control">
                                    <input type="text" class="input"
                                        placeholder="例如：public、gov"
                                        value={(*form_platform_code).clone()}
                                        oninput={
                                            let form_platform_code = form_platform_code.clone();
                                            Callback::from(move |e: InputEvent| {
                                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                form_platform_code.set(input.value());
                                            })
                                        }
                                    />
                                </div>
                            </div>

                            <div class="field">
                                <label class="label">{ "描述" }</label>
                                <div class="control">
