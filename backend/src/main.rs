use axum::{
    routing::{get, post, delete},
    Router,
};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
};
use shared::{
    Asset, NetworkZone, PortInfo, ZoneConfig,
    User, Role,
};
use chrono::Utc;
use uuid::Uuid;

mod state;
mod utils;
mod handlers;

use state::AppState;
use handlers::{
    auth::login,
    users::{get_users, create_user, delete_user},
    logs::get_audit_logs,
    assets::{get_assets, add_asset, update_asset, delete_asset, add_asset_port, update_asset_port, delete_asset_port, bind_port},
    tasks::{get_tasks, create_task, update_task, delete_task, trigger_scan},
    risks::{get_risks, resolve_risk, update_risk_status},
    zones::{get_zones, create_zone, update_zone, delete_zone},
    cloud_assets::{get_cloud_assets, get_cloud_asset, create_cloud_asset, update_cloud_asset, delete_cloud_asset, get_cloud_asset_stats, sync_cloud_assets},
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
            created_at: Utc::now(),
        },
        User {
            id: Uuid::new_v4().to_string(),
            username: "sec".to_string(),
            password: "sec".to_string(),
            role: Role::SecAdmin,
            created_at: Utc::now(),
        },
        User {
            id: Uuid::new_v4().to_string(),
            username: "audit".to_string(),
            password: "audit".to_string(),
            role: Role::Auditor,
            created_at: Utc::now(),
        },
    ];

    let state = AppState {
        assets: Arc::new(Mutex::new(initial_assets)),
        tasks: Arc::new(Mutex::new(vec![])),
        risks: Arc::new(Mutex::new(vec![])),
        zones: Arc::new(Mutex::new(vec![
            ZoneConfig { id: "1".to_string(), name: "Intranet".to_string(), cidr: "192.168.0.0/16".to_string(), priority: 10 },
            ZoneConfig { id: "2".to_string(), name: "DMZ".to_string(), cidr: "10.0.0.0/8".to_string(), priority: 20 },
        ])),
        users: Arc::new(Mutex::new(initial_users)),
        audit_logs: Arc::new(Mutex::new(vec![])),
    };

    let app = Router::new()
        // Auth
        .route("/api/login", post(login))
        .route("/api/users", get(get_users).post(create_user))
        .route("/api/users/:id", delete(delete_user))
        .route("/api/logs", get(get_audit_logs))
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
        // Cloud Assets (Multi-Cloud Management)
        .route("/api/cloud-assets", get(get_cloud_assets).post(create_cloud_asset))
        .route("/api/cloud-assets/stats", get(get_cloud_asset_stats))
        .route("/api/cloud-assets/sync", post(sync_cloud_assets))
        .route("/api/cloud-assets/:id", get(get_cloud_asset).put(update_cloud_asset).delete(delete_cloud_asset))
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
