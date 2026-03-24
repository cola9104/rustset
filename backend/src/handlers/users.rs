use crate::database::{
    delete_user as db_delete_user, get_db, get_department_by_id, get_organization_by_id,
    get_user_by_id as db_get_user_by_id, get_user_by_username as db_get_user_by_username,
    get_users as db_get_users, insert_user as db_insert_user, update_user as db_update_user,
};
use crate::middleware::auth_middleware::AuthUser;
use crate::middleware::ApiError;
use crate::password;
use crate::state::AppState;
use crate::utils::{
    get_current_user_from_auth, get_current_user_from_headers, log_action, remove_cached_user,
    sync_cached_user, sync_cached_users,
};
use axum::{
    extract::{Json, Path, State},
    http::{HeaderMap, StatusCode},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use shared::{CreateUserRequest, PasswordPolicy, Permissions, Role, UpdateUserRequest, User};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, utoipa::ToSchema)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

// 计算密码强度
pub fn calculate_password_strength(password: &str) -> (String, u32) {
    let mut score = 0;

    // 长度检查
    if password.len() >= 8 {
        score += 1;
    }
    if password.len() >= 12 {
        score += 1;
    }

    // 包含小写字母
    if password.chars().any(|c| c.is_ascii_lowercase()) {
        score += 1;
    }

    // 包含大写字母
    if password.chars().any(|c| c.is_ascii_uppercase()) {
        score += 1;
    }

    // 包含数字
    if password.chars().any(|c| c.is_ascii_digit()) {
        score += 1;
    }

    // 包含特殊字符
    if password.chars().any(|c| !c.is_alphanumeric()) {
        score += 1;
    }

    let strength = match score {
        0..=2 => "weak",
        3..=4 => "medium",
        _ => "strong",
    };

    (strength.to_string(), score)
}

// 根据策略验证密码
pub fn validate_password_policy(password: &str, policy: &PasswordPolicy) -> Result<(), String> {
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
        return Err(format!(
            "密码强度不足，需要{}强度或更高",
            policy.min_strength
        ));
    }

    Ok(())
}

async fn load_user_by_id(state: &AppState, user_id: &str) -> Option<User> {
    if crate::database::get_db().is_some() {
        return match db_get_user_by_id(user_id).await {
            Ok(Some(user)) => {
                sync_cached_user(&state.users, &user);
                Some(user)
            }
            _ => None,
        };
    }

    state
        .users
        .read()
        .ok()?
        .iter()
        .find(|user| user.id == user_id)
        .cloned()
}

async fn load_user_by_username(state: &AppState, username: &str) -> Option<User> {
    if crate::database::get_db().is_some() {
        return match db_get_user_by_username(username).await {
            Ok(Some(user)) => {
                sync_cached_user(&state.users, &user);
                Some(user)
            }
            _ => None,
        };
    }

    state
        .users
        .read()
        .ok()?
        .iter()
        .find(|user| user.username == username)
        .cloned()
}

async fn load_all_users(state: &AppState) -> Result<Vec<User>, ApiError> {
    match db_get_users().await {
        Ok(users) => {
            sync_cached_users(&state.users, users.clone());
            Ok(users)
        }
        Err(_e) if crate::database::get_db().is_none() => state
            .users
            .read()
            .map(|users| users.clone())
            .map_err(|lock_error| {
                ApiError::internal(format!("Failed to read users cache: {}", lock_error))
            }),
        Err(e) => Err(ApiError::internal(format!(
            "Failed to load users from database: {}",
            e
        ))),
    }
}

async fn persist_user(state: &AppState, user: &User) -> Result<(), ApiError> {
    if crate::database::get_db().is_some() {
        db_update_user(&user.id, user)
            .await
            .map_err(|e| ApiError::internal(format!("Failed to persist user: {}", e)))?;
    }

    sync_cached_user(&state.users, user);
    Ok(())
}

fn is_builtin_system_account(username: &str) -> bool {
    matches!(username, "admin" | "sec" | "audit")
}

async fn validate_user_binding(
    username: &str,
    real_name: Option<&str>,
    organization_id: Option<i32>,
    department_id: Option<i32>,
) -> Result<(), ApiError> {
    if is_builtin_system_account(username) {
        return Ok(());
    }

    if real_name.is_none_or(|value| value.trim().is_empty()) {
        return Err(ApiError::bad_request("普通账号必须填写姓名"));
    }

    let organization_id =
        organization_id.ok_or_else(|| ApiError::bad_request("普通账号必须绑定组织/单位"))?;
    let department_id =
        department_id.ok_or_else(|| ApiError::bad_request("普通账号必须绑定部门"))?;

    let conn = get_db().ok_or_else(|| ApiError::internal("Database not available"))?;
    get_organization_by_id(&conn, organization_id)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to load organization: {}", e)))?
        .ok_or_else(|| ApiError::bad_request("绑定的组织/单位不存在"))?;

    let department = get_department_by_id(&conn, department_id)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to load department: {}", e)))?
        .ok_or_else(|| ApiError::bad_request("绑定的部门不存在"))?;

    if department.organization_id != organization_id {
        return Err(ApiError::bad_request("绑定的部门不属于所选组织/单位"));
    }

    Ok(())
}

/// Get all users (SysAdmin only)
#[utoipa::path(
    get,
    path = "/api/users",
    responses(
        (status = 200, description = "List of users", body = Vec<User>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - SysAdmin only")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "users"
)]
pub async fn get_users(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<User>>, ApiError> {
    let user = get_current_user_from_headers(&headers, &state.users)
        .await
        .ok_or_else(|| ApiError::unauthorized("Unauthorized"))?;

    if user.role != Role::SysAdmin {
        return Err(ApiError::forbidden("Access denied: SysAdmin only"));
    }

    Ok(Json(load_all_users(&state).await?))
}

/// Create a new user (SysAdmin only)
#[utoipa::path(
    post,
    path = "/api/users",
    request_body = CreateUserRequest,
    responses(
        (status = 200, description = "User created", body = User),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 409, description = "Username already exists")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "users"
)]
pub async fn create_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<User>, ApiError> {
    let current_user = get_current_user_from_headers(&headers, &state.users)
        .await
        .ok_or_else(|| ApiError::unauthorized("Unauthorized"))?;

    if current_user.role != Role::SysAdmin {
        return Err(ApiError::forbidden("Access denied: SysAdmin only"));
    }

    if load_user_by_username(&state, &req.username).await.is_some() {
        return Err(ApiError::conflict("Username already exists"));
    }

    validate_user_binding(
        &req.username,
        req.real_name.as_deref(),
        req.organization_id,
        req.department_id,
    )
    .await?;

    let (password_strength, _) = calculate_password_strength(&req.password);
    let password_hash = password::hash_password(&req.password)
        .map_err(|e| ApiError::internal(format!("Failed to hash password: {}", e)))?;

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
        real_name: req.real_name.clone(),
        password: password_hash,
        role: req.role,
        permissions,
        created_at: Utc::now(),
        password_changed_at: Some(Utc::now()),
        password_strength: Some(password_strength),
        force_password_change: Some(false),
        last_login_at: None,
        email: req.email.clone(),
        phone: req.phone.clone(),
        status: Some(req.status.clone().unwrap_or_else(|| "active".to_string())),
        organization_id: req.organization_id,
        department_id: req.department_id,
        failed_login_attempts: Some(0),
        locked_until: None,
    };

    if crate::database::get_db().is_some() {
        db_insert_user(&new_user)
            .await
            .map_err(|e| ApiError::internal(format!("Failed to create user: {}", e)))?;
    }

    sync_cached_user(&state.users, &new_user);

    log_action(
        &state.audit_logs,
        &current_user,
        "CREATE_USER",
        &new_user.username,
        "Created new user",
    );

    Ok(Json(new_user))
}

pub async fn update_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<User>, ApiError> {
    let current_user = get_current_user_from_headers(&headers, &state.users)
        .await
        .ok_or_else(|| ApiError::unauthorized("Unauthorized"))?;

    if current_user.role != Role::SysAdmin {
        return Err(ApiError::forbidden("Access denied: SysAdmin only"));
    }

    let mut target_user = load_user_by_id(&state, &id)
        .await
        .ok_or_else(|| ApiError::not_found("User not found"))?;

    let next_real_name = req.real_name.clone().or(target_user.real_name.clone());
    let next_org_id = req.organization_id.or(target_user.organization_id);
    let next_department_id = req.department_id.or(target_user.department_id);
    validate_user_binding(
        &target_user.username,
        next_real_name.as_deref(),
        next_org_id,
        next_department_id,
    )
    .await?;

    if let Some(value) = req.real_name {
        target_user.real_name = if value.trim().is_empty() {
            None
        } else {
            Some(value)
        };
    }
    if let Some(value) = req.role {
        target_user.role = value;
    }
    if let Some(value) = req.email {
        target_user.email = if value.trim().is_empty() {
            None
        } else {
            Some(value)
        };
    }
    if let Some(value) = req.phone {
        target_user.phone = if value.trim().is_empty() {
            None
        } else {
            Some(value)
        };
    }
    if let Some(value) = req.status {
        target_user.status = Some(value);
    }
    if req.organization_id.is_some() {
        target_user.organization_id = req.organization_id;
    }
    if req.department_id.is_some() {
        target_user.department_id = req.department_id;
    }
    if let Some(value) = req.password {
        let password_hash = password::hash_password(&value)
            .map_err(|e| ApiError::internal(format!("Failed to hash password: {}", e)))?;
        let (password_strength, _) = calculate_password_strength(&value);
        target_user.password = password_hash;
        target_user.password_changed_at = Some(Utc::now());
        target_user.password_strength = Some(password_strength);
    }

    persist_user(&state, &target_user).await?;

    log_action(
        &state.audit_logs,
        &current_user,
        "UPDATE_USER",
        &target_user.username,
        "Updated user profile",
    );

    Ok(Json(target_user))
}

/// Delete a user (SysAdmin only)
#[utoipa::path(
    delete,
    path = "/api/users/{id}",
    params(
        ("id" = String, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User deleted", body = String),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "User not found")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "users"
)]
pub async fn delete_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<String>, ApiError> {
    let current_user = get_current_user_from_headers(&headers, &state.users)
        .await
        .ok_or_else(|| ApiError::unauthorized("Unauthorized"))?;

    if current_user.role != Role::SysAdmin {
        return Err(ApiError::forbidden("Access denied: SysAdmin only"));
    }

    let removed_user = load_user_by_id(&state, &id)
        .await
        .ok_or_else(|| ApiError::not_found(format!("User with ID '{}' not found", id)))?;

    if crate::database::get_db().is_some() {
        db_delete_user(&id)
            .await
            .map_err(|e| ApiError::internal(format!("Failed to delete user: {}", e)))?;
    }

    remove_cached_user(&state.users, &id);

    log_action(
        &state.audit_logs,
        &current_user,
        "DELETE_USER",
        &removed_user.username,
        "Deleted user",
    );
    Ok(Json("Deleted".to_string()))
}

/// Update user permissions
#[utoipa::path(
    put,
    path = "/api/users/{id}/permissions",
    params(
        ("id" = String, Path, description = "User ID")
    ),
    request_body = Permissions,
    responses(
        (status = 200, description = "Permissions updated", body = User),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "User not found")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "users"
)]
pub async fn update_user_permissions(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(permissions): Json<Permissions>,
) -> Result<Json<User>, (StatusCode, String)> {
    let current_user = get_current_user_from_headers(&headers, &state.users)
        .await
        .ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;

    // Check if current user has permission to manage permissions
    let current_perms = current_user
        .permissions
        .as_ref()
        .ok_or((StatusCode::FORBIDDEN, "No permissions set".to_string()))?;

    if !current_perms.can_manage_permissions {
        return Err((StatusCode::FORBIDDEN, "Permission denied".to_string()));
    }

    let mut target_user = load_user_by_id(&state, &id)
        .await
        .ok_or((StatusCode::NOT_FOUND, "User not found".to_string()))?;
    target_user.permissions = Some(permissions.clone());

    persist_user(&state, &target_user)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    log_action(
        &state.audit_logs,
        &current_user,
        "UPDATE_PERMISSIONS",
        &target_user.username,
        "Updated user permissions",
    );

    Ok(Json(target_user))
}

/// Change current user's password
#[utoipa::path(
    post,
    path = "/api/users/change-password",
    request_body = ChangePasswordRequest,
    responses(
        (status = 200, description = "Password changed successfully", body = String),
        (status = 400, description = "Invalid password"),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "users"
)]
pub async fn change_password(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<Json<String>, (StatusCode, String)> {
    let current_user = get_current_user_from_headers(&headers, &state.users)
        .await
        .ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;

    // 获取密码策略
    let policy = state.password_policy.read().unwrap().clone();

    // 验证新密码是否符合策略
    if let Err(e) = validate_password_policy(&req.new_password, &policy) {
        return Err((StatusCode::BAD_REQUEST, e));
    }

    // 计算密码强度
    let (strength, _) = calculate_password_strength(&req.new_password);

    let mut target_user = load_user_by_id(&state, &current_user.id)
        .await
        .ok_or((StatusCode::NOT_FOUND, "User not found".to_string()))?;
    let username = target_user.username.clone();

    let password_valid = password::verify_password(&req.current_password, &target_user.password)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Password verification failed: {}", e),
            )
        })?;

    if !password_valid {
        return Err((
            StatusCode::BAD_REQUEST,
            "Current password is incorrect".to_string(),
        ));
    }

    let new_password_hash = password::hash_password(&req.new_password).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to hash password: {}", e),
        )
    })?;
    target_user.password = new_password_hash.clone();
    target_user.password_changed_at = Some(Utc::now());
    target_user.password_strength = Some(strength.clone());

    persist_user(&state, &target_user)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 记录密码历史
    {
        let mut history = state.password_history.write().unwrap();
        history.push((target_user.id.clone(), new_password_hash, Utc::now()));
    }

    log_action(
        &state.audit_logs,
        &current_user,
        "CHANGE_PASSWORD",
        &username,
        &format!("Password changed, strength: {}", strength),
    );

    Ok(Json("Password changed successfully".to_string()))
}

// 获取密码策略
pub async fn get_password_policy(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<PasswordPolicy>, (StatusCode, String)> {
    let user = get_current_user_from_headers(&headers, &state.users)
        .await
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
    let current_user = get_current_user_from_headers(&headers, &state.users)
        .await
        .ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;

    // 只有 admin 可以修改密码策略
    if current_user.role != Role::SysAdmin {
        return Err((StatusCode::FORBIDDEN, "Access denied".to_string()));
    }

    let mut policy_state = state.password_policy.write().unwrap();
    *policy_state = policy.clone();

    log_action(
        &state.audit_logs,
        &current_user,
        "UPDATE_PASSWORD_POLICY",
        "system",
        &format!(
            "Updated password policy: min_length={}, require_uppercase={}, require_number={}",
            policy.min_length, policy.require_uppercase, policy.require_number
        ),
    );

    Ok(Json(policy))
}

// 获取当前用户信息
/// Get current user information
#[utoipa::path(
    get,
    path = "/api/users/me",
    responses(
        (status = 200, description = "Current user info", body = User),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "users"
)]
pub async fn get_current_user_info(
    auth_user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<User>, ApiError> {
    let user = get_current_user_from_auth(&auth_user, &state.users)
        .await
        .ok_or_else(|| ApiError::unauthorized("User not found"))?;

    Ok(Json(user))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_password_strength_weak() {
        // Short password with only lowercase
        let (strength, score) = calculate_password_strength("abc");
        assert_eq!(strength, "weak");
        assert!(score <= 2);
    }

    #[test]
    fn test_calculate_password_strength_medium() {
        // Medium password
        let (strength, score) = calculate_password_strength("Abc123");
        assert_eq!(strength, "medium");
        assert!((3..=4).contains(&score));
    }

    #[test]
    fn test_calculate_password_strength_strong() {
        // Strong password
        let (strength, score) = calculate_password_strength("StrongP@ss123");
        assert_eq!(strength, "strong");
        assert!(score >= 5);
    }

    #[test]
    fn test_calculate_password_strength_all_criteria() {
        let (strength, score) = calculate_password_strength("VeryStr0ng!Pass");

        assert_eq!(strength, "strong");

        // Verify score includes all criteria
        assert!(score >= 5); // Has at least 5 criteria met
    }

    #[test]
    fn test_calculate_password_strength_empty() {
        let (strength, score) = calculate_password_strength("");
        assert_eq!(strength, "weak");
        assert_eq!(score, 0);
    }

    #[test]
    fn test_validate_password_policy_success() {
        let policy = PasswordPolicy {
            min_length: 8,
            require_uppercase: true,
            require_lowercase: true,
            require_number: true,
            require_special: true,
            max_age_days: None,
            prevent_reuse: 0,
            min_strength: "medium".to_string(),
            max_login_attempts: Some(3),
            lockout_duration_minutes: 30,
        };

        let result = validate_password_policy("StrongP@ss123", &policy);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_password_policy_too_short() {
        let policy = PasswordPolicy {
            min_length: 8,
            require_uppercase: true,
            require_lowercase: true,
            require_number: true,
            require_special: true,
            max_age_days: None,
            prevent_reuse: 0,
            min_strength: "medium".to_string(),
            max_login_attempts: Some(3),
            lockout_duration_minutes: 30,
        };

        let result = validate_password_policy("Short1!", &policy);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("密码长度至少为"));
    }

    #[test]
    fn test_validate_password_policy_missing_uppercase() {
        let policy = PasswordPolicy {
            min_length: 8,
            require_uppercase: true,
            require_lowercase: true,
            require_number: true,
            require_special: true,
            max_age_days: None,
            prevent_reuse: 0,
            min_strength: "medium".to_string(),
            max_login_attempts: Some(3),
            lockout_duration_minutes: 30,
        };

        let result = validate_password_policy("lowercase123!", &policy);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("大写字母"));
    }

    #[test]
    fn test_validate_password_policy_missing_lowercase() {
        let policy = PasswordPolicy {
            min_length: 8,
            require_uppercase: true,
            require_lowercase: true,
            require_number: true,
            require_special: true,
            max_age_days: None,
            prevent_reuse: 0,
            min_strength: "medium".to_string(),
            max_login_attempts: Some(3),
            lockout_duration_minutes: 30,
        };

        let result = validate_password_policy("UPPERCASE123!", &policy);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("小写字母"));
    }

    #[test]
    fn test_validate_password_policy_missing_number() {
        let policy = PasswordPolicy {
            min_length: 8,
            require_uppercase: true,
            require_lowercase: true,
            require_number: true,
            require_special: true,
            max_age_days: None,
            prevent_reuse: 0,
            min_strength: "medium".to_string(),
            max_login_attempts: Some(3),
            lockout_duration_minutes: 30,
        };

        let result = validate_password_policy("NoNumbers!", &policy);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("数字"));
    }

    #[test]
    fn test_validate_password_policy_missing_special() {
        let policy = PasswordPolicy {
            min_length: 8,
            require_uppercase: true,
            require_lowercase: true,
            require_number: true,
            require_special: true,
            max_age_days: None,
            prevent_reuse: 0,
            min_strength: "medium".to_string(),
            max_login_attempts: Some(3),
            lockout_duration_minutes: 30,
        };

        let result = validate_password_policy("NoSpecial123", &policy);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("特殊字符"));
    }

    #[test]
    fn test_validate_password_policy_weak_strength() {
        let policy = PasswordPolicy {
            min_length: 8,
            require_uppercase: false,
            require_lowercase: false,
            require_number: false,
            require_special: false,
            max_age_days: None,
            prevent_reuse: 0,
            min_strength: "strong".to_string(),
            max_login_attempts: Some(3),
            lockout_duration_minutes: 30,
        };

        let result = validate_password_policy("weakpass", &policy);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("密码强度不足"));
    }

    #[test]
    fn test_validate_password_policy_no_requirements() {
        let policy = PasswordPolicy {
            min_length: 1,
            require_uppercase: false,
            require_lowercase: false,
            require_number: false,
            require_special: false,
            max_age_days: None,
            prevent_reuse: 0,
            min_strength: "weak".to_string(),
            max_login_attempts: None,
            lockout_duration_minutes: 0,
        };

        let result = validate_password_policy("a", &policy);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_password_policy_strong_strength() {
        let policy = PasswordPolicy {
            min_length: 8,
            require_uppercase: true,
            require_lowercase: true,
            require_number: true,
            require_special: true,
            max_age_days: None,
            prevent_reuse: 0,
            min_strength: "strong".to_string(),
            max_login_attempts: Some(3),
            lockout_duration_minutes: 30,
        };

        let result = validate_password_policy("VeryStr0ng!Pass", &policy);
        assert!(result.is_ok());
    }
}
