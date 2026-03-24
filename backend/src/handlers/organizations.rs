use axum::{
    extract::{Path, State},
    response::{IntoResponse, Json},
};
use chrono::Utc;
use serde::Deserialize;
use serde_json::json;

use crate::database::{
    delete_organization, get_all_departments, get_all_organizations, get_db,
    get_organization_by_id, get_users, insert_organization, update_organization,
};
use crate::middleware::{ApiError, AuthUser};
use crate::state::AppState;
use crate::utils::log_action_auth;
use shared::{Organization, Role};

#[derive(Debug, Deserialize)]
pub struct CreateOrganizationRequest {
    pub name: String,
    pub code: String,
    pub status: Option<String>,
    pub remarks: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOrganizationRequest {
    pub name: Option<String>,
    pub code: Option<String>,
    pub status: Option<String>,
    pub remarks: Option<String>,
}

pub async fn get_organizations(
    State(_state): State<AppState>,
    _user: AuthUser,
) -> Result<impl IntoResponse, ApiError> {
    let conn = get_db().ok_or_else(|| ApiError::internal("Database not available"))?;
    let items = get_all_organizations(&conn)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to load organizations: {}", e)))?;
    Ok(Json(items).into_response())
}

pub async fn create_organization(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<CreateOrganizationRequest>,
) -> Result<impl IntoResponse, ApiError> {
    if user.role != Role::SysAdmin && user.role != Role::SecAdmin {
        return Err(ApiError::forbidden("Access denied"));
    }
    if req.name.trim().is_empty() || req.code.trim().is_empty() {
        return Err(ApiError::bad_request("组织名称和编码不能为空"));
    }

    let conn = get_db().ok_or_else(|| ApiError::internal("Database not available"))?;
    let now = Utc::now().to_rfc3339();
    let status = req.status.clone().unwrap_or_else(|| "active".to_string());
    let id = insert_organization(
        &conn,
        req.name.trim(),
        req.code.trim(),
        &status,
        req.remarks.as_deref(),
        &now,
    )
    .await
    .map_err(|e| ApiError::internal(format!("Failed to create organization: {}", e)))?;

    let item = Organization {
        id,
        name: req.name.trim().to_string(),
        code: req.code.trim().to_string(),
        status,
        remarks: req.remarks.clone(),
        created_at: now,
        updated_at: None,
    };

    log_action_auth(
        &state.audit_logs,
        &user,
        "CREATE_ORGANIZATION",
        &item.name,
        &format!("Created organization {}", item.name),
    );

    Ok(Json(json!({"message": "组织创建成功", "data": item})).into_response())
}

pub async fn update_organization_handler(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i32>,
    Json(req): Json<UpdateOrganizationRequest>,
) -> Result<impl IntoResponse, ApiError> {
    if user.role != Role::SysAdmin && user.role != Role::SecAdmin {
        return Err(ApiError::forbidden("Access denied"));
    }

    let conn = get_db().ok_or_else(|| ApiError::internal("Database not available"))?;
    get_organization_by_id(&conn, id)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to load organization: {}", e)))?
        .ok_or_else(|| ApiError::not_found("Organization not found"))?;

    let now = Utc::now().to_rfc3339();
    update_organization(
        &conn,
        id,
        req.name.as_deref(),
        req.code.as_deref(),
        req.status.as_deref(),
        req.remarks.as_deref(),
        Some(&now),
    )
    .await
    .map_err(|e| ApiError::internal(format!("Failed to update organization: {}", e)))?;

    let updated = get_organization_by_id(&conn, id)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to reload organization: {}", e)))?
        .ok_or_else(|| ApiError::not_found("Organization not found"))?;

    log_action_auth(
        &state.audit_logs,
        &user,
        "UPDATE_ORGANIZATION",
        &updated.name,
        &format!("Updated organization {}", updated.name),
    );

    Ok(Json(json!({"message": "组织更新成功", "data": updated})).into_response())
}

pub async fn delete_organization_handler(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApiError> {
    if user.role != Role::SysAdmin && user.role != Role::SecAdmin {
        return Err(ApiError::forbidden("Access denied"));
    }

    let conn = get_db().ok_or_else(|| ApiError::internal("Database not available"))?;
    let organization = get_organization_by_id(&conn, id)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to load organization: {}", e)))?
        .ok_or_else(|| ApiError::not_found("Organization not found"))?;

    let users = get_users().await.unwrap_or_default();
    if users.iter().any(|item| item.organization_id == Some(id)) {
        return Err(ApiError::bad_request("该组织仍有绑定用户，无法删除"));
    }

    let departments = get_all_departments(&conn)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to load departments: {}", e)))?;
    if departments.iter().any(|item| item.organization_id == id) {
        return Err(ApiError::bad_request("该组织下仍有部门，无法删除"));
    }

    delete_organization(&conn, id)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to delete organization: {}", e)))?;

    log_action_auth(
        &state.audit_logs,
        &user,
        "DELETE_ORGANIZATION",
        &organization.name,
        &format!("Deleted organization {}", organization.name),
    );

    Ok(Json(json!({"message": "组织删除成功"})).into_response())
}
