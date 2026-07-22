use dioxus::prelude::*;
use crate::components::*;

#[component] pub fn RoomPage() -> Element {
    {crate::pages::infra_crud("机房管理", "machine-room", vec![
        Column{key:"roomName".to_string(),title:"名称".to_string(),width:None},
        Column{key:"roomCode".to_string(),title:"编码".to_string(),width:None},
        Column{key:"facilityType".to_string(),title:"设施类型".to_string(),width:None},
        Column{key:"address".to_string(),title:"地址".to_string(),width:None},
        Column{key:"contactPerson".to_string(),title:"联系人".to_string(),width:None},
        Column{key:"status".to_string(),title:"状态".to_string(),width:Some("80px".to_string())},
    ])}
}
