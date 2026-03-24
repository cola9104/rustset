//! CORS Middleware Configuration

use axum::http::{header, HeaderValue, Method};
use tower_http::cors::CorsLayer;

fn parse_origins(origins: &str) -> Option<Vec<HeaderValue>> {
    let parsed: Vec<Result<HeaderValue, _>> = origins
        .split(',')
        .map(|origin| origin.trim().parse::<HeaderValue>())
        .collect();

    if parsed.iter().all(|origin| origin.is_ok()) {
        Some(parsed.into_iter().filter_map(Result::ok).collect())
    } else {
        None
    }
}

/// 创建 CORS 中间件
/// 根据环境变量 FRONTEND_URL 配置允许的来源
pub fn create_cors_layer() -> CorsLayer {
    let valid_origins = std::env::var("FRONTEND_URL")
        .ok()
        .and_then(|url| parse_origins(&url))
        .or_else(|| parse_origins("http://127.0.0.1:8080,http://localhost:8080"))
        .unwrap_or_default();

    CorsLayer::new()
        .allow_origin(valid_origins)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
            Method::PATCH,
        ])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE, header::ACCEPT])
        .allow_credentials(true)
}
