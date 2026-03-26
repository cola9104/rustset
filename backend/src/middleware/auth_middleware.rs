use axum::{
    extract::{FromRequestParts, Request, State},
    http::{header::AUTHORIZATION, request::Parts, Method},
    middleware::Next,
    response::Response,
};
use chrono::Utc;
use std::future::{ready, Future};
use tower_sessions::Session;

use crate::auth::{verify_token, Claims};
use crate::middleware::ApiError;
use crate::state::AppState;
use crate::utils::get_current_user_from_auth;
use shared::Role;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: String,
    pub username: String,
    pub role: Role,
    #[allow(dead_code)]
    pub exp: usize,
}

impl From<Claims> for AuthUser {
    fn from(claims: Claims) -> Self {
        Self {
            user_id: claims.user_id,
            username: claims.username,
            role: claims.role,
            exp: claims.exp,
        }
    }
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        ready(
            parts
                .extensions
                .get::<AuthUser>()
                .cloned()
                .ok_or_else(|| ApiError::unauthorized("Unauthorized")),
        )
    }
}

fn is_public_path(method: &Method, path: &str) -> bool {
    matches!(
        (method.as_str(), path),
        ("POST", "/api/login")
            | ("POST", "/api/logout")
            | ("POST", "/api/refresh-token")
            | ("GET", "/api/health")
            | ("GET", "/api/ready")
            | ("GET", "/api/live")
            | ("GET", "/api/metrics")
            | ("GET", "/health")
            | ("GET", "/ready")
            | ("GET", "/live")
            | ("GET", "/metrics")
    )
}

fn auth_user_from_session_claims(claims: Claims) -> AuthUser {
    claims.into()
}

fn auth_user_from_authorization_header(header_value: &str) -> Result<Option<AuthUser>, ApiError> {
    let token = header_value.trim();
    if token.is_empty() {
        return Ok(None);
    }

    let token = token
        .strip_prefix("Bearer ")
        .or_else(|| token.strip_prefix("bearer "))
        .unwrap_or(token);

    let claims = verify_token(token)?;

    Ok(Some(AuthUser {
        user_id: claims.user_id,
        username: claims.username,
        role: claims.role,
        exp: claims.exp,
    }))
}

async fn validate_current_user(
    state: &AppState,
    auth_user: AuthUser,
) -> Result<AuthUser, ApiError> {
    let user = get_current_user_from_auth(&auth_user, &state.users)
        .await
        .ok_or_else(|| ApiError::unauthorized("User not found"))?;

    if user.status.as_deref() == Some("disabled") {
        return Err(ApiError::forbidden("账户已被禁用"));
    }

    if let Some(locked_until) = user.locked_until {
        if Utc::now() < locked_until {
            return Err(ApiError::forbidden("账户已锁定"));
        }
    }

    Ok(auth_user)
}

/// 认证中间件 - 从 Session 中获取用户信息
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let method = req.method().clone();
    let path = req.uri().path().to_string();

    // 公开路径或 OPTIONS 预检请求直接通过
    if method == Method::OPTIONS || is_public_path(&method, &path) {
        return Ok(next.run(req).await);
    }

    // 从请求 extensions 中获取 Session（由 SessionManagerLayer 注入）
    let session = req.extensions_mut().get::<Session>();
    if let Some(session) = session {
        // 尝试从 Session 中获取用户信息
        let session_result = session.get::<Claims>("user").await;
        tracing::debug!("Session get result for path {}: {:?}", path, session_result);

        if let Ok(Some(claims)) = session_result {
            let auth_user =
                validate_current_user(&state, auth_user_from_session_claims(claims.clone()))
                    .await?;
            tracing::debug!("Authenticated user: {}", claims.username);
            req.extensions_mut().insert(auth_user);
            return Ok(next.run(req).await);
        }
    } else {
        tracing::warn!("No session found in request extensions for path: {}", path);
    }

    if let Some(auth_header) = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
    {
        if let Some(auth_user) = auth_user_from_authorization_header(auth_header)? {
            let auth_user = validate_current_user(&state, auth_user).await?;
            tracing::debug!(
                "Authenticated user via Authorization header: {}",
                auth_user.username
            );
            req.extensions_mut().insert(auth_user);
            return Ok(next.run(req).await);
        }
    }

    tracing::warn!("Authentication failed for path: {}", path);
    Err(ApiError::unauthorized("Unauthorized: Please login"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::generate_token;
    use axum::{
        body::Body,
        http::{header, Request, StatusCode},
        middleware::from_fn_with_state,
        response::IntoResponse,
        routing::get,
        Router,
    };
    use chrono::{Duration, Utc};
    use shared::{PasswordPolicy, Permissions, User};
    use std::sync::{Arc, RwLock};
    use tower::ServiceExt;
    use tower_cookies::CookieManagerLayer;
    use tower_sessions::{MemoryStore, SessionManagerLayer};

    async fn protected_route(_auth_user: AuthUser) -> impl IntoResponse {
        StatusCode::OK
    }

    fn test_state() -> AppState {
        AppState {
            users: Arc::new(RwLock::new(vec![
                User {
                    id: "active".to_string(),
                    username: "active".to_string(),
                    real_name: None,
                    password: "hashed".to_string(),
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
                    organization_id: None,
                    department_id: None,
                    failed_login_attempts: Some(0),
                    locked_until: None,
                },
                User {
                    id: "disabled".to_string(),
                    username: "disabled".to_string(),
                    real_name: None,
                    password: "hashed".to_string(),
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
                    organization_id: None,
                    department_id: None,
                    failed_login_attempts: Some(0),
                    locked_until: None,
                },
                User {
                    id: "locked".to_string(),
                    username: "locked".to_string(),
                    real_name: None,
                    password: "hashed".to_string(),
                    role: Role::Auditor,
                    permissions: Some(Permissions::auditor()),
                    created_at: Utc::now(),
                    password_changed_at: Some(Utc::now()),
                    password_strength: Some("medium".to_string()),
                    force_password_change: Some(false),
                    last_login_at: None,
                    email: None,
                    phone: None,
                    status: Some("locked".to_string()),
                    organization_id: None,
                    department_id: None,
                    failed_login_attempts: Some(3),
                    locked_until: Some(Utc::now() + Duration::minutes(10)),
                },
            ])),
            assets: Arc::new(RwLock::new(vec![])),
            tasks: Arc::new(RwLock::new(vec![])),
            risks: Arc::new(RwLock::new(vec![])),
            zones: Arc::new(RwLock::new(vec![])),
            audit_logs: Arc::new(RwLock::new(vec![])),
            advanced_tasks: Arc::new(RwLock::new(vec![])),
            custom_roles: Arc::new(RwLock::new(vec![])),
            scan_manager: Arc::new(tokio::sync::RwLock::new(None)),
            password_policy: Arc::new(RwLock::new(PasswordPolicy::default())),
            password_history: Arc::new(RwLock::new(vec![])),
            port_details: Arc::new(RwLock::new(vec![])),
            scanners: Arc::new(RwLock::new(vec![])),
            scan_results: Arc::new(RwLock::new(vec![])),
        }
    }

    fn auth_app(state: AppState) -> Router {
        Router::new()
            .route("/protected", get(protected_route))
            .with_state(state.clone())
            .layer(from_fn_with_state(state, auth_middleware))
            .layer(SessionManagerLayer::new(MemoryStore::default()))
            .layer(CookieManagerLayer::new())
    }

    fn auth_header(user: &User) -> String {
        std::env::set_var("JWT_SECRET", "test-jwt-secret");
        format!("Bearer {}", generate_token(user).unwrap())
    }

    #[tokio::test]
    async fn rejects_disabled_user_even_with_valid_token() {
        let state = test_state();
        let disabled_user = state.users.read().unwrap()[1].clone();
        let app = auth_app(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .header(header::AUTHORIZATION, auth_header(&disabled_user))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_locked_user_even_with_valid_token() {
        let state = test_state();
        let locked_user = state.users.read().unwrap()[2].clone();
        let app = auth_app(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .header(header::AUTHORIZATION, auth_header(&locked_user))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn allows_active_user_with_valid_token() {
        let state = test_state();
        let active_user = state.users.read().unwrap()[0].clone();
        let app = auth_app(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .header(header::AUTHORIZATION, auth_header(&active_user))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
