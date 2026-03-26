//! 集成测试：port_details.rs - 端口详细信息管理

use axum::{
    body::Body,
    http::{header, Method, Request, StatusCode},
    middleware::from_fn_with_state,
    Router,
};
use backend::auth::generate_token;
use backend::middleware::auth_middleware::auth_middleware;
use backend::state::AppState;
use chrono::Utc;
use shared::{Role, User};
use tower::ServiceExt;

/// 创建测试用的 Router
async fn create_test_app(state: AppState) -> Router {
    std::env::set_var("JWT_SECRET", "test-jwt-secret");

    Router::new()
        .route(
            "/api/port-details",
            axum::routing::get(backend::handlers::port_details::get_port_details)
                .post(backend::handlers::port_details::create_port_detail),
        )
        .layer(from_fn_with_state(state.clone(), auth_middleware))
        .with_state(state)
}

fn create_test_user() -> User {
    User {
        id: "test_user_1".to_string(),
        username: "admin".to_string(),
        real_name: None,
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
        organization_id: None,
        department_id: None,
        failed_login_attempts: Some(0),
        locked_until: None,
    }
}

fn auth_header_value() -> String {
    std::env::set_var("JWT_SECRET", "test-jwt-secret");
    format!("Bearer {}", generate_token(&create_test_user()).unwrap())
}

/// 创建测试用的 AppState
async fn create_test_state() -> AppState {
    use std::sync::{Arc, RwLock};

    let test_users = vec![create_test_user()];

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
        port_details: Arc::new(RwLock::new(vec![])),
        scanners: Arc::new(RwLock::new(vec![])),
        scan_results: Arc::new(RwLock::new(vec![])),
    }
}

#[tokio::test]
async fn test_get_port_details() {
    let state = create_test_state().await;
    let app = create_test_app(state).await;

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/port-details")
        .header(header::AUTHORIZATION, auth_header_value())
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_create_port_detail_with_service_detection() {
    let state = create_test_state().await;
    let app = create_test_app(state).await;

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/port-details")
        .header(header::AUTHORIZATION, auth_header_value())
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"port":22,"protocol":"tcp"}"#))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(result["port"], 22);
    assert_eq!(result["service"], "SSH");
}

#[tokio::test]
async fn test_create_port_detail_http() {
    let state = create_test_state().await;
    let app = create_test_app(state).await;

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/port-details")
        .header(header::AUTHORIZATION, auth_header_value())
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"port":443,"protocol":"tcp"}"#))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(result["port"], 443);
    assert_eq!(result["service"], "HTTPS");
}
