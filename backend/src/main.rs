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
use chrono::{DateTime, Utc};
use uuid::Uuid;

mod state;
mod utils;
mod handlers;
mod scanners;
mod database;
mod config;
mod entities;
mod migration;

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
    cloud_providers::{
        get_cloud_provider_configs, get_cloud_provider_config, create_cloud_provider_config,
        update_cloud_provider_config, delete_cloud_provider_config, test_cloud_provider_connection,
        get_cloud_provider_options, get_active_cloud_provider_configs,
    },
    cloud_zones::{
        get_cloud_zones, get_cloud_zone, create_cloud_zone, update_cloud_zone, delete_cloud_zone,
    },
    cloud_platforms::{
        get_cloud_platforms, get_cloud_platform, create_cloud_platform, update_cloud_platform,
        delete_cloud_platform, get_platforms_by_zone,
    },
    cloud_service_assets::{
        get_cloud_service_assets,
        get_cloud_service_stats,
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

    // 初始化数据库 (使用 SeaORM)
    let db_config = config::DatabaseConfig::from_env();
    database::init_db(&db_config.connection_string)
        .await
        .expect("Failed to initialize database");
    println!("Database initialized: type={}, url={}",
        db_config.db_type,
        db_config.connection_string.chars().take(50).collect::<String>() // 只显示前50个字符避免泄露密码
    );

    // 获取数据库连接
    let db_conn = database::get_db().expect("Database not initialized");

    // 从数据库加载数据 (使用 SeaORM)
    let loaded_users = load_users_from_db(&db_conn).await;
    println!("Loaded {} users from database", loaded_users.len());
    let initial_users = if loaded_users.is_empty() { initial_users } else { loaded_users };

    let loaded_audit_logs = load_audit_logs_from_db(&db_conn).await;
    println!("Loaded {} audit logs from database", loaded_audit_logs.len());

    // Load cloud zones and platforms from database
    let loaded_cloud_zones = load_cloud_zones_from_db(&db_conn).await;
    println!("Loaded {} cloud zones from database", loaded_cloud_zones.len());

    let loaded_cloud_platforms = load_cloud_platforms_from_db(&db_conn).await;
    println!("Loaded {} cloud platforms from database", loaded_cloud_platforms.len());

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
        audit_logs: Arc::new(StdMutex::new(loaded_audit_logs)),
        advanced_tasks: Arc::new(StdMutex::new(vec![])),
        custom_roles: Arc::new(StdMutex::new(vec![])),
        scan_manager: Arc::new(TokioMutex::new(scan_manager)),
        password_policy: Arc::new(StdMutex::new(PasswordPolicy::default())),
        password_history: Arc::new(StdMutex::new(vec![])),
        cloud_zones: Arc::new(StdMutex::new(loaded_cloud_zones)),
        cloud_platforms: Arc::new(StdMutex::new(loaded_cloud_platforms)),
    };

    // 数据加载辅助函数 (使用 SeaORM)
    async fn load_users_from_db(conn: &sea_orm::DatabaseConnection) -> Vec<shared::User> {
        match database::get_users_with_conn(conn).await {
            Ok(users) => users,
            Err(e) => {
                eprintln!("Error loading users from database: {}", e);
                vec![]
            }
        }
    }

    async fn load_audit_logs_from_db(conn: &sea_orm::DatabaseConnection) -> Vec<shared::AuditLog> {
        use crate::database::get_audit_logs_with_conn as get_audit_logs_db;
        match get_audit_logs_db(conn, Some(1000)).await {
            Ok(logs) => logs.into_iter().map(|db_log| {
                shared::AuditLog {
                    id: db_log.id,
                    user_id: db_log.user_id,
                    username: db_log.username,
                    action: db_log.action,
                    target: db_log.target,
                    details: db_log.details,
                    timestamp: chrono::DateTime::parse_from_rfc3339(&db_log.timestamp)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                }
            }).collect(),
            Err(e) => {
                eprintln!("Error loading audit logs from database: {}", e);
                vec![]
            }
        }
    }

    async fn load_cloud_zones_from_db(conn: &sea_orm::DatabaseConnection) -> Vec<shared::CloudZone> {
        use crate::database::get_all_cloud_zones;
        use chrono::TimeZone;
        match get_all_cloud_zones(conn).await {
            Ok(zones) => zones.into_iter().map(|db| shared::CloudZone {
                id: Some(db.id),
                zone_name: db.zone_name.clone(),
                zone_code: db.zone_code.clone(),
                description: db.description.clone(),
                created_at: chrono::DateTime::parse_from_rfc3339(&db.created_at)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            }).collect(),
            Err(e) => {
                eprintln!("Error loading cloud zones from database: {}", e);
                vec![]
            }
        }
    }

    async fn load_cloud_platforms_from_db(conn: &sea_orm::DatabaseConnection) -> Vec<shared::CloudPlatform> {
        use crate::database::get_all_cloud_platforms;
        use chrono::TimeZone;
        match get_all_cloud_platforms(conn).await {
            Ok(platforms) => platforms.into_iter().map(|db| shared::CloudPlatform {
                id: Some(db.id),
                zone_id: db.zone_id,
                platform_name: db.platform_name.clone(),
                platform_code: db.platform_code.clone(),
                description: db.description.clone(),
                created_at: chrono::DateTime::parse_from_rfc3339(&db.created_at)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            }).collect(),
            Err(e) => {
                eprintln!("Error loading cloud platforms from database: {}", e);
                vec![]
            }
        }
    }

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
        // Cloud Provider Configuration (云厂商对接)
        .route("/api/cloud-provider-configs", get(get_cloud_provider_configs).post(create_cloud_provider_config))
        .route("/api/cloud-provider-configs/options", get(get_cloud_provider_options))
        .route("/api/cloud-provider-configs/active", get(get_active_cloud_provider_configs))
        .route("/api/cloud-provider-configs/:id", get(get_cloud_provider_config).put(update_cloud_provider_config).delete(delete_cloud_provider_config))
        .route("/api/cloud-provider-configs/:id/test", post(test_cloud_provider_connection))
        // Cloud Zones (云区管理)
        .route("/api/cloud-zones", get(get_cloud_zones).post(create_cloud_zone))
        .route("/api/cloud-zones/:id", get(get_cloud_zone).put(update_cloud_zone).delete(delete_cloud_zone))
        // Cloud Platforms (云平台管理)
        .route("/api/cloud-platforms", get(get_cloud_platforms).post(create_cloud_platform))
        .route("/api/cloud-platforms/:id", get(get_cloud_platform).put(update_cloud_platform).delete(delete_cloud_platform))
        .route("/api/cloud-platforms/zone/:zone_id", get(get_platforms_by_zone))
        // Cloud Service Assets (云服务资产 - 统一视图)
        .route("/api/cloud-service-assets", get(get_cloud_service_assets))
        .route("/api/cloud-service-assets/stats", get(get_cloud_service_stats))
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
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3003));
    println!("Backend listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
