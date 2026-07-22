use dioxus::prelude::*;
use crate::components::*;

#[component] pub fn BusinessAppPage() -> Element {
    let mut tab = use_signal(|| "apps");
    let t = tab();
    rsx! {
        div {
            h1 { class: "text-xl font-bold text-gray-800 mb-4", "业务应用管理" }
            div { class: "flex gap-2 mb-4",
                button { class: if t == "apps" { "ant-btn ant-btn-primary" } else { "ant-btn" }, onclick: move |_| tab.set("apps"), "应用列表" }
                button { class: if t == "endpoints" { "ant-btn ant-btn-primary" } else { "ant-btn" }, onclick: move |_| tab.set("endpoints"), "端点管理" }
            }
            {match t {
                "endpoints" => crate::pages::infra_crud("端点管理", "application-endpoint", vec![Column{key:"endpointPath".to_string(),title:"路径".to_string(),width:None},Column{key:"endpointMethod".to_string(),title:"方法".to_string(),width:Some("80px".to_string())},Column{key:"applicationId".to_string(),title:"应用ID".to_string(),width:Some("80px".to_string())}]),
                _ => crate::pages::infra_crud("应用列表", "business-application", vec![Column{key:"applicationName".to_string(),title:"名称".to_string(),width:None},Column{key:"applicationCode".to_string(),title:"编码".to_string(),width:None},Column{key:"applicationStatus".to_string(),title:"状态".to_string(),width:Some("80px".to_string())}]),
            }}
        }
    }
}
