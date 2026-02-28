//! Audit logs page component

use yew::prelude::*;
use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use shared::AuditLog;
use crate::{api_url, Language, get_auth_token};

#[function_component]
pub fn AuditLogs() -> Html {
    let lang = use_state(|| Language::Zh);
    let logs = use_state(|| Vec::new());
    let loading = use_state(|| true);

    let token = get_auth_token();

    use_effect_with((), {
        let logs = logs.clone();
        let loading = loading.clone();
        let token = token.clone();

        move |_| {
            spawn_local(async move {
                if let Ok(resp) = Request::get(&api_url("logs")).header("Authorization", &token).send().await {
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
