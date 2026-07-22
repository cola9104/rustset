use dioxus::prelude::*;
use crate::components::*;

#[component] pub fn CloudPlatformPage() -> Element {
    let mut tab = use_signal(|| "zones");
    let t = tab();
    rsx! {
        div {
            h1 { class: "text-xl font-bold text-gray-800 mb-4", "云平台管理" }
            div { class: "flex gap-2 mb-4",
                button { class: if t == "zones" { "ant-btn ant-btn-primary" } else { "ant-btn" }, onclick: move |_| tab.set("zones"), "云区" }
                button { class: if t == "platforms" { "ant-btn ant-btn-primary" } else { "ant-btn" }, onclick: move |_| tab.set("platforms"), "云平台" }
                button { class: if t == "configs" { "ant-btn ant-btn-primary" } else { "ant-btn" }, onclick: move |_| tab.set("configs"), "对接配置" }
            }
            {match t {
                "platforms" => crate::pages::infra_crud("云平台", "cloud-platform", vec![Column{key:"platformName".to_string(),title:"名称".to_string(),width:None},Column{key:"platformCode".to_string(),title:"编码".to_string(),width:None},Column{key:"zoneId".to_string(),title:"云区ID".to_string(),width:Some("80px".to_string())}]),
                "configs" => crate::pages::infra_crud("对接配置", "cloud-provider-config", vec![Column{key:"accountName".to_string(),title:"账号".to_string(),width:None},Column{key:"provider".to_string(),title:"厂商".to_string(),width:None},Column{key:"regionName".to_string(),title:"区域".to_string(),width:None},Column{key:"status".to_string(),title:"状态".to_string(),width:Some("80px".to_string())}]),
                _ => crate::pages::infra_crud("云区", "cloud-zone", vec![Column{key:"zoneName".to_string(),title:"名称".to_string(),width:None},Column{key:"zoneCode".to_string(),title:"编码".to_string(),width:None},Column{key:"description".to_string(),title:"描述".to_string(),width:None}]),
            }}
        }
    }
}
