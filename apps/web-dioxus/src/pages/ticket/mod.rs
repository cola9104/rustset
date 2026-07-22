use dioxus::prelude::*;
use crate::components::*;

#[component] pub fn TicketPage() -> Element {
    {crate::pages::infra_crud("资源工单", "resource-ticket", vec![
        Column{key:"ecsName".to_string(),title:"名称".to_string(),width:None},
        Column{key:"resourceType".to_string(),title:"类型".to_string(),width:Some("80px".to_string())},
        Column{key:"ticketStatus".to_string(),title:"工单状态".to_string(),width:None},
        Column{key:"customerName".to_string(),title:"客户".to_string(),width:None},
        Column{key:"applicantName".to_string(),title:"申请人".to_string(),width:None},
        Column{key:"deliveryStatus".to_string(),title:"交付状态".to_string(),width:None},
    ])}
}
