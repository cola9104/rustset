use crate::auth::{generate_token, verify_token};
use crate::middleware::{auth_middleware::Claims, ApiError};
use crate::password;
use crate::state::AppState;
use crate::utils::{get_current_user_from_auth, log_action, sync_cached_user};
use axum::{
    extract::{Json, State},
    http::{header::AUTHORIZATION, HeaderMap},
};
use chrono::Utc;
use shared::{LoginRequest, LoginResponse};
use tower_sessions::Session;

async fn load_user_by_username(state: &AppState, username: &str) -> Option<shared::User> {
    if crate::database::get_db().is_some() {
        return match crate::database::get_user_by_username(username).await {
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

async fn persist_user_state(state: &AppState, user: &shared::User) -> Result<(), ApiError> {
    if crate::database::get_db().is_some() {
        crate::database::update_user(&user.id, user)
            .await
            .map_err(|e| ApiError::internal(format!("Failed to persist user state: {}", e)))?;
    }

    sync_cached_user(&state.users, user);
    Ok(())
}

/// User login endpoint
#[utoipa::path(
    post,
    path = "/api/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 401, description = "Invalid credentials")
    ),
    tag = "auth"
)]
pub async fn login(
    State(state): State<AppState>,
    session: Session,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    // Read password policy (using read lock for better performance)
    let policy = state
        .password_policy
        .read()
        .map_err(|e| ApiError::internal(format!("Failed to read password policy: {}", e)))?
        .clone();

    let mut user = load_user_by_username(&state, &req.username)
        .await
        .ok_or_else(|| ApiError::unauthorized("用户名或密码错误"))?;

    // 检查账户是否被禁用
    if user.status.as_deref() == Some("disabled") {
        return Err(ApiError::forbidden("账户已被禁用"));
    }

    // 检查账户是否被锁定
    if let Some(locked_until) = user.locked_until {
        if Utc::now() < locked_until {
            let remaining = (locked_until - Utc::now()).num_minutes() + 1;
            return Err(ApiError::forbidden(format!(
                "账户已锁定，请在 {} 分钟后重试",
                remaining
            )));
        } else {
            // 锁定期已过，清除锁定状态
            user.locked_until = None;
            user.failed_login_attempts = Some(0);
            user.status = Some("active".to_string());
        }
    }

    let password_valid = password::verify_password(&req.password, &user.password)
        .map_err(|e| ApiError::internal(format!("Password verification failed: {}", e)))?;

    if !password_valid {
        let current_attempts = user.failed_login_attempts.unwrap_or(0) + 1;
        user.failed_login_attempts = Some(current_attempts);

        if let Some(max_attempts) = policy.max_login_attempts {
            if current_attempts >= max_attempts {
                let lock_until =
                    Utc::now() + chrono::Duration::minutes(policy.lockout_duration_minutes as i64);
                user.locked_until = Some(lock_until);
                user.status = Some("locked".to_string());

                persist_user_state(&state, &user).await?;

                log_action(
                    &state.audit_logs,
                    &user,
                    "ACCOUNT_LOCKED",
                    &user.username,
                    &format!(
                        "Account locked after {} failed login attempts",
                        current_attempts
                    ),
                );

                return Err(ApiError::forbidden(format!(
                    "登录失败次数过多，账户已锁定 {} 分钟",
                    policy.lockout_duration_minutes
                )));
            }
        }

        persist_user_state(&state, &user).await?;

        log_action(
            &state.audit_logs,
            &user,
            "LOGIN_FAILED",
            &user.username,
            &format!(
                "Failed login attempt {}/{}",
                current_attempts,
                policy
                    .max_login_attempts
                    .map(|n| n.to_string())
                    .unwrap_or("∞".to_string())
            ),
        );

        let remaining = policy.max_login_attempts.map(|max| max - current_attempts);
        return match remaining {
            Some(0) => Err(ApiError::forbidden("账户已被锁定")),
            Some(n) => Err(ApiError::unauthorized(format!(
                "密码错误，还有 {} 次尝试机会",
                n
            ))),
            None => Err(ApiError::unauthorized("密码错误")),
        };
    }

    user.failed_login_attempts = Some(0);
    user.last_login_at = Some(Utc::now());
    if user.status.as_deref() == Some("locked") {
        user.status = Some("active".to_string());
    }

    persist_user_state(&state, &user).await?;

    log_action(
        &state.audit_logs,
        &user,
        "LOGIN",
        &user.username,
        "User logged in successfully",
    );

    let user_id = user.id.clone();
    let username = user.username.clone();
    let role = user.role.clone();

    if user_id.is_empty() {
        return Err(ApiError::unauthorized("用户名或密码错误"));
    }

    // 保存用户信息到 Session (在锁释放后)
    let claims = Claims {
        user_id,
        username: username.clone(),
        role,
        exp: (Utc::now().timestamp() + 24 * 3600) as usize, // 24小时
    };
    session
        .insert("user", claims)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to create session: {}", e)))?;

    let token = generate_token(&user)?;

    Ok(Json(LoginResponse { token, user }))
}

/// User logout endpoint
#[utoipa::path(
    post,
    path = "/api/logout",
    responses(
        (status = 200, description = "Logout successful"),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "auth"
)]
pub async fn logout(
    State(state): State<AppState>,
    session: Session,
) -> Result<Json<serde_json::Value>, ApiError> {
    let auth_user = session
        .get::<Claims>("user")
        .await
        .ok()
        .flatten()
        .map(Into::into);

    // 清除 Session
    session
        .flush()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to flush session: {}", e)))?;

    // 记录审计日志
    if let Some(auth_user) = auth_user {
        if let Some(user) = get_current_user_from_auth(&auth_user, &state.users).await {
            log_action(
                &state.audit_logs,
                &user,
                "LOGOUT",
                &user.username,
                "User logged out",
            );
        }
    }

    Ok(Json(serde_json::json!({
        "message": "Logged out successfully"
    })))
}

/// Refresh authentication token
#[utoipa::path(
    post,
    path = "/api/refresh-token",
    responses(
        (status = 200, description = "Token refreshed", body = LoginResponse),
        (status = 401, description = "Invalid token")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "auth"
)]
pub async fn refresh_token(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<LoginResponse>, ApiError> {
    let auth_header = headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| ApiError::unauthorized("No authorization token"))?;

    let token = auth_header
        .trim()
        .strip_prefix("Bearer ")
        .or_else(|| auth_header.trim().strip_prefix("bearer "))
        .ok_or_else(|| ApiError::unauthorized("Invalid authorization scheme"))?;

    let claims = verify_token(token)?;

    let auth_user = crate::middleware::AuthUser {
        user_id: claims.user_id,
        username: claims.username,
        role: claims.role,
        exp: claims.exp,
    };

    let user = get_current_user_from_auth(&auth_user, &state.users)
        .await
        .ok_or_else(|| ApiError::unauthorized("User not found"))?;

    // 检查用户状态
    if user.status.as_deref() == Some("disabled") {
        return Err(ApiError::forbidden("账户已被禁用"));
    }

    if let Some(locked_until) = user.locked_until {
        if Utc::now() < locked_until {
            return Err(ApiError::forbidden("账户已锁定"));
        }
    }

    let token = generate_token(&user)?;

    Ok(Json(LoginResponse {
        token,
        user: user.clone(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::{generate_token, verify_token};
    use crate::password;
    use crate::state::AppState;
    use axum::http::HeaderMap;
    use shared::{Permissions, Role, User};
    use std::sync::{Arc, RwLock};
    use tower_sessions::{MemoryStore, Session};

    fn setup_jwt_secret() {
        std::env::set_var("JWT_SECRET", "test-jwt-secret");
    }

    fn create_test_app_state() -> AppState {
        AppState {
            users: Arc::new(RwLock::new(vec![
                User {
                    id: "1".to_string(),
                    username: "admin".to_string(),
                    password: password::hash_password("admin123").unwrap(),
                    role: Role::SysAdmin,
                    permissions: Some(Permissions::sys_admin()),
                    created_at: Utc::now(),
                    password_changed_at: Some(Utc::now()),
                    password_strength: Some("strong".to_string()),
                    force_password_change: Some(false),
                    last_login_at: None,
                    email: None,
                    phone: None,
                    status: Some("active".to_string()),
                    failed_login_attempts: Some(0),
                    locked_until: None,
                },
                User {
                    id: "2".to_string(),
                    username: "disabled_user".to_string(),
                    password: password::hash_password("password123").unwrap(),
                    role: Role::Auditor,
                    permissions: Some(Permissions::auditor()),
                    created_at: Utc::now(),
                    password_changed_at: Some(Utc::now()),
                    password_strength: Some("medium".to_string()),
                    force_password_change: Some(false),
                    last_login_at: None,
                    email: None,
                    phone: None,
                    status: Some("disabled".to_string()),
                    failed_login_attempts: Some(0),
                    locked_until: None,
                },
            ])),
            assets: Arc::new(RwLock::new(Vec::new())),
            tasks: Arc::new(RwLock::new(Vec::new())),
            risks: Arc::new(RwLock::new(Vec::new())),
            zones: Arc::new(RwLock::new(Vec::new())),
            audit_logs: Arc::new(RwLock::new(Vec::new())),
            advanced_tasks: Arc::new(RwLock::new(Vec::new())),
            custom_roles: Arc::new(RwLock::new(Vec::new())),
            scan_manager: Arc::new(tokio::sync::RwLock::new(None)),
            password_policy: Arc::new(RwLock::new(shared::PasswordPolicy {
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
            })),
            password_history: Arc::new(RwLock::new(Vec::new())),
            port_details: Arc::new(RwLock::new(Vec::new())),
            scanners: Arc::new(RwLock::new(Vec::new())),
            scan_results: Arc::new(RwLock::new(Vec::new())),
        }
    }

    fn create_test_session() -> Session {
        Session::new(None, Arc::new(MemoryStore::default()), None)
    }

    fn session_claims_for_user(user: &User) -> Claims {
        Claims {
            user_id: user.id.clone(),
            username: user.username.clone(),
            role: user.role.clone(),
            exp: (Utc::now().timestamp() + 24 * 3600) as usize,
        }
    }

    #[tokio::test]
    async fn test_login_success() {
        setup_jwt_secret();
        let state = create_test_app_state();
        let session = create_test_session();

        let req = LoginRequest {
            username: "admin".to_string(),
            password: "admin123".to_string(),
        };

        let result = login(State(state), session.clone(), Json(req)).await;

        assert!(result.is_ok());

        let response = result.unwrap();
        let claims = verify_token(&response.token).unwrap();
        let session_claims = session.get::<Claims>("user").await.unwrap().unwrap();

        assert_eq!(claims.user_id, "1");
        assert_eq!(claims.username, "admin");
        assert_eq!(claims.role, Role::SysAdmin);
        assert_eq!(response.user.username, "admin");
        assert_eq!(session_claims.user_id, "1");
        assert_eq!(session_claims.username, "admin");
    }

    #[tokio::test]
    async fn test_login_invalid_username() {
        let state = create_test_app_state();
        let session = create_test_session();
        let req = LoginRequest {
            username: "nonexistent".to_string(),
            password: "password123".to_string(),
        };

        let result = login(State(state), session, Json(req)).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::Unauthorized(msg) => {
                assert!(msg.contains("用户名或密码错误"));
            }
            _ => panic!("Expected Unauthorized error"),
        }
    }

    #[tokio::test]
    async fn test_login_invalid_password() {
        let state = create_test_app_state();
        let session = create_test_session();
        let req = LoginRequest {
            username: "admin".to_string(),
            password: "wrongpassword".to_string(),
        };

        let result = login(State(state), session, Json(req)).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::Unauthorized(msg) => {
                assert!(msg.contains("密码错误"));
            }
            _ => panic!("Expected Unauthorized error"),
        }
    }

    #[tokio::test]
    async fn test_login_disabled_account() {
        let state = create_test_app_state();
        let session = create_test_session();
        let req = LoginRequest {
            username: "disabled_user".to_string(),
            password: "password123".to_string(),
        };

        let result = login(State(state), session, Json(req)).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::Forbidden(msg) => {
                assert!(msg.contains("账户已被禁用"));
            }
            _ => panic!("Expected Forbidden error"),
        }
    }

    #[tokio::test]
    async fn test_login_account_lockout_after_max_attempts() {
        let state = create_test_app_state();

        // Attempt 3 failed logins
        for _ in 0..3 {
            let session = create_test_session();
            let req = LoginRequest {
                username: "admin".to_string(),
                password: "wrongpassword".to_string(),
            };
            let _ = login(State(state.clone()), session, Json(req)).await;
        }

        // 4th attempt should be locked
        let session = create_test_session();
        let req = LoginRequest {
            username: "admin".to_string(),
            password: "admin123".to_string(),
        };

        let result = login(State(state), session, Json(req)).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::Forbidden(msg) => {
                assert!(msg.contains("账户已锁定"));
            }
            _ => panic!("Expected Forbidden error for locked account"),
        }
    }

    #[tokio::test]
    async fn test_logout_success() {
        setup_jwt_secret();
        let state = create_test_app_state();
        let admin_user = state.users.read().unwrap()[0].clone();
        let session = create_test_session();

        session
            .insert("user", session_claims_for_user(&admin_user))
            .await
            .unwrap();

        let result = logout(State(state), session.clone()).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response["message"], "Logged out successfully");
        assert!(session.get::<Claims>("user").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_logout_without_auth_header() {
        let state = create_test_app_state();
        let session = create_test_session();

        let result = logout(State(state), session).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response["message"], "Logged out successfully");
    }

    #[tokio::test]
    async fn test_refresh_token_success() {
        setup_jwt_secret();
        let state = create_test_app_state();
        let admin_user = state.users.read().unwrap()[0].clone();
        let token = generate_token(&admin_user).unwrap();

        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {}", token).parse().unwrap(),
        );

        let result = refresh_token(State(state), headers).await;

        assert!(result.is_ok());
        let response = result.unwrap();

        let claims = verify_token(&response.token).unwrap();
        assert_eq!(claims.user_id, "1");
        assert_eq!(claims.username, "admin");
        assert_eq!(response.user.username, "admin");
    }

    #[tokio::test]
    async fn test_refresh_token_no_auth_header() {
        let state = create_test_app_state();
        let headers = HeaderMap::new();

        let result = refresh_token(State(state), headers).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::Unauthorized(msg) => {
                assert!(msg.contains("No authorization token"));
            }
            _ => panic!("Expected Unauthorized error"),
        }
    }

    #[tokio::test]
    async fn test_refresh_token_invalid_user() {
        setup_jwt_secret();
        let state = create_test_app_state();

        let fake_user = User {
            id: "999".to_string(),
            username: "nonexistent".to_string(),
            password: password::hash_password("password123").unwrap(),
            role: Role::Auditor,
            permissions: Some(Permissions::auditor()),
            created_at: Utc::now(),
            password_changed_at: Some(Utc::now()),
            password_strength: Some("medium".to_string()),
            force_password_change: Some(false),
            last_login_at: None,
            email: None,
            phone: None,
            status: Some("active".to_string()),
            failed_login_attempts: Some(0),
            locked_until: None,
        };
        let token = generate_token(&fake_user).unwrap();

        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {}", token).parse().unwrap(),
        );

        let result = refresh_token(State(state), headers).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::Unauthorized(msg) => {
                assert!(msg.contains("User not found"));
            }
            _ => panic!("Expected Unauthorized error"),
        }
    }

    #[tokio::test]
    async fn test_refresh_token_disabled_account() {
        setup_jwt_secret();
        let state = create_test_app_state();
        let disabled_user = state.users.read().unwrap()[1].clone();
        let token = generate_token(&disabled_user).unwrap();

        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {}", token).parse().unwrap(),
        );

        let result = refresh_token(State(state), headers).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::Forbidden(msg) => {
                assert!(msg.contains("账户已被禁用"));
            }
            _ => panic!("Expected Forbidden error"),
        }
    }
}
