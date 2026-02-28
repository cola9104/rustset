//! Login page component

use yew::prelude::*;
use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use web_sys::{InputEvent, Event};
use shared::{LoginRequest, LoginResponse};
use crate::{api_url, Language, Page, set_auth};

#[derive(Properties, PartialEq)]
pub struct LoginProps {
    pub current_page: UseStateHandle<Page>,
}

#[function_component]
pub fn Login(props: &LoginProps) -> Html {
    let current_page = props.current_page.clone();
    let username = use_state(|| String::new());
    let password = use_state(|| String::new());
    let error_msg = use_state(|| None::<String>);
    let loading = use_state(|| false);
    let lang = use_state(|| Language::Zh);

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
                let http_req = Request::post(&api_url("login"))
                    .header("Content-Type", "application/json")
                    .body(json)
                    .unwrap();
                let resp = http_req.send().await;

                match resp {
                    Ok(response) if response.ok() => {
                        let text = response.text().await.unwrap_or_default();
                        gloo_console::log!("Login response text:", &text);
                        if let Ok(login_resp) = serde_json::from_str::<LoginResponse>(&text) {
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
                                    <div class="content mt-4 has-text-centered has-text-grey is-size-7">
                                        <p>{ lang.t("default_account_hint") }</p>
                                        <p class="is-family-monospace">{ "admin / admin" }</p>
                                        <p class="has-text-warning-dark mt-2">{ lang.t("change_password_hint") }</p>
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
