use gloo_net::http::Request;
use serde_json::json;
use crate::state::resource_ticket::{ResourceTicket, ResourceType, TicketStatus};

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

/// 获取资源工单列表
pub async fn fetch_resource_tickets() -> Result<Vec<ResourceTicket>, String> {
    let response = Request::get(&format!("{}/resource-tickets", API_BASE))
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

/// 创建资源工单的请求体
#[derive(Clone, Debug, serde::Serialize)]
pub struct CreateTicketRequest {
    pub resource_type: String,
    pub ecs_name: String,
    pub provider_id: Option<i32>,
    pub cloud_platform_id: Option<i32>,
    pub machine_room_id: Option<i32>,
    pub cloud_region: Option<String>,
    pub cloud_category: Option<String>,
    pub zone_name: Option<String>,
    pub zone_cabinet: Option<String>,
    pub rack_units: Option<i32>,
    pub customer_name: Option<String>,
    pub application_name: Option<String>,
    pub contract_name: Option<String>,
    pub ecs_type: Option<String>,
    pub ecs_os: Option<String>,
    pub cpu_cores: Option<i32>,
    pub memory_gb: Option<i32>,
    pub system_disk: Option<String>,
    pub system_disk_size_gb: Option<i32>,
    pub data_disk: Option<String>,
    pub has_security_product: Option<bool>,
    pub ip_address: Option<String>,
    pub remarks: Option<String>,
    pub fw_source_zone: Option<String>,
    pub fw_source_address: Option<String>,
    pub fw_dest_zone: Option<String>,
    pub fw_dest_address: Option<String>,
    pub fw_protocol: Option<String>,
    pub fw_port: Option<String>,
    pub fw_direction: Option<String>,
    pub fw_valid_until: Option<String>,
    pub fw_firewall_name: Option<String>,
}

impl From<&ResourceTicket> for CreateTicketRequest {
    fn from(ticket: &ResourceTicket) -> Self {
        Self {
            resource_type: ticket.resource_type.to_api_str().to_string(),
            ecs_name: ticket.ecs_name.clone(),
            provider_id: ticket.provider_id,
            cloud_platform_id: ticket.cloud_platform_id,
            machine_room_id: ticket.machine_room_id,
            cloud_region: if ticket.cloud_region.is_empty() { None } else { Some(ticket.cloud_region.clone()) },
            cloud_category: if ticket.cloud_category.is_empty() { None } else { Some(ticket.cloud_category.clone()) },
            zone_name: if ticket.zone_name.is_empty() { None } else { Some(ticket.zone_name.clone()) },
            zone_cabinet: if ticket.zone_cabinet.is_empty() { None } else { Some(ticket.zone_cabinet.clone()) },
            rack_units: if ticket.rack_units == 0 { None } else { Some(ticket.rack_units) },
            customer_name: if ticket.customer_name.is_empty() { None } else { Some(ticket.customer_name.clone()) },
            application_name: if ticket.application_name.is_empty() { None } else { Some(ticket.application_name.clone()) },
            contract_name: if ticket.contract_name.is_empty() { None } else { Some(ticket.contract_name.clone()) },
            ecs_type: if ticket.ecs_type.is_empty() { None } else { Some(ticket.ecs_type.clone()) },
            ecs_os: if ticket.ecs_os.is_empty() { None } else { Some(ticket.ecs_os.clone()) },
            cpu_cores: if ticket.cpu_cores == 0 { None } else { Some(ticket.cpu_cores) },
            memory_gb: if ticket.memory_gb == 0 { None } else { Some(ticket.memory_gb) },
            system_disk: if ticket.system_disk.is_empty() { None } else { Some(ticket.system_disk.clone()) },
            system_disk_size_gb: if ticket.system_disk_size_gb == 0 { None } else { Some(ticket.system_disk_size_gb) },
            data_disk: if ticket.data_disk.is_empty() { None } else { Some(ticket.data_disk.clone()) },
            has_security_product: Some(ticket.has_security_product),
            ip_address: if ticket.ip_address.is_empty() { None } else { Some(ticket.ip_address.clone()) },
            remarks: if ticket.remarks.is_empty() { None } else { Some(ticket.remarks.clone()) },
            fw_source_zone: ticket.fw_source_zone.clone(),
            fw_source_address: ticket.fw_source_address.clone(),
            fw_dest_zone: ticket.fw_dest_zone.clone(),
            fw_dest_address: ticket.fw_dest_address.clone(),
            fw_protocol: ticket.fw_protocol.clone(),
            fw_port: ticket.fw_port.clone(),
            fw_direction: ticket.fw_direction.clone(),
            fw_valid_until: ticket.fw_valid_until.clone(),
            fw_firewall_name: ticket.fw_firewall_name.clone(),
        }
    }
}

/// 创建资源工单
pub async fn create_resource_ticket(ticket: &ResourceTicket) -> Result<ResourceTicket, String> {
    let request_body = CreateTicketRequest::from(ticket);

    let response = Request::post(&format!("{}/resource-tickets", API_BASE))
        .json(&request_body)
        .map_err(|e| format!("构建请求失败: {}", e))?
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    if !response.ok() {
        return Err(format!("服务器错误: {}", response.status()));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("解析失败: {}", e))?;

    if let Some(data) = json.get("data") {
        let created: BackendResourceTicket = serde_json::from_value(data.clone())
            .map_err(|e| format!("解析响应失败: {}", e))?;
        Ok(created.to_frontend())
    } else {
        Err("响应格式错误".to_string())
    }
}

/// 更新资源工单
pub async fn update_resource_ticket(id: i32, ticket: &ResourceTicket) -> Result<ResourceTicket, String> {
    let request_body = CreateTicketRequest::from(ticket);

    let response = Request::put(&format!("{}/resource-tickets/{}", API_BASE, id))
        .json(&request_body)
        .map_err(|e| format!("构建请求失败: {}", e))?
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    if !response.ok() {
        return Err(format!("服务器错误: {}", response.status()));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("解析失败: {}", e))?;

    if let Some(data) = json.get("data") {
        let updated: BackendResourceTicket = serde_json::from_value(data.clone())
            .map_err(|e| format!("解析响应失败: {}", e))?;
        Ok(updated.to_frontend())
    } else {
        Err("响应格式错误".to_string())
    }
}

/// 删除资源工单
pub async fn delete_resource_ticket(id: i32) -> Result<(), String> {
    let response = Request::delete(&format!("{}/resource-tickets/{}", API_BASE, id))
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    if !response.ok() {
        return Err(format!("服务器错误: {}", response.status()));
    }

    Ok(())
}

/// 审批工单
pub async fn approve_ticket(id: i32, approved: bool, comment: Option<String>) -> Result<ResourceTicket, String> {
    let body = json!({
        "approved": approved,
        "comment": comment
    });

    let response = Request::post(&format!("{}/resource-tickets/{}/approve", API_BASE, id))
        .json(&body)
        .map_err(|e| format!("构建请求失败: {}", e))?
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    if !response.ok() {
        return Err(format!("服务器错误: {}", response.status()));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("解析失败: {}", e))?;

    if let Some(data) = json.get("data") {
        let updated: BackendResourceTicket = serde_json::from_value(data.clone())
            .map_err(|e| format!("解析响应失败: {}", e))?;
        Ok(updated.to_frontend())
    } else {
        Err("响应格式错误".to_string())
    }
}

/// 配置工单
pub async fn provision_ticket(id: i32, ip_address: Option<String>, details: Option<String>) -> Result<ResourceTicket, String> {
    let body = json!({
        "details": details,
        "ip_address": ip_address
    });

    let response = Request::post(&format!("{}/resource-tickets/{}/provision", API_BASE, id))
        .json(&body)
        .map_err(|e| format!("构建请求失败: {}", e))?
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    if !response.ok() {
        return Err(format!("服务器错误: {}", response.status()));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("解析失败: {}", e))?;

    if let Some(data) = json.get("data") {
        let updated: BackendResourceTicket = serde_json::from_value(data.clone())
            .map_err(|e| format!("解析响应失败: {}", e))?;
        Ok(updated.to_frontend())
    } else {
        Err("响应格式错误".to_string())
    }
}

/// 交付工单
pub async fn deliver_ticket(id: i32, comment: Option<String>) -> Result<ResourceTicket, String> {
    let body = json!({
        "comment": comment
    });

    let response = Request::post(&format!("{}/resource-tickets/{}/deliver", API_BASE, id))
        .json(&body)
        .map_err(|e| format!("构建请求失败: {}", e))?
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    if !response.ok() {
        return Err(format!("服务器错误: {}", response.status()));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("解析失败: {}", e))?;

    if let Some(data) = json.get("data") {
        let updated: BackendResourceTicket = serde_json::from_value(data.clone())
            .map_err(|e| format!("解析响应失败: {}", e))?;
        Ok(updated.to_frontend())
    } else {
        Err("响应格式错误".to_string())
    }
}
