use dioxus::prelude::*;
use crate::components::*;

#[component] pub fn TaskPage() -> Element {
    {crate::pages::infra_crud("任务中心", "task", vec![
        Column{key:"name".to_string(),title:"名称".to_string(),width:None},
        Column{key:"target".to_string(),title:"目标".to_string(),width:None},
        Column{key:"status".to_string(),title:"状态".to_string(),width:Some("80px".to_string())},
        Column{key:"portPolicy".to_string(),title:"端口策略".to_string(),width:Some("100px".to_string())},
        Column{key:"foundAssets".to_string(),title:"发现资产".to_string(),width:Some("80px".to_string())},
        Column{key:"foundRisks".to_string(),title:"发现风险".to_string(),width:Some("80px".to_string())},
    ])}
}
