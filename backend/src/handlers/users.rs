use axum::{
    extract::{State, Json, Path},
    http::{StatusCode, HeaderMap},
};
use serde::{Deserialize, Serialize};
use shared::{User, CreateUserRequest, Role, Permissions, PasswordPolicy};
use crate::state::AppState;
use crate::utils::{get_current_user, log_action};
use crate::database::{get_users as db_get_users, insert_user as db_insert_user, delete_user as db_delete_user, update_user as db_update_user};
use crate::middleware::ApiError;
use uuid::Uuid;
use chrono::Utc;

#[derive(Debug, Deserialize, Serialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

// 计算密码强度
fn calculate_password_strength(password: &str) -> (String, u32) {
    let mut score = 0;

    // 长度检查
    if password.len() >= 8 { score += 1; }
    if password.len() >= 12 { score += 1; }

    // 包含小写字母
    if password.chars().any(|c| c.is_ascii_lowercase()) { score += 1; }

    // 包含大写字母
    if password.chars().any(|c| c.is_ascii_uppercase()) { score += 1; }

    // 包含数字
    if password.chars().any(|c| c.is_ascii_digit()) { score += 1; }

    // 包含特殊字符
    if password.chars().any(|c| !c.is_alphanumeric()) { score += 1; }

    let strength = match score {
        0..=2 => "weak",
        3..=4 => "medium",
        _ => "strong",
    };

    (strength.to_string(), score)
}

// 根据策略验证密码
fn validate_password_policy(password: &str, policy: &PasswordPolicy) -> Result<(), String> {
    // 长度检查
    if password.len() < policy.min_length as usize {
        return Err(format!("密码长度至少为 {}", policy.min_length));
    }

    // 大写字母检查
    if policy.require_uppercase && !password.chars().any(|c| c.is_ascii_uppercase()) {
        return Err("密码必须包含至少一个大写字母".to_string());
    }

    // 小写字母检查
    if policy.require_lowercase && !password.chars().any(|c| c.is_ascii_lowercase()) {
        return Err("密码必须包含至少一个小写字母".to_string());
    }

    // 数字检查
    if policy.require_number && !password.chars().any(|c| c.is_ascii_digit()) {
        return Err("密码必须包含至少一个数字".to_string());
    }

    // 特殊字符检查
    if policy.require_special && !password.chars().any(|c| !c.is_alphanumeric()) {
        return Err("密码必须包含至少一个特殊字符".to_string());
    }

    // 强度检查
    let (_, score) = calculate_password_strength(password);
    let strength_score = match policy.min_strength.as_str() {
        "weak" => 0,
        "medium" => 3,
        "strong" => 5,
        _ => 0,
    };

    if score < strength_score {
        return Err(format!("密码强度不足，需要{}强度或更高", policy.min_strength));
    }

    Ok(())
}

pub async fn get_users(State(state): State<AppState>, headers: HeaderMap) -> Result<Json<Vec<User>>, ApiError> {
    let user = get_current_user(&headers, &state.users)
        .ok_or_else(|| ApiError::unauthorized("Unauthorized"))?;

    if user.role != Role::SysAdmin {
        return Err(ApiError::forbidden("Access denied: SysAdmin only"));
    }

    // 使用新的 database API
    match db_get_users().await {
        Ok(db_users) => {
            // Update in-memory cache
            *state.users.write()
                .map_err(|e| ApiError::internal(format!("Failed to write users cache: {}", e)))? = db_users.clone();
            return Ok(Json(db_users));
        }
        Err(e) => {
            eprintln!("Error loading users from database: {}", e);
            // Fallback to memory cache
            let users = state.users.read()
                .map_err(|e| ApiError::internal(format!("Failed to read users cache: {}", e)))?;
            Ok(Json(users.clone()))
        }
    }
}

pub async fn create_user(State(state): State<AppState>, headers: HeaderMap, Json(req): Json<CreateUserRequest>) -> Result<Json<User>, ApiError> {
    let current_user = get_current_user(&headers, &state.users)
        .ok_or_else(|| ApiError::unauthorized("Unauthorized"))?;

    if current_user.role != Role::SysAdmin {
        return Err(ApiError::forbidden("Access denied: SysAdmin only"));
    }

    // Check if username exists in memory
    {
        let users = state.users.read()
            .map_err(|e| ApiError::internal(format!("Failed to read users: {}", e)))?;
        if users.iter().any(|u| u.username == req.username) {
            return Err(ApiError::conflict("Username already exists"));
        }
    }

    // Set default permissions based on role
    let permissions = match &req.role {
        Role::SysAdmin => Some(shared::Permissions::sys_admin()),
        Role::SecAdmin => Some(shared::Permissions::sec_admin()),
        Role::Auditor => Some(shared::Permissions::auditor()),
        Role::Custom(_) => None, // Custom roles need explicit permissions set later
    };

    let new_user = User {
        id: Uuid::new_v4().to_string(),
        username: req.username.clone(),
        password: req.password,
        role: req.role,
        permissions,
        created_at: Utc::now(),
        password_changed_at: Some(Utc::now()),
        password_strength: Some("weak".to_string()),
        force_password_change: Some(false),
        last_login_at: None,
        email: None,
        phone: None,
        status: Some("active".to_string()),
        failed_login_attempts: Some(0),
        locked_until: None,
    };

    // Add to in-memory storage
    {
        let mut users = state.users.write()
            .map_err(|e| ApiError::internal(format!("Failed to write users: {}", e)))?;
        users.push(new_user.clone());
    }

    // Persist to database
    if let Err(e) = db_insert_user(&new_user).await {
        eprintln!("Error inserting user to database: {}", e);
    }

    log_action(&state.audit_logs, &current_user, "CREATE_USER", &new_user.username, "Created new user");

    Ok(Json(new_user))
}

pub async fn delete_user(State(state): State<AppState>, headers: HeaderMap, Path(id): Path<String>) -> Result<Json<String>, ApiError> {
    let current_user = get_current_user(&headers, &state.users)
        .ok_or_else(|| ApiError::unauthorized("Unauthorized"))?;

    if current_user.role != Role::SysAdmin {
        return Err(ApiError::forbidden("Access denied: SysAdmin only"));
    }

    let removed = {
        let mut users = state.users.write()
            .map_err(|e| ApiError::internal(format!("Failed to write users: {}", e)))?;
        users.iter().position(|u| u.id == id).map(|idx| users.remove(idx))
    };

    if let Some(removed_user) = removed {
        // Persist to database
        if let Err(e) = db_delete_user(&id).await {
            eprintln!("Error deleting user from database: {}", e);
        }

        log_action(&state.audit_logs, &current_user, "DELETE_USER", &removed_user.username, "Deleted user");
        Ok(Json("Deleted".to_string()))
    } else {
        Err(ApiError::not_found(format!("User with ID '{}' not found", id)))
    }
}

pub async fn update_user_permissions(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(permissions): Json<Permissions>,
) -> Result<Json<User>, (StatusCode, String)> {
    println!("📝 收到权限更新请求 - 用户ID: {}", id);
    println!("📝 通用模块: can_access_general={}", permissions.can_access_general);
    println!("📝 任务中心: can_view_tasks={}, can_create_task={}, can_delete_task={}, can_update_task={}",
        permissions.can_view_tasks, permissions.can_create_task, permissions.can_delete_task, permissions.can_update_task);
    println!("📝 高级扫描: can_view_advanced_scan={}, can_create_scan={}, can_delete_scan={}, can_export_scan={}",
        permissions.can_view_advanced_scan, permissions.can_create_scan, permissions.can_delete_scan, permissions.can_export_scan);
    println!("📝 资产风险: can_access_assets_risks={}", permissions.can_access_assets_risks);
    println!("📝 云资产: can_view_cloud_assets={}, can_create_cloud_asset={}, can_update_cloud_asset={}, can_delete_cloud_asset={}",
        permissions.can_view_cloud_assets, permissions.can_create_cloud_asset, permissions.can_update_cloud_asset, permissions.can_delete_cloud_asset);

    let current_user = get_current_user(&headers, &state.users).ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;

    println!("📝 当前用户: {}", current_user.username);

    // Check if current user has permission to manage permissions
    let current_perms = current_user.permissions.as_ref().ok_or((StatusCode::FORBIDDEN, "No permissions set".to_string()))?;
    println!("📝 当前用户权限 - can_manage_permissions: {}", current_perms.can_manage_permissions);

    if !current_perms.can_manage_permissions {
        println!("❌ 权限不足: 用户没有管理权限的权限");
        return Err((StatusCode::FORBIDDEN, "Permission denied".to_string()));
    }

    // Find the user and clone necessary data before async operations
    let (user_id, username) = {
        let mut users = state.users.write().unwrap();
        if let Some(idx) = users.iter().position(|u| u.id == id) {
            let user = &mut users[idx];
            println!("✅ 找到用户: {}, 更新权限", user.username);
            user.permissions = Some(permissions.clone());
            (user.id.clone(), user.username.clone())
        } else {
            println!("❌ 用户未找到: {}", id);
            return Err((StatusCode::NOT_FOUND, "User not found".to_string()));
        }
    };

    // Persist to database (after releasing the lock)
    let user_for_db = {
        let users = state.users.read().unwrap();
        users.iter().find(|u| u.id == user_id).cloned()
    };

    if let Some(user) = user_for_db {
        if let Err(e) = db_update_user(&user.id, &user).await {
            eprintln!("Error updating user permissions in database: {}", e);
        }

        log_action(&state.audit_logs, &current_user, "UPDATE_PERMISSIONS", &username, "Updated user permissions");
        println!("✅ 权限更新成功");

        // Get updated user for response
        let users = state.users.read().unwrap();
        if let Some(user) = users.iter().find(|u| u.id == user_id) {
            Ok(Json(user.clone()))
        } else {
            Err((StatusCode::NOT_FOUND, "User not found".to_string()))
        }
    } else {
        Err((StatusCode::NOT_FOUND, "User not found".to_string()))
    }
}

pub async fn change_password(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<Json<String>, (StatusCode, String)> {
    let current_user = get_current_user(&headers, &state.users)
        .ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;

    // 获取密码策略
    let policy = state.password_policy.read().unwrap().clone();

    // 验证新密码是否符合策略
    if let Err(e) = validate_password_policy(&req.new_password, &policy) {
        return Err((StatusCode::BAD_REQUEST, e));
    }

    // 计算密码强度
    let (strength, _) = calculate_password_strength(&req.new_password);

    // Find user and validate current password
    let (user_id, username) = {
        let users = state.users.read().unwrap();
        if let Some(user) = users.iter().find(|u| u.id == current_user.id) {
            // 验证当前密码
            if user.password != req.current_password {
                return Err((StatusCode::BAD_REQUEST, "Current password is incorrect".to_string()));
            }
            (user.id.clone(), user.username.clone())
        } else {
            return Err((StatusCode::NOT_FOUND, "User not found".to_string()));
        }
    };

    // Update password (release lock before async operation)
    let new_password = req.new_password.clone();
    let password_changed_at = Utc::now();

    {
        let mut users = state.users.write().unwrap();
        if let Some(user) = users.iter_mut().find(|u| u.id == user_id) {
            user.password = new_password.clone();
            user.password_changed_at = Some(password_changed_at);
            user.password_strength = Some(strength.clone());
        }
    }

    // Persist to database (after releasing the lock)
    let user_for_db = {
        let users = state.users.read().unwrap();
        users.iter().find(|u| u.id == user_id).cloned()
    };

    if let Some(user) = user_for_db {
        if let Err(e) = db_update_user(&user.id, &user).await {
            eprintln!("Error updating user password in database: {}", e);
        }
    }

    // 记录密码历史
    {
        let mut history = state.password_history.write().unwrap();
        history.push((user_id.clone(), new_password, Utc::now()));
    }

    log_action(&state.audit_logs, &current_user, "CHANGE_PASSWORD", &username,
               &format!("Password changed, strength: {}", strength));

    Ok(Json("Password changed successfully".to_string()))
}

// 获取密码策略
pub async fn get_password_policy(State(state): State<AppState>, headers: HeaderMap) -> Result<Json<PasswordPolicy>, (StatusCode, String)> {
    let user = get_current_user(&headers, &state.users)
        .ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;

    // 只有 admin 可以查看密码策略
    if user.role != Role::SysAdmin {
        return Err((StatusCode::FORBIDDEN, "Access denied".to_string()));
    }

    let policy = state.password_policy.read().unwrap();
    Ok(Json(policy.clone()))
}

// 更新密码策略
pub async fn update_password_policy(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(policy): Json<PasswordPolicy>,
) -> Result<Json<PasswordPolicy>, (StatusCode, String)> {
    let current_user = get_current_user(&headers, &state.users)
        .ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;

    // 只有 admin 可以修改密码策略
    if current_user.role != Role::SysAdmin {
        return Err((StatusCode::FORBIDDEN, "Access denied".to_string()));
    }

    let mut policy_state = state.password_policy.write().unwrap();
    *policy_state = policy.clone();

    log_action(&state.audit_logs, &current_user, "UPDATE_PASSWORD_POLICY", "system",
               &format!("Updated password policy: min_length={}, require_uppercase={}, require_number={}",
                       policy.min_length, policy.require_uppercase, policy.require_number));

    Ok(Json(policy))
}

// 获取当前用户信息
pub async fn get_current_user_info(State(state): State<AppState>, headers: HeaderMap) -> Result<Json<User>, (StatusCode, String)> {
    let user = get_current_user(&headers, &state.users).ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;
    Ok(Json(user))
}
