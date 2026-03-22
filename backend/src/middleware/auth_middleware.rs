use axum::{
    extract::{FromRequestParts, Request},
    http::{header::AUTHORIZATION, request::Parts, HeaderMap, Method},
    middleware::Next,
    response::Response,
};
use std::future::{Future, ready};
use shared::Role;

use crate::auth::{verify_token, Claims};
use crate::middleware::ApiError;

#[derive(Debug, Clone)]
pub struct AuthUser {
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
        ready(parts
            .extensions
            .get::<AuthUser>()
            .cloned()
            .ok_or_else(|| ApiError::unauthorized("Unauthorized")))
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
            | ("GET", "/api/metrics")
            | ("GET", "/health")
            | ("GET", "/ready")
            | ("GET", "/metrics")
    )
}

fn extract_bearer_token(headers: &HeaderMap) -> Result<&str, ApiError> {
    let auth_value = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| ApiError::unauthorized("Missing Authorization header"))?;

    auth_value
        .trim()
        .strip_prefix("Bearer ")
        .or_else(|| auth_value.trim().strip_prefix("bearer "))
        .ok_or_else(|| ApiError::unauthorized("Invalid authorization scheme"))
}

pub async fn auth_middleware(
    mut req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let method = req.method().clone();
    let path = req.uri().path().to_string();

    if method == Method::OPTIONS || is_public_path(&method, &path) {
        return Ok(next.run(req).await);
    }

    let token = extract_bearer_token(req.headers())?;
    let claims = verify_token(token)?;

    req.extensions_mut().insert(AuthUser::from(claims));

    Ok(next.run(req).await)
}
