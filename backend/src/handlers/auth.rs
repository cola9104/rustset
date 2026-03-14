use axum::{
    extract::{State, Json},
    http::{HeaderMap, header::AUTHORIZATION},
};
use shared::{LoginRequest, LoginResponse};
use crate::state::AppState;
use crate::utils::log_action;
use crate::password;
use crate::middleware::ApiError;
use chrono::Utc;

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>
) -> Result<Json<LoginResponse>, ApiError> {
    // Read password policy (using read lock for better performance)
    let policy = state.password_policy.read()
        .map_err(|e| ApiError::internal(format!("Failed to read password policy: {}", e)))?
        .clone();

    // Check if user exists (read lock)
    let user_idx = {
        let users = state.users.read()
            .map_err(|e| ApiError::internal(format!("Failed to acquire read lock: {}", e)))?;
        users.iter().position(|u| u.username == req.username)
    };

    if user_idx.is_none() {
        return Err(ApiError::unauthorized("用户名或密码错误"));
    }

    let user_idx = user_idx.unwrap();

    // Acquire write lock to check and update user state
    let mut users = state.users.write()
        .map_err(|e| ApiError::internal(format!("Failed to acquire write lock: {}", e)))?;
    let user = &mut users[user_idx];

    // 检查账户是否被禁用
    if user.status.as_deref() == Some("disabled") {
        return Err(ApiError::forbidden("账户已被禁用"));
    }

    // 检查账户是否被锁定
    if let Some(locked_until) = user.locked_until {
        if Utc::now() < locked_until {
            let remaining = (locked_until - Utc::now()).num_minutes() + 1;
            return Err(ApiError::forbidden(format!("账户已锁定，请在 {} 分钟后重试", remaining)));
        } else {
            // 锁定期已过，清除锁定状态
            user.locked_until = None;
            user.failed_login_attempts = Some(0);
            user.status = Some("active".to_string());
        }
    }

    // 验证密码 (使用 Argon2 哈希验证)
    let password_valid = password::verify_password(&req.password, &user.password)
        .map_err(|e| ApiError::internal(format!("Password verification failed: {}", e)))?;

    if password_valid {
        // 登录成功，清除失败计数
        user.failed_login_attempts = Some(0);
        user.last_login_at = Some(Utc::now());

        // 记录审计日志
        log_action(&state.audit_logs, user, "LOGIN", &user.username, "User logged in successfully");

        Ok(Json(LoginResponse {
            token: user.username.clone(),
            user: user.clone(),
        }))
    } else {
        // 登录失败，增加失败计数
        let current_attempts = user.failed_login_attempts.unwrap_or(0) + 1;
        user.failed_login_attempts = Some(current_attempts);

        // 检查是否达到最大失败次数
        if let Some(max_attempts) = policy.max_login_attempts {
            if current_attempts >= max_attempts {
                // 锁定账户
                let lock_until = Utc::now() + chrono::Duration::minutes(policy.lockout_duration_minutes as i64);
                user.locked_until = Some(lock_until);
                user.status = Some("locked".to_string());

                // 记录审计日志
                log_action(&state.audit_logs, user, "ACCOUNT_LOCKED", &user.username,
                           &format!("Account locked after {} failed login attempts", current_attempts));

                return Err(ApiError::forbidden(
                    format!("登录失败次数过多，账户已锁定 {} 分钟", policy.lockout_duration_minutes)
                ));
            }
        }

        // 记录失败的登录尝试
        log_action(&state.audit_logs, user, "LOGIN_FAILED", &user.username,
                   &format!("Failed login attempt {}/{}", current_attempts,
                           policy.max_login_attempts.map(|n| n.to_string()).unwrap_or("∞".to_string())));

        let remaining = policy.max_login_attempts.map(|max| max - current_attempts);
        match remaining {
            Some(0) => Err(ApiError::forbidden("账户已被锁定")),
            Some(n) => Err(ApiError::unauthorized(
                format!("密码错误，还有 {} 次尝试机会", n)
            )),
            None => Err(ApiError::unauthorized("密码错误")),
        }
    }
}

/// 登出
pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    let username = headers.get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    // 记录审计日志
    if let Some(user) = state.users.read().ok().and_then(|u| u.iter().find(|u| u.username == username).cloned()) {
        log_action(&state.audit_logs, &user, "LOGOUT", &username, "User logged out");
    }

    Ok(Json(serde_json::json!({
        "message": "Logged out successfully"
    })))
}

/// 刷新 Token
pub async fn refresh_token(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<LoginResponse>, ApiError> {
    let username = headers.get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| ApiError::unauthorized("No authorization token"))?;

    let users = state.users.read()
        .map_err(|e| ApiError::internal(format!("Failed to read users: {}", e)))?;

    let user = users.iter()
        .find(|u| u.username == username)
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

    Ok(Json(LoginResponse {
        token: user.username.clone(),
        user: user.clone(),
    }))
}
