use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Json},
};
use chrono::Utc;
use serde_json::json;

use crate::database::{
    delete_resource_ticket as db_delete_resource_ticket, get_cloud_platform_config_by_id, get_db,
    get_department_by_id, get_machine_room_by_id, get_organization_by_id,
    get_resource_ticket as db_get_resource_ticket, get_resource_tickets as db_get_resource_tickets,
    get_service_provider_by_id, insert_resource_ticket_wrapper,
    update_resource_ticket as db_update_resource_ticket,
};
use crate::middleware::{ApiError, AuthUser};
use crate::state::AppState;
use crate::utils::{effective_permissions, get_current_user_from_auth, log_action_auth};
use shared::{
    ApproveTicketRequest, CreateResourceTicketRequest, DataScope, DeliverTicketRequest,
    ProvisionTicketRequest, ResourceTicket, ResourceTicketQuery, TicketStatus,
    UpdateResourceTicketRequest,
};

/// 获取资源工单列表
pub async fn get_resource_tickets(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<ResourceTicketQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let current_user = load_current_user(&state, &user).await?;
    let permissions = effective_permissions(&state, &current_user).await;

    if !permissions.can_view_resource_tickets {
        return Err(ApiError::forbidden("Access denied"));
    }

    let mut tickets = db_get_resource_tickets()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to load resource tickets: {}", e)))?;

    tickets.retain(|t| {
        let scope_match = can_access_ticket(t, &current_user, permissions.resource_ticket_scope);
        let keyword_match = query.search_keyword.as_ref().is_none_or(|keyword| {
            let keyword = keyword.to_lowercase();
            t.ecs_name.to_lowercase().contains(&keyword)
                || t.customer_name
                    .as_ref()
                    .is_some_and(|v| v.to_lowercase().contains(&keyword))
                || t.application_name
                    .as_ref()
                    .is_some_and(|v| v.to_lowercase().contains(&keyword))
                || t.ip_address
                    .as_ref()
                    .is_some_and(|v| v.to_lowercase().contains(&keyword))
        });

        let resource_type_match = query
            .resource_type
            .as_ref()
            .is_none_or(|resource_type| t.resource_type.as_str() == resource_type.as_str());

        let status_match = query
            .ticket_status
            .as_ref()
            .is_none_or(|status| t.ticket_status.as_str() == status.as_str());

        let provider_match = query
            .provider_id
            .is_none_or(|provider_id| t.provider_id == Some(provider_id));

        scope_match && keyword_match && resource_type_match && status_match && provider_match
    });

    tickets.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(Json(tickets).into_response())
}

/// 获取单个资源工单
pub async fn get_resource_ticket(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApiError> {
    let current_user = load_current_user(&state, &user).await?;
    let permissions = effective_permissions(&state, &current_user).await;

    if !permissions.can_view_resource_tickets {
        return Err(ApiError::forbidden("Access denied"));
    }

    let ticket = db_get_resource_ticket(id)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to load ticket: {}", e)))?
        .ok_or_else(|| ApiError::not_found("Ticket not found"))?;

    if !can_access_ticket(&ticket, &current_user, permissions.resource_ticket_scope) {
        return Err(ApiError::forbidden("Access denied"));
    }

    Ok(Json(ticket).into_response())
}

/// 创建资源工单
pub async fn create_resource_ticket(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<CreateResourceTicketRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let current_user = load_current_user(&state, &user).await?;
    let permissions = effective_permissions(&state, &current_user).await;

    if !permissions.can_create_resource_tickets {
        return Err(ApiError::forbidden("Access denied"));
    }

    let created_at = Utc::now().format("%Y-%m-%d %H:%M").to_string();
    let (provider_name, cloud_platform_name, machine_room_name) =
        resolve_related_names(req.provider_id, req.cloud_platform_id, req.machine_room_id).await?;
    let applicant_profile = resolve_ticket_user_profile(&state, &user).await?;

    let mut ticket = ResourceTicket {
        id: None,
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
        security_products: req.security_products,
        ip_address: req.ip_address,
        delivery_status: Some("未交付".to_string()),
        remarks: req.remarks,
        created_at: created_at.clone(),
        updated_at: Some(created_at.clone()),
        created_by: user.username.clone(),
        applicant_name: applicant_profile.display_name,
        organization_id: applicant_profile.organization_id,
        organization_name: applicant_profile.organization_name,
        department_id: applicant_profile.department_id,
        department_name: applicant_profile.department_name,
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

    let id = insert_resource_ticket_wrapper(&ticket)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to create ticket: {}", e)))?;
    ticket.id = Some(id);

    log_action_auth(
        &state.audit_logs,
        &user,
        "CREATE_RESOURCE_TICKET",
        &id.to_string(),
        &format!(
            "创建资源工单: {} ({})",
            ticket.ecs_name,
            ticket.resource_type.as_str()
        ),
    );

    Ok(Json(json!({
        "message": "工单创建成功",
        "data": ticket
    }))
    .into_response())
}

/// 更新资源工单
pub async fn update_resource_ticket(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i32>,
    Json(req): Json<UpdateResourceTicketRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let current_user = load_current_user(&state, &user).await?;
    let permissions = effective_permissions(&state, &current_user).await;

    if !permissions.can_approve_resource_tickets {
        return Err(ApiError::forbidden("Access denied"));
    }

    let mut ticket = db_get_resource_ticket(id)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to load ticket: {}", e)))?
        .ok_or_else(|| ApiError::not_found("Ticket not found"))?;

    if !can_access_ticket(&ticket, &current_user, permissions.resource_ticket_scope) {
        return Err(ApiError::forbidden("Access denied"));
    }

    if let Some(value) = req.ecs_name {
        ticket.ecs_name = value;
    }
    if let Some(value) = req.provider_id {
        ticket.provider_id = Some(value);
    }
    if let Some(value) = req.cloud_platform_id {
        ticket.cloud_platform_id = Some(value);
    }
    if let Some(value) = req.machine_room_id {
        ticket.machine_room_id = Some(value);
    }
    if let Some(value) = req.cloud_region {
        ticket.cloud_region = Some(value);
    }
    if let Some(value) = req.cloud_category {
        ticket.cloud_category = Some(value);
    }
    if let Some(value) = req.zone_name {
        ticket.zone_name = Some(value);
    }
    if let Some(value) = req.zone_cabinet {
        ticket.zone_cabinet = Some(value);
    }
    if let Some(value) = req.rack_units {
        ticket.rack_units = value;
    }
    if let Some(value) = req.customer_name {
        ticket.customer_name = Some(value);
    }
    if let Some(value) = req.application_name {
        ticket.application_name = Some(value);
    }
    if let Some(value) = req.contract_name {
        ticket.contract_name = Some(value);
    }
    if let Some(value) = req.ecs_type {
        ticket.ecs_type = Some(value);
    }
    if let Some(value) = req.ecs_os {
        ticket.ecs_os = Some(value);
    }
    if let Some(value) = req.cpu_cores {
        ticket.cpu_cores = value;
    }
    if let Some(value) = req.memory_gb {
        ticket.memory_gb = value;
    }
    if let Some(value) = req.system_disk {
        ticket.system_disk = Some(value);
    }
    if let Some(value) = req.system_disk_size_gb {
        ticket.system_disk_size_gb = value;
    }
    if let Some(value) = req.data_disk {
        ticket.data_disk = Some(value);
    }
    if let Some(value) = req.has_security_product {
        ticket.has_security_product = value;
    }
    if let Some(value) = req.security_products {
        ticket.security_products = Some(value);
    }
    if let Some(value) = req.ip_address {
        ticket.ip_address = Some(value);
    }
    if let Some(value) = req.delivery_status {
        ticket.delivery_status = Some(value);
    }
    if let Some(value) = req.remarks {
        ticket.remarks = Some(value);
    }

    let (provider_name, cloud_platform_name, machine_room_name) = resolve_related_names(
        ticket.provider_id,
        ticket.cloud_platform_id,
        ticket.machine_room_id,
    )
    .await?;
    ticket.provider_name = provider_name;
    ticket.cloud_platform_name = cloud_platform_name;
    ticket.machine_room_name = machine_room_name;
    ticket.updated_at = Some(Utc::now().format("%Y-%m-%d %H:%M").to_string());

    db_update_resource_ticket(id, &ticket)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to update ticket: {}", e)))?;

    log_action_auth(
        &state.audit_logs,
        &user,
        "UPDATE_RESOURCE_TICKET",
        &id.to_string(),
        &format!("更新资源工单: {}", id),
    );

    Ok(Json(json!({
        "message": "工单更新成功",
        "data": ticket
    }))
    .into_response())
}

/// 删除资源工单
pub async fn delete_resource_ticket(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApiError> {
    let current_user = load_current_user(&state, &user).await?;
    let permissions = effective_permissions(&state, &current_user).await;

    if !permissions.can_delete_resource_tickets {
        return Err(ApiError::forbidden("Access denied"));
    }

    let ticket = db_get_resource_ticket(id)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to load ticket: {}", e)))?
        .ok_or_else(|| ApiError::not_found("Ticket not found"))?;

    if !can_access_ticket(&ticket, &current_user, permissions.resource_ticket_scope) {
        return Err(ApiError::forbidden("Access denied"));
    }

    db_delete_resource_ticket(id)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to delete ticket: {}", e)))?;

    log_action_auth(
        &state.audit_logs,
        &user,
        "DELETE_RESOURCE_TICKET",
        &id.to_string(),
        &format!("删除资源工单: {}", id),
    );

    Ok(Json(json!({
        "message": "工单删除成功"
    }))
    .into_response())
}

/// 审批工单
pub async fn approve_ticket(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i32>,
    Json(req): Json<ApproveTicketRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let current_user = load_current_user(&state, &user).await?;
    let permissions = effective_permissions(&state, &current_user).await;

    if !permissions.can_approve_resource_tickets {
        return Err(ApiError::forbidden("Access denied"));
    }

    let mut ticket = db_get_resource_ticket(id)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to load ticket: {}", e)))?
        .ok_or_else(|| ApiError::not_found("Ticket not found"))?;

    if !can_access_ticket(&ticket, &current_user, permissions.resource_ticket_scope) {
        return Err(ApiError::forbidden("Access denied"));
    }

    if ticket.ticket_status != TicketStatus::PendingApproval {
        return Err(ApiError::bad_request("只能审批待审批状态的工单"));
    }

    let now = Utc::now().format("%Y-%m-%d %H:%M").to_string();
    let operator_display_name = resolve_operator_display_name(&state, &user).await?;
    ticket.approver = Some(operator_display_name);
    ticket.approve_time = Some(now.clone());
    ticket.approve_comment = req.comment.clone();
    ticket.ticket_status = if req.approved {
        TicketStatus::PendingProvision
    } else {
        TicketStatus::Rejected
    };
    ticket.updated_at = Some(now);

    db_update_resource_ticket(id, &ticket)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to update ticket: {}", e)))?;

    log_action_auth(
        &state.audit_logs,
        &user,
        "APPROVE_RESOURCE_TICKET",
        &id.to_string(),
        &format!(
            "审批工单 {}: {}",
            id,
            if req.approved { "通过" } else { "拒绝" }
        ),
    );

    Ok(Json(json!({
        "message": if req.approved { "工单审批通过" } else { "工单已拒绝" },
        "data": ticket
    }))
    .into_response())
}

/// 开始配置工单
pub async fn provision_ticket(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i32>,
    Json(req): Json<ProvisionTicketRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let current_user = load_current_user(&state, &user).await?;
    let permissions = effective_permissions(&state, &current_user).await;

    if !permissions.can_provision_resource_tickets {
        return Err(ApiError::forbidden("Access denied"));
    }

    let mut ticket = db_get_resource_ticket(id)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to load ticket: {}", e)))?
        .ok_or_else(|| ApiError::not_found("Ticket not found"))?;

    if !can_access_ticket(&ticket, &current_user, permissions.resource_ticket_scope) {
        return Err(ApiError::forbidden("Access denied"));
    }

    if ticket.ticket_status != TicketStatus::PendingProvision
        && ticket.ticket_status != TicketStatus::Approved
    {
        return Err(ApiError::bad_request("只能配置待配置状态的工单"));
    }

    let now = Utc::now().format("%Y-%m-%d %H:%M").to_string();
    let operator_display_name = resolve_operator_display_name(&state, &user).await?;
    ticket.provisioner = Some(operator_display_name);
    ticket.provision_time = Some(now.clone());
    ticket.provision_details = req.details.clone();
    ticket.ticket_status = TicketStatus::PendingDelivery;
    ticket.updated_at = Some(now);

    db_update_resource_ticket(id, &ticket)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to update ticket: {}", e)))?;

    log_action_auth(
        &state.audit_logs,
        &user,
        "PROVISION_RESOURCE_TICKET",
        &id.to_string(),
        &format!("配置工单: {}", id),
    );

    Ok(Json(json!({
        "message": "工单配置完成",
        "data": ticket
    }))
    .into_response())
}

/// 交付工单
pub async fn deliver_ticket(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i32>,
    Json(req): Json<DeliverTicketRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let current_user = load_current_user(&state, &user).await?;
    let permissions = effective_permissions(&state, &current_user).await;

    if !permissions.can_deliver_resource_tickets {
        return Err(ApiError::forbidden("Access denied"));
    }

    let mut ticket = db_get_resource_ticket(id)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to load ticket: {}", e)))?
        .ok_or_else(|| ApiError::not_found("Ticket not found"))?;

    if !can_access_ticket(&ticket, &current_user, permissions.resource_ticket_scope) {
        return Err(ApiError::forbidden("Access denied"));
    }

    if ticket.ticket_status != TicketStatus::PendingDelivery {
        return Err(ApiError::bad_request("只能交付待交付状态的工单"));
    }

    let now = Utc::now().format("%Y-%m-%d %H:%M").to_string();
    let operator_display_name = resolve_operator_display_name(&state, &user).await?;
    ticket.deliverer = Some(operator_display_name);
    ticket.deliver_time = Some(now.clone());
    ticket.deliver_comment = req.comment.clone();
    ticket.ticket_status = TicketStatus::Delivered;
    ticket.delivery_status = Some("已交付".to_string());
    ticket.updated_at = Some(now);

    db_update_resource_ticket(id, &ticket)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to update ticket: {}", e)))?;

    log_action_auth(
        &state.audit_logs,
        &user,
        "DELIVER_RESOURCE_TICKET",
        &id.to_string(),
        &format!("交付工单: {}", id),
    );

    Ok(Json(json!({
        "message": "工单交付完成",
        "data": ticket
    }))
    .into_response())
}

struct TicketUserProfile {
    display_name: Option<String>,
    organization_id: Option<i32>,
    organization_name: Option<String>,
    department_id: Option<i32>,
    department_name: Option<String>,
}

async fn resolve_ticket_user_profile(
    state: &AppState,
    user: &AuthUser,
) -> Result<TicketUserProfile, ApiError> {
    let current_user = get_current_user_from_auth(user, &state.users)
        .await
        .ok_or_else(|| ApiError::unauthorized("User not found"))?;

    let conn = get_db().ok_or_else(|| ApiError::internal("Database not available"))?;

    let organization_name = match current_user.organization_id {
        Some(id) => get_organization_by_id(conn.as_ref(), id)
            .await
            .map_err(|e| ApiError::internal(format!("Failed to load organization: {}", e)))?
            .map(|item| item.name),
        None => None,
    };

    let department_name = match current_user.department_id {
        Some(id) => get_department_by_id(conn.as_ref(), id)
            .await
            .map_err(|e| ApiError::internal(format!("Failed to load department: {}", e)))?
            .map(|item| item.name),
        None => None,
    };

    Ok(TicketUserProfile {
        display_name: Some(display_name_for_user(&current_user)),
        organization_id: current_user.organization_id,
        organization_name,
        department_id: current_user.department_id,
        department_name,
    })
}

async fn resolve_operator_display_name(
    state: &AppState,
    user: &AuthUser,
) -> Result<String, ApiError> {
    let current_user = get_current_user_from_auth(user, &state.users)
        .await
        .ok_or_else(|| ApiError::unauthorized("User not found"))?;
    Ok(display_name_for_user(&current_user))
}

fn display_name_for_user(user: &shared::User) -> String {
    user.real_name
        .clone()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| user.username.clone())
}

async fn load_current_user(state: &AppState, user: &AuthUser) -> Result<shared::User, ApiError> {
    get_current_user_from_auth(user, &state.users)
        .await
        .ok_or_else(|| ApiError::unauthorized("User not found"))
}

fn can_access_ticket(ticket: &ResourceTicket, user: &shared::User, scope: DataScope) -> bool {
    match scope {
        DataScope::SelfOnly => ticket.created_by == user.username,
        DataScope::Department => {
            user.department_id.is_some() && ticket.department_id == user.department_id
        }
        DataScope::Organization => {
            user.organization_id.is_some() && ticket.organization_id == user.organization_id
        }
        DataScope::All => true,
    }
}

async fn resolve_related_names(
    provider_id: Option<i32>,
    cloud_platform_id: Option<i32>,
    machine_room_id: Option<i32>,
) -> Result<(Option<String>, Option<String>, Option<String>), ApiError> {
    let conn =
        crate::database::get_db().ok_or_else(|| ApiError::internal("Database not available"))?;

    let provider_name = match provider_id {
        Some(id) => get_service_provider_by_id(conn.as_ref(), id)
            .await
            .map_err(|e| ApiError::internal(format!("Failed to load service provider: {}", e)))?
            .map(|provider| provider.short_name),
        None => None,
    };

    let cloud_platform_name = match cloud_platform_id {
        Some(id) => get_cloud_platform_config_by_id(conn.as_ref(), id)
            .await
            .map_err(|e| ApiError::internal(format!("Failed to load cloud platform: {}", e)))?
            .map(|platform| platform.platform_name),
        None => None,
    };

    let machine_room_name = match machine_room_id {
        Some(id) => get_machine_room_by_id(conn.as_ref(), id)
            .await
            .map_err(|e| ApiError::internal(format!("Failed to load machine room: {}", e)))?
            .map(|room| room.room_name),
        None => None,
    };

    Ok((provider_name, cloud_platform_name, machine_room_name))
}
