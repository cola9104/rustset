use axum::{
    extract::{State, Json, Path},
    routing::{get, post, delete, put},
    Router,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use std::net::{SocketAddr, IpAddr};
use std::sync::{Arc, Mutex};
use tower_http::cors::CorsLayer;
use shared::{Asset, NetworkZone, PortInfo, ScanRequest, PortBindingRequest, Task, TaskStatus, CreateTaskRequest, Risk, RiskStatus, ZoneConfig};
use tokio::time::{sleep, Duration};
use ipnetwork::IpNetwork;

#[derive(Clone)]
struct AppState {
    assets: Arc<Mutex<Vec<Asset>>>,
    tasks: Arc<Mutex<Vec<Task>>>,
    risks: Arc<Mutex<Vec<Risk>>>,
    zones: Arc<Mutex<Vec<ZoneConfig>>>,
}

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
                    owner: Some("Admin".to_string()),
                    system_name: Some("Gateway Portal".to_string()),
                    middleware: Some("Nginx".to_string()),
                },
            ],
            last_scanned: Some(chrono::Utc::now()),
        }
    ];

    let state = AppState {
        assets: Arc::new(Mutex::new(initial_assets)),
        tasks: Arc::new(Mutex::new(vec![])),
        risks: Arc::new(Mutex::new(vec![])),
        zones: Arc::new(Mutex::new(vec![
            ZoneConfig { id: "1".to_string(), name: "Intranet".to_string(), cidr: "192.168.0.0/16".to_string(), priority: 10 },
            ZoneConfig { id: "2".to_string(), name: "DMZ".to_string(), cidr: "10.0.0.0/8".to_string(), priority: 20 },
        ])),
    };

    // Periodic scanner task (simplified)
    let _scanner_state = state.clone();
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(30)).await;
            // ... (keep existing simple scanner logic if needed, or rely on tasks)
        }
    });

    let app = Router::new()
        .route("/api/assets", get(get_assets).post(add_asset))
        .route("/api/assets/:id", delete(delete_asset).put(update_asset))
        .route("/api/scan", post(trigger_scan))
        .route("/api/assets/:ip/ports/:port/bind", post(bind_port))
        // New Task & Risk Routes
        .route("/api/tasks", get(get_tasks).post(create_task))
        .route("/api/tasks/:id", delete(delete_task).put(update_task))
        .route("/api/risks", get(get_risks))
        .route("/api/risks/:id/resolve", post(resolve_risk))
        // Zone Routes
        .route("/api/zones", get(get_zones).post(create_zone))
        .route("/api/zones/:id", delete(delete_zone).put(update_zone))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3003));
    println!("Backend listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn determine_zone(ip_str: &str, zones: &[ZoneConfig]) -> NetworkZone {
    if let Ok(ip) = ip_str.parse::<IpAddr>() {
        let mut matching_zones: Vec<&ZoneConfig> = zones.iter()
            .filter(|z| {
                if let Ok(net) = z.cidr.parse::<IpNetwork>() {
                    net.contains(ip)
                } else {
                    false
                }
            })
            .collect();
        
        matching_zones.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        if let Some(best_match) = matching_zones.first() {
            return match best_match.name.as_str() {
                "Internet" => NetworkZone::Internet,
                "DMZ" => NetworkZone::DMZ,
                "Intranet" => NetworkZone::Intranet,
                other => NetworkZone::Custom(other.to_string()),
            };
        }
    }
    NetworkZone::Internet // Default fallback if no match (e.g. public IP)
}

async fn get_assets(State(state): State<AppState>) -> Json<Vec<Asset>> {
    let assets = state.assets.lock().unwrap();
    Json(assets.clone())
}

async fn add_asset(State(state): State<AppState>, Json(mut asset): Json<Asset>) -> Json<Asset> {
    // Determine zone automatically
    {
        let zones = state.zones.lock().unwrap();
        asset.zone = determine_zone(&asset.ip, &zones);
    } // Release lock

    let mut assets = state.assets.lock().unwrap();
    let new_id = assets.len() as i32 + 1;
    asset.id = Some(new_id);
    assets.push(asset.clone());
    Json(asset)
}

async fn update_asset(State(state): State<AppState>, Path(id): Path<i32>, Json(req): Json<Asset>) -> Json<Option<Asset>> {
    // Lock zones first to avoid deadlock with update_zone (which locks zones then assets)
    let zones = state.zones.lock().unwrap();
    let mut assets = state.assets.lock().unwrap();
    
    if let Some(asset) = assets.iter_mut().find(|a| a.id == Some(id)) {
        asset.name = req.name;
        // Check if IP changed, might need to re-evaluate zone
        if asset.ip != req.ip {
             asset.ip = req.ip.clone();
             // Re-evaluate zone
             asset.zone = determine_zone(&asset.ip, &zones);
        }
        return Json(Some(asset.clone()));
    }
    Json(None)
}

async fn delete_asset(State(state): State<AppState>, Path(id): Path<i32>) -> Json<String> {
    let mut assets = state.assets.lock().unwrap();
    assets.retain(|a| a.id != Some(id));
    Json("Deleted".to_string())
}

async fn update_zone(State(state): State<AppState>, Path(id): Path<String>, Json(req): Json<ZoneConfig>) -> Json<Option<ZoneConfig>> {
    let mut zones = state.zones.lock().unwrap();
    if let Some(zone) = zones.iter_mut().find(|z| z.id == id) {
        zone.name = req.name;
        zone.cidr = req.cidr;
        zone.priority = req.priority;
        return Json(Some(zone.clone()));
    }
    Json(None)
}

// Keep old scan trigger for backward compatibility or simple scans
async fn trigger_scan(State(state): State<AppState>, Json(req): Json<ScanRequest>) -> Json<String> {
    let target_ip = req.target_ip.clone();
    // Simulate finding open ports immediately for demo
    let mut assets = state.assets.lock().unwrap();
    if let Some(asset) = assets.iter_mut().find(|a| a.ip == target_ip) {
        for port in req.ports {
            if !asset.ports.iter().any(|p| p.port == port) {
                asset.ports.push(PortInfo {
                    port,
                    is_open: true,
                    service: Some("Unknown".to_string()),
                    is_bound: false,
                    owner: None,
                    system_name: None,
                    middleware: None,
                });
            }
        }
        asset.last_scanned = Some(chrono::Utc::now());
    }
    Json(format!("Scan initiated for {}", target_ip))
}

async fn bind_port(
    State(state): State<AppState>,
    Path((ip, port)): Path<(String, u16)>,
    Json(req): Json<PortBindingRequest>
) -> Json<Option<Asset>> {
    let mut assets = state.assets.lock().unwrap();
    if let Some(asset) = assets.iter_mut().find(|a| a.ip == ip) {
        if let Some(p) = asset.ports.iter_mut().find(|p| p.port == port) {
            p.is_bound = true;
            p.owner = Some(req.owner);
            p.system_name = Some(req.system_name);
            p.middleware = Some(req.middleware);
        }
        return Json(Some(asset.clone()));
    }
    Json(None)
}

// Zone Handlers
async fn get_zones(State(state): State<AppState>) -> Json<Vec<ZoneConfig>> {
    let zones = state.zones.lock().unwrap();
    Json(zones.clone())
}

async fn create_zone(State(state): State<AppState>, Json(req): Json<ZoneConfig>) -> Json<ZoneConfig> {
    let mut zones = state.zones.lock().unwrap();
    let mut new_zone = req;
    if new_zone.id.is_empty() {
        new_zone.id = uuid::Uuid::new_v4().to_string();
    }
    zones.push(new_zone.clone());
    Json(new_zone)
}

async fn delete_zone(State(state): State<AppState>, Path(id): Path<String>) -> Result<Json<String>, (StatusCode, String)> {
    let mut zones = state.zones.lock().unwrap();
    
    // Find the zone to get its name (for checking against assets)
    let zone_name = if let Some(z) = zones.iter().find(|z| z.id == id) {
        z.name.clone()
    } else {
        return Ok(Json("Zone not found".to_string()));
    };

    // Check if any asset belongs to this zone
    let assets = state.assets.lock().unwrap();
    
    // Logic: Check if asset.zone matches the zone we are deleting.
    // Asset.zone is NetworkZone enum.
    // If zone_name is "Internet", "DMZ", "Intranet", we check for those variants.
    // Otherwise check for Custom(name).
    
    let target_zone_enum = match zone_name.as_str() {
        "Internet" => NetworkZone::Internet,
        "DMZ" => NetworkZone::DMZ,
        "Intranet" => NetworkZone::Intranet,
        other => NetworkZone::Custom(other.to_string()),
    };
    
    if assets.iter().any(|a| a.zone == target_zone_enum) {
        return Err((StatusCode::BAD_REQUEST, "Cannot delete zone: One or more assets are assigned to this zone.".to_string()));
    }

    zones.retain(|z| z.id != id);
    Ok(Json("Deleted".to_string()))
}

// --- New Task & Risk Handlers ---

async fn get_tasks(State(state): State<AppState>) -> Json<Vec<Task>> {
    let tasks = state.tasks.lock().unwrap();
    Json(tasks.clone())
}

async fn get_risks(State(state): State<AppState>) -> Json<Vec<Risk>> {
    let risks = state.risks.lock().unwrap();
    Json(risks.clone())
}

async fn create_task(State(state): State<AppState>, Json(req): Json<CreateTaskRequest>) -> Json<Task> {
    let mut tasks = state.tasks.lock().unwrap();
    let task_id = uuid::Uuid::new_v4().to_string();
    
    let new_task = Task {
        id: task_id.clone(),
        name: req.name.clone(),
        target: req.target.clone(),
        status: TaskStatus::Pending,
        start_time: Some(chrono::Utc::now().to_string()),
        end_time: None,
        found_assets: 0,
        found_risks: 0,
        port_policy: req.port_policy.clone(),
        domain_brute: req.domain_brute,
        service_detection: req.service_detection,
        os_detection: req.os_detection,
        site_identify: req.site_identify,
    };
    tasks.push(new_task.clone());

    // Spawn async task simulation
    let task_state = state.clone();
    let target = req.target.clone();
    let tid = task_id.clone();
    let policy = req.port_policy.clone();
    let _service_det = req.service_detection;
    let _os_det = req.os_detection;
    
    tokio::spawn(async move {
        // 1. Pending -> Running
        sleep(Duration::from_secs(2)).await;
        {
            let mut tasks = task_state.tasks.lock().unwrap();
            if let Some(t) = tasks.iter_mut().find(|t| t.id == tid) {
                t.status = TaskStatus::Running;
            }
        }

        // 2. Simulate Scanning (Port Scan -> Service Discovery)
        sleep(Duration::from_secs(5)).await;
        
        let mut found_assets_count = 0;
        let mut found_risks_count = 0;

        // Mock Logic: If target is an IP, check if it exists or create it
        {
            let mut assets = task_state.assets.lock().unwrap();
            let mut risks = task_state.risks.lock().unwrap();
            
            // Check if asset exists, if not create
            let asset_exists = assets.iter().any(|a| a.ip == target);
            if !asset_exists {
                // Determine zone
                let zone = {
                    let zones = task_state.zones.lock().unwrap();
                    determine_zone(&target, &zones)
                };

                let new_id = (assets.len() as i32) + 1;
                assets.push(Asset {
                    id: Some(new_id),
                    name: format!("Scanned Host {}", target),
                    ip: target.clone(),
                    zone,
                    ports: vec![],
                    last_scanned: Some(chrono::Utc::now()),
                });
                found_assets_count += 1;
            }

            if let Some(asset) = assets.iter_mut().find(|a| a.ip == target) {
                // Simulate finding generic ports based on policy
                let common_ports = match policy.as_str() {
                    "TEST" => vec![80, 443],
                    "TOP100" => vec![80, 443, 8080, 22, 3306, 21, 23, 25, 53, 110, 445, 1433, 1521, 3389, 5432, 6379, 27017],
                    _ => vec![80, 443, 8080, 22, 3306, 21, 23, 25, 53, 110, 445, 1433, 1521, 3389, 5432, 6379, 27017, 9000, 9200, 5000], // Simplified ALL/TOP1000
                };

                for port in common_ports {
                    // 50% chance to be open
                    if rand::random::<f32>() > 0.5 {
                        if !asset.ports.iter().any(|p| p.port == port) {
                            asset.ports.push(PortInfo {
                                port,
                                is_open: true,
                                service: Some("Unknown".to_string()),
                                is_bound: false,
                                owner: None,
                                system_name: None,
                                middleware: None,
                            });
                            
                            // Create a risk for unbound port 8080 or 3306
                            if port == 8080 || port == 3306 {
                                risks.push(Risk {
                                        id: uuid::Uuid::new_v4().to_string(),
                                        asset_ip: target.clone(),
                                        port,
                                        severity: "High".to_string(),
                                        description: format!("Port {} is open and unbound/unauthenticated", port),
                                        solution: Some("Close port or enable authentication".to_string()),
                                        status: RiskStatus::Open,
                                    });
                                found_risks_count += 1;
                            }
                        }
                    }
                }
                asset.last_scanned = Some(chrono::Utc::now());
            }
        }

        // 3. Completed
        {
            let mut tasks = task_state.tasks.lock().unwrap();
            if let Some(t) = tasks.iter_mut().find(|t| t.id == tid) {
                t.status = TaskStatus::Completed;
                t.end_time = Some(chrono::Utc::now().to_string());
                t.found_assets = found_assets_count;
                t.found_risks = found_risks_count;
            }
        }
    });

    Json(new_task)
}

async fn delete_task(State(state): State<AppState>, Path(id): Path<String>) -> Json<String> {
    let mut tasks = state.tasks.lock().unwrap();
    tasks.retain(|t| t.id != id);
    Json("Deleted".to_string())
}

async fn update_task(State(state): State<AppState>, Path(id): Path<String>, Json(req): Json<CreateTaskRequest>) -> Json<Option<Task>> {
    let mut tasks = state.tasks.lock().unwrap();
    if let Some(task) = tasks.iter_mut().find(|t| t.id == id) {
        task.name = req.name;
        task.target = req.target;
        task.port_policy = req.port_policy;
        task.domain_brute = req.domain_brute;
        task.service_detection = req.service_detection;
        task.os_detection = req.os_detection;
        task.site_identify = req.site_identify;
        return Json(Some(task.clone()));
    }
    Json(None)
}

async fn resolve_risk(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Json<Option<Risk>> {
    let mut risks = state.risks.lock().unwrap();
    if let Some(risk) = risks.iter_mut().find(|r| r.id == id) {
        risk.status = RiskStatus::Resolved;
        return Json(Some(risk.clone()));
    }
    Json(None)
}
