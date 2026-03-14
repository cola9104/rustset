//! 集成测试：scanners.rs - 扫描器接口

use axum::{
    body::Body,
    http::{header, Method, Request, StatusCode},
    Router,
};
use tower::ServiceExt;
use backend::state::AppState;

/// 创建测试用的 Router
async fn create_test_app(state: AppState) -> Router {
    Router::new()
        .route("/api/scan-ip", axum::routing::post(backend::handlers::scanners::scan_ip))
        .route("/api/batch-scan-ips", axum::routing::post(backend::handlers::scanners::batch_scan_ips))
        .route("/api/scan-results", axum::routing::get(backend::handlers::scanners::get_scan_results))
        .with_state(state)
}

/// 创建测试用的 AppState
async fn create_test_state() -> AppState {
    use std::sync::{Arc, RwLock};
    use shared::{User, Role};
    use chrono::Utc;

    let test_users = vec![
        User {
            id: "test_user_1".to_string(),
            username: "admin".to_string(),
            password: "test_hash".to_string(),
            role: Role::SysAdmin,
            permissions: Some(shared::Permissions::sys_admin()),
            created_at: Utc::now(),
            password_changed_at: Some(Utc::now()),
            password_strength: Some("strong".to_string()),
            force_password_change: Some(false),
            last_login_at: None,
            email: None,
            phone: None,
            status: Some("active".to_string()),
            failed_login_attempts: Some(0),
            locked_until: None,
        },
    ];

    AppState {
        assets: Arc::new(RwLock::new(vec![])),
        tasks: Arc::new(RwLock::new(vec![])),
        risks: Arc::new(RwLock::new(vec![])),
        zones: Arc::new(RwLock::new(vec![])),
        users: Arc::new(RwLock::new(test_users)),
        audit_logs: Arc::new(RwLock::new(vec![])),
        advanced_tasks: Arc::new(RwLock::new(vec![])),
        custom_roles: Arc::new(RwLock::new(vec![])),
        scan_manager: Arc::new(tokio::sync::RwLock::new(None)),
        password_policy: Arc::new(RwLock::new(shared::PasswordPolicy::default())),
        password_history: Arc::new(RwLock::new(vec![])),
        cloud_zones: Arc::new(RwLock::new(vec![])),
        cloud_platforms: Arc::new(RwLock::new(vec![])),
        port_details: Arc::new(RwLock::new(vec![])),
        scan_results: Arc::new(RwLock::new(vec![])),
    }
}

#[tokio::test]
async fn test_scan_ip() {
    let state = create_test_state().await;
    let app = create_test_app(state).await;

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/scan-ip")
        .header("Authorization", "admin")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"target":"192.168.1.1"}"#))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(result["target"], "192.168.1.1");
    assert_eq!(result["status"], "completed");
}

#[tokio::test]
async fn test_batch_scan_ips() {
    let state = create_test_state().await;
    let app = create_test_app(state).await;

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/batch-scan-ips")
        .header("Authorization", "admin")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"targets":["192.168.1.1","192.168.1.2","192.168.1.3"]}"#))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert!(result.is_array());
    assert_eq!(result.as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn test_get_scan_results() {
    let state = create_test_state().await;
    let app = create_test_app(state).await;

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/scan-results")
        .header("Authorization", "admin")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_scan_with_custom_ports() {
    let state = create_test_state().await;
    let app = create_test_app(state).await;

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/scan-ip")
        .header("Authorization", "admin")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"target":"192.168.1.1","ports":[22,80,443]}"#))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert!(result["ports"].is_array());
    // 应该有 3 个端口结果
    assert_eq!(result["ports"].as_array().unwrap().len(), 3);
}
