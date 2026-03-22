use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Json},
};
use serde_json::json;
use std::sync::RwLock;
use chrono::Utc;

use crate::middleware::{AuthUser, ApiError};
use crate::state::AppState;
use crate::utils::log_action_auth;
use shared::{
    ResourceTicket, ResourceType, TicketStatus,
    CreateResourceTicketRequest, UpdateResourceTicketRequest,
    ApproveTicketRequest, ProvisionTicketRequest, DeliverTicketRequest,
    ResourceTicketQuery, Role,
};

// 内存缓存
pub static RESOURCE_TICKETS: RwLock<Vec<ResourceTicket>> = RwLock::new(Vec::new());

/// 获取资源工单列表
pub async fn get_resource_tickets(
    State(_state): State<AppState>,
    _user: AuthUser,
    Query(query): Query<ResourceTicketQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let tickets = RESOURCE_TICKETS.read().unwrap();
    let mut filtered: Vec<_> = tickets.iter()
        .filter(|t| {
            let mut matches = true;

            if let Some(ref keyword) = query.search_keyword {
                let keyword = keyword.to_lowercase();
                let search_match = t.ecs_name.to_lowercase().contains(&keyword)
                    || t.customer_name.as_ref().map(|s| s.to_lowercase().contains(&keyword)).unwrap_or(false)
                    || t.application_name.as_ref().map(|s| s.to_lowercase().contains(&keyword)).unwrap_or(false)
                    || t.ip_address.as_ref().map(|s| s.to_lowercase().contains(&keyword)).unwrap_or(false);
                matches = matches && search_match;
            }

            if let Some(ref rt) = query.resource_type {
                let rt_match = match rt.as_str() {
                    "physical" => t.resource_type == ResourceType::Physical,
                    "cloud" => t.resource_type == ResourceType::Cloud,
                    "network" => t.resource_type == ResourceType::Network,
                    _ => true,
                };
                matches = matches && rt_match;
            }

            if let Some(ref status) = query.ticket_status {
                let status_match = match status.as_str() {
                    "pending_approval" => t.ticket_status == TicketStatus::PendingApproval,
                    "approved" => t.ticket_status == TicketStatus::Approved,
                    "rejected" => t.ticket_status == TicketStatus::Rejected,
                    "pending_provision" => t.ticket_status == TicketStatus::PendingProvision,
                    "provisioning" => t.ticket_status == TicketStatus::Provisioning,
                    "pending_delivery" => t.ticket_status == TicketStatus::PendingDelivery,
                    "delivered" => t.ticket_status == TicketStatus::Delivered,
                    _ => true,
                };
                matches = matches && status_match;
            }

            if let Some(provider_id) = query.provider_id {
                matches = matches && t.provider_id == Some(provider_id);
            }

            matches
        })
        .cloned()
        .collect();

    // 按创建时间倒序排列
    filtered.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    Ok(Json(filtered).into_response())
}

/// 获取单个资源工单
pub async fn get_resource_ticket(
    State(_state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApiError> {
    let tickets = RESOURCE_TICKETS.read().unwrap();
    if let Some(ticket) = tickets.iter().find(|t| t.id == Some(id)) {
        Ok(Json(ticket.clone()).into_response())
    } else {
        Err(ApiError::not_found("Ticket not found"))
    }
}

/// 创建资源工单
pub async fn create_resource_ticket(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<CreateResourceTicketRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // 检查权限
    if user.role != Role::SysAdmin && user.role != Role::SecAdmin {
        return Err(ApiError::forbidden("Access denied"));
    }

    let now = Utc::now();
    let created_at_str = now.format("%Y-%m-%d %H:%M").to_string();

    // 获取新的ID
    let new_id = {
        let tickets = RESOURCE_TICKETS.read().unwrap();
        tickets.iter().filter_map(|t| t.id).max().unwrap_or(0) + 1
    };

    // 获取关联的名称
    let (provider_name, cloud_platform_name, machine_room_name) = get_related_names(&req);

    let ticket = ResourceTicket {
        id: Some(new_id),
        resource_type: req.resource_type.clone(),
        ecs_name: req.ecs_name.clone(),
        ticket_status: TicketStatus::PendingApproval,

        provider_id: req.provider_id,
        provider_name,
        cloud_platform_id: req.cloud_platform_id,
        cloud_platform_name,
        machine_room_id: req.machine_room_id,
        machine_room_name,

        cloud_region: req.cloud_region,
        cloud_category: req.cloud_category,
        zone_name: req.zone_name,
        zone_cabinet: req.zone_cabinet,
        rack_units: req.rack_units.unwrap_or(0),

        customer_name: req.customer_name,
        application_name: req.application_name,
        contract_name: req.contract_name,
        ecs_type: req.ecs_type,
        ecs_os: req.ecs_os,
        cpu_cores: req.cpu_cores.unwrap_or(0),
        memory_gb: req.memory_gb.unwrap_or(0),
        system_disk: req.system_disk,
        system_disk_size_gb: req.system_disk_size_gb.unwrap_or(0),
        data_disk: req.data_disk,
        has_security_product: req.has_security_product.unwrap_or(false),
        security_products: req.security_products.clone(),
        ip_address: req.ip_address,
        delivery_status: Some("未交付".to_string()),
        remarks: req.remarks,

        created_at: created_at_str,
        updated_at: None,
        created_by: user.username.clone(),

        approver: None,
        approve_time: None,
        approve_comment: None,

        provisioner: None,
        provision_time: None,
        provision_details: None,

        deliverer: None,
        deliver_time: None,
        deliver_comment: None,

        fw_source_zone: req.fw_source_zone,
        fw_source_address: req.fw_source_address,
        fw_dest_zone: req.fw_dest_zone,
        fw_dest_address: req.fw_dest_address,
        fw_protocol: req.fw_protocol,
        fw_port: req.fw_port,
        fw_direction: req.fw_direction,
        fw_valid_until: req.fw_valid_until,
        fw_firewall_name: req.fw_firewall_name,
    };

    // 添加到内存
    {
        let mut tickets = RESOURCE_TICKETS.write().unwrap();
        tickets.push(ticket.clone());
    }

    log_action_auth(
        &state.audit_logs,
        &user,
        "CREATE_RESOURCE_TICKET",
        &format!("{}", new_id),
        &format!("创建资源工单: {} ({})", ticket.ecs_name, ticket.resource_type.as_str()),
    );

    Ok(Json(json!({
        "message": "工单创建成功",
        "data": ticket
    })).into_response())
}

/// 更新资源工单
pub async fn update_resource_ticket(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i32>,
    Json(req): Json<UpdateResourceTicketRequest>,
) -> Result<impl IntoResponse, ApiError> {
    if user.role != Role::SysAdmin && user.role != Role::SecAdmin {
        return Err(ApiError::forbidden("Access denied"));
    }

    let now = Utc::now();
    let updated_at_str = now.format("%Y-%m-%d %H:%M").to_string();

    let mut tickets = RESOURCE_TICKETS.write().unwrap();
    if let Some(ticket) = tickets.iter_mut().find(|t| t.id == Some(id)) {
        // 更新字段
        if let Some(v) = req.ecs_name { ticket.ecs_name = v; }
        if let Some(v) = req.provider_id { ticket.provider_id = Some(v); }
        if let Some(v) = req.cloud_platform_id { ticket.cloud_platform_id = Some(v); }
        if let Some(v) = req.machine_room_id { ticket.machine_room_id = Some(v); }
        if let Some(v) = req.cloud_region { ticket.cloud_region = Some(v); }
        if let Some(v) = req.cloud_category { ticket.cloud_category = Some(v); }
        if let Some(v) = req.zone_name { ticket.zone_name = Some(v); }
        if let Some(v) = req.zone_cabinet { ticket.zone_cabinet = Some(v); }
        if let Some(v) = req.rack_units { ticket.rack_units = v; }
        if let Some(v) = req.customer_name { ticket.customer_name = Some(v); }
        if let Some(v) = req.application_name { ticket.application_name = Some(v); }
        if let Some(v) = req.contract_name { ticket.contract_name = Some(v); }
        if let Some(v) = req.ecs_type { ticket.ecs_type = Some(v); }
        if let Some(v) = req.ecs_os { ticket.ecs_os = Some(v); }
        if let Some(v) = req.cpu_cores { ticket.cpu_cores = v; }
        if let Some(v) = req.memory_gb { ticket.memory_gb = v; }
        if let Some(v) = req.system_disk { ticket.system_disk = Some(v); }
        if let Some(v) = req.system_disk_size_gb { ticket.system_disk_size_gb = v; }
        if let Some(v) = req.data_disk { ticket.data_disk = Some(v); }
        if let Some(v) = req.has_security_product { ticket.has_security_product = v; }
        if let Some(v) = req.ip_address { ticket.ip_address = Some(v); }
        if let Some(v) = req.delivery_status { ticket.delivery_status = Some(v); }
        if let Some(v) = req.remarks { ticket.remarks = Some(v); }

        ticket.updated_at = Some(updated_at_str);

        let updated = ticket.clone();

        log_action_auth(
            &state.audit_logs,
            &user,
            "UPDATE_RESOURCE_TICKET",
            &format!("{}", id),
            &format!("更新资源工单: {}", id),
        );

        Ok(Json(json!({
            "message": "工单更新成功",
            "data": updated
        })).into_response())
    } else {
        Err(ApiError::not_found("Ticket not found"))
    }
}

/// 删除资源工单
pub async fn delete_resource_ticket(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApiError> {
    if user.role != Role::SysAdmin {
        return Err(ApiError::forbidden("Only SysAdmin can delete tickets"));
    }

    let mut tickets = RESOURCE_TICKETS.write().unwrap();
    if let Some(pos) = tickets.iter().position(|t| t.id == Some(id)) {
        tickets.remove(pos);

        log_action_auth(
            &state.audit_logs,
            &user,
            "DELETE_RESOURCE_TICKET",
            &format!("{}", id),
            &format!("删除资源工单: {}", id),
        );

        Ok(Json(json!({
            "message": "工单删除成功"
        })).into_response())
    } else {
        Err(ApiError::not_found("Ticket not found"))
    }
}

/// 审批工单
pub async fn approve_ticket(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i32>,
    Json(req): Json<ApproveTicketRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // 只有管理员和审批员可以审批
    if user.role != Role::SysAdmin && user.role != Role::SecAdmin {
        return Err(ApiError::forbidden("Access denied"));
    }

    let now = Utc::now();
    let time_str = now.format("%Y-%m-%d %H:%M").to_string();

    let mut tickets = RESOURCE_TICKETS.write().unwrap();
    if let Some(ticket) = tickets.iter_mut().find(|t| t.id == Some(id)) {
        if ticket.ticket_status != TicketStatus::PendingApproval {
            return Err(ApiError::bad_request("只能审批待审批状态的工单"));
        }

        ticket.approver = Some(user.username.clone());
        ticket.approve_time = Some(time_str.clone());
        ticket.approve_comment = req.comment.clone();

        if req.approved {
            ticket.ticket_status = TicketStatus::Approved;
        } else {
            ticket.ticket_status = TicketStatus::Rejected;
        }

        ticket.updated_at = Some(time_str);

        let updated = ticket.clone();

        log_action_auth(
            &state.audit_logs,
            &user,
            "APPROVE_RESOURCE_TICKET",
            &format!("{}", id),
            &format!("审批工单 {}: {}", id, if req.approved { "通过" } else { "拒绝" }),
        );

        Ok(Json(json!({
            "message": if req.approved { "工单审批通过" } else { "工单已拒绝" },
            "data": updated
        })).into_response())
    } else {
        Err(ApiError::not_found("Ticket not found"))
    }
}

/// 开始配置工单
pub async fn provision_ticket(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i32>,
    Json(req): Json<ProvisionTicketRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // 只有运维人员可以配置
    if user.role != Role::SysAdmin && user.role != Role::SecAdmin {
        return Err(ApiError::forbidden("Access denied"));
    }

    let now = Utc::now();
    let time_str = now.format("%Y-%m-%d %H:%M").to_string();

    let mut tickets = RESOURCE_TICKETS.write().unwrap();
    if let Some(ticket) = tickets.iter_mut().find(|t| t.id == Some(id)) {
        if ticket.ticket_status != TicketStatus::Approved {
            return Err(ApiError::bad_request("只能配置已批准的工单"));
        }

        ticket.provisioner = Some(user.username.clone());
        ticket.provision_time = Some(time_str.clone());
        ticket.provision_details = req.details.clone();
        ticket.ticket_status = TicketStatus::PendingDelivery;
        ticket.updated_at = Some(time_str);

        let updated = ticket.clone();

        log_action_auth(
            &state.audit_logs,
            &user,
            "PROVISION_RESOURCE_TICKET",
            &format!("{}", id),
            &format!("配置工单: {}", id),
        );

        Ok(Json(json!({
            "message": "工单配置完成",
            "data": updated
        })).into_response())
    } else {
        Err(ApiError::not_found("Ticket not found"))
    }
}

/// 交付工单
pub async fn deliver_ticket(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i32>,
    Json(req): Json<DeliverTicketRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // 只有运维和交付人员可以交付
    if user.role != Role::SysAdmin && user.role != Role::SecAdmin {
        return Err(ApiError::forbidden("Access denied"));
    }

    let now = Utc::now();
    let time_str = now.format("%Y-%m-%d %H:%M").to_string();

    let mut tickets = RESOURCE_TICKETS.write().unwrap();
    if let Some(ticket) = tickets.iter_mut().find(|t| t.id == Some(id)) {
        if ticket.ticket_status != TicketStatus::PendingDelivery {
            return Err(ApiError::bad_request("只能交付待交付状态的工单"));
        }

        ticket.deliverer = Some(user.username.clone());
        ticket.deliver_time = Some(time_str.clone());
        ticket.deliver_comment = req.comment.clone();
        ticket.ticket_status = TicketStatus::Delivered;
        ticket.delivery_status = Some("已交付".to_string());
        ticket.updated_at = Some(time_str);

        let updated = ticket.clone();

        log_action_auth(
            &state.audit_logs,
            &user,
            "DELIVER_RESOURCE_TICKET",
            &format!("{}", id),
            &format!("交付工单: {}", id),
        );

        Ok(Json(json!({
            "message": "工单交付完成",
            "data": updated
        })).into_response())
    } else {
        Err(ApiError::not_found("Ticket not found"))
    }
}

/// 辅助函数：获取关联名称
fn get_related_names(_req: &CreateResourceTicketRequest) -> (Option<String>, Option<String>, Option<String>) {
    // 这里简化处理，实际应该从数据库查询
    // 暂时返回 None，前端会提供名称
    (None, None, None)
}
