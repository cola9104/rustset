use axum::{
    extract::{State, Json},
    http::StatusCode,
};
use shared::{LoginRequest, LoginResponse};
use crate::state::AppState;
use crate::utils::log_action;
use crate::password;
use chrono::Utc;

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>
) -> Result<Json<LoginResponse>, (StatusCode, String)> {
    // Read password policy (using read lock for better performance)
    let policy = state.password_policy.read().unwrap().clone();

    // Check if user exists (read lock)
    let user_idx = {
        let users = state.users.read().unwrap();
        users.iter().position(|u| u.username == req.username)
    };

    if user_idx.is_none() {
        return Err((StatusCode::UNAUTHORIZED, "用户名或密码错误".to_string()));
    }

    let user_idx = user_idx.unwrap();

    // Acquire write lock to check and update user state
    let mut users = state.users.write().unwrap();
    let user = &mut users[user_idx];

    // 检查账户是否被禁用
    if user.status.as_ref().map(|s| s.as_str()) == Some("disabled") {
        return Err((StatusCode::FORBIDDEN, "账户已被禁用".to_string()));
    }

    // 检查账户是否被锁定
    if let Some(locked_until) = user.locked_until {
        if Utc::now() < locked_until {
            let remaining = (locked_until - Utc::now()).num_minutes() + 1;
            return Err((StatusCode::FORBIDDEN, format!("账户已锁定，请在 {} 分钟后重试", remaining)));
        } else {
            // 锁定期已过，清除锁定状态
            user.locked_until = None;
            user.failed_login_attempts = Some(0);
            user.status = Some("active".to_string());
        }
    }

    // 验证密码 (使用 Argon2 哈希验证)
    let password_valid = password::verify_password(&req.password, &user.password)
        .unwrap_or(false);

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

                return Err((StatusCode::LOCKED,
                           format!("登录失败次数过多，账户已锁定 {} 分钟", policy.lockout_duration_minutes)));
            }
        }

        // 记录失败的登录尝试
        log_action(&state.audit_logs, user, "LOGIN_FAILED", &user.username,
                   &format!("Failed login attempt {}/{}", current_attempts,
                           policy.max_login_attempts.map(|n| n.to_string()).unwrap_or("∞".to_string())));

        let remaining = policy.max_login_attempts.map(|max| max - current_attempts);
        match remaining {
            Some(0) => Err((StatusCode::LOCKED, "账户已被锁定".to_string())),
            Some(n) => Err((StatusCode::UNAUTHORIZED,
                           format!("密码错误，还有 {} 次尝试机会", n))),
            None => Err((StatusCode::UNAUTHORIZED, "密码错误".to_string())),
        }
    }
}
