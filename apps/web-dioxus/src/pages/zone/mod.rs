use dioxus::prelude::*;
use crate::components::*;

#[component] pub fn ZonePage() -> Element {
    {crate::pages::infra_crud("网络区域", "network-zone", vec![
        Column{key:"name".to_string(),title:"名称".to_string(),width:None},
        Column{key:"cidr".to_string(),title:"CIDR".to_string(),width:None},
        Column{key:"priority".to_string(),title:"优先级".to_string(),width:Some("80px".to_string())},
        Column{key:"cloudPlatformName".to_string(),title:"云平台".to_string(),width:None},
        Column{key:"machineRoomName".to_string(),title:"机房".to_string(),width:None},
    ])}
}
