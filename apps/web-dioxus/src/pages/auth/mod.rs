use crate::services::api;
use dioxus::prelude::*;

#[component]
pub fn LoginPage() -> Element {
    let mut username = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut tenants = use_signal(Vec::<api::Tenant>::new);
    let mut tenant_id = use_signal(|| None::<i64>);
    let mut error = use_signal(|| None::<String>);
    let mut loading = use_signal(|| false);

    use_effect(move || {
        spawn(async move {
            let hostname = web_sys::window()
                .and_then(|window| window.location().hostname().ok())
                .unwrap_or_default();
            match api::tenant_list().await {
                Ok(list) => {
                    let website_tenant = api::tenant_by_website(&hostname).await.ok().flatten();
                    let selected = website_tenant
                        .filter(|tenant| list.iter().any(|item| item.id == tenant.id))
                        .map(|tenant| tenant.id)
                        .or_else(|| list.first().map(|tenant| tenant.id));
                    tenants.set(list);
                    tenant_id.set(selected);
                }
                Err(message) => error.set(Some(message)),
            }
        });
    });

    let mut on_submit = move |_| {
        let u = username();
        let p = password();
        let selected_tenant = tenant_id();
        if u.is_empty() || p.is_empty() {
            error.set(Some("请输入用户名和密码".to_string()));
            return;
        }
        if selected_tenant.is_none() {
            error.set(Some("请选择公司".to_string()));
            return;
        }
        loading.set(true);
        error.set(None);
        spawn(async move {
            match api::login(&api::LoginRequest {
                username: u,
                password: p,
                tenant_id: selected_tenant,
            })
            .await
            {
                Ok(_) => {
                    let _ = web_sys::window().unwrap().location().set_href("/");
                }
                Err(e) => {
                    error.set(Some(e));
                    loading.set(false);
                }
            }
        });
    };

    rsx! {
        div { class: "min-h-screen flex items-center justify-center bg-gradient-to-br from-primary-600 to-primary-900",
            div { class: "bg-white rounded-2xl shadow-2xl p-10 w-[400px]",
                div { class: "text-center mb-8",
                    h1 { class: "text-3xl font-bold text-primary-600", "RustSet" }
                    p { class: "text-gray-500 mt-2", "资产管理平台" }
                }
                if let Some(ref err) = error() {
                    div { class: "bg-red-50 text-red-600 px-4 py-3 rounded-lg mb-4 text-sm", "{err}" }
                }
                div { class: "mb-4",
                    label { class: "block text-sm font-medium text-gray-700 mb-1", "公司" }
                    select {
                        class: "ant-input",
                        value: tenant_id().map(|id| id.to_string()).unwrap_or_default(),
                        onchange: move |event| tenant_id.set(event.value().parse::<i64>().ok()),
                        disabled: tenants().is_empty(),
                        if tenants().is_empty() {
                            option { value: "", "暂无可用公司" }
                        }
                        for tenant in tenants() {
                            option { value: "{tenant.id}", "{tenant.name}" }
                        }
                    }
                }
                div { class: "mb-4",
                    label { class: "block text-sm font-medium text-gray-700 mb-1", "用户名" }
                    input { class: "ant-input", r#type: "text", placeholder: "请输入用户名", oninput: move |e| username.set(e.value()) }
                }
                div { class: "mb-6",
                    label { class: "block text-sm font-medium text-gray-700 mb-1", "密码" }
                    input { class: "ant-input", r#type: "password", placeholder: "请输入密码", oninput: move |e| password.set(e.value()),
                        onkeydown: move |e| { if e.key() == Key::Enter { on_submit(()); } }
                    }
                }
                button { class: "ant-btn ant-btn-primary w-full justify-center py-2.5 text-base", disabled: loading(), onclick: move |_| on_submit(()),
                    if loading() { "登录中..." } else { "登 录" }
                }
            }
        }
    }
}
