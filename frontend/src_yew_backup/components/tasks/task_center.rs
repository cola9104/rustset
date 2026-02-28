//! Task center page component

use yew::prelude::*;
use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use shared::{Task, TaskStatus};
use crate::{api_url, Language, get_auth_token};

#[function_component]
pub fn TaskCenter() -> Html {
    let lang = use_state(|| Language::Zh);
    let tasks = use_state(|| Vec::new());
    let loading = use_state(|| true);

    let token = get_auth_token();

    use_effect_with((), {
        let tasks = tasks.clone();
        let loading = loading.clone();
        let token = token.clone();

        move |_| {
            spawn_local(async move {
                if let Ok(resp) = Request::get(&api_url("tasks")).header("Authorization", &token).send().await {
                    if let Ok(data) = resp.json::<Vec<Task>>().await {
                        tasks.set(data);
                    }
                }
                loading.set(false);
            });
            || ()
        }
    });

    let status_class = |status: &TaskStatus| -> &'static str {
        match status {
            TaskStatus::Pending => "is-warning",
            TaskStatus::Running => "is-info",
            TaskStatus::Completed => "is-success",
            TaskStatus::Failed => "is-danger",
        }
    };

    let status_to_lowercase = |status: &TaskStatus| -> String {
        format!("{:?}", status).to_lowercase()
    };

    html! {
        <div class="container p-4">
            <h1 class="title">{ lang.t("task_center") }</h1>
            <div class="box">
                if (*tasks).is_empty() && *loading {
                    <p>{ "Loading..." }</p>
                } else if (*tasks).is_empty() {
                    <p>{ "No tasks yet" }</p>
                } else {
                    <table class="table is-fullwidth is-hoverable">
                        <thead>
                            <tr>
                                <th>{ lang.t("name") }</th>
                                <th>{ lang.t("target") }</th>
                                <th>{ lang.t("status") }</th>
                                <th>{ lang.t("assets_found") }</th>
                                <th>{ lang.t("risks_found") }</th>
                            </tr>
                        </thead>
                        <tbody>
                            { for tasks.iter().map(|task| {
                                let status_str = format!("{:?}", task.status);
                                html! {
                                    <tr>
                                        <td>{ &task.name }</td>
                                        <td><code>{ &task.target }</code></td>
                                        <td>
                                            <span class={classes!(status_class(&task.status), format!("status-{}", status_to_lowercase(&task.status)))}>
                                                { status_str }
                                            </span>
                                        </td>
                                        <td>{ task.found_assets }</td>
                                        <td>{ task.found_risks }</td>
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
