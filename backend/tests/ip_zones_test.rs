//! 集成测试：ip_zones.rs - IP 区域管理

use axum::{
    body::Body,
    http::{header, Method, Request, StatusCode},
    middleware::from_fn_with_state,
    Router,
};
use backend::auth::generate_token;
use backend::handlers::ip_zones::{create_ip_zone, delete_ip_zone, find_zone_by_ip, get_ip_zones};
use backend::middleware::auth_middleware::auth_middleware;
use backend::state::AppState;
use chrono::Utc;
use shared::{Role, User, ZoneConfig};
use tower::ServiceExt;

/// 创建测试用的 Router
async fn create_test_app(state: AppState) -> Router {
    std::env::set_var("JWT_SECRET", "test-jwt-secret");

    Router::new()
        .route(
            "/api/ip-zones",
            axum::routing::get(get_ip_zones).post(create_ip_zone),
        )
        .route("/api/ip-zones/{id}", axum::routing::delete(delete_ip_zone))
        .route("/api/ip-zones/find", axum::routing::get(find_zone_by_ip))
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
        zones: Arc::new(RwLock::new(vec![
            ZoneConfig {
                id: "1".to_string(),
                name: "Intranet".to_string(),
                cidr: "192.168.0.0/16".to_string(),
                priority: 10,
                cloud_platform_id: None,
                cloud_platform_name: None,
                machine_room_id: None,
                machine_room_name: None,
            },
            ZoneConfig {
                id: "2".to_string(),
                name: "DMZ".to_string(),
                cidr: "10.0.0.0/8".to_string(),
                priority: 20,
                cloud_platform_id: None,
                cloud_platform_name: None,
                machine_room_id: None,
                machine_room_name: None,
            },
        ])),
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
async fn test_get_ip_zones() {
    let state = create_test_state().await;
    let app = create_test_app(state).await;

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/ip-zones")
        .header(header::AUTHORIZATION, auth_header_value())
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_find_zone_by_ip_intranet() {
    let state = create_test_state().await;
    let app = create_test_app(state).await;

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/ip-zones/find?ip=192.168.1.100")
        .header(header::AUTHORIZATION, auth_header_value())
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(result["zone"], "Intranet");
    assert_eq!(result["matched_cidr"], "192.168.0.0/16");
}

#[tokio::test]
async fn test_find_zone_by_ip_internet() {
    let state = create_test_state().await;
    let app = create_test_app(state).await;

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/ip-zones/find?ip=8.8.8.8")
        .header(header::AUTHORIZATION, auth_header_value())
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(result["zone"], "Internet");
    assert!(result["matched_cidr"].is_null());
}

#[tokio::test]
async fn test_create_ip_zone_unauthorized() {
    let state = create_test_state().await;
    let app = create_test_app(state).await;

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/ip-zones")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            r#"{"name":"Test","cidr":"172.16.0.0/12","priority":5}"#,
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
