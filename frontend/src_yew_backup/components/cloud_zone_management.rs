// ============== Cloud Zone Management Component (运营商/厂家管理) ==============

use crate::{api_url, Language, get_auth_token};
use shared::{CloudZone, CreateCloudZoneRequest, UpdateCloudZoneRequest};
use wasm_bindgen_futures::spawn_local;
use web_sys::{Event, HtmlInputElement, HtmlTextAreaElement};
use yew::prelude::*;
use gloo_net::http::Request;
use serde_json;

#[function_component]
pub fn CloudZoneManagement() -> Html {
    let zones = use_state(|| Vec::new());
    let loading = use_state(|| true);
    let error_message = use_state(|| None as Option<String>);
    let success_message = use_state(|| None as Option<String>);

    let show_create_modal = use_state(|| false);
    let show_edit_modal = use_state(|| false);
    let editing_zone_id = use_state(|| None as Option<i32>);

    let form_zone_name = use_state(|| String::new());
    let form_zone_code = use_state(|| String::new());
    let form_description = use_state(|| None as Option<String>);

    let token = get_auth_token();

    let fetch_zones = {
        let zones = zones.clone();
        let loading = loading.clone();
        let error_message = error_message.clone();
        let token = token.clone();

        Callback::from(move |_| {
            let zones = zones.clone();
            let loading = loading.clone();
            let error_message = error_message.clone();
            let token = token.clone();

            spawn_local(async move {
                loading.set(true);
                error_message.set(None);

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
                                error_message.set(Some(format!("解析响应失败: {}", e)));
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

                loading.set(false);
            });
        })
    };

    use_effect_with((), {
        let fetch_zones = fetch_zones.clone();
        move |_| {
            fetch_zones.emit(());
            || ()
        }
    });

    let on_open_create_modal = Callback::from({
        let show_create_modal = show_create_modal.clone();
        move |_| show_create_modal.set(true)
    });

    let on_close_create_modal = {
        let show_create_modal = show_create_modal.clone();
        let form_zone_name = form_zone_name.clone();
        let form_zone_code = form_zone_code.clone();
        let form_description = form_description.clone();

        Callback::from(move |_: web_sys::MouseEvent| {
            show_create_modal.set(false);
            form_zone_name.set(String::new());
            form_zone_code.set(String::new());
            form_description.set(None);
        })
    };

    let on_create_zone = {
        let fetch_zones = fetch_zones.clone();
        let show_create_modal = show_create_modal.clone();
        let success_message = success_message.clone();
        let error_message = error_message.clone();
        let token = token.clone();
        let form_zone_name = form_zone_name.clone();
        let form_zone_code = form_zone_code.clone();
        let form_description = form_description.clone();
        let on_close_create_modal = on_close_create_modal.clone();

        Callback::from(move |_: web_sys::MouseEvent| {
            let zone_name = (*form_zone_name).clone();
            let zone_code = (*form_zone_code).clone();
            let description = (*form_description).clone();

            if zone_name.is_empty() || zone_code.is_empty() {
                error_message.set(Some("请填写必填字段".to_string()));
                return;
            }

            let fetch_zones = fetch_zones.clone();
            let show_create_modal = show_create_modal.clone();
            let success_message = success_message.clone();
            let error_message = error_message.clone();
            let token = token.clone();

            spawn_local(async move {
                let request = CreateCloudZoneRequest {
                    zone_name: zone_name.clone(),
                    zone_code: zone_code.clone(),
                    description,
                };

                match Request::post(&api_url("providers"))
                    .header("Authorization", &token)
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_string(&request).unwrap())
                    .unwrap()
                    .send()
                    .await
                {
                    Ok(resp) if resp.ok() => {
                        success_message.set(Some(format!("技术底座配置创建成功")));
                        fetch_zones.emit(());
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
            form_zone_name.set(String::new());
            form_zone_code.set(String::new());
            form_description.set(None);
        })
    };

    let on_delete_zone = {
        let fetch_zones = fetch_zones.clone();
        let success_message = success_message.clone();
        let error_message = error_message.clone();
        let token = token.clone();

        Callback::from(move |id: i32| {
            let fetch_zones = fetch_zones.clone();
            let success_message = success_message.clone();
            let error_message = error_message.clone();
            let token = token.clone();

            spawn_local(async move {
                match Request::delete(&format!("{}/{}", api_url("providers"), id))
                    .header("Authorization", &token)
                    .send()
                    .await
                {
                    Ok(resp) if resp.ok() => {
                        success_message.set(Some(format!("技术底座配置 {} 删除成功", id)));
                        fetch_zones.emit(());
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
        let editing_zone_id = editing_zone_id.clone();
        let form_zone_name = form_zone_name.clone();
        let form_zone_code = form_zone_code.clone();
        let form_description = form_description.clone();

        Callback::from(move |zone: CloudZone| {
            editing_zone_id.set(zone.id);
            form_zone_name.set(zone.zone_name.clone());
            form_zone_code.set(zone.zone_code.clone());
            form_description.set(zone.description);
            show_edit_modal.set(true);
        })
    };

    let on_close_edit_modal = {
        let show_edit_modal = show_edit_modal.clone();
        Callback::from(move |_: web_sys::MouseEvent| {
            show_edit_modal.set(false);
        })
    };

    let on_update_zone = {
        let fetch_zones = fetch_zones.clone();
        let show_edit_modal = show_edit_modal.clone();
        let success_message = success_message.clone();
        let error_message = error_message.clone();
        let token = token.clone();
        let editing_zone_id = editing_zone_id.clone();
        let form_zone_name = form_zone_name.clone();
        let form_zone_code = form_zone_code.clone();
        let form_description = form_description.clone();

        Callback::from(move |_: web_sys::MouseEvent| {
            let id = (*editing_zone_id).unwrap();
            let zone_name = (*form_zone_name).clone();
            let zone_code = (*form_zone_code).clone();
            let description = (*form_description).clone();

            let fetch_zones = fetch_zones.clone();
            let show_edit_modal = show_edit_modal.clone();
            let success_message = success_message.clone();
            let error_message = error_message.clone();
            let token = token.clone();

            spawn_local(async move {
                let request = UpdateCloudZoneRequest {
                    zone_name: if zone_name.is_empty() { None } else { Some(zone_name) },
                    zone_code: if zone_code.is_empty() { None } else { Some(zone_code) },
                    description,
                };

                match Request::put(&format!("{}/{}", api_url("providers"), id))
                    .header("Authorization", &token)
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_string(&request).unwrap())
                    .unwrap()
                    .send()
                    .await
                {
                    Ok(resp) if resp.ok() => {
                        success_message.set(Some(format!("云区 {} 更新成功", id)));
                        fetch_zones.emit(());
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

    html! {
        <div class="container" style="margin-top: 20px;">
            <h1 class="title">{ "🗺️ 云区管理" }</h1>

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
                        <span>{ "新建云区" }</span>
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
                                <th>{ "云区名称" }</th>
                                <th>{ "云区代码" }</th>
                                <th>{ "描述" }</th>
                                <th>{ "创建时间" }</th>
                                <th>{ "操作" }</th>
                            </tr>
                        </thead>
                        <tbody>
                            { for zones.iter().map(|zone| {
                                let zone_clone = zone.clone();
                                let on_edit = on_open_edit_modal.clone();
                                let id = zone.id.unwrap();
                                let on_delete = on_delete_zone.clone();
                                html! {
                                    <tr key={zone.id.unwrap()}>
                                        <td>{ zone.id.unwrap() }</td>
                                        <td><strong>{ &zone.zone_name }</strong></td>
                                        <td><code>{ &zone.zone_code }</code></td>
                                        <td>{ zone.description.clone().unwrap_or_else(|| "-".to_string()) }</td>
                                        <td>{ zone.created_at.format("%Y-%m-%d %H:%M:%S").to_string() }</td>
                                        <td>
                                            <button class="button is-small is-info" onclick={
                                                let on_edit = on_edit.clone();
                                                Callback::from(move |_| on_edit.emit(zone_clone.clone()))
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
                            <p class="modal-card-title">{ "新建云区" }</p>
                            <button class="delete" onclick={on_close_create_modal.clone()}></button>
                        </header>
                        <section class="modal-card-body">
                            <div class="field">
                                <label class="label">{ "云区名称" }</label>
                                <div class="control">
                                    <input type="text" class="input"
                                        placeholder="例如：华北区、华南区"
                                        value={(*form_zone_name).clone()}
                                        oninput={
                                            let form_zone_name = form_zone_name.clone();
                                            Callback::from(move |e: InputEvent| {
                                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                form_zone_name.set(input.value());
                                            })
                                        }
                                    />
                                </div>
                            </div>

                            <div class="field">
                                <label class="label">{ "云区代码" }</label>
                                <div class="control">
                                    <input type="text" class="input"
                                        placeholder="例如：north、south"
                                        value={(*form_zone_code).clone()}
                                        oninput={
                                            let form_zone_code = form_zone_code.clone();
                                            Callback::from(move |e: InputEvent| {
                                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                form_zone_code.set(input.value());
                                            })
                                        }
                                    />
                                </div>
                            </div>

                            <div class="field">
                                <label class="label">{ "描述" }</label>
                                <div class="control">
                                    <input type="text" class="input"
                                        placeholder="选填"
                                        value={(*form_description).clone().unwrap_or_default()}
                                        oninput={
                                            let form_description = form_description.clone();
                                            Callback::from(move |e: InputEvent| {
                                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                form_description.set(Some(input.value()));
                                            })
                                        }
                                    />
                                </div>
                            </div>
                        </section>
                        <footer class="modal-card-foot" style="justify-content: flex-end;">
                            <button class="button" onclick={on_close_create_modal.clone()}>{ "取消" }</button>
                            <button class="button is-primary" onclick={on_create_zone}>{ "创建" }</button>
                        </footer>
                    </div>
                </div>
            }

            // Edit Modal
            if *show_edit_modal {
                <div class="modal is-active">
                    <div class="modal-background" onclick={on_close_edit_modal.clone()}></div>
                    <div class="modal-card">
                        <header class="modal-card-head">
                            <p class="modal-card-title">{ "编辑云区" }</p>
                            <button class="delete" onclick={on_close_edit_modal.clone()}></button>
                        </header>
                        <section class="modal-card-body">
                            <div class="field">
                                <label class="label">{ "云区名称" }</label>
                                <div class="control">
                                    <input type="text" class="input"
                                        value={(*form_zone_name).clone()}
                                        oninput={
                                            let form_zone_name = form_zone_name.clone();
                                            Callback::from(move |e: InputEvent| {
                                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                form_zone_name.set(input.value());
                                            })
                                        }
                                    />
                                </div>
                            </div>

                            <div class="field">
                                <label class="label">{ "云区代码" }</label>
                                <div class="control">
                                    <input type="text" class="input"
                                        value={(*form_zone_code).clone()}
                                        oninput={
                                            let form_zone_code = form_zone_code.clone();
                                            Callback::from(move |e: InputEvent| {
                                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                form_zone_code.set(input.value());
                                            })
                                        }
                                    />
                                </div>
                            </div>

                            <div class="field">
                                <label class="label">{ "描述" }</label>
                                <div class="control">
                                    <input type="text" class="input"
                                        value={(*form_description).clone().unwrap_or_default()}
                                        oninput={
                                            let form_description = form_description.clone();
                                            Callback::from(move |e: InputEvent| {
                                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                form_description.set(Some(input.value()));
                                            })
                                        }
                                    />
                                </div>
                            </div>
                        </section>
                        <footer class="modal-card-foot" style="justify-content: flex-end;">
                            <button class="button" onclick={on_close_edit_modal.clone()}>{ "取消" }</button>
                            <button class="button is-primary" onclick={on_update_zone}>{ "更新" }</button>
                        </footer>
                    </div>
                </div>
            }

            <div class="box mt-5">
                <p class="heading">{ "🏢 运营商/厂家管理说明" }</p>
                <ul>
                    <li>{ "运营商/厂家用于管理不同的云服务提供商（如阿里云、腾讯云、华为云等）或设备厂家" }</li>
                    <li>{ "每个运营商/厂家下可以有多个云服务（如对外服务、内部核心服务等）" }</li>
                    <li>{ "运营商/厂家代码用于系统内部标识，建议使用英文简写" }</li>
                </ul>
            </div>
        </div>
    }
}

// ============== Cloud Platform Management Component ==============

