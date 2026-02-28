//! Dashboard page component

use yew::prelude::*;
use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use shared::Asset;
use crate::{api_url, Language, get_auth_token};

#[function_component]
pub fn Dashboard() -> Html {
    let lang = use_state(|| Language::Zh);
    let stats = use_state(|| (0, 0, 0, 0));
    let loading = use_state(|| true);

    let token = get_auth_token();

    use_effect_with((), {
        let stats = stats.clone();
        let loading = loading.clone();
        let token = token.clone();

        move |_| {
            spawn_local(async move {
                loading.set(true);

                let assets_req = Request::get(&api_url("assets"))
                    .header("Authorization", &token)
                    .send()
                    .await;
                let tasks_req = Request::get(&api_url("tasks"))
                    .header("Authorization", &token)
                    .send()
                    .await;

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
                    if let Ok(tasks) = resp.json::<Vec<shared::Task>>().await {
                        task_count = tasks.len();
                    }
                }

                stats.set((asset_count, port_count, unbound_count, task_count));
                loading.set(false);
            });
            || ()
        }
    });

    let (total_assets, total_ports, unbound_ports, total_tasks) = *stats;

    html! {
        <div class="container p-4">
            <h1 class="title">{ lang.t("dashboard") }</h1>
            <div class="columns is-multiline">
                <div class="column is-3">
                    <div class="box has-background-info-light">
                        <div class="heading">{ lang.t("total_assets") }</div>
                        <div class="title">{ total_assets }</div>
                    </div>
                </div>
                <div class="column is-3">
                    <div class="box has-background-primary-light">
                        <div class="heading">{ lang.t("total_ports") }</div>
                        <div class="title">{ total_ports }</div>
                    </div>
                </div>
                <div class="column is-3">
                    <div class="box has-background-danger-light">
                        <div class="heading">{ lang.t("unbound_ports") }</div>
                        <div class="title has-text-danger">{ unbound_ports }</div>
                    </div>
                </div>
                <div class="column is-3">
                    <div class="box has-background-warning-light">
                        <div class="heading">{ lang.t("active_tasks") }</div>
                        <div class="title">{ total_tasks }</div>
                    </div>
                </div>
            </div>
            <div class="box">
                <h2 class="subtitle">{ lang.t("system_status") }</h2>
                <p>{ lang.t("system_running_msg") }</p>
            </div>
        </div>
    }
}
