use gloo_net::http::Request;
use serde_json::json;
use crate::state::resource_ticket::{ResourceTicket, ResourceType, TicketStatus};
use web_sys::RequestCredentials;

const API_BASE: &str = "http://localhost:3003/api";

/// 后端返回的资源工单结构（与shared库匹配）
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct BackendResourceTicket {
    id: Option<i32>,
    resource_type: String,
    ecs_name: String,
    ticket_status: String,
    provider_id: Option<i32>,
    provider_name: Option<String>,
    cloud_platform_id: Option<i32>,
    cloud_platform_name: Option<String>,
    machine_room_id: Option<i32>,
    machine_room_name: Option<String>,
    cloud_region: Option<String>,
    cloud_category: Option<String>,
    zone_name: Option<String>,
    zone_cabinet: Option<String>,
    rack_units: i32,
    customer_name: Option<String>,
    application_name: Option<String>,
    contract_name: Option<String>,
    ecs_type: Option<String>,
    ecs_os: Option<String>,
    cpu_cores: i32,
    memory_gb: i32,
    system_disk: Option<String>,
    system_disk_size_gb: i32,
    data_disk: Option<String>,
    has_security_product: bool,
    security_products: Option<String>,
    ip_address: Option<String>,
    delivery_status: Option<String>,
    remarks: Option<String>,
    created_at: String,
    updated_at: Option<String>,
    created_by: String,
    approver: Option<String>,
    approve_time: Option<String>,
    approve_comment: Option<String>,
    provisioner: Option<String>,
    provision_time: Option<String>,
    provision_details: Option<String>,
    deliverer: Option<String>,
    deliver_time: Option<String>,
    deliver_comment: Option<String>,
    fw_source_zone: Option<String>,
    fw_source_address: Option<String>,
    fw_dest_zone: Option<String>,
    fw_dest_address: Option<String>,
    fw_protocol: Option<String>,
    fw_port: Option<String>,
    fw_direction: Option<String>,
    fw_valid_until: Option<String>,
    fw_firewall_name: Option<String>,
}

impl BackendResourceTicket {
    fn to_frontend(&self) -> ResourceTicket {
        ResourceTicket {
            id: self.id.unwrap_or(0),
            resource_type: ResourceType::from_api_str(&self.resource_type),
            ecs_name: self.ecs_name.clone(),
            ticket_status: TicketStatus::from_api_str(&self.ticket_status),
            provider_id: self.provider_id,
            provider_name: self.provider_name.clone().unwrap_or_default(),
            cloud_platform_id: self.cloud_platform_id,
            cloud_platform_name: self.cloud_platform_name.clone().unwrap_or_default(),
            machine_room_id: self.machine_room_id,
            machine_room_name: self.machine_room_name.clone().unwrap_or_default(),
            cloud_region: self.cloud_region.clone().unwrap_or_default(),
            cloud_category: self.cloud_category.clone().unwrap_or_default(),
            zone_name: self.zone_name.clone().unwrap_or_default(),
            zone_cabinet: self.zone_cabinet.clone().unwrap_or_default(),
            rack_units: self.rack_units,
            customer_name: self.customer_name.clone().unwrap_or_default(),
            application_name: self.application_name.clone().unwrap_or_default(),
            contract_name: self.contract_name.clone().unwrap_or_default(),
            ecs_type: self.ecs_type.clone().unwrap_or_default(),
            ecs_os: self.ecs_os.clone().unwrap_or_default(),
            cpu_cores: self.cpu_cores,
            memory_gb: self.memory_gb,
            system_disk: self.system_disk.clone().unwrap_or_default(),
            system_disk_size_gb: self.system_disk_size_gb,
            data_disk: self.data_disk.clone().unwrap_or_default(),
            has_security_product: self.has_security_product,
            security_products: self.security_products.clone().unwrap_or_default(),
            ip_address: self.ip_address.clone().unwrap_or_default(),
            delivery_status: self.delivery_status.clone().unwrap_or_else(|| "未交付".to_string()),
            remarks: self.remarks.clone().unwrap_or_default(),
            created_at: self.created_at.clone(),
            updated_at: self.updated_at.clone().unwrap_or_default(),
            created_by: self.created_by.clone(),
            approver: self.approver.clone(),
            approve_time: self.approve_time.clone(),
            approve_comment: self.approve_comment.clone(),
            provisioner: self.provisioner.clone(),
            provision_time: self.provision_time.clone(),
            provision_details: self.provision_details.clone(),
            deliverer: self.deliverer.clone(),
            deliver_time: self.deliver_time.clone(),
            deliver_comment: self.deliver_comment.clone(),
            fw_source_zone: self.fw_source_zone.clone(),
            fw_source_address: self.fw_source_address.clone(),
            fw_dest_zone: self.fw_dest_zone.clone(),
            fw_dest_address: self.fw_dest_address.clone(),
            fw_protocol: self.fw_protocol.clone(),
            fw_port: self.fw_port.clone(),
            fw_direction: self.fw_direction.clone(),
            fw_valid_until: self.fw_valid_until.clone(),
            fw_firewall_name: self.fw_firewall_name.clone(),
        }
    }
}

/// 获取资源工单列表 (使用 Session Cookie 认证)
pub async fn fetch_resource_tickets() -> Result<Vec<ResourceTicket>, String> {
    let response = Request::get(&format!("{}/resource-tickets", API_BASE))
        .credentials(RequestCredentials::Include)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    if !response.ok() {
        return Err(format!("服务器错误: {}", response.status()));
    }

    let backend_tickets: Vec<BackendResourceTicket> = response
        .json()
        .await
        .map_err(|e| format!("解析失败: {}", e))?;

    Ok(backend_tickets.iter().map(|t| t.to_frontend()).collect())
}
