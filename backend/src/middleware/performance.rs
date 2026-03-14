//! Performance Monitoring Middleware

use axum::{
    body::Body,
    http::{Request, Response},
    middleware::Next,
};
use std::time::Instant;

/// 性能监控中间件
pub async fn performance_monitoring(
    req: Request<Body>,
    next: Next,
) -> Response<Body> {
    let start = Instant::now();
    let method = req.method().to_string();
    let path = req.uri().path().to_string();
    
    let response = next.run(req).await;
    
    let duration = start.elapsed();
    let status = response.status().as_u16();
    
    // 记录慢请求 (> 1s)
    if duration.as_millis() > 1000 {
        tracing::warn!(
            "Slow request: {} {} - {}ms - status {}",
            method,
            path,
            duration.as_millis(),
            status
        );
    } else {
        tracing::debug!(
            "Request: {} {} - {}ms - status {}",
            method,
            path,
            duration.as_millis(),
            status
        );
    }
    
    response
}
