use crate::database::{
    db_custom_role_to_shared, delete_custom_role, get_custom_roles, get_users,
    insert_custom_role_wrapper, update_custom_role, update_user as db_update_user,
};
use crate::middleware::ApiError;
use crate::state::AppState;
use crate::utils::{effective_permissions, get_current_user, log_action, sync_cached_user};
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use shared::{CreateRoleRequest, CustomRole, Permissions, UpdateRoleRequest};

/// Get all roles (including system predefined roles and custom roles)
#[utoipa::path(
    get,
    path = "/api/roles",
    responses(
        (status = 200, description = "List of all roles", body = Vec<serde_json::Value>),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "roles"
)]
pub async fn get_roles(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<serde_json::Value>>, ApiError> {
    let current_user = get_current_user(&headers, &state.users)
        .ok_or_else(|| ApiError::unauthorized("Unauthorized"))?;

    if !effective_permissions(&state, &current_user)
        .await
        .can_manage_permissions
    {
        return Err(ApiError::forbidden("Access denied"));
    }

    // Try to load custom roles from database first
    let custom_roles = match get_custom_roles().await {
        Ok(db_roles) => {
            let roles: Vec<CustomRole> =
                db_roles.into_iter().map(db_custom_role_to_shared).collect();
            // Update in-memory cache
            *state.custom_roles.write().map_err(|e| {
                ApiError::internal(format!("Failed to write custom roles cache: {}", e))
            })? = roles.clone();
            roles
        }
        Err(e) => {
            eprintln!("Error loading custom roles from database: {}", e);
            // Fallback to memory cache
            state
                .custom_roles
                .read()
                .map_err(|e| {
                    ApiError::internal(format!("Failed to read custom roles cache: {}", e))
                })?
                .clone()
        }
    };

    // 构建角色列表,包括系统角色和自定义角色
    let mut roles = vec![
        serde_json::json!({
            "id": "sys_admin",
            "name": "系统管理员",
            "description": "系统内置角色,拥有所有权限",
            "is_system": true,
            "role": "SysAdmin",
            "permissions": Permissions::sys_admin()
        }),
        serde_json::json!({
            "id": "sec_admin",
            "name": "安全管理员",
            "description": "系统内置角色,负责资产、扫描和风险管理",
            "is_system": true,
            "role": "SecAdmin",
            "permissions": Permissions::sec_admin()
        }),
        serde_json::json!({
            "id": "auditor",
            "name": "审计员",
            "description": "系统内置角色,只能查看日志",
            "is_system": true,
            "role": "Auditor",
            "permissions": Permissions::auditor()
        }),
    ];

    // 添加自定义角色
    for role in custom_roles.iter() {
        roles.push(serde_json::json!({
            "id": role.id.map(|id| id.to_string()),
            "name": role.name,
            "description": role.description,
            "is_system": false,
            "permissions": role.permissions,
            "created_at": role.created_at,
            "updated_at": role.updated_at
        }));
    }

    Ok(Json(roles))
}

/// Get a single role by ID
#[utoipa::path(
    get,
    path = "/api/roles/{id}",
    params(
        ("id" = String, Path, description = "Role ID (sys_admin, sec_admin, auditor, or custom role ID)")
    ),
    responses(
        (status = 200, description = "Role details", body = serde_json::Value),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Role not found")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "roles"
)]
pub async fn get_role(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    let current_user = get_current_user(&headers, &state.users)
        .ok_or_else(|| ApiError::unauthorized("Unauthorized"))?;

    if !effective_permissions(&state, &current_user)
        .await
        .can_manage_permissions
    {
        return Err(ApiError::forbidden("Access denied"));
    }

    // 检查系统角色
    match id.as_str() {
        "sys_admin" => {
            return Ok(Json(serde_json::json!({
                "id": "sys_admin",
                "name": "系统管理员",
                "description": "系统内置角色,拥有所有权限",
                "is_system": true,
                "role": "SysAdmin",
                "permissions": Permissions::sys_admin()
            })));
        }
        "sec_admin" => {
            return Ok(Json(serde_json::json!({
                "id": "sec_admin",
                "name": "安全管理员",
                "description": "系统内置角色,负责资产、扫描和风险管理",
                "is_system": true,
                "role": "SecAdmin",
                "permissions": Permissions::sec_admin()
            })));
        }
        "auditor" => {
            return Ok(Json(serde_json::json!({
                "id": "auditor",
                "name": "审计员",
                "description": "系统内置角色,只能查看日志",
                "is_system": true,
                "role": "Auditor",
                "permissions": Permissions::auditor()
            })));
        }
        _ => {}
    }

    // Try to find custom role in database
    if let Ok(role_id) = id.parse::<i32>() {
        match get_custom_roles().await {
            Ok(db_roles) => {
                if let Some(db_role) = db_roles.iter().find(|r| r.id == role_id) {
                    let role = db_custom_role_to_shared(db_role.clone());
                    return Ok(Json(serde_json::json!({
                        "id": role.id.map(|id| id.to_string()),
                        "name": role.name,
                        "description": role.description,
                        "is_system": false,
                        "permissions": role.permissions,
                        "created_at": role.created_at,
                        "updated_at": role.updated_at
                    })));
                }
            }
            Err(e) => {
                eprintln!("Error loading custom roles from database: {}", e);
                // Fallback to memory cache
                let custom_roles = state.custom_roles.read().map_err(|e| {
                    ApiError::internal(format!("Failed to read custom roles: {}", e))
                })?;
                if let Some(role) = custom_roles.iter().find(|r| r.id == Some(role_id)) {
                    return Ok(Json(serde_json::json!({
                        "id": role.id.map(|id| id.to_string()),
                        "name": role.name,
                        "description": role.description,
                        "is_system": false,
                        "permissions": role.permissions,
                        "created_at": role.created_at,
                        "updated_at": role.updated_at
                    })));
                }
            }
        }
    }

    Err(ApiError::not_found(format!(
        "Role with ID '{}' not found",
        id
    )))
}

/// Create a custom role
#[utoipa::path(
    post,
    path = "/api/roles",
    request_body = CreateRoleRequest,
    responses(
        (status = 200, description = "Role created", body = serde_json::Value),
        (status = 401, description = "Unauthorized"),
        (status = 400, description = "Role name already exists")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "roles"
)]
pub async fn create_role(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateRoleRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Get current user for audit logging
    let current_user = get_current_user(&headers, &state.users)
        .ok_or_else(|| ApiError::unauthorized("Unauthorized"))?;

    if !effective_permissions(&state, &current_user)
        .await
        .can_manage_permissions
    {
        return Err(ApiError::forbidden("Access denied"));
    }

    // 检查角色名称是否已存在
    {
        let custom_roles = state
            .custom_roles
            .read()
            .map_err(|e| ApiError::internal(format!("Failed to read custom roles: {}", e)))?;
        if custom_roles.iter().any(|r| r.name == req.name) {
            return Ok(Json(serde_json::json!({
                "error": "角色名称已存在"
            })));
        }
    }

    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let new_role = CustomRole {
        id: None, // Database will assign ID
        name: req.name.clone(),
        description: req.description,
        permissions: req.permissions,
        created_at: Some(now.clone()),
        updated_at: Some(now),
    };

    // Persist to database
    let new_id = match insert_custom_role_wrapper(&new_role).await {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Error inserting custom role to database: {}", e);
            // Fallback to in-memory with generated ID
            let mut custom_roles = state
                .custom_roles
                .write()
                .map_err(|e| ApiError::internal(format!("Failed to write custom roles: {}", e)))?;
            let id = custom_roles.len() as i32 + 1;
            let mut role_with_id = new_role.clone();
            role_with_id.id = Some(id);
            custom_roles.push(role_with_id);

            // Audit log
            log_action(
                &state.audit_logs,
                &current_user,
                "ROLE_CREATED",
                &req.name,
                &format!("Created custom role with ID {}", id),
            );

            return Ok(Json(serde_json::json!({
                "id": id,
                "name": req.name,
                "message": "角色创建成功"
            })));
        }
    };

    // Update in-memory cache
    {
        let mut custom_roles = state
            .custom_roles
            .write()
            .map_err(|e| ApiError::internal(format!("Failed to write custom roles: {}", e)))?;
        let mut role_with_id = new_role.clone();
        role_with_id.id = Some(new_id);
        custom_roles.push(role_with_id);
    }

    // Audit log
    log_action(
        &state.audit_logs,
        &current_user,
        "ROLE_CREATED",
        &req.name,
        &format!("Created custom role with ID {}", new_id),
    );

    Ok(Json(serde_json::json!({
        "id": new_id,
        "name": req.name,
        "message": "角色创建成功"
    })))
}

/// Update a custom role
#[utoipa::path(
    put,
    path = "/api/roles/{id}",
    params(
        ("id" = String, Path, description = "Role ID")
    ),
    request_body = UpdateRoleRequest,
    responses(
        (status = 200, description = "Role updated", body = serde_json::Value),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Role not found")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "roles"
)]
pub async fn update_role(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(req): Json<UpdateRoleRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Get current user for audit logging
    let current_user = get_current_user(&headers, &state.users)
        .ok_or_else(|| ApiError::unauthorized("Unauthorized"))?;

    if !effective_permissions(&state, &current_user)
        .await
        .can_manage_permissions
    {
        return Err(ApiError::forbidden("Access denied"));
    }

    // 不允许修改系统角色
    if matches!(id.as_str(), "sys_admin" | "sec_admin" | "auditor") {
        return Ok(Json(serde_json::json!({
            "error": "不能修改系统内置角色"
        })));
    }

    let role_id = id
        .parse::<i32>()
        .map_err(|_| ApiError::bad_request("Invalid role ID format"))?;
    let target_id = role_id;

    // Find current role data
    let (found, current_role) = {
        let custom_roles = state
            .custom_roles
            .read()
            .map_err(|e| ApiError::internal(format!("Failed to read custom roles: {}", e)))?;
        if let Some(role) = custom_roles.iter().find(|r| r.id == Some(target_id)) {
            (true, role.clone())
        } else {
            (
                false,
                CustomRole {
                    id: Some(target_id),
                    name: String::new(),
                    description: None,
                    permissions: Default::default(),
                    created_at: None,
                    updated_at: None,
                },
            )
        }
    };

    if !found {
        return Err(ApiError::not_found(format!(
            "Role with ID '{}' not found",
            id
        )));
    }

    // Build updated role
    let old_role_name = current_role.name.clone();
    let mut updated_role = current_role.clone();
    if let Some(name) = req.name {
        updated_role.name = name;
    }
    if let Some(description) = req.description {
        updated_role.description = Some(description);
    }
    if let Some(permissions) = req.permissions {
        updated_role.permissions = permissions;
    }
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    updated_role.updated_at = Some(now);

    // Persist to database (after releasing lock)
    if let Err(e) = update_custom_role(target_id, &updated_role).await {
        eprintln!("Error updating custom role in database: {}", e);
    }

    // Update in-memory cache
    {
        let mut custom_roles = state
            .custom_roles
            .write()
            .map_err(|e| ApiError::internal(format!("Failed to write custom roles: {}", e)))?;
        if let Some(role) = custom_roles.iter_mut().find(|r| r.id == Some(target_id)) {
            *role = updated_role.clone();
        }
    }

    sync_users_for_custom_role(&state, &old_role_name, &updated_role).await?;

    // Audit log
    log_action(
        &state.audit_logs,
        &current_user,
        "ROLE_UPDATED",
        id.as_str(),
        &format!("Updated custom role with ID {}", target_id),
    );

    Ok(Json(serde_json::json!({
        "message": "角色更新成功"
    })))
}

/// Delete a custom role
#[utoipa::path(
    delete,
    path = "/api/roles/{id}",
    params(
        ("id" = String, Path, description = "Role ID")
    ),
    responses(
        (status = 200, description = "Role deleted", body = serde_json::Value),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Role not found")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "roles"
)]
pub async fn delete_role(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Get current user for audit logging
    let current_user = get_current_user(&headers, &state.users)
        .ok_or_else(|| ApiError::unauthorized("Unauthorized"))?;

    if !effective_permissions(&state, &current_user)
        .await
        .can_manage_permissions
    {
        return Err(ApiError::forbidden("Access denied"));
    }

    // 不允许删除系统角色
    if matches!(id.as_str(), "sys_admin" | "sec_admin" | "auditor") {
        return Ok(Json(serde_json::json!({
            "error": "不能删除系统内置角色"
        })));
    }

    let role_id = id
        .parse::<i32>()
        .map_err(|_| ApiError::bad_request("Invalid role ID format"))?;
    let target_id = role_id;

    let role_name = {
        let custom_roles = state
            .custom_roles
            .read()
            .map_err(|e| ApiError::internal(format!("Failed to read custom roles: {}", e)))?;
        let Some(role) = custom_roles.iter().find(|r| r.id == Some(target_id)) else {
            return Err(ApiError::not_found(format!(
                "Role with ID '{}' not found",
                id
            )));
        };
        role.name.clone()
    };

    let assigned_users = load_users_assigned_to_custom_role(&state, &role_name).await?;
    if !assigned_users.is_empty() {
        return Err(ApiError::bad_request(format!(
            "角色“{}”仍绑定了 {} 个用户，不能删除",
            role_name,
            assigned_users.len()
        )));
    }

    // Remove from in-memory storage
    {
        let mut custom_roles = state
            .custom_roles
            .write()
            .map_err(|e| ApiError::internal(format!("Failed to write custom roles: {}", e)))?;
        custom_roles.retain(|r| r.id != Some(target_id));
    }

    // Persist to database (after releasing lock)
    if let Err(e) = delete_custom_role(target_id).await {
        eprintln!("Error deleting custom role from database: {}", e);
    }

    // Audit log
    log_action(
        &state.audit_logs,
        &current_user,
        "ROLE_DELETED",
        id.as_str(),
        &format!("Deleted custom role with ID {}", target_id),
    );

    Ok(Json(serde_json::json!({
        "message": "角色删除成功"
    })))
}

async fn sync_users_for_custom_role(
    state: &AppState,
    old_role_name: &str,
    role: &CustomRole,
) -> Result<(), ApiError> {
    let all_users = load_users_for_role_sync(state).await?;

    for mut user in all_users
        .into_iter()
        .filter(|user| matches!(&user.role, shared::Role::Custom(name) if name == old_role_name))
    {
        user.role = shared::Role::Custom(role.name.clone());
        user.permissions = Some(role.permissions.clone());

        if crate::database::get_db().is_some() {
            db_update_user(&user.id, &user)
                .await
                .map_err(|e| ApiError::internal(format!("Failed to sync role users: {}", e)))?;
        }

        sync_cached_user(&state.users, &user);
    }

    Ok(())
}

async fn load_users_assigned_to_custom_role(
    state: &AppState,
    role_name: &str,
) -> Result<Vec<shared::User>, ApiError> {
    load_users_for_role_sync_by_name(role_name, Some(state)).await
}

async fn load_users_for_role_sync(state: &AppState) -> Result<Vec<shared::User>, ApiError> {
    if crate::database::get_db().is_some() {
        return get_users()
            .await
            .map_err(|e| ApiError::internal(format!("Failed to load users: {}", e)));
    }

    state
        .users
        .read()
        .map(|users| users.clone())
        .map_err(|e| ApiError::internal(format!("Failed to read users cache: {}", e)))
}

async fn load_users_for_role_sync_by_name(
    role_name: &str,
    state: Option<&AppState>,
) -> Result<Vec<shared::User>, ApiError> {
    let users = if let Some(state) = state {
        load_users_for_role_sync(state).await?
    } else {
        get_users()
            .await
            .map_err(|e| ApiError::internal(format!("Failed to load users: {}", e)))?
    };

    Ok(users
        .into_iter()
        .filter(|user| matches!(&user.role, shared::Role::Custom(name) if name == role_name))
        .collect())
}
