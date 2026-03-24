use axum::{
    extract::{FromRequestParts, Request},
    http::{header::AUTHORIZATION, request::Parts, Method},
    middleware::Next,
    response::Response,
};
use std::future::{ready, Future};
use tower_sessions::Session;

use crate::auth::verify_token;
use crate::middleware::ApiError;
use shared::Role;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: String,
    pub username: String,
    pub role: Role,
    pub exp: usize,
}

/// 用户 Claims（存储在 Session 中）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Claims {
    pub user_id: String,
    pub username: String,
    pub role: Role,
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
    AuthUser {
        user_id: claims.user_id,
        username: claims.username,
        role: claims.role,
        exp: claims.exp,
    }
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

/// 认证中间件 - 从 Session 中获取用户信息
pub async fn auth_middleware(mut req: Request, next: Next) -> Result<Response, ApiError> {
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
            let auth_user = auth_user_from_session_claims(claims.clone());
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
