//! CORS Middleware Configuration

use axum::http::{header, HeaderValue, Method};
use tower_http::cors::{Any, CorsLayer};

/// 创建 CORS 中间件
/// 根据环境变量 FRONTEND_URL 配置允许的来源
pub fn create_cors_layer() -> CorsLayer {
    // 从环境变量读取前端 URL，默认为 permissive（开发环境）
    let frontend_url = std::env::var("FRONTEND_URL").ok();

    if let Some(url) = frontend_url {
        // 生产环境：限制特定来源
        let origins: Vec<Result<HeaderValue, _>> = url
            .split(',')
            .map(|s| s.trim().parse::<HeaderValue>())
            .collect();

        // 如果所有 URL 都有效，使用它们
        if origins.iter().all(|r| r.is_ok()) {
            let valid_origins: Vec<HeaderValue> =
                origins.into_iter().filter_map(|r| r.ok()).collect();

            return CorsLayer::new()
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
                .allow_credentials(true);
        }
    }

    // 开发环境：允许所有来源
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
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE, header::ACCEPT])
        .allow_credentials(false)
}
