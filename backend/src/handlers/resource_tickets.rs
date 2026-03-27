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
use crate::utils::{
    effective_permissions, get_current_user_from_auth, is_builtin_system_account, log_action_auth,
};
use sea_orm::DatabaseConnection;
use shared::{
    ApproveTicketRequest, CreateResourceTicketRequest, DataScope, DeliverTicketRequest,
    Permissions, ProvisionTicketRequest, ResourceTicket, ResourceTicketQuery, TicketStatus,
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

    if !can_access_resource_ticket_module(&permissions) {
        return Err(ApiError::forbidden("Access denied"));
    }

    let mut tickets = db_get_resource_tickets()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to load resource tickets: {}", e)))?;
    let conn = get_db().ok_or_else(|| ApiError::internal("Database not available"))?;

    for ticket in &mut tickets {
        prepare_ticket_for_response(conn.as_ref(), ticket).await?;
    }

    tickets.retain(|t| {
        let scope_match = can_access_ticket(t, &current_user, permissions.resource_ticket_scope);
        let keyword_match = query.search_keyword.as_ref().is_none_or(|keyword| {
            let keyword = keyword.to_lowercase();
            t.ecs_name.to_lowercase().contains(&keyword)
                || t.created_by.to_lowercase().contains(&keyword)
                || t.applicant_name
                    .as_ref()
                    .is_some_and(|v| v.to_lowercase().contains(&keyword))
                || t.organization_name
                    .as_ref()
                    .is_some_and(|v| v.to_lowercase().contains(&keyword))
                || t.department_name
                    .as_ref()
                    .is_some_and(|v| v.to_lowercase().contains(&keyword))
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

    if !can_access_resource_ticket_module(&permissions) {
        return Err(ApiError::forbidden("Access denied"));
    }

    let mut ticket = db_get_resource_ticket(id)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to load ticket: {}", e)))?
        .ok_or_else(|| ApiError::not_found("Ticket not found"))?;
    let conn = get_db().ok_or_else(|| ApiError::internal("Database not available"))?;
    prepare_ticket_for_response(conn.as_ref(), &mut ticket).await?;

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
    validate_ticket_user_profile_for_request(&current_user, &applicant_profile)?;

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
        application_endpoint_id: req.application_endpoint_id,
        application_domain: req.application_domain,
        contract_name: req.contract_name,
        ecs_type: req.ecs_type,
        ecs_os: req.ecs_os,
        resource_count: req.resource_count.unwrap_or(0),
        cpu_cores: req.cpu_cores.unwrap_or(0),
        memory_gb: req.memory_gb.unwrap_or(0),
        system_disk: req.system_disk,
        system_disk_size_gb: req.system_disk_size_gb.unwrap_or(0),
        data_disk: req.data_disk,
        expire_at: req.expire_at,
        has_security_product: req.has_security_product.unwrap_or(false),
        security_products: req.security_products,
        ip_address: req.ip_address,
        delivery_status: Some("未交付".to_string()),
        remarks: req.remarks,
        created_at: created_at.clone(),
        updated_at: Some(created_at.clone()),
        created_by: user.username.clone(),
        applicant_name: Some(applicant_profile.display_name.clone()),
        organization_id: applicant_profile.organization_id,
        organization_name: optional_non_empty(&applicant_profile.organization_name),
        department_id: applicant_profile.department_id,
        department_name: optional_non_empty(&applicant_profile.department_name),
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
        fw_source_port: req.fw_source_port,
        fw_dest_zone: req.fw_dest_zone,
        fw_dest_address: req.fw_dest_address,
        fw_dest_port: req.fw_dest_port,
        fw_protocol: req.fw_protocol,
        fw_port: req.fw_port,
        fw_direction: req.fw_direction,
        fw_valid_until: req.fw_valid_until,
        fw_firewall_name: req.fw_firewall_name,
    };
    normalize_ticket_fields(&mut ticket);
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
    normalize_ticket_fields(&mut ticket);

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
    if let Some(value) = req.application_endpoint_id {
        ticket.application_endpoint_id = Some(value);
    }
    if let Some(value) = req.application_domain {
        ticket.application_domain = Some(value);
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
    if let Some(value) = req.resource_count {
        ticket.resource_count = value;
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
    if let Some(value) = req.expire_at {
        ticket.expire_at = Some(value);
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
    if let Some(value) = req.fw_source_zone {
        ticket.fw_source_zone = Some(value);
    }
    if let Some(value) = req.fw_source_address {
        ticket.fw_source_address = Some(value);
    }
    if let Some(value) = req.fw_source_port {
        ticket.fw_source_port = Some(value);
    }
    if let Some(value) = req.fw_dest_zone {
        ticket.fw_dest_zone = Some(value);
    }
    if let Some(value) = req.fw_dest_address {
        ticket.fw_dest_address = Some(value);
    }
    if let Some(value) = req.fw_dest_port {
        ticket.fw_dest_port = Some(value);
    }
    if let Some(value) = req.fw_protocol {
        ticket.fw_protocol = Some(value);
    }
    if let Some(value) = req.fw_port {
        ticket.fw_port = Some(value);
    }
    if let Some(value) = req.fw_direction {
        ticket.fw_direction = Some(value);
    }
    if let Some(value) = req.fw_valid_until {
        ticket.fw_valid_until = Some(value);
    }
    if let Some(value) = req.fw_firewall_name {
        ticket.fw_firewall_name = Some(value);
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
    normalize_ticket_fields(&mut ticket);
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
    normalize_ticket_fields(&mut ticket);

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
    normalize_ticket_fields(&mut ticket);

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
    normalize_ticket_fields(&mut ticket);

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
    normalize_ticket_fields(&mut ticket);

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
    normalize_ticket_fields(&mut ticket);

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
    normalize_ticket_fields(&mut ticket);

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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct TicketUserProfile {
    display_name: String,
    organization_id: Option<i32>,
    organization_name: String,
    department_id: Option<i32>,
    department_name: String,
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
        display_name: display_name_for_user(&current_user),
        organization_id: current_user.organization_id,
        organization_name: organization_name.unwrap_or_default(),
        department_id: current_user.department_id,
        department_name: department_name.unwrap_or_default(),
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

fn can_access_resource_ticket_module(permissions: &Permissions) -> bool {
    permissions.can_view_resource_tickets
        || permissions.can_create_resource_tickets
        || permissions.can_approve_resource_tickets
        || permissions.can_provision_resource_tickets
        || permissions.can_deliver_resource_tickets
}

fn optional_non_empty(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn normalize_ticket_status(status: TicketStatus) -> TicketStatus {
    match status {
        TicketStatus::Approved | TicketStatus::Provisioning => TicketStatus::PendingProvision,
        other => other,
    }
}

fn normalize_ticket_fields(ticket: &mut ResourceTicket) {
    ticket.ticket_status = normalize_ticket_status(ticket.ticket_status);

    if ticket
        .fw_dest_port
        .as_ref()
        .is_none_or(|value| value.trim().is_empty())
    {
        ticket.fw_dest_port = ticket.fw_port.clone();
    }

    if ticket
        .fw_port
        .as_ref()
        .is_none_or(|value| value.trim().is_empty())
    {
        ticket.fw_port = ticket.fw_dest_port.clone();
    }

    if ticket.resource_type == shared::ResourceType::Network {
        clear_network_ticket_application_fields(ticket);
    } else {
        ticket.application_domain = ticket
            .application_domain
            .as_deref()
            .map(normalize_domain)
            .filter(|value| !value.is_empty());
    }

    if ticket
        .applicant_name
        .as_ref()
        .is_none_or(|value| value.trim().is_empty())
    {
        ticket.applicant_name = Some(ticket.created_by.clone());
    }

    if ticket
        .delivery_status
        .as_ref()
        .is_none_or(|value| value.trim().is_empty())
    {
        let next_status = if ticket.ticket_status == TicketStatus::Delivered {
            "已交付"
        } else {
            "未交付"
        };
        ticket.delivery_status = Some(next_status.to_string());
    }
}

async fn prepare_ticket_for_response(
    conn: &DatabaseConnection,
    ticket: &mut ResourceTicket,
) -> Result<(), ApiError> {
    if clear_network_ticket_application_fields(ticket) {
        if let Some(id) = ticket.id {
            db_update_resource_ticket(id, ticket)
                .await
                .map_err(|e| ApiError::internal(format!("Failed to cleanup network ticket: {}", e)))?;
        }
    }

    if ticket
        .organization_name
        .as_ref()
        .is_none_or(|value| value.trim().is_empty())
    {
        if let Some(id) = ticket.organization_id {
            ticket.organization_name = get_organization_by_id(conn, id)
                .await
                .map_err(|e| ApiError::internal(format!("Failed to load organization: {}", e)))?
                .map(|item| item.name);
        }
    }

    if ticket
        .department_name
        .as_ref()
        .is_none_or(|value| value.trim().is_empty())
    {
        if let Some(id) = ticket.department_id {
            ticket.department_name = get_department_by_id(conn, id)
                .await
                .map_err(|e| ApiError::internal(format!("Failed to load department: {}", e)))?
                .map(|item| item.name);
        }
    }

    normalize_ticket_fields(ticket);
    Ok(())
}

fn clear_network_ticket_application_fields(ticket: &mut ResourceTicket) -> bool {
    if ticket.resource_type != shared::ResourceType::Network {
        return false;
    }

    let had_values = ticket
        .application_name
        .as_ref()
        .is_some_and(|value| !value.trim().is_empty())
        || ticket.application_endpoint_id.is_some()
        || ticket
            .application_domain
            .as_ref()
            .is_some_and(|value| !value.trim().is_empty());

    ticket.application_name = None;
    ticket.application_endpoint_id = None;
    ticket.application_domain = None;

    had_values
}

fn normalize_domain(value: &str) -> String {
    value
        .trim()
        .trim_matches('.')
        .to_ascii_lowercase()
}

fn validate_ticket_user_profile_for_request(
    current_user: &shared::User,
    profile: &TicketUserProfile,
) -> Result<(), ApiError> {
    if profile.display_name.trim().is_empty() {
        return Err(ApiError::bad_request("当前用户缺少姓名，请先完善用户资料"));
    }

    if is_builtin_system_account(&current_user.username) {
        return Ok(());
    }

    if profile.organization_id.is_none() || profile.organization_name.trim().is_empty() {
        return Err(ApiError::bad_request(
            "当前用户缺少所属公司/组织，请先完善用户资料",
        ));
    }

    if profile.department_id.is_none() || profile.department_name.trim().is_empty() {
        return Err(ApiError::bad_request(
            "当前用户缺少所属部门，请先完善用户资料",
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use shared::{Permissions, Role, User};

    fn sample_user() -> User {
        User {
            id: "u-1".to_string(),
            username: "alice".to_string(),
            real_name: Some("Alice".to_string()),
            password: "hashed".to_string(),
            role: Role::Custom("申请人".to_string()),
            permissions: None,
            created_at: Utc::now(),
            password_changed_at: None,
            password_strength: None,
            force_password_change: Some(false),
            last_login_at: None,
            email: None,
            phone: None,
            status: Some("active".to_string()),
            organization_id: Some(10),
            department_id: Some(20),
            failed_login_attempts: Some(0),
            locked_until: None,
        }
    }

    fn sample_ticket() -> ResourceTicket {
        ResourceTicket {
            id: Some(1),
            resource_type: shared::ResourceType::Cloud,
            ecs_name: "ecs-1".to_string(),
            ticket_status: TicketStatus::Approved,
            provider_id: None,
            provider_name: None,
            cloud_platform_id: None,
            cloud_platform_name: None,
            machine_room_id: None,
            machine_room_name: None,
            cloud_region: None,
            cloud_category: None,
            zone_name: None,
            zone_cabinet: None,
            rack_units: 0,
            customer_name: None,
            application_name: Some("OA".to_string()),
            application_endpoint_id: None,
            application_domain: None,
            contract_name: None,
            ecs_type: None,
            ecs_os: None,
            resource_count: 0,
            cpu_cores: 0,
            memory_gb: 0,
            system_disk: None,
            system_disk_size_gb: 0,
            data_disk: None,
            expire_at: None,
            has_security_product: false,
            security_products: None,
            ip_address: None,
            delivery_status: None,
            remarks: None,
            created_at: "2026-03-25 10:00".to_string(),
            updated_at: Some("2026-03-25 10:00".to_string()),
            created_by: "alice".to_string(),
            applicant_name: None,
            organization_id: Some(10),
            organization_name: None,
            department_id: Some(20),
            department_name: None,
            approver: None,
            approve_time: None,
            approve_comment: None,
            provisioner: None,
            provision_time: None,
            provision_details: None,
            deliverer: None,
            deliver_time: None,
            deliver_comment: None,
            fw_source_zone: None,
            fw_source_address: None,
            fw_source_port: None,
            fw_dest_zone: None,
            fw_dest_address: None,
            fw_dest_port: None,
            fw_protocol: None,
            fw_port: None,
            fw_direction: None,
            fw_valid_until: None,
            fw_firewall_name: None,
        }
    }

    #[test]
    fn resource_ticket_module_access_accepts_workflow_permissions() {
        let mut permissions = Permissions::default();
        permissions.can_approve_resource_tickets = true;
        assert!(can_access_resource_ticket_module(&permissions));

        let view_only = Permissions {
            can_view_resource_tickets: true,
            ..Default::default()
        };
        assert!(can_access_resource_ticket_module(&view_only));

        assert!(!can_access_resource_ticket_module(&Permissions::default()));
    }

    #[test]
    fn ticket_scope_respects_self_department_org_and_all() {
        let user = sample_user();
        let ticket = sample_ticket();

        assert!(can_access_ticket(&ticket, &user, DataScope::SelfOnly));
        assert!(can_access_ticket(&ticket, &user, DataScope::Department));
        assert!(can_access_ticket(&ticket, &user, DataScope::Organization));
        assert!(can_access_ticket(&ticket, &user, DataScope::All));
    }

    #[test]
    fn ticket_normalization_aligns_legacy_status_and_defaults() {
        let mut ticket = sample_ticket();
        normalize_ticket_fields(&mut ticket);

        assert_eq!(ticket.ticket_status, TicketStatus::PendingProvision);
        assert_eq!(ticket.applicant_name.as_deref(), Some("alice"));
        assert_eq!(ticket.delivery_status.as_deref(), Some("未交付"));
    }

    #[test]
    fn normal_accounts_require_complete_profile_for_ticket_requests() {
        let user = sample_user();
        let incomplete = TicketUserProfile {
            display_name: "Alice".to_string(),
            organization_id: None,
            organization_name: String::new(),
            department_id: Some(20),
            department_name: "研发部".to_string(),
        };

        assert!(validate_ticket_user_profile_for_request(&user, &incomplete).is_err());

        let complete = TicketUserProfile {
            display_name: "Alice".to_string(),
            organization_id: Some(10),
            organization_name: "示例公司".to_string(),
            department_id: Some(20),
            department_name: "研发部".to_string(),
        };

        assert!(validate_ticket_user_profile_for_request(&user, &complete).is_ok());
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
