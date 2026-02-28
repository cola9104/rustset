//! Risk center page component

use yew::prelude::*;
use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use shared::Risk;
use crate::{api_url, Language, get_auth_token};

#[function_component]
pub fn RiskCenter() -> Html {
    let lang = use_state(|| Language::Zh);
    let risks = use_state(|| Vec::new());
    let loading = use_state(|| true);

    let token = get_auth_token();

    use_effect_with((), {
        let risks = risks.clone();
        let loading = loading.clone();
        let token = token.clone();

        move |_| {
            spawn_local(async move {
                if let Ok(resp) = Request::get(&api_url("risks")).header("Authorization", &token).send().await {
                    if let Ok(data) = resp.json::<Vec<Risk>>().await {
                        risks.set(data);
                    }
                }
                loading.set(false);
            });
            || ()
        }
    });

    let severity_class = |sev: &str| -> &'static str {
        match sev {
            "Critical" => "is-danger",
            "High" => "is-warning",
            "Medium" => "is-info",
            _ => "is-light",
        }
    };

    html! {
        <div class="container p-4">
            <h1 class="title">{ lang.t("risk_monitoring") }</h1>
            if (*risks).is_empty() && *loading {
                <div class="box has-text-centered">
                    <p class="has-text-grey">{ "Loading..." }</p>
                </div>
            } else if (*risks).is_empty() {
                <div class="box has-text-centered">
                    <p class="has-text-grey">{ lang.t("no_risks") }</p>
                </div>
            } else {
                <div class="box">
                    <table class="table is-fullwidth is-hoverable">
                        <thead>
                            <tr>
                                <th>{ lang.t("severity") }</th>
                                <th>{ lang.t("asset") }</th>
                                <th>{ lang.t("port") }</th>
                                <th>{ lang.t("description") }</th>
                                <th>{ lang.t("status") }</th>
                            </tr>
                        </thead>
                        <tbody>
                            { for risks.iter().map(|risk| {
                                let status_str = format!("{:?}", risk.status);
                                html! {
                                    <tr>
                                        <td>
                                            <span class={classes!("tag", severity_class(&risk.severity))}>
                                                { &risk.severity }
                                            </span>
                                        </td>
                                        <td>{ &risk.asset_ip }</td>
                                        <td>{ risk.port }</td>
                                        <td>{ &risk.description }</td>
                                        <td>{ status_str }</td>
                                    </tr>
                                }
                            })}
                        </tbody>
                    </table>
                </div>
            }
        </div>
    }
}
