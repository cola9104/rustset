use axum::{
    routing::{get, post, delete, put},
    Router,
};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex as StdMutex};
use tokio::sync::Mutex as TokioMutex;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
};
use shared::{
    Asset, NetworkZone, PortInfo, ZoneConfig,
    User, Role, PasswordPolicy,
};
use chrono::Utc;
use uuid::Uuid;

mod state;
mod utils;
mod handlers;
mod scanners;

use state::AppState;
use handlers::{
    auth::login,
    users::{get_users, create_user, delete_user, update_user_permissions, change_password, get_password_policy, update_password_policy, get_current_user_info},
    logs::get_audit_logs,
    assets::{get_assets, add_asset, update_asset, delete_asset, add_asset_port, update_asset_port, delete_asset_port, bind_port},
    tasks::{get_tasks, create_task, update_task, delete_task, trigger_scan},
    risks::{get_risks, resolve_risk, update_risk_status},
    zones::{get_zones, create_zone, update_zone, delete_zone},
    business_resources::{get_business_resources, create_business_resource, update_business_resource, delete_business_resource},
    advanced_scan::{
        execute_advanced_scan, get_advanced_tasks, get_advanced_task,
        cancel_advanced_scan, delete_advanced_scan, export_scan_results,
        get_scan_engines_status, scan_progress_stream,
    },
};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Initial mock data
    let initial_assets = vec![
        Asset {
            id: Some(1),
            name: "Gateway Server".to_string(),
            ip: "192.168.1.1".to_string(),
            zone: NetworkZone::Intranet,
            ports: vec![
                PortInfo { 
                    port: 80, 
                    is_open: true, 
                    service: Some("HTTP".to_string()), 
                    version: Some("1.18.0".to_string()),
                    banner: Some("Server: nginx/1.18.0".to_string()),
                    is_bound: true,
                    system_name: Some("Gateway Portal".to_string()),
                    middleware: Some("Nginx".to_string()),
                    created_by: Some("system".to_string()),
                    updated_by: None,
                },
            ],
            last_scanned: Some(Utc::now()),
            contact_person: Some("Admin".to_string()),
            contact_phone: Some("13800000000".to_string()),
            created_by: Some("system".to_string()),
            updated_by: None,
            owner: Some("IT Dept".to_string()),
            weight: 80,
            labels: vec!["Core".to_string(), "Gateway".to_string()],
            os: Some("Linux".to_string()),
            device_type: Some("Server".to_string()),
        }
    ];

    // Default Users
    let initial_users = vec![
        User {
            id: Uuid::new_v4().to_string(),
            username: "admin".to_string(),
            password: "admin".to_string(), // Plain text for demo
            role: Role::SysAdmin,
            permissions: Some(shared::Permissions::sys_admin()),
            created_at: Utc::now(),
            password_changed_at: Some(Utc::now()),
            password_strength: Some("weak".to_string()),
            force_password_change: Some(false),
            last_login_at: None,
            email: Some("admin@rustset.local".to_string()),
            phone: None,
            status: Some("active".to_string()),
            failed_login_attempts: Some(0),
            locked_until: None,
        },
        User {
            id: Uuid::new_v4().to_string(),
            username: "sec".to_string(),
            password: "sec".to_string(),
            role: Role::SecAdmin,
            permissions: Some(shared::Permissions::sec_admin()),
            created_at: Utc::now(),
            password_changed_at: Some(Utc::now()),
            password_strength: Some("weak".to_string()),
            force_password_change: Some(false),
            last_login_at: None,
            email: Some("sec@rustset.local".to_string()),
            phone: None,
            status: Some("active".to_string()),
            failed_login_attempts: Some(0),
            locked_until: None,
        },
        User {
            id: Uuid::new_v4().to_string(),
            username: "audit".to_string(),
            password: "audit".to_string(),
            role: Role::Auditor,
            permissions: Some(shared::Permissions::auditor()),
            created_at: Utc::now(),
            password_changed_at: Some(Utc::now()),
            password_strength: Some("weak".to_string()),
            force_password_change: Some(false),
            last_login_at: None,
            email: Some("audit@rustset.local".to_string()),
            phone: None,
            status: Some("active".to_string()),
            failed_login_attempts: Some(0),
            locked_until: None,
        },
    ];

    // 初始化扫描管理器
    let scan_manager = scanners::engine::ScanManager::new().await.ok();

    let state = AppState {
        assets: Arc::new(StdMutex::new(initial_assets)),
        tasks: Arc::new(StdMutex::new(vec![])),
        risks: Arc::new(StdMutex::new(vec![])),
        zones: Arc::new(StdMutex::new(vec![
            ZoneConfig { id: "1".to_string(), name: "Intranet".to_string(), cidr: "192.168.0.0/16".to_string(), priority: 10 },
            ZoneConfig { id: "2".to_string(), name: "DMZ".to_string(), cidr: "10.0.0.0/8".to_string(), priority: 20 },
        ])),
        users: Arc::new(StdMutex::new(initial_users)),
        audit_logs: Arc::new(StdMutex::new(vec![])),
        advanced_tasks: Arc::new(StdMutex::new(vec![])),
        cloud_assets: Arc::new(StdMutex::new(vec![])),
        custom_roles: Arc::new(StdMutex::new(vec![])),
        scan_manager: Arc::new(TokioMutex::new(scan_manager)),
        password_policy: Arc::new(StdMutex::new(PasswordPolicy::default())),
        password_history: Arc::new(StdMutex::new(vec![])),
    };

    let app = Router::new()
        // Auth
        .route("/api/login", post(login))
        .route("/api/users/me", get(get_current_user_info)) // 必须在 :id 之前
        .route("/api/users", get(get_users).post(create_user))
        .route("/api/users/:id", delete(delete_user))
        .route("/api/users/:id/permissions", put(update_user_permissions))
        .route("/api/users/change-password", post(change_password))
        .route("/api/password-policy", get(get_password_policy).put(update_password_policy))
        .route("/api/logs", get(get_audit_logs))
        // Roles
        .route("/api/roles", get(handlers::roles::get_roles).post(handlers::roles::create_role))
        .route("/api/roles/:id", get(handlers::roles::get_role).put(handlers::roles::update_role).delete(handlers::roles::delete_role))
        // Assets
        .route("/api/assets", get(get_assets).post(add_asset))
        .route("/api/assets/:id", delete(delete_asset).put(update_asset))
        .route("/api/assets/:id/ports", post(add_asset_port))
        .route("/api/assets/:id/ports/:port", delete(delete_asset_port).put(update_asset_port))
        .route("/api/assets/:ip/ports/:port/bind", post(bind_port))
        // Tasks & Risks
        .route("/api/tasks", get(get_tasks).post(create_task))
        .route("/api/tasks/:id", delete(delete_task).put(update_task))
        .route("/api/risks", get(get_risks))
        .route("/api/risks/:id/resolve", post(resolve_risk))
        .route("/api/risks/:id/status/:status", post(update_risk_status))
        // Zones
        .route("/api/zones", get(get_zones).post(create_zone))
        .route("/api/zones/:id", delete(delete_zone).put(update_zone))
        // Business Resources (业务申请)
        .route("/api/business-resources", get(get_business_resources).post(create_business_resource))
        .route("/api/business-resources/:id", put(update_business_resource).delete(delete_business_resource))
        // Advanced Scanning (新增高级扫描 API)
        .route("/api/scan/advanced", post(execute_advanced_scan))
        .route("/api/scan/advanced/tasks", get(get_advanced_tasks))
        .route("/api/scan/advanced/tasks/:id", get(get_advanced_task).delete(delete_advanced_scan))
        .route("/api/scan/advanced/tasks/:id/cancel", post(cancel_advanced_scan))
        .route("/api/scan/advanced/tasks/:id/export", get(export_scan_results))
        .route("/api/scan/advanced/engines/status", get(get_scan_engines_status))
        .route("/api/scan/advanced/tasks/:id/progress", get(scan_progress_stream))
        // IP Zones (TODO: implement handlers)
        // .route("/api/ip-zones", get(get_ip_zones).post(create_ip_zone))
        // .route("/api/ip-zones/:id", get(get_ip_zone).delete(delete_ip_zone).put(update_ip_zone))
        // .route("/api/ip-zones/find/:ip", get(find_zone_by_ip))
        // Port Details (端口详细信息表) (TODO: implement handlers)
        // .route("/api/port-details", get(get_port_details).post(create_port_detail))
        // .route("/api/port-details/:id", get(get_port_detail).put(update_port_detail).delete(delete_port_detail))
        // .route("/api/port-details/batch-bind", post(batch_bind_ports))
        // Scanners (扫描器) (TODO: implement handlers)
        // .route("/api/scan-ip", post(scan_ip))
        // .route("/api/batch-scan-ips", post(batch_scan_ips))
        // .route("/api/scan-results", get(get_scan_results))
        // .route("/api/ip-scan-results", get(get_ip_scan_results))
        // Cloud Assets (Multi-Cloud Management) - 暂时禁用
        // .route("/api/cloud-assets", get(get_cloud_assets).post(create_cloud_asset))
        // .route("/api/cloud-assets/stats", get(get_cloud_asset_stats))
        // .route("/api/cloud-assets/sync", post(sync_cloud_assets))
        // .route("/api/cloud-assets/:id", get(get_cloud_asset).put(update_cloud_asset).delete(delete_cloud_asset))
        // Scan
        .route("/api/scan", post(trigger_scan))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3003));
    println!("Backend listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
