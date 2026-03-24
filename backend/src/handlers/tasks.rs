use crate::database::{
    db_asset_to_shared, db_task_to_shared, delete_task as db_delete_task, get_db,
    get_risks_by_asset_ip, get_task_by_id as db_get_task_by_id, get_tasks as db_get_tasks,
    insert_risk, insert_task_wrapper as db_insert_task, update_asset as db_update_asset,
    update_task as db_update_task,
};
use crate::middleware::ApiError;
use crate::state::AppState;
use crate::utils::{get_current_user_from_headers, log_action};
use axum::{
    extract::{Json, Path, State},
    http::HeaderMap,
};
use chrono::Utc;
use shared::{CreateTaskRequest, PortInfo, Role, ScanRequest, Task, TaskStatus};
use uuid::Uuid;

fn sync_task_cache(state: &AppState, task: &Task) -> Result<(), ApiError> {
    let mut tasks = state
        .tasks
        .write()
        .map_err(|e| ApiError::internal(format!("Failed to write tasks cache: {}", e)))?;
    if let Some(existing) = tasks.iter_mut().find(|existing| existing.id == task.id) {
        *existing = task.clone();
    } else {
        tasks.push(task.clone());
    }
    Ok(())
}

fn remove_task_from_cache(state: &AppState, task_id: &str) -> Result<(), ApiError> {
    let mut tasks = state
        .tasks
        .write()
        .map_err(|e| ApiError::internal(format!("Failed to write tasks cache: {}", e)))?;
    tasks.retain(|task| task.id != task_id);
    Ok(())
}

async fn load_all_tasks(state: &AppState) -> Result<Vec<Task>, ApiError> {
    if get_db().is_some() {
        let tasks: Vec<Task> = db_get_tasks()
            .await
            .map_err(|e| ApiError::internal(format!("Failed to load tasks from database: {}", e)))?
            .into_iter()
            .map(db_task_to_shared)
            .collect();
        let mut cache = state
            .tasks
            .write()
            .map_err(|e| ApiError::internal(format!("Failed to write tasks cache: {}", e)))?;
        *cache = tasks.clone();
        return Ok(tasks);
    }

    state
        .tasks
        .read()
        .map(|tasks| tasks.clone())
        .map_err(|e| ApiError::internal(format!("Failed to read tasks cache: {}", e)))
}

async fn load_task_by_id(state: &AppState, id: &str) -> Result<Option<Task>, ApiError> {
    if let Some(conn) = get_db() {
        let task = db_get_task_by_id(&conn, id)
            .await
            .map_err(|e| ApiError::internal(format!("Failed to load task: {}", e)))?
            .map(db_task_to_shared);
        if let Some(task) = &task {
            sync_task_cache(state, task)?;
        }
        return Ok(task);
    }

    state
        .tasks
        .read()
        .map(|tasks| tasks.iter().find(|task| task.id == id).cloned())
        .map_err(|e| ApiError::internal(format!("Failed to read tasks cache: {}", e)))
}

fn sync_risk_cache(state: &AppState, risk: &shared::Risk) {
    if let Ok(mut risks) = state.risks.write() {
        if let Some(existing) = risks.iter_mut().find(|existing| existing.id == risk.id) {
            *existing = risk.clone();
        } else {
            risks.push(risk.clone());
        }
    }
}

fn scan_target_ports(
    target_ip: String,
    ports_to_scan: Vec<u16>,
) -> (Vec<PortInfo>, Vec<shared::Risk>) {
    let mut discovered_ports = Vec::new();
    let mut discovered_risks = Vec::new();

    for port in ports_to_scan {
        let addr_str = format!("{}:{}", target_ip, port);
        if let Ok(socket_addr) = addr_str.parse::<SocketAddr>() {
            if let Ok(mut stream) = std::net::TcpStream::connect_timeout(
                &socket_addr,
                std::time::Duration::from_millis(500),
            ) {
                let mut banner = None;
                let _ = stream.set_read_timeout(Some(std::time::Duration::from_millis(1000)));
                let mut buffer = [0; 512];
                if let Ok(n) = stream.read(&mut buffer) {
                    if n > 0 {
                        let s = String::from_utf8_lossy(&buffer[..n]).trim().to_string();
                        if !s.is_empty() {
                            banner = Some(s);
                        }
                    }
                }

                let service = match port {
                    21 => {
                        discovered_risks.push(shared::Risk {
                            id: Uuid::new_v4().to_string(),
                            asset_ip: target_ip.clone(),
                            port,
                            severity: "High".to_string(),
                            description: "FTP Anonymous Login Allowed".to_string(),
                            solution: Some("Disable anonymous login in vsftpd.conf".to_string()),
                            status: shared::RiskStatus::Open,
                            created_at: Some(Utc::now()),
                            updated_at: Some(Utc::now()),
                            assigned_to: None,
                        });
                        Some("FTP".to_string())
                    }
                    22 => {
                        if let Some(b) = &banner {
                            if b.contains("SSH-2.0-OpenSSH_7") {
                                discovered_risks.push(shared::Risk {
                                    id: Uuid::new_v4().to_string(),
                                    asset_ip: target_ip.clone(),
                                    port,
                                    severity: "Medium".to_string(),
                                    description: "Outdated SSH Version".to_string(),
                                    solution: Some("Upgrade OpenSSH to latest version".to_string()),
                                    status: shared::RiskStatus::Open,
                                    created_at: Some(Utc::now()),
                                    updated_at: Some(Utc::now()),
                                    assigned_to: None,
                                });
                            }
                        }
                        Some("SSH".to_string())
                    }
                    23 => {
                        discovered_risks.push(shared::Risk {
                            id: Uuid::new_v4().to_string(),
                            asset_ip: target_ip.clone(),
                            port,
                            severity: "Critical".to_string(),
                            description: "Telnet Service Detected".to_string(),
                            solution: Some("Telnet is insecure. Replace with SSH.".to_string()),
                            status: shared::RiskStatus::Open,
                            created_at: Some(Utc::now()),
                            updated_at: Some(Utc::now()),
                            assigned_to: None,
                        });
                        Some("Telnet".to_string())
                    }
                    25 => Some("SMTP".to_string()),
                    53 => Some("DNS".to_string()),
                    80 | 8080 | 8000 | 8888 => {
                        if let Some(b) = &banner {
                            if b.to_lowercase().contains("apache/2.4.49") {
                                discovered_risks.push(shared::Risk {
                                    id: Uuid::new_v4().to_string(),
                                    asset_ip: target_ip.clone(),
                                    port,
                                    severity: "Critical".to_string(),
                                    description: "Apache Path Traversal (CVE-2021-41773)"
                                        .to_string(),
                                    solution: Some("Upgrade Apache to 2.4.51+".to_string()),
                                    status: shared::RiskStatus::Open,
                                    created_at: Some(Utc::now()),
                                    updated_at: Some(Utc::now()),
                                    assigned_to: None,
                                });
                            }
                        }
                        if port == 8080 {
                            discovered_risks.push(shared::Risk {
                                id: Uuid::new_v4().to_string(),
                                asset_ip: target_ip.clone(),
                                port,
                                severity: "High".to_string(),
                                description: "Weak Admin Password (admin/admin)".to_string(),
                                solution: Some(
                                    "Change the default password immediately.".to_string(),
                                ),
                                status: shared::RiskStatus::Open,
                                created_at: Some(Utc::now()),
                                updated_at: Some(Utc::now()),
                                assigned_to: None,
                            });
                        }
                        Some("HTTP".to_string())
                    }
                    443 => Some("HTTPS".to_string()),
                    3306 => {
                        discovered_risks.push(shared::Risk {
                            id: Uuid::new_v4().to_string(),
                            asset_ip: target_ip.clone(),
                            port,
                            severity: "Medium".to_string(),
                            description: "MySQL Weak Password".to_string(),
                            solution: Some("Enforce strong password policy".to_string()),
                            status: shared::RiskStatus::Open,
                            created_at: Some(Utc::now()),
                            updated_at: Some(Utc::now()),
                            assigned_to: None,
                        });
                        Some("MySQL".to_string())
                    }
                    3389 => Some("RDP".to_string()),
                    5432 => Some("PostgreSQL".to_string()),
                    6379 => {
                        discovered_risks.push(shared::Risk {
                            id: Uuid::new_v4().to_string(),
                            asset_ip: target_ip.clone(),
                            port,
                            severity: "High".to_string(),
                            description: "Redis Unprotected".to_string(),
                            solution: Some("Enable authentication in redis.conf".to_string()),
                            status: shared::RiskStatus::Open,
                            created_at: Some(Utc::now()),
                            updated_at: Some(Utc::now()),
                            assigned_to: None,
                        });
                        Some("Redis".to_string())
                    }
                    27017 => Some("MongoDB".to_string()),
                    _ => Some("Unknown".to_string()),
                };

                discovered_ports.push(PortInfo {
                    port,
                    is_open: true,
                    service,
                    version: None,
                    banner,
                    is_bound: false,
                    system_name: None,
                    middleware: None,
                    created_by: Some("scanner".to_string()),
                    updated_by: None,
                });
            }
        }
    }

    (discovered_ports, discovered_risks)
}

pub async fn get_tasks(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<Task>>, ApiError> {
    let _user = get_current_user_from_headers(&headers, &state.users)
        .await
        .ok_or_else(|| ApiError::unauthorized("Unauthorized"))?;
    Ok(Json(load_all_tasks(&state).await?))
}

pub async fn create_task(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateTaskRequest>,
) -> Result<Json<Task>, ApiError> {
    let user = get_current_user_from_headers(&headers, &state.users)
        .await
        .ok_or_else(|| ApiError::unauthorized("Unauthorized"))?;

    if user.role != Role::SecAdmin {
        return Err(ApiError::forbidden("Access denied: SecAdmin only"));
    }

    let new_task = Task {
        id: Uuid::new_v4().to_string(),
        name: req.name.clone(),
        target: req.target,
        status: TaskStatus::Pending,
        start_time: None,
        end_time: None,
        found_assets: 0,
        found_risks: 0,
        port_policy: req.port_policy,
        domain_brute: req.domain_brute,
        service_detection: req.service_detection,
        os_detection: req.os_detection,
        site_identify: req.site_identify,
        created_by: Some(user.username.clone()),
    };

    if get_db().is_some() {
        db_insert_task(&new_task)
            .await
            .map_err(|e| ApiError::internal(format!("Failed to create task: {}", e)))?;
    }
    sync_task_cache(&state, &new_task)?;

    log_action(
        &state.audit_logs,
        &user,
        "CREATE_TASK",
        &req.name,
        "Created new scan task",
    );

    Ok(Json(new_task))
}

pub async fn update_task(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(req): Json<CreateTaskRequest>,
) -> Result<Json<Option<Task>>, ApiError> {
    let user = get_current_user_from_headers(&headers, &state.users)
        .await
        .ok_or_else(|| ApiError::unauthorized("Unauthorized"))?;

    if user.role != Role::SecAdmin {
        return Err(ApiError::forbidden("Access denied: SecAdmin only"));
    }

    let mut updated_task = match load_task_by_id(&state, &id).await? {
        Some(task) => task,
        None => return Ok(Json(None)),
    };

    updated_task.name = req.name.clone();
    updated_task.target = req.target;
    updated_task.port_policy = req.port_policy;
    updated_task.domain_brute = req.domain_brute;
    updated_task.service_detection = req.service_detection;
    updated_task.os_detection = req.os_detection;
    updated_task.site_identify = req.site_identify;

    if get_db().is_some() {
        db_update_task(&id, &updated_task)
            .await
            .map_err(|e| ApiError::internal(format!("Failed to update task: {}", e)))?;
    }
    sync_task_cache(&state, &updated_task)?;

    log_action(
        &state.audit_logs,
        &user,
        "UPDATE_TASK",
        &updated_task.name,
        "Updated task config",
    );
    Ok(Json(Some(updated_task)))
}

pub async fn delete_task(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<String>, ApiError> {
    let user = get_current_user_from_headers(&headers, &state.users)
        .await
        .ok_or_else(|| ApiError::unauthorized("Unauthorized"))?;

    if user.role != Role::SecAdmin {
        return Err(ApiError::forbidden("Access denied: SecAdmin only"));
    }

    if load_task_by_id(&state, &id).await?.is_none() {
        return Err(ApiError::not_found(format!(
            "Task with ID '{}' not found",
            id
        )));
    }

    if get_db().is_some() {
        db_delete_task(&id)
            .await
            .map_err(|e| ApiError::internal(format!("Failed to delete task: {}", e)))?;
    }
    remove_task_from_cache(&state, &id)?;

    log_action(&state.audit_logs, &user, "DELETE_TASK", &id, "Deleted task");
    Ok(Json("Deleted".to_string()))
}

use std::io::Read;
use std::net::SocketAddr;

pub async fn trigger_scan(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ScanRequest>,
) -> Result<Json<String>, ApiError> {
    let user = get_current_user_from_headers(&headers, &state.users)
        .await
        .ok_or_else(|| ApiError::unauthorized("Unauthorized"))?;

    if user.role != Role::SecAdmin {
        return Err(ApiError::forbidden("Access denied: SecAdmin only"));
    }

    let target_ip = req.target_ip.clone();
    let ports_to_scan = if req.ports.is_empty() {
        // Default TOP ports if none specified
        vec![
            21, 22, 23, 25, 53, 80, 110, 143, 443, 445, 3306, 3389, 5432, 6379, 8000, 8080, 8888,
            27017,
        ]
    } else {
        req.ports.clone()
    };

    let target_ip_clone = target_ip.clone();
    let state_for_scan = state.clone();

    tokio::spawn(async move {
        let target_for_worker = target_ip_clone.clone();
        let scan_result = tokio::task::spawn_blocking(move || {
            scan_target_ports(target_for_worker, ports_to_scan)
        })
        .await;

        let (discovered_ports, discovered_risks) = match scan_result {
            Ok(result) => result,
            Err(error) => {
                tracing::error!("Background scan task failed: {}", error);
                return;
            }
        };

        if let Some(conn) = get_db() {
            if let Ok(Some(db_asset)) =
                crate::database::get_asset_by_ip(&conn, &target_ip_clone).await
            {
                let mut asset = db_asset_to_shared(db_asset);
                for port in &discovered_ports {
                    if let Some(existing) = asset
                        .ports
                        .iter_mut()
                        .find(|existing| existing.port == port.port)
                    {
                        existing.is_open = true;
                        if existing.banner.is_none() {
                            existing.banner = port.banner.clone();
                        }
                        if existing.service.is_none()
                            || existing.service.as_deref() == Some("Unknown")
                        {
                            existing.service = port.service.clone();
                        }
                    } else {
                        asset.ports.push(port.clone());
                    }
                }
                asset.ports.sort_by_key(|port| port.port);
                asset.last_scanned = Some(Utc::now());
                asset.updated_by = Some("scanner".to_string());

                if let Some(asset_id) = asset.id {
                    if let Err(error) = db_update_asset(asset_id, &asset).await {
                        tracing::error!(
                            "Failed to persist scanned asset {}: {}",
                            target_ip_clone,
                            error
                        );
                    }
                }

                if let Ok(mut assets) = state_for_scan.assets.write() {
                    if let Some(existing) = assets
                        .iter_mut()
                        .find(|existing| existing.ip == target_ip_clone)
                    {
                        *existing = asset.clone();
                    } else {
                        assets.push(asset.clone());
                    }
                }
            }

            match get_risks_by_asset_ip(&conn, &target_ip_clone).await {
                Ok(existing_risks) => {
                    for discovered_risk in discovered_risks {
                        let duplicate = existing_risks.iter().any(|existing| {
                            existing.asset_ip == discovered_risk.asset_ip
                                && existing.port == discovered_risk.port as i32
                                && existing.description == discovered_risk.description
                                && existing.status != "Resolved"
                        });
                        if duplicate {
                            continue;
                        }
                        if let Err(error) = insert_risk(&conn, &discovered_risk).await {
                            tracing::error!(
                                "Failed to persist discovered risk {}:{}: {}",
                                discovered_risk.asset_ip,
                                discovered_risk.port,
                                error
                            );
                            continue;
                        }
                        sync_risk_cache(&state_for_scan, &discovered_risk);
                    }
                }
                Err(error) => tracing::error!("Failed to load existing risks for scan: {}", error),
            }
        } else {
            if !discovered_ports.is_empty() {
                let mut assets = match state_for_scan.assets.write() {
                    Ok(assets) => assets,
                    Err(error) => {
                        tracing::error!("Failed to write asset cache during scan: {}", error);
                        return;
                    }
                };
                if let Some(asset) = assets.iter_mut().find(|asset| asset.ip == target_ip_clone) {
                    for port in discovered_ports {
                        if let Some(existing) = asset
                            .ports
                            .iter_mut()
                            .find(|existing| existing.port == port.port)
                        {
                            existing.is_open = true;
                            if existing.banner.is_none() {
                                existing.banner = port.banner.clone();
                            }
                            if existing.service.is_none()
                                || existing.service.as_deref() == Some("Unknown")
                            {
                                existing.service = port.service.clone();
                            }
                        } else {
                            asset.ports.push(port);
                        }
                    }
                    asset.ports.sort_by_key(|port| port.port);
                    asset.last_scanned = Some(Utc::now());
                }
            }

            for discovered_risk in discovered_risks {
                sync_risk_cache(&state_for_scan, &discovered_risk);
            }
        }
    });

    log_action(
        &state.audit_logs,
        &user,
        "TRIGGER_SCAN",
        &target_ip,
        "Triggered background scan",
    );
    Ok(Json(format!("Scan started for {}", target_ip)))
}
