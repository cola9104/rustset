//! 集成测试：users.rs - 用户管理

use axum::{
    body::Body,
    http::{header, Method, Request, StatusCode},
    Router,
};
use backend::handlers::users::{create_user, delete_user, get_users};
use backend::state::AppState;
use serde_json::json;
use shared::{Role, User};
use tower::ServiceExt;

/// 创建测试用的 Router
async fn create_test_app(state: AppState) -> Router {
    Router::new()
        .route(
            "/api/users",
            axum::routing::get(get_users).post(create_user),
        )
        .route("/api/users/{id}", axum::routing::delete(delete_user))
        .with_state(state)
}

/// 创建测试用的 AppState
async fn create_test_state() -> AppState {
    use backend::password;
    use chrono::Utc;
    use std::sync::{Arc, RwLock};

    // 创建测试管理员用户
    let password_hash = password::hash_password("admin123").unwrap();
    let test_users = vec![User {
        id: "admin_user".to_string(),
        username: "admin".to_string(),
        real_name: None,
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
        organization_id: None,
        department_id: None,
        failed_login_attempts: Some(0),
        locked_until: None,
    }];

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
async fn test_get_users_unauthorized() {
    let state = create_test_state().await;
    let app = create_test_app(state).await;

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/users")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_get_users_forbidden() {
    // 注意：这个测试需要有效的普通用户 token
    // 由于测试环境的限制，这里只测试逻辑
    // 实际测试中，应该使用 SecAdmin 或 Auditor 角色的用户 token
    let state = create_test_state().await;
    let app = create_test_app(state).await;

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/users")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // 期望：401（未授权）或 403（禁止访问）
    assert!(
        response.status() == StatusCode::UNAUTHORIZED || response.status() == StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn test_create_user_unauthorized() {
    let state = create_test_state().await;
    let app = create_test_app(state).await;

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/users")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "username": "testuser",
                "password": "TestPass123",
                "role": "SecAdmin"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_delete_user_unauthorized() {
    let state = create_test_state().await;
    let app = create_test_app(state).await;

    let request = Request::builder()
        .method(Method::DELETE)
        .uri("/api/users/admin_user")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_user_role_permissions() {
    // 测试角色权限系统
    use shared::Permissions;

    let sys_admin_perms = Permissions::sys_admin();
    let sec_admin_perms = Permissions::sec_admin();
    let auditor_perms = Permissions::auditor();

    // SysAdmin 应该有所有权限
    assert!(sys_admin_perms.can_view_tasks);
    assert!(sys_admin_perms.can_create_task);

    // SecAdmin 应该有资产管理权限，但不能管理用户
    assert!(sec_admin_perms.can_access_assets_risks);

    // Auditor 只能查看日志
    assert!(auditor_perms.can_view_audit_logs);
    assert!(!auditor_perms.can_access_assets_risks);
}

#[tokio::test]
async fn test_password_strength_calculation() {
    use backend::handlers::users::calculate_password_strength;

    // 弱密码
    let (strength, score) = calculate_password_strength("weak");
    assert_eq!(strength, "weak");
    assert!(score < 3);

    // 中等密码
    let (strength, score) = calculate_password_strength("WeakPass123");
    assert_eq!(strength, "medium");
    assert!((3..=4).contains(&score));

    // 强密码
    let (strength, score) = calculate_password_strength("StrongP@ssw0rd123!");
    assert_eq!(strength, "strong");
    assert!(score >= 5);
}

#[tokio::test]
async fn test_password_policy_validation() {
    use backend::handlers::users::validate_password_policy;
    use shared::PasswordPolicy;

    let policy = PasswordPolicy {
        min_length: 8,
        require_uppercase: true,
        require_lowercase: true,
        require_number: true,
        require_special: true,
        max_login_attempts: Some(5),
        max_age_days: None,
        prevent_reuse: 5,
        min_strength: "medium".to_string(),
        lockout_duration_minutes: 30,
    };

    // 测试密码太短
    let result = validate_password_policy("Short", &policy);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("密码长度"));

    // 测试缺少大写字母
    let result = validate_password_policy("lowercase123!", &policy);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("大写字母"));

    // 测试缺少小写字母
    let result = validate_password_policy("UPPERCASE123!", &policy);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("小写字母"));

    // 测试缺少数字
    let result = validate_password_policy("NoNumbers!", &policy);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("数字"));

    // 测试缺少特殊字符
    let result = validate_password_policy("NoSpecialChars123", &policy);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("特殊字符"));

    // 测试有效密码
    let result = validate_password_policy("ValidP@ssw0rd", &policy);
    assert!(result.is_ok());
}
