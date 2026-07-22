use dioxus::prelude::*;
use crate::components::*;

#[component] pub fn RiskPage() -> Element {
    {crate::pages::infra_crud("风险中心", "risk", vec![
        Column{key:"assetIp".to_string(),title:"资产IP".to_string(),width:None},
        Column{key:"port".to_string(),title:"端口".to_string(),width:Some("70px".to_string())},
        Column{key:"severity".to_string(),title:"严重程度".to_string(),width:Some("90px".to_string())},
        Column{key:"description".to_string(),title:"描述".to_string(),width:None},
        Column{key:"status".to_string(),title:"状态".to_string(),width:Some("90px".to_string())},
    ])}
}
