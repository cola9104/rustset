/// Rate Limiting 集成测试
///
/// 测试 Rate Limiting 中间件在实际 HTTP 请求场景中的行为
use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use tower::ServiceExt;

/// 创建测试应用
async fn create_test_app() -> Router {
    use std::sync::Arc;

    use backend::handlers::health::{health_check, readiness_check};
    use backend::middleware::rate_limit::{
        init_rate_limiter, rate_limit_middleware, RateLimitConfig,
    };
    use backend::state::AppState;

    // 初始化 Rate Limiter（较小的限制用于测试）
    let rate_limit_config = RateLimitConfig {
        requests_per_minute: 5,     // 每分钟5次请求
        block_duration_seconds: 10, // 阻塞10秒
    };
    init_rate_limiter(rate_limit_config);

    // 创建一个简单的 AppState
    let state = AppState {
        assets: Arc::new(std::sync::RwLock::new(vec![])),
        tasks: Arc::new(std::sync::RwLock::new(vec![])),
        risks: Arc::new(std::sync::RwLock::new(vec![])),
        zones: Arc::new(std::sync::RwLock::new(vec![])),
        users: Arc::new(std::sync::RwLock::new(vec![])),
        audit_logs: Arc::new(std::sync::RwLock::new(vec![])),
        advanced_tasks: Arc::new(std::sync::RwLock::new(vec![])),
        custom_roles: Arc::new(std::sync::RwLock::new(vec![])),
        scan_manager: Arc::new(tokio::sync::RwLock::new(None)),
        password_policy: Arc::new(std::sync::RwLock::new(shared::PasswordPolicy::default())),
        password_history: Arc::new(std::sync::RwLock::new(vec![])),
        port_details: Arc::new(std::sync::RwLock::new(vec![])),
        scanners: Arc::new(std::sync::RwLock::new(vec![])),
        scan_results: Arc::new(std::sync::RwLock::new(vec![])),
    };

    Router::new()
        .route("/health", axum::routing::get(health_check))
        .route("/ready", axum::routing::get(readiness_check))
        .layer(axum::middleware::from_fn(rate_limit_middleware))
        .with_state(state)
}

#[tokio::test]
async fn test_rate_limit_within_bounds() {
    let app = create_test_app().await;

    // 发送3个请求，应该在限制内
    for i in 0..3 {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .header("X-Forwarded-For", "198.51.100.10")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::OK,
            "Request {} should succeed",
            i
        );
    }
}

#[tokio::test]
async fn test_rate_limit_exceeded() {
    let app = create_test_app().await;

    // 发送超过限制的请求（6次，限制是5次）
    let mut success_count = 0;
    let mut rate_limited_count = 0;

    for _i in 0..6 {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .header("X-Forwarded-For", "198.51.100.11")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        if response.status() == StatusCode::OK {
            success_count += 1;
        } else if response.status() == StatusCode::TOO_MANY_REQUESTS {
            rate_limited_count += 1;
        }
    }

    // 应该有5次成功，第6次被限流
    assert_eq!(success_count, 5, "Should have 5 successful requests");
    assert_eq!(rate_limited_count, 1, "Should have 1 rate-limited request");
}

#[tokio::test]
async fn test_rate_limit_with_different_ips() {
    let app = create_test_app().await;

    // 模拟来自不同 IP 的请求
    let ips = vec!["192.168.1.1", "192.168.1.2", "192.168.1.3"];

    for ip in &ips {
        // 每个IP发送6个请求（超过限制）
        let mut success_count = 0;
        let mut rate_limited_count = 0;

        for _ in 0..6 {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri("/health")
                        .header("X-Forwarded-For", *ip)
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();

            if response.status() == StatusCode::OK {
                success_count += 1;
            } else if response.status() == StatusCode::TOO_MANY_REQUESTS {
                rate_limited_count += 1;
            }
        }

        // 每个IP应该独立计算限流
        assert_eq!(
            success_count, 5,
            "IP {} should have 5 successful requests",
            ip
        );
        assert_eq!(
            rate_limited_count, 1,
            "IP {} should have 1 rate-limited request",
            ip
        );
    }
}

#[tokio::test]
async fn test_rate_limit_x_real_ip_header() {
    let app = create_test_app().await;

    // 使用 X-Real-IP 头
    let mut success_count = 0;
    let mut rate_limited_count = 0;

    for _ in 0..6 {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .header("X-Real-IP", "10.0.0.1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        if response.status() == StatusCode::OK {
            success_count += 1;
        } else if response.status() == StatusCode::TOO_MANY_REQUESTS {
            rate_limited_count += 1;
        }
    }

    assert_eq!(success_count, 5, "Should have 5 successful requests");
    assert_eq!(rate_limited_count, 1, "Should have 1 rate-limited request");
}

#[tokio::test]
async fn test_rate_limit_cf_connecting_ip_header() {
    let app = create_test_app().await;

    // 使用 CF-Connecting-IP 头（Cloudflare）
    let mut success_count = 0;
    let mut rate_limited_count = 0;

    for _ in 0..6 {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .header("CF-Connecting-IP", "203.0.113.1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        if response.status() == StatusCode::OK {
            success_count += 1;
        } else if response.status() == StatusCode::TOO_MANY_REQUESTS {
            rate_limited_count += 1;
        }
    }

    assert_eq!(success_count, 5, "Should have 5 successful requests");
    assert_eq!(rate_limited_count, 1, "Should have 1 rate-limited request");
}
