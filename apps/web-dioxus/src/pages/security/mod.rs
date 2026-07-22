use dioxus::prelude::*;
use crate::components::*;

#[component] pub fn SecurityPage() -> Element {
    {crate::pages::infra_crud("安全产品", "security-product", vec![
        Column{key:"name".to_string(),title:"名称".to_string(),width:None},
        Column{key:"category".to_string(),title:"类别".to_string(),width:None},
        Column{key:"vendor".to_string(),title:"厂商".to_string(),width:None},
        Column{key:"model".to_string(),title:"型号".to_string(),width:None},
        Column{key:"licenseType".to_string(),title:"许可证".to_string(),width:None},
        Column{key:"status".to_string(),title:"状态".to_string(),width:Some("80px".to_string())},
    ])}
}
