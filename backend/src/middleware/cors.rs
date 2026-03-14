//! CORS Middleware Configuration

use tower_http::cors::{Any, CorsLayer};
use axum::http::{Method, header};

/// 创建 CORS 中间件
pub fn create_cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
            Method::PATCH,
        ])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::ACCEPT,
        ])
        .allow_credentials(false)
}
