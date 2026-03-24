use axum::{
    extract::{Path, State},
    response::{IntoResponse, Json},
};
use chrono::Utc;
use serde::Deserialize;
use serde_json::json;

use crate::database::{
    delete_department, get_all_departments, get_db, get_department_by_id, get_organization_by_id,
    get_users, insert_department, update_department,
};
use crate::middleware::{ApiError, AuthUser};
use crate::state::AppState;
use crate::utils::log_action_auth;
use shared::{Department, Role};

#[derive(Debug, Deserialize)]
pub struct CreateDepartmentRequest {
    pub organization_id: i32,
    pub name: String,
    pub code: String,
    pub parent_id: Option<i32>,
    pub level: Option<u32>,
    pub status: Option<String>,
    pub remarks: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDepartmentRequest {
    pub organization_id: Option<i32>,
    pub name: Option<String>,
    pub code: Option<String>,
    pub parent_id: Option<i32>,
    pub level: Option<u32>,
    pub status: Option<String>,
    pub remarks: Option<String>,
}

pub async fn get_departments(
    State(_state): State<AppState>,
    _user: AuthUser,
) -> Result<impl IntoResponse, ApiError> {
    let conn = get_db().ok_or_else(|| ApiError::internal("Database not available"))?;
    let items = get_all_departments(&conn)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to load departments: {}", e)))?;
    Ok(Json(items).into_response())
}

pub async fn create_department(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<CreateDepartmentRequest>,
) -> Result<impl IntoResponse, ApiError> {
    if user.role != Role::SysAdmin && user.role != Role::SecAdmin {
        return Err(ApiError::forbidden("Access denied"));
    }
    if req.name.trim().is_empty() || req.code.trim().is_empty() {
        return Err(ApiError::bad_request("部门名称和编码不能为空"));
    }

    let conn = get_db().ok_or_else(|| ApiError::internal("Database not available"))?;
    get_organization_by_id(&conn, req.organization_id)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to load organization: {}", e)))?
        .ok_or_else(|| ApiError::bad_request("所属组织不存在"))?;

    let now = Utc::now().to_rfc3339();
    let status = req.status.clone().unwrap_or_else(|| "active".to_string());
    let level = req.level.unwrap_or(1);
    let id = insert_department(
        &conn,
        req.organization_id,
        req.name.trim(),
        req.code.trim(),
        req.parent_id,
        level,
        &status,
        req.remarks.as_deref(),
        &now,
    )
    .await
    .map_err(|e| ApiError::internal(format!("Failed to create department: {}", e)))?;

    let item = Department {
        id,
        organization_id: req.organization_id,
        name: req.name.trim().to_string(),
        code: req.code.trim().to_string(),
        parent_id: req.parent_id,
        level,
        status,
        remarks: req.remarks.clone(),
        created_at: now,
        updated_at: None,
    };

    log_action_auth(
        &state.audit_logs,
        &user,
        "CREATE_DEPARTMENT",
        &item.name,
        &format!("Created department {}", item.name),
    );

    Ok(Json(json!({"message": "部门创建成功", "data": item})).into_response())
}

pub async fn update_department_handler(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i32>,
    Json(req): Json<UpdateDepartmentRequest>,
) -> Result<impl IntoResponse, ApiError> {
    if user.role != Role::SysAdmin && user.role != Role::SecAdmin {
        return Err(ApiError::forbidden("Access denied"));
    }

    let conn = get_db().ok_or_else(|| ApiError::internal("Database not available"))?;
    get_department_by_id(&conn, id)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to load department: {}", e)))?
        .ok_or_else(|| ApiError::not_found("Department not found"))?;

    if let Some(organization_id) = req.organization_id {
        get_organization_by_id(&conn, organization_id)
            .await
            .map_err(|e| ApiError::internal(format!("Failed to load organization: {}", e)))?
            .ok_or_else(|| ApiError::bad_request("所属组织不存在"))?;
    }

    let now = Utc::now().to_rfc3339();
    update_department(
        &conn,
        id,
        req.organization_id,
        req.name.as_deref(),
        req.code.as_deref(),
        req.parent_id,
        req.level,
        req.status.as_deref(),
        req.remarks.as_deref(),
        Some(&now),
    )
    .await
    .map_err(|e| ApiError::internal(format!("Failed to update department: {}", e)))?;

    let updated = get_department_by_id(&conn, id)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to reload department: {}", e)))?
        .ok_or_else(|| ApiError::not_found("Department not found"))?;

    log_action_auth(
        &state.audit_logs,
        &user,
        "UPDATE_DEPARTMENT",
        &updated.name,
        &format!("Updated department {}", updated.name),
    );

    Ok(Json(json!({"message": "部门更新成功", "data": updated})).into_response())
}

pub async fn delete_department_handler(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApiError> {
    if user.role != Role::SysAdmin && user.role != Role::SecAdmin {
        return Err(ApiError::forbidden("Access denied"));
    }

    let conn = get_db().ok_or_else(|| ApiError::internal("Database not available"))?;
    let department = get_department_by_id(&conn, id)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to load department: {}", e)))?
        .ok_or_else(|| ApiError::not_found("Department not found"))?;

    let users = get_users().await.unwrap_or_default();
    if users.iter().any(|item| item.department_id == Some(id)) {
        return Err(ApiError::bad_request("该部门仍有绑定用户，无法删除"));
    }

    let all_departments = get_all_departments(&conn)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to load departments: {}", e)))?;
    if all_departments
        .iter()
        .any(|item| item.parent_id == Some(id))
    {
        return Err(ApiError::bad_request("该部门下仍有子部门，无法删除"));
    }

    delete_department(&conn, id)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to delete department: {}", e)))?;

    log_action_auth(
        &state.audit_logs,
        &user,
        "DELETE_DEPARTMENT",
        &department.name,
        &format!("Deleted department {}", department.name),
    );

    Ok(Json(json!({"message": "部门删除成功"})).into_response())
}
