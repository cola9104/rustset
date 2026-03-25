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
use shared::{CreateRoleRequest, CustomRole, DataScope, Permissions, UpdateRoleRequest};

fn normalize_role_name(name: &str) -> String {
    name.trim().to_string()
}

fn is_reserved_role_name(name: &str) -> bool {
    matches!(
        name,
        "系统管理员" | "安全管理员" | "审计员" | "SysAdmin" | "SecAdmin" | "Auditor"
    )
}

fn enable_if_any(target: &mut bool, sources: &[bool]) {
    if sources.iter().any(|enabled| *enabled) {
        *target = true;
    }
}

fn normalize_role_permissions(permissions: &mut Permissions) {
    enable_if_any(
        &mut permissions.can_view_tasks,
        &[
            permissions.can_create_task,
            permissions.can_update_task,
            permissions.can_delete_task,
        ],
    );
    enable_if_any(
        &mut permissions.can_view_advanced_scan,
        &[
            permissions.can_create_scan,
            permissions.can_delete_scan,
            permissions.can_export_scan,
        ],
    );
    enable_if_any(
        &mut permissions.can_view_cloud_assets,
        &[
            permissions.can_create_cloud_asset,
            permissions.can_update_cloud_asset,
            permissions.can_delete_cloud_asset,
        ],
    );
    enable_if_any(
        &mut permissions.can_view_risks,
        &[permissions.can_resolve_risk, permissions.can_delete_risk],
    );
    enable_if_any(
        &mut permissions.can_view_business_applications,
        &[
            permissions.can_create_business_application,
            permissions.can_approve_business_application,
            permissions.can_supplement_business_application,
            permissions.can_delete_business_application,
        ],
    );
    enable_if_any(
        &mut permissions.can_view_operations_management,
        &[permissions.can_manage_operations],
    );
    enable_if_any(
        &mut permissions.can_view_automation_orchestration,
        &[
            permissions.can_execute_orchestration,
            permissions.can_manage_orchestration,
        ],
    );
    enable_if_any(
        &mut permissions.can_view_cloud_providers,
        &[permissions.can_manage_cloud_providers],
    );
    enable_if_any(
        &mut permissions.can_view_users,
        &[
            permissions.can_create_user,
            permissions.can_update_user,
            permissions.can_delete_user,
            permissions.can_manage_permissions,
        ],
    );
    enable_if_any(
        &mut permissions.can_view_password_policy,
        &[permissions.can_manage_password_policy],
    );
    enable_if_any(
        &mut permissions.can_view_resource_tickets,
        &[
            permissions.can_create_resource_tickets,
            permissions.can_approve_resource_tickets,
            permissions.can_provision_resource_tickets,
            permissions.can_deliver_resource_tickets,
            permissions.can_delete_resource_tickets,
        ],
    );
    enable_if_any(
        &mut permissions.can_access_general,
        &[
            permissions.can_view_dashboard,
            permissions.can_view_tasks,
            permissions.can_create_task,
            permissions.can_update_task,
            permissions.can_delete_task,
            permissions.can_view_advanced_scan,
            permissions.can_create_scan,
            permissions.can_delete_scan,
            permissions.can_export_scan,
        ],
    );
    enable_if_any(
        &mut permissions.can_access_assets_risks,
        &[
            permissions.can_view_cloud_assets,
            permissions.can_create_cloud_asset,
            permissions.can_update_cloud_asset,
            permissions.can_delete_cloud_asset,
            permissions.can_view_risks,
            permissions.can_resolve_risk,
            permissions.can_delete_risk,
            permissions.can_view_business_process,
            permissions.can_view_business_applications,
            permissions.can_create_business_application,
            permissions.can_approve_business_application,
            permissions.can_supplement_business_application,
            permissions.can_delete_business_application,
            permissions.can_view_operations_management,
            permissions.can_manage_operations,
            permissions.can_view_automation_orchestration,
            permissions.can_execute_orchestration,
            permissions.can_manage_orchestration,
        ],
    );
    enable_if_any(
        &mut permissions.can_access_cloud,
        &[
            permissions.can_view_cloud_providers,
            permissions.can_manage_cloud_providers,
        ],
    );
    enable_if_any(
        &mut permissions.can_access_user_management,
        &[
            permissions.can_view_users,
            permissions.can_create_user,
            permissions.can_update_user,
            permissions.can_delete_user,
            permissions.can_manage_permissions,
            permissions.can_view_password_policy,
            permissions.can_manage_password_policy,
        ],
    );
    enable_if_any(
        &mut permissions.can_access_audit,
        &[permissions.can_view_audit_logs],
    );
}

fn validate_role_name(name: &str) -> Result<(), ApiError> {
    if name.trim().is_empty() {
        return Err(ApiError::bad_request("角色名称不能为空"));
    }

    if is_reserved_role_name(name) {
        return Err(ApiError::bad_request("角色名称不能与系统内置角色重名"));
    }

    Ok(())
}

fn data_scope_label(scope: DataScope) -> &'static str {
    match scope {
        DataScope::SelfOnly => "仅自己",
        DataScope::Department => "本部门",
        DataScope::Organization => "本公司/组织",
        DataScope::All => "全部",
    }
}

fn enabled_permission_count(permissions: &Permissions) -> usize {
    [
        permissions.can_access_general,
        permissions.can_view_dashboard,
        permissions.can_view_tasks,
        permissions.can_create_task,
        permissions.can_delete_task,
        permissions.can_update_task,
        permissions.can_view_advanced_scan,
        permissions.can_create_scan,
        permissions.can_delete_scan,
        permissions.can_export_scan,
        permissions.can_access_assets_risks,
        permissions.can_view_cloud_assets,
        permissions.can_create_cloud_asset,
        permissions.can_update_cloud_asset,
        permissions.can_delete_cloud_asset,
        permissions.can_view_risks,
        permissions.can_resolve_risk,
        permissions.can_delete_risk,
        permissions.can_view_business_process,
        permissions.can_view_business_applications,
        permissions.can_create_business_application,
        permissions.can_approve_business_application,
        permissions.can_supplement_business_application,
        permissions.can_delete_business_application,
        permissions.can_view_operations_management,
        permissions.can_manage_operations,
        permissions.can_view_automation_orchestration,
        permissions.can_execute_orchestration,
        permissions.can_manage_orchestration,
        permissions.can_access_cloud,
        permissions.can_view_cloud_providers,
        permissions.can_manage_cloud_providers,
        permissions.can_access_user_management,
        permissions.can_view_users,
        permissions.can_create_user,
        permissions.can_update_user,
        permissions.can_delete_user,
        permissions.can_manage_permissions,
        permissions.can_view_password_policy,
        permissions.can_manage_password_policy,
        permissions.can_access_audit,
        permissions.can_view_audit_logs,
        permissions.can_view_resource_tickets,
        permissions.can_create_resource_tickets,
        permissions.can_approve_resource_tickets,
        permissions.can_provision_resource_tickets,
        permissions.can_deliver_resource_tickets,
        permissions.can_delete_resource_tickets,
    ]
    .into_iter()
    .filter(|enabled| *enabled)
    .count()
}

fn role_workflow_summary(permissions: &Permissions) -> String {
    let mut items = Vec::new();

    if permissions.can_create_resource_tickets {
        items.push("提交");
    }
    if permissions.can_approve_resource_tickets {
        items.push("审批");
    }
    if permissions.can_provision_resource_tickets {
        items.push("配置");
    }
    if permissions.can_deliver_resource_tickets {
        items.push("交付");
    }
    if permissions.can_delete_resource_tickets {
        items.push("删除");
    }

    if items.is_empty() {
        "无工单流程权限".to_string()
    } else {
        format!("工单能力: {}", items.join("/"))
    }
}

fn role_audit_summary(role: &CustomRole, affected_users: usize) -> String {
    format!(
        "角色名: {}, 数据范围: {}, 布尔权限数: {}, {}, 影响用户: {}",
        role.name,
        data_scope_label(role.permissions.resource_ticket_scope),
        enabled_permission_count(&role.permissions),
        role_workflow_summary(&role.permissions),
        affected_users
    )
}

fn role_name_exists(
    state: &AppState,
    name: &str,
    exclude_id: Option<i32>,
) -> Result<bool, ApiError> {
    let custom_roles = state
        .custom_roles
        .read()
        .map_err(|e| ApiError::internal(format!("Failed to read custom roles: {}", e)))?;

    Ok(custom_roles.iter().any(|role| {
        role.id != exclude_id && normalize_role_name(&role.name).eq_ignore_ascii_case(name)
    }))
}

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

    let role_name = normalize_role_name(&req.name);
    validate_role_name(&role_name)?;
    if role_name_exists(&state, &role_name, None)? {
        return Err(ApiError::conflict("角色名称已存在"));
    }

    let mut permissions = req.permissions;
    normalize_role_permissions(&mut permissions);

    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let new_role = CustomRole {
        id: None, // Database will assign ID
        name: role_name.clone(),
        description: req.description,
        permissions,
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
                &role_name,
                &format!(
                    "创建自定义角色(ID: {})，{}",
                    id,
                    role_audit_summary(&new_role, 0)
                ),
            );

            return Ok(Json(serde_json::json!({
                "id": id,
                "name": role_name,
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
        &role_name,
        &format!(
            "创建自定义角色(ID: {})，{}",
            new_id,
            role_audit_summary(&new_role, 0)
        ),
    );

    Ok(Json(serde_json::json!({
        "id": new_id,
        "name": role_name,
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
        return Err(ApiError::bad_request("不能修改系统内置角色"));
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
        let normalized_name = normalize_role_name(&name);
        validate_role_name(&normalized_name)?;
        if role_name_exists(&state, &normalized_name, Some(target_id))? {
            return Err(ApiError::conflict("角色名称已存在"));
        }
        updated_role.name = normalized_name;
    }
    if let Some(description) = req.description {
        updated_role.description = Some(description);
    }
    if let Some(permissions) = req.permissions {
        let mut permissions = permissions;
        normalize_role_permissions(&mut permissions);
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

    let affected_users = load_users_assigned_to_custom_role(&state, &old_role_name)
        .await?
        .len();
    sync_users_for_custom_role(&state, &old_role_name, &updated_role).await?;

    // Audit log
    log_action(
        &state.audit_logs,
        &current_user,
        "ROLE_UPDATED",
        id.as_str(),
        &format!(
            "更新自定义角色(ID: {})，旧名称: {}，{}",
            target_id,
            old_role_name,
            role_audit_summary(&updated_role, affected_users)
        ),
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
        return Err(ApiError::bad_request("不能删除系统内置角色"));
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
        &format!(
            "删除自定义角色(ID: {})，角色名: {}，影响用户: 0",
            target_id, role_name
        ),
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

#[cfg(test)]
mod tests {
    use super::*;
    use shared::DataScope;

    #[test]
    fn normalize_role_permissions_enables_parent_permissions() {
        let mut permissions = Permissions {
            can_manage_operations: true,
            can_approve_resource_tickets: true,
            can_manage_password_policy: true,
            ..Default::default()
        };

        normalize_role_permissions(&mut permissions);

        assert!(permissions.can_view_operations_management);
        assert!(permissions.can_access_assets_risks);
        assert!(permissions.can_view_resource_tickets);
        assert!(permissions.can_view_password_policy);
        assert!(permissions.can_access_user_management);
    }

    #[test]
    fn validate_role_name_rejects_reserved_and_blank_names() {
        assert!(validate_role_name("").is_err());
        assert!(validate_role_name("  ").is_err());
        assert!(validate_role_name("系统管理员").is_err());
        assert!(validate_role_name("SecAdmin").is_err());
        assert!(validate_role_name("运维管理员").is_ok());
    }

    #[test]
    fn normalize_role_permissions_keeps_scope_intact() {
        let mut permissions = Permissions {
            can_deliver_resource_tickets: true,
            resource_ticket_scope: DataScope::Organization,
            ..Default::default()
        };

        normalize_role_permissions(&mut permissions);

        assert!(permissions.can_view_resource_tickets);
        assert_eq!(permissions.resource_ticket_scope, DataScope::Organization);
    }
}
