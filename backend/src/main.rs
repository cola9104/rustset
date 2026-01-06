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
    risks::{get_risks, resolve_risk},
    zones::{get_zones, create_zone, update_zone, delete_zone},
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
        // Zones
        .route("/api/zones", get(get_zones).post(create_zone))
        .route("/api/zones/:id", delete(delete_zone).put(update_zone))
        .route("/api/scan", post(trigger_scan))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3003));
    println!("Backend listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
