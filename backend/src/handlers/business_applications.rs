use axum::{
    extract::{Json, Path, State},
    response::{IntoResponse, Json as ResponseJson},
};
use sea_orm::DatabaseConnection;

use crate::database::{
    delete_application_endpoint_with_conn, delete_application_endpoints_by_application_id_with_conn,
    delete_business_application_with_conn, find_application_endpoint_l4_with_conn,
    find_application_endpoint_l7_with_conn, get_all_business_applications_with_conn,
    get_application_endpoint_by_id_with_conn, get_application_endpoints_by_application_ids_with_conn,
    get_business_application_by_id_with_conn, get_business_application_by_name_with_conn, get_db,
    insert_application_endpoint_with_conn, insert_business_application_with_conn,
    update_application_endpoint_with_conn, update_business_application_with_conn,
};
use crate::middleware::{ApiError, AuthUser};
use crate::state::AppState;
use crate::utils::{effective_permissions, log_action_auth, require_current_user_from_auth};
use shared::{
    ApplicationEndpointRecord, BusinessApplicationRecord, CreateApplicationEndpointRequest,
    CreateBusinessApplicationRequest, Permissions, UpdateApplicationEndpointRequest,
    UpdateBusinessApplicationRequest,
};

pub async fn get_business_applications(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, ApiError> {
    let current_user = require_current_user_from_auth(&auth_user, &state).await?;
    let permissions = effective_permissions(&state, &current_user).await;
    if !can_access_business_application_module(&permissions) {
        return Err(ApiError::forbidden("Access denied"));
    }

    let conn = get_db().ok_or_else(|| ApiError::internal("Database not available"))?;
    let response = load_business_application_records(conn.as_ref()).await?;
    Ok(ResponseJson(response).into_response())
}

pub async fn create_business_application(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(req): Json<CreateBusinessApplicationRequest>,
) -> Result<impl IntoResponse, ApiError> {
    ensure_can_create_application(&state, &auth_user).await?;
    let conn = get_db().ok_or_else(|| ApiError::internal("Database not available"))?;

    let (name, description) = normalize_application_payload(&req.name, req.description.as_deref())?;
    if get_business_application_by_name_with_conn(conn.as_ref(), &name)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to query business application: {}", e)))?
        .is_some()
    {
        return Err(ApiError::conflict("业务应用名称已存在"));
    }

    let application = insert_business_application_with_conn(
        conn.as_ref(),
        &name,
        description.as_deref(),
        Some(&auth_user.username),
    )
    .await
    .map_err(|e| ApiError::internal(format!("Failed to create business application: {}", e)))?;

    log_action_auth(
        &state.audit_logs,
        &auth_user,
        "CREATE_BUSINESS_APPLICATION",
        &application.name,
        "Created business application",
    );

    Ok(ResponseJson(BusinessApplicationRecord {
        id: Some(application.id),
        name: application.name,
        description: application.description,
        created_at: application.created_at,
        updated_at: application.updated_at,
        created_by: application.created_by,
        endpoints: Vec::new(),
    })
    .into_response())
}

pub async fn update_business_application(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<i32>,
    Json(req): Json<UpdateBusinessApplicationRequest>,
) -> Result<impl IntoResponse, ApiError> {
    ensure_can_update_application(&state, &auth_user).await?;
    let conn = get_db().ok_or_else(|| ApiError::internal("Database not available"))?;

    let (name, description) = normalize_application_payload(&req.name, req.description.as_deref())?;
    if let Some(existing) = get_business_application_by_name_with_conn(conn.as_ref(), &name)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to query business application: {}", e)))?
    {
        if existing.id != id {
            return Err(ApiError::conflict("业务应用名称已存在"));
        }
    }

    let Some(application) = update_business_application_with_conn(
        conn.as_ref(),
        id,
        &name,
        description.as_deref(),
    )
    .await
    .map_err(|e| ApiError::internal(format!("Failed to update business application: {}", e)))?
    else {
        return Err(ApiError::not_found("Business application not found"));
    };

    log_action_auth(
        &state.audit_logs,
        &auth_user,
        "UPDATE_BUSINESS_APPLICATION",
        &application.name,
        "Updated business application",
    );

    let application_name = application.name.clone();
    Ok(ResponseJson(BusinessApplicationRecord {
        id: Some(application.id),
        name: application.name,
        description: application.description,
        created_at: application.created_at,
        updated_at: application.updated_at,
        created_by: application.created_by,
        endpoints: load_application_endpoints(conn.as_ref(), application.id, &application_name).await?,
    })
    .into_response())
}

pub async fn delete_business_application(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApiError> {
    ensure_can_delete_application(&state, &auth_user).await?;
    let conn = get_db().ok_or_else(|| ApiError::internal("Database not available"))?;

    let Some(application) = get_business_application_by_id_with_conn(conn.as_ref(), id)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to query business application: {}", e)))?
    else {
        return Err(ApiError::not_found("Business application not found"));
    };

    delete_application_endpoints_by_application_id_with_conn(conn.as_ref(), id)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to delete application endpoints: {}", e)))?;
    delete_business_application_with_conn(conn.as_ref(), id)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to delete business application: {}", e)))?;

    log_action_auth(
        &state.audit_logs,
        &auth_user,
        "DELETE_BUSINESS_APPLICATION",
        &application.name,
        "Deleted business application",
    );

    Ok(ResponseJson(serde_json::json!({
        "message": "业务应用已删除"
    }))
    .into_response())
}

pub async fn create_application_endpoint(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(req): Json<CreateApplicationEndpointRequest>,
) -> Result<impl IntoResponse, ApiError> {
    ensure_can_create_application(&state, &auth_user).await?;
    let conn = get_db().ok_or_else(|| ApiError::internal("Database not available"))?;

    let (application, protocol, dest_ip, nat_ip, dest_port, domain) =
        validate_endpoint_payload(conn.as_ref(), req.business_application_id, &req.protocol, &req.dest_ip, &req.nat_ip, &req.dest_port, req.domain.as_deref()).await?;

    ensure_endpoint_unique(
        conn.as_ref(),
        None,
        application.id,
        &application.name,
        &protocol,
        &dest_ip,
        &dest_port,
        domain.as_deref(),
    )
    .await?;

    let endpoint = insert_application_endpoint_with_conn(
        conn.as_ref(),
        application.id,
        &protocol,
        &dest_ip,
        Some(&nat_ip),
        &dest_port,
        domain.as_deref(),
        Some(&auth_user.username),
    )
    .await
    .map_err(|e| ApiError::internal(format!("Failed to create application endpoint: {}", e)))?;

    log_action_auth(
        &state.audit_logs,
        &auth_user,
        "CREATE_APPLICATION_ENDPOINT",
        &application.name,
        "Created application endpoint",
    );

    Ok(ResponseJson(endpoint_to_record(&application.name, endpoint)).into_response())
}

pub async fn update_application_endpoint(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<i32>,
    Json(req): Json<UpdateApplicationEndpointRequest>,
) -> Result<impl IntoResponse, ApiError> {
    ensure_can_update_application(&state, &auth_user).await?;
    let conn = get_db().ok_or_else(|| ApiError::internal("Database not available"))?;

    let Some(existing_endpoint) = get_application_endpoint_by_id_with_conn(conn.as_ref(), id)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to query application endpoint: {}", e)))?
    else {
        return Err(ApiError::not_found("Application endpoint not found"));
    };

    let (application, protocol, dest_ip, nat_ip, dest_port, domain) =
        validate_endpoint_payload(conn.as_ref(), req.business_application_id, &req.protocol, &req.dest_ip, &req.nat_ip, &req.dest_port, req.domain.as_deref()).await?;

    ensure_endpoint_unique(
        conn.as_ref(),
        Some(id),
        application.id,
        &application.name,
        &protocol,
        &dest_ip,
        &dest_port,
        domain.as_deref(),
    )
    .await?;

    let Some(endpoint) = update_application_endpoint_with_conn(
        conn.as_ref(),
        id,
        application.id,
        &protocol,
        &dest_ip,
        Some(&nat_ip),
        &dest_port,
        domain.as_deref(),
    )
    .await
    .map_err(|e| ApiError::internal(format!("Failed to update application endpoint: {}", e)))?
    else {
        return Err(ApiError::not_found("Application endpoint not found"));
    };

    log_action_auth(
        &state.audit_logs,
        &auth_user,
        "UPDATE_APPLICATION_ENDPOINT",
        &application.name,
        &format!("Updated application endpoint {}", existing_endpoint.id),
    );

    Ok(ResponseJson(endpoint_to_record(&application.name, endpoint)).into_response())
}

pub async fn delete_application_endpoint(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApiError> {
    ensure_can_delete_application(&state, &auth_user).await?;
    let conn = get_db().ok_or_else(|| ApiError::internal("Database not available"))?;

    let Some(endpoint) = get_application_endpoint_by_id_with_conn(conn.as_ref(), id)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to query application endpoint: {}", e)))?
    else {
        return Err(ApiError::not_found("Application endpoint not found"));
    };

    let application_name = get_business_application_by_id_with_conn(conn.as_ref(), endpoint.business_application_id)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to query business application: {}", e)))?
        .map(|item| item.name)
        .unwrap_or_else(|| "未知应用".to_string());

    delete_application_endpoint_with_conn(conn.as_ref(), id)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to delete application endpoint: {}", e)))?;

    log_action_auth(
        &state.audit_logs,
        &auth_user,
        "DELETE_APPLICATION_ENDPOINT",
        &application_name,
        &format!("Deleted application endpoint {}", id),
    );

    Ok(ResponseJson(serde_json::json!({
        "message": "应用端点已删除"
    }))
    .into_response())
}

async fn load_business_application_records(
    conn: &DatabaseConnection,
) -> Result<Vec<BusinessApplicationRecord>, ApiError> {
    let applications = get_all_business_applications_with_conn(conn)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to load business applications: {}", e)))?;

    let application_ids = applications.iter().map(|item| item.id).collect::<Vec<_>>();
    let endpoints = get_application_endpoints_by_application_ids_with_conn(conn, &application_ids)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to load application endpoints: {}", e)))?;

    Ok(applications
        .into_iter()
        .map(|application| {
            let records = endpoints
                .iter()
                .filter(|endpoint| endpoint.business_application_id == application.id)
                .cloned()
                .map(|endpoint| endpoint_to_record(&application.name, endpoint))
                .collect::<Vec<_>>();

            BusinessApplicationRecord {
                id: Some(application.id),
                name: application.name,
                description: application.description,
                created_at: application.created_at,
                updated_at: application.updated_at,
                created_by: application.created_by,
                endpoints: records,
            }
        })
        .collect())
}

async fn load_application_endpoints(
    conn: &DatabaseConnection,
    application_id: i32,
    application_name: &str,
) -> Result<Vec<ApplicationEndpointRecord>, ApiError> {
    let endpoints = get_application_endpoints_by_application_ids_with_conn(conn, &[application_id])
        .await
        .map_err(|e| ApiError::internal(format!("Failed to load application endpoints: {}", e)))?;

    Ok(endpoints
        .into_iter()
        .map(|endpoint| endpoint_to_record(application_name, endpoint))
        .collect())
}

fn endpoint_to_record(
    application_name: &str,
    endpoint: crate::entities::application_endpoint::Model,
) -> ApplicationEndpointRecord {
    ApplicationEndpointRecord {
        id: Some(endpoint.id),
        business_application_id: endpoint.business_application_id,
        business_application_name: application_name.to_string(),
        protocol: endpoint.protocol,
        dest_ip: endpoint.dest_ip,
        nat_ip: endpoint.nat_ip,
        dest_port: endpoint.dest_port,
        domain: endpoint.domain,
        created_at: endpoint.created_at,
        updated_at: endpoint.updated_at,
        created_by: endpoint.created_by,
    }
}

fn normalize_application_payload(
    name: &str,
    description: Option<&str>,
) -> Result<(String, Option<String>), ApiError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(ApiError::bad_request("业务应用名称不能为空"));
    }

    Ok((
        name.to_string(),
        description
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string),
    ))
}

async fn validate_endpoint_payload(
    conn: &DatabaseConnection,
    business_application_id: i32,
    protocol: &str,
    dest_ip: &str,
    nat_ip: &str,
    dest_port: &str,
    domain: Option<&str>,
) -> Result<
    (
        crate::entities::business_application::Model,
        String,
        String,
        String,
        String,
        Option<String>,
    ),
    ApiError,
> {
    let Some(application) = get_business_application_by_id_with_conn(conn, business_application_id)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to query business application: {}", e)))?
    else {
        return Err(ApiError::bad_request("业务应用不存在"));
    };

    let protocol = protocol.trim().to_ascii_uppercase();
    let dest_ip = dest_ip.trim().to_string();
    let nat_ip = nat_ip.trim().to_string();
    let dest_port = dest_port.trim().to_string();
    let domain = domain.map(normalize_domain).filter(|value| !value.is_empty());

    if protocol.is_empty() {
        return Err(ApiError::bad_request("协议不能为空"));
    }
    if dest_ip.is_empty() {
        return Err(ApiError::bad_request("互联网地址不能为空"));
    }
    if nat_ip.is_empty() {
        return Err(ApiError::bad_request("NAT地址不能为空"));
    }
    if dest_port.is_empty() {
        return Err(ApiError::bad_request("目的端口不能为空"));
    }

    Ok((application, protocol, dest_ip, nat_ip, dest_port, domain))
}

async fn ensure_endpoint_unique(
    conn: &DatabaseConnection,
    current_endpoint_id: Option<i32>,
    business_application_id: i32,
    application_name: &str,
    protocol: &str,
    dest_ip: &str,
    dest_port: &str,
    domain: Option<&str>,
) -> Result<(), ApiError> {
    let existing_endpoint = if let Some(domain) = domain {
        find_application_endpoint_l7_with_conn(conn, protocol, dest_port, domain)
            .await
            .map_err(|e| ApiError::internal(format!("Failed to query application endpoint: {}", e)))?
    } else {
        find_application_endpoint_l4_with_conn(conn, protocol, dest_ip, dest_port)
            .await
            .map_err(|e| ApiError::internal(format!("Failed to query application endpoint: {}", e)))?
    };

    if let Some(endpoint) = existing_endpoint {
        if current_endpoint_id == Some(endpoint.id) {
            return Ok(());
        }

        if endpoint.business_application_id == business_application_id {
            return Err(ApiError::conflict("当前端点已存在，无需重复创建"));
        }

        let bound_name = get_business_application_by_id_with_conn(conn, endpoint.business_application_id)
            .await
            .map_err(|e| ApiError::internal(format!("Failed to query business application: {}", e)))?
            .map(|item| item.name)
            .unwrap_or_else(|| "未知应用".to_string());
        return Err(ApiError::conflict(format!(
            "当前端点已归属业务应用：{}，不能再绑定到 {}",
            bound_name, application_name
        )));
    }

    Ok(())
}

async fn ensure_can_create_application(state: &AppState, auth_user: &AuthUser) -> Result<(), ApiError> {
    let current_user = require_current_user_from_auth(auth_user, state).await?;
    let permissions = effective_permissions(state, &current_user).await;
    if !permissions.can_create_business_application {
        return Err(ApiError::forbidden("Access denied"));
    }
    Ok(())
}

async fn ensure_can_update_application(state: &AppState, auth_user: &AuthUser) -> Result<(), ApiError> {
    let current_user = require_current_user_from_auth(auth_user, state).await?;
    let permissions = effective_permissions(state, &current_user).await;
    if !(permissions.can_create_business_application || permissions.can_supplement_business_application)
    {
        return Err(ApiError::forbidden("Access denied"));
    }
    Ok(())
}

async fn ensure_can_delete_application(state: &AppState, auth_user: &AuthUser) -> Result<(), ApiError> {
    let current_user = require_current_user_from_auth(auth_user, state).await?;
    let permissions = effective_permissions(state, &current_user).await;
    if !permissions.can_delete_business_application {
        return Err(ApiError::forbidden("Access denied"));
    }
    Ok(())
}

fn normalize_domain(value: &str) -> String {
    value
        .trim()
        .trim_matches('.')
        .to_ascii_lowercase()
}

fn can_access_business_application_module(permissions: &Permissions) -> bool {
    permissions.can_view_business_applications
        || permissions.can_create_business_application
        || permissions.can_approve_business_application
        || permissions.can_supplement_business_application
        || permissions.can_delete_business_application
}
