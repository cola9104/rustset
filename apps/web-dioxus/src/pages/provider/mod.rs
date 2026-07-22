use dioxus::prelude::*;
use crate::components::*;

#[component] pub fn ProviderPage() -> Element {
    {crate::pages::infra_crud("服务商管理", "service-provider", vec![
        Column{key:"providerName".to_string(),title:"名称".to_string(),width:None},
        Column{key:"shortName".to_string(),title:"简称".to_string(),width:None},
        Column{key:"contactPerson".to_string(),title:"联系人".to_string(),width:None},
        Column{key:"contactPhone".to_string(),title:"电话".to_string(),width:None},
        Column{key:"headquarters".to_string(),title:"总部".to_string(),width:None},
        Column{key:"status".to_string(),title:"状态".to_string(),width:Some("80px".to_string())},
    ])}
}
