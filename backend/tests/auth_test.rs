//! 集成测试：auth.rs - 登录、权限验证

use axum::{
    body::Body,
    http::{header, Method, Request, StatusCode},
    Router,
};
use serde_json::json;
use tower::ServiceExt;
use backend::state::AppState;
use backend::handlers::auth::login;
use shared::LoginRequest;

/// 创建测试用的 Router
async fn create_test_app(state: AppState) -> Router {
    Router::new()
        .route("/api/login", axum::routing::post(login))
        .with_state(state)
}

/// 创建测试用的 AppState
async fn create_test_state() -> AppState {
    use std::sync::{Arc, RwLock};
    use shared::{User, Role};
    use chrono::Utc;
    use backend::password;

    // 创建测试用户
    let password_hash = password::hash_password("admin123").unwrap();
    let test_users = vec![
        User {
            id: "test_user_1".to_string(),
            username: "admin".to_string(),
            password: password_hash,
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
        password_policy: Arc::new(RwLock::new(shared::PasswordPolicy {
            min_length: 8,
            require_uppercase: true,
            require_lowercase: true,
            require_number: true,
            require_special: true,
            max_login_attempts: Some(5),
            lockout_duration_minutes: 30,
            prevent_reuse: 5,
            max_age_days: Some(90),
            min_strength: "medium".to_string(),
        })),
        password_history: Arc::new(RwLock::new(vec![])),
        cloud_zones: Arc::new(RwLock::new(vec![])),
        cloud_platforms: Arc::new(RwLock::new(vec![])),
    }
}

#[tokio::test]
async fn test_login_success() {
    let state = create_test_state().await;
    let app = create_test_app(state).await;

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json!({
            "username": "admin",
            "password": "admin123"
        }).to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_login_invalid_credentials() {
    let state = create_test_state().await;
    let app = create_test_app(state).await;

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json!({
            "username": "admin",
            "password": "wrongpassword"
        }).to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_login_user_not_found() {
    let state = create_test_state().await;
    let app = create_test_app(state).await;

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json!({
            "username": "nonexistent",
            "password": "password"
        }).to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_login_missing_fields() {
    let state = create_test_state().await;
    let app = create_test_app(state).await;

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json!({
            "username": "admin"
        }).to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Axum returns 422 for missing required fields during JSON deserialization
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}
