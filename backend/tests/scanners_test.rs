//! 集成测试：scanners.rs - 扫描器接口

use axum::{
    body::Body,
    http::{header, Method, Request, StatusCode},
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
            "/api/scan-ip",
            axum::routing::post(backend::handlers::scanners::scan_ip),
        )
        .route(
            "/api/batch-scan-ips",
            axum::routing::post(backend::handlers::scanners::batch_scan_ips),
        )
        .route(
            "/api/scan-results",
            axum::routing::get(backend::handlers::scanners::get_scan_results),
        )
        .route(
            "/api/scanners",
            axum::routing::get(backend::handlers::scanners::get_scanners)
                .post(backend::handlers::scanners::create_scanner),
        )
        .route(
            "/api/scanners/{id}",
            axum::routing::put(backend::handlers::scanners::update_scanner)
                .delete(backend::handlers::scanners::delete_scanner),
        )
        .with_state(state)
        .layer(axum::middleware::from_fn(auth_middleware))
}

fn create_test_user(username: &str) -> User {
    match username {
        "admin" => User {
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
        },
        "secadmin" => User {
            id: "test_user_2".to_string(),
            username: "secadmin".to_string(),
            real_name: None,
            password: "test_hash".to_string(),
            role: Role::SecAdmin,
            permissions: Some(shared::Permissions::sec_admin()),
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
        },
        "auditor" => User {
            id: "test_user_3".to_string(),
            username: "auditor".to_string(),
            real_name: None,
            password: "test_hash".to_string(),
            role: Role::Auditor,
            permissions: Some(shared::Permissions::auditor()),
            created_at: Utc::now(),
            password_changed_at: Some(Utc::now()),
            password_strength: Some("medium".to_string()),
            force_password_change: Some(false),
            last_login_at: None,
            email: None,
            phone: None,
            status: Some("active".to_string()),
            organization_id: None,
            department_id: None,
            failed_login_attempts: Some(0),
            locked_until: None,
        },
        other => panic!("unknown test user: {}", other),
    }
}

fn auth_header_value(username: &str) -> String {
    std::env::set_var("JWT_SECRET", "test-jwt-secret");
    format!(
        "Bearer {}",
        generate_token(&create_test_user(username)).unwrap()
    )
}

/// 创建测试用的 AppState
async fn create_test_state() -> AppState {
    use std::sync::{Arc, RwLock};

    let test_users = vec![
        create_test_user("admin"),
        create_test_user("secadmin"),
        create_test_user("auditor"),
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
        port_details: Arc::new(RwLock::new(vec![])),
        scanners: Arc::new(RwLock::new(vec![])),
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
        .header(header::AUTHORIZATION, auth_header_value("admin"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"target":"192.168.1.1"}"#))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
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
        .header(header::AUTHORIZATION, auth_header_value("admin"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            r#"{"targets":["192.168.1.1","192.168.1.2","192.168.1.3"]}"#,
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
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
        .header(header::AUTHORIZATION, auth_header_value("admin"))
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
        .header(header::AUTHORIZATION, auth_header_value("admin"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            r#"{"target":"192.168.1.1","ports":[22,80,443]}"#,
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(result["ports"].is_array());
    // 应该有 3 个端口结果
    assert_eq!(result["ports"].as_array().unwrap().len(), 3);
}

// ============ Scanner Configuration Tests ============

#[tokio::test]
async fn test_get_scanners_empty() {
    let state = create_test_state().await;
    let app = create_test_app(state).await;

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/scanners")
        .header(header::AUTHORIZATION, auth_header_value("admin"))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(result.is_array());
    assert_eq!(result.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_create_scanner_as_admin() {
    let state = create_test_state().await;
    let app = create_test_app(state).await;

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/scanners")
        .header(header::AUTHORIZATION, auth_header_value("admin"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"name":"test-scanner","scanner_type":"rustscan","enabled":true,"config":{"timeout":30}}"#))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(result["name"], "test-scanner");
    assert_eq!(result["scanner_type"], "rustscan");
    assert!(result["id"].is_string());
}

#[tokio::test]
async fn test_create_scanner_as_secadmin() {
    let state = create_test_state().await;
    let app = create_test_app(state).await;

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/scanners")
        .header(header::AUTHORIZATION, auth_header_value("secadmin"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"name":"sec-scanner","scanner_type":"nmap","enabled":true,"config":{"timeout":60}}"#))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_create_scanner_as_auditor_forbidden() {
    let state = create_test_state().await;
    let app = create_test_app(state).await;

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/scanners")
        .header(header::AUTHORIZATION, auth_header_value("auditor"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            r#"{"name":"test-scanner","scanner_type":"rustscan","config":{}}"#,
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_create_scanner_unauthorized() {
    let state = create_test_state().await;
    let app = create_test_app(state).await;

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/scanners")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            r#"{"name":"test-scanner","scanner_type":"rustscan","config":{}}"#,
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_get_scanners_after_create() {
    let state = create_test_state().await;
    let app = create_test_app(state).await;

    // Create first scanner
    let request1 = Request::builder()
        .method(Method::POST)
        .uri("/api/scanners")
        .header(header::AUTHORIZATION, auth_header_value("admin"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            r#"{"name":"scanner-1","scanner_type":"rustscan","config":{}}"#,
        ))
        .unwrap();
    app.clone().oneshot(request1).await.unwrap();

    // Create second scanner
    let request2 = Request::builder()
        .method(Method::POST)
        .uri("/api/scanners")
        .header(header::AUTHORIZATION, auth_header_value("admin"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            r#"{"name":"scanner-2","scanner_type":"nmap","config":{}}"#,
        ))
        .unwrap();
    app.clone().oneshot(request2).await.unwrap();

    // Get all scanners
    let request3 = Request::builder()
        .method(Method::GET)
        .uri("/api/scanners")
        .header(header::AUTHORIZATION, auth_header_value("admin"))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request3).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(result.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_update_scanner_as_admin() {
    let state = create_test_state().await;
    let app = create_test_app(state).await;

    // Create a scanner first
    let create_request = Request::builder()
        .method(Method::POST)
        .uri("/api/scanners")
        .header(header::AUTHORIZATION, auth_header_value("admin"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            r#"{"name":"original-name","scanner_type":"rustscan","config":{}}"#,
        ))
        .unwrap();

    let create_response = app.clone().oneshot(create_request).await.unwrap();
    let create_body = axum::body::to_bytes(create_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let create_result: serde_json::Value = serde_json::from_slice(&create_body).unwrap();
    let scanner_id = create_result["id"].as_str().unwrap();

    // Update the scanner
    let update_request = Request::builder()
        .method(Method::PUT)
        .uri(format!("/api/scanners/{}", scanner_id))
        .header(header::AUTHORIZATION, auth_header_value("admin"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"name":"updated-name","enabled":false}"#))
        .unwrap();

    let update_response = app.clone().oneshot(update_request).await.unwrap();
    assert_eq!(update_response.status(), StatusCode::OK);

    let update_body = axum::body::to_bytes(update_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let update_result: serde_json::Value = serde_json::from_slice(&update_body).unwrap();

    assert_eq!(update_result["name"], "updated-name");
    assert_eq!(update_result["enabled"], false);
}

#[tokio::test]
async fn test_update_scanner_not_found() {
    let state = create_test_state().await;
    let app = create_test_app(state).await;

    let request = Request::builder()
        .method(Method::PUT)
        .uri("/api/scanners/non-existent-id")
        .header(header::AUTHORIZATION, auth_header_value("admin"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"name":"updated"}"#))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_scanner_as_admin() {
    let state = create_test_state().await;
    let app = create_test_app(state).await;

    // Create a scanner first
    let create_request = Request::builder()
        .method(Method::POST)
        .uri("/api/scanners")
        .header(header::AUTHORIZATION, auth_header_value("admin"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            r#"{"name":"to-delete","scanner_type":"rustscan","config":{}}"#,
        ))
        .unwrap();

    let create_response = app.clone().oneshot(create_request).await.unwrap();
    let create_body = axum::body::to_bytes(create_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let create_result: serde_json::Value = serde_json::from_slice(&create_body).unwrap();
    let scanner_id = create_result["id"].as_str().unwrap();

    // Delete the scanner
    let delete_request = Request::builder()
        .method(Method::DELETE)
        .uri(format!("/api/scanners/{}", scanner_id))
        .header(header::AUTHORIZATION, auth_header_value("admin"))
        .body(Body::empty())
        .unwrap();

    let delete_response = app.clone().oneshot(delete_request).await.unwrap();
    assert_eq!(delete_response.status(), StatusCode::OK);

    // Verify it's deleted
    let get_request = Request::builder()
        .method(Method::GET)
        .uri("/api/scanners")
        .header(header::AUTHORIZATION, auth_header_value("admin"))
        .body(Body::empty())
        .unwrap();

    let get_response = app.clone().oneshot(get_request).await.unwrap();
    let get_body = axum::body::to_bytes(get_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let get_result: serde_json::Value = serde_json::from_slice(&get_body).unwrap();

    assert_eq!(get_result.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_create_scanner_invalid_type() {
    let state = create_test_state().await;
    let app = create_test_app(state).await;

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/scanners")
        .header(header::AUTHORIZATION, auth_header_value("admin"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            r#"{"name":"invalid-scanner","scanner_type":"invalid_type","config":{}}"#,
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    // 应该返回 422 (ValidationError) 或 400
    assert!(
        response.status() == StatusCode::UNPROCESSABLE_ENTITY
            || response.status() == StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn test_update_scanner_invalid_type() {
    let state = create_test_state().await;
    let app = create_test_app(state).await;

    // Create a scanner first
    let create_request = Request::builder()
        .method(Method::POST)
        .uri("/api/scanners")
        .header(header::AUTHORIZATION, auth_header_value("admin"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            r#"{"name":"test","scanner_type":"rustscan","config":{}}"#,
        ))
        .unwrap();

    let create_response = app.clone().oneshot(create_request).await.unwrap();
    let create_body = axum::body::to_bytes(create_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let create_result: serde_json::Value = serde_json::from_slice(&create_body).unwrap();
    let scanner_id = create_result["id"].as_str().unwrap();

    // Try to update with invalid type
    let update_request = Request::builder()
        .method(Method::PUT)
        .uri(format!("/api/scanners/{}", scanner_id))
        .header(header::AUTHORIZATION, auth_header_value("admin"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"scanner_type":"invalid_type"}"#))
        .unwrap();

    let update_response = app.oneshot(update_request).await.unwrap();
    // 应该返回 422 (ValidationError) 或 400
    assert!(
        update_response.status() == StatusCode::UNPROCESSABLE_ENTITY
            || update_response.status() == StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn test_delete_scanner_not_found() {
    let state = create_test_state().await;
    let app = create_test_app(state).await;

    let request = Request::builder()
        .method(Method::DELETE)
        .uri("/api/scanners/non-existent-id")
        .header(header::AUTHORIZATION, auth_header_value("admin"))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_update_scanner_as_auditor_forbidden() {
    let state = create_test_state().await;
    let app = create_test_app(state).await;

    // Create a scanner first as admin
    let create_request = Request::builder()
        .method(Method::POST)
        .uri("/api/scanners")
        .header(header::AUTHORIZATION, auth_header_value("admin"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            r#"{"name":"test","scanner_type":"rustscan","config":{}}"#,
        ))
        .unwrap();

    let create_response = app.clone().oneshot(create_request).await.unwrap();
    let create_body = axum::body::to_bytes(create_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let create_result: serde_json::Value = serde_json::from_slice(&create_body).unwrap();
    let scanner_id = create_result["id"].as_str().unwrap();

    // Try to update as auditor
    let update_request = Request::builder()
        .method(Method::PUT)
        .uri(format!("/api/scanners/{}", scanner_id))
        .header(header::AUTHORIZATION, auth_header_value("auditor"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"name":"hacked"}"#))
        .unwrap();

    let update_response = app.clone().oneshot(update_request).await.unwrap();
    assert_eq!(update_response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_delete_scanner_as_auditor_forbidden() {
    let state = create_test_state().await;
    let app = create_test_app(state).await;

    // Create a scanner first as admin
    let create_request = Request::builder()
        .method(Method::POST)
        .uri("/api/scanners")
        .header(header::AUTHORIZATION, auth_header_value("admin"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            r#"{"name":"test","scanner_type":"rustscan","config":{}}"#,
        ))
        .unwrap();

    let create_response = app.clone().oneshot(create_request).await.unwrap();
    let create_body = axum::body::to_bytes(create_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let create_result: serde_json::Value = serde_json::from_slice(&create_body).unwrap();
    let scanner_id = create_result["id"].as_str().unwrap();

    // Try to delete as auditor
    let delete_request = Request::builder()
        .method(Method::DELETE)
        .uri(format!("/api/scanners/{}", scanner_id))
        .header(header::AUTHORIZATION, auth_header_value("auditor"))
        .body(Body::empty())
        .unwrap();

    let delete_response = app.clone().oneshot(delete_request).await.unwrap();
    assert_eq!(delete_response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_get_scanners_unauthorized() {
    let state = create_test_state().await;
    let app = create_test_app(state).await;

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/scanners")
        // No Authorization header
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_update_scanner_as_secadmin() {
    let state = create_test_state().await;
    let app = create_test_app(state).await;

    // Create a scanner first as admin
    let create_request = Request::builder()
        .method(Method::POST)
        .uri("/api/scanners")
        .header(header::AUTHORIZATION, auth_header_value("admin"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            r#"{"name":"test","scanner_type":"rustscan","config":{}}"#,
        ))
        .unwrap();

    let create_response = app.clone().oneshot(create_request).await.unwrap();
    let create_body = axum::body::to_bytes(create_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let create_result: serde_json::Value = serde_json::from_slice(&create_body).unwrap();
    let scanner_id = create_result["id"].as_str().unwrap();

    // Update as secadmin
    let update_request = Request::builder()
        .method(Method::PUT)
        .uri(format!("/api/scanners/{}", scanner_id))
        .header(header::AUTHORIZATION, auth_header_value("secadmin"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            r#"{"name":"updated-by-secadmin","enabled":false}"#,
        ))
        .unwrap();

    let update_response = app.clone().oneshot(update_request).await.unwrap();
    assert_eq!(update_response.status(), StatusCode::OK);

    let update_body = axum::body::to_bytes(update_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let update_result: serde_json::Value = serde_json::from_slice(&update_body).unwrap();
    assert_eq!(update_result["name"], "updated-by-secadmin");
}

#[tokio::test]
async fn test_delete_scanner_as_secadmin() {
    let state = create_test_state().await;
    let app = create_test_app(state).await;

    // Create a scanner first as admin
    let create_request = Request::builder()
        .method(Method::POST)
        .uri("/api/scanners")
        .header(header::AUTHORIZATION, auth_header_value("admin"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            r#"{"name":"test","scanner_type":"rustscan","config":{}}"#,
        ))
        .unwrap();

    let create_response = app.clone().oneshot(create_request).await.unwrap();
    let create_body = axum::body::to_bytes(create_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let create_result: serde_json::Value = serde_json::from_slice(&create_body).unwrap();
    let scanner_id = create_result["id"].as_str().unwrap();

    // Delete as secadmin
    let delete_request = Request::builder()
        .method(Method::DELETE)
        .uri(format!("/api/scanners/{}", scanner_id))
        .header(header::AUTHORIZATION, auth_header_value("secadmin"))
        .body(Body::empty())
        .unwrap();

    let delete_response = app.clone().oneshot(delete_request).await.unwrap();
    assert_eq!(delete_response.status(), StatusCode::OK);

    // Verify it's deleted
    let get_request = Request::builder()
        .method(Method::GET)
        .uri("/api/scanners")
        .header(header::AUTHORIZATION, auth_header_value("admin"))
        .body(Body::empty())
        .unwrap();

    let get_response = app.clone().oneshot(get_request).await.unwrap();
    let get_body = axum::body::to_bytes(get_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let get_result: serde_json::Value = serde_json::from_slice(&get_body).unwrap();
    assert_eq!(get_result.as_array().unwrap().len(), 0);
}
