use axum::{
    extract::{State, Json, Path},
    http::{StatusCode, HeaderMap},
};
use shared::{Task, CreateTaskRequest, TaskStatus, ScanRequest, PortInfo, Role};
use crate::state::AppState;
use crate::utils::{get_current_user, log_action};
use crate::database::{get_tasks as db_get_tasks, insert_task_wrapper as db_insert_task, update_task as db_update_task, delete_task as db_delete_task, db_task_to_shared};
use crate::middleware::ApiError;
use uuid::Uuid;
use chrono::Utc;

pub async fn get_tasks(State(state): State<AppState>, headers: HeaderMap) -> Result<Json<Vec<Task>>, ApiError> {
    let _user = get_current_user(&headers, &state.users)
        .ok_or_else(|| ApiError::unauthorized("Unauthorized"))?;

    // Try to load from database first
    match db_get_tasks().await {
        Ok(db_tasks) => {
            let tasks: Vec<Task> = db_tasks.into_iter().map(db_task_to_shared).collect();
            // Update in-memory cache
            *state.tasks.write()
                .map_err(|e| ApiError::internal(format!("Failed to write tasks cache: {}", e)))? = tasks.clone();
            return Ok(Json(tasks));
        }
        Err(e) => {
            eprintln!("Error loading tasks from database: {}", e);
            // Fallback to memory cache
            let tasks = state.tasks.read()
                .map_err(|e| ApiError::internal(format!("Failed to read tasks cache: {}", e)))?;
            Ok(Json(tasks.clone()))
        }
    }
}

pub async fn create_task(State(state): State<AppState>, headers: HeaderMap, Json(req): Json<CreateTaskRequest>) -> Result<Json<Task>, ApiError> {
    let user = get_current_user(&headers, &state.users)
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

    // Add to in-memory storage
    {
        let mut tasks = state.tasks.write()
            .map_err(|e| ApiError::internal(format!("Failed to write tasks: {}", e)))?;
        tasks.push(new_task.clone());
    }

    // Persist to database
    let _ = db_insert_task(&new_task).await;

    log_action(&state.audit_logs, &user, "CREATE_TASK", &req.name, "Created new scan task");

    Ok(Json(new_task))
}

pub async fn update_task(State(state): State<AppState>, headers: HeaderMap, Path(id): Path<String>, Json(req): Json<CreateTaskRequest>) -> Result<Json<Option<Task>>, ApiError> {
    let user = get_current_user(&headers, &state.users)
        .ok_or_else(|| ApiError::unauthorized("Unauthorized"))?;

    if user.role != Role::SecAdmin {
        return Err(ApiError::forbidden("Access denied: SecAdmin only"));
    }

    let (found, updated_task) = {
        let mut tasks = state.tasks.write()
            .map_err(|e| ApiError::internal(format!("Failed to write tasks: {}", e)))?;
        if let Some(task) = tasks.iter_mut().find(|t| t.id == id) {
            task.name = req.name.clone();
            task.target = req.target;
            task.port_policy = req.port_policy;
            task.domain_brute = req.domain_brute;
            task.service_detection = req.service_detection;
            task.os_detection = req.os_detection;
            task.site_identify = req.site_identify;
            (true, task.clone())
        } else {
            (false, Task {
                id: id.clone(),
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
            })
        }
    };

    if found {
        // Persist to database (after releasing lock)
        let _ = db_update_task(&id, &updated_task).await;

        log_action(&state.audit_logs, &user, "UPDATE_TASK", &updated_task.name, "Updated task config");
        Ok(Json(Some(updated_task)))
    } else {
        Ok(Json(None))
    }
}

pub async fn delete_task(State(state): State<AppState>, headers: HeaderMap, Path(id): Path<String>) -> Result<Json<String>, ApiError> {
    let user = get_current_user(&headers, &state.users)
        .ok_or_else(|| ApiError::unauthorized("Unauthorized"))?;

    if user.role != Role::SecAdmin {
        return Err(ApiError::forbidden("Access denied: SecAdmin only"));
    }

    // Remove from in-memory storage
    {
        let mut tasks = state.tasks.write()
            .map_err(|e| ApiError::internal(format!("Failed to write tasks: {}", e)))?;
        tasks.retain(|t| t.id != id);
    }

    // Persist to database (after releasing lock)
    let _ = db_delete_task(&id).await;

    log_action(&state.audit_logs, &user, "DELETE_TASK", &id, "Deleted task");
    Ok(Json("Deleted".to_string()))
}

use std::io::Read;
use std::net::SocketAddr;

pub async fn trigger_scan(State(state): State<AppState>, headers: HeaderMap, Json(req): Json<ScanRequest>) -> Result<Json<String>, ApiError> {
    let user = get_current_user(&headers, &state.users)
        .ok_or_else(|| ApiError::unauthorized("Unauthorized"))?;

    if user.role != Role::SecAdmin {
        return Err(ApiError::forbidden("Access denied: SecAdmin only"));
    }

    let target_ip = req.target_ip.clone();
    let ports_to_scan = if req.ports.is_empty() {
        // Default TOP ports if none specified
        vec![21, 22, 23, 25, 53, 80, 110, 143, 443, 445, 3306, 3389, 5432, 6379, 8000, 8080, 8888, 27017]
    } else {
        req.ports.clone()
    };

    let assets_handle = state.assets.clone();
    let risks_handle = state.risks.clone();
    let target_ip_clone = target_ip.clone();

    // Spawn scanning task
    tokio::task::spawn_blocking(move || {
        let mut discovered_ports = Vec::new();
        let mut discovered_risks = Vec::new();

        for port in ports_to_scan {
            let addr_str = format!("{}:{}", target_ip_clone, port);
            if let Ok(socket_addr) = addr_str.parse::<SocketAddr>() {
                // 500ms connect timeout
                if let Ok(mut stream) = std::net::TcpStream::connect_timeout(&socket_addr, std::time::Duration::from_millis(500)) {
                    // Try to grab banner
                    let mut banner = None;
                    let _ = stream.set_read_timeout(Some(std::time::Duration::from_millis(1000)));
                    let mut buffer = [0; 512];
                    // Send some bytes to trigger response for some protocols? 
                    // For now just read.
                    if let Ok(n) = stream.read(&mut buffer) {
                        if n > 0 {
                            let s = String::from_utf8_lossy(&buffer[..n]).trim().to_string();
                            if !s.is_empty() {
                                banner = Some(s);
                            }
                        }
                    }

                    // Simple service guess & Mock Risk Detection
                    let service = match port {
                        21 => {
                            discovered_risks.push((port, "High", "FTP Anonymous Login Allowed", "Disable anonymous login in vsftpd.conf"));
                            Some("FTP".to_string())
                        },
                        22 => {
                            if let Some(b) = &banner {
                                if b.contains("SSH-2.0-OpenSSH_7") {
                                    discovered_risks.push((port, "Medium", "Outdated SSH Version", "Upgrade OpenSSH to latest version"));
                                }
                            }
                            Some("SSH".to_string())
                        },
                        23 => {
                            discovered_risks.push((port, "Critical", "Telnet Service Detected", "Telnet is insecure. Replace with SSH."));
                            Some("Telnet".to_string())
                        },
                        25 => Some("SMTP".to_string()),
                        53 => Some("DNS".to_string()),
                        80 | 8080 | 8000 | 8888 => {
                            if let Some(b) = &banner {
                                if b.to_lowercase().contains("apache/2.4.49") {
                                    discovered_risks.push((port, "Critical", "Apache Path Traversal (CVE-2021-41773)", "Upgrade Apache to 2.4.51+"));
                                }
                            }
                            // Randomly simulate weak password for HTTP admin
                            if port == 8080 {
                                discovered_risks.push((port, "High", "Weak Admin Password (admin/admin)", "Change the default password immediately."));
                            }
                            Some("HTTP".to_string())
                        },
                        443 => Some("HTTPS".to_string()),
                        3306 => {
                             discovered_risks.push((port, "Medium", "MySQL Weak Password", "Enforce strong password policy"));
                             Some("MySQL".to_string())
                        },
                        3389 => Some("RDP".to_string()),
                        5432 => Some("PostgreSQL".to_string()),
                        6379 => {
                            discovered_risks.push((port, "High", "Redis Unprotected", "Enable authentication in redis.conf"));
                            Some("Redis".to_string())
                        },
                        27017 => Some("MongoDB".to_string()),
                        _ => Some("Unknown".to_string()),
                    };

                    discovered_ports.push(PortInfo {
                        port,
                        is_open: true,
                        service,
                        version: None, // Logic for version parsing would go here
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

        // Update assets state
        if !discovered_ports.is_empty() {
            let mut assets = assets_handle.write().unwrap();
            if let Some(asset) = assets.iter_mut().find(|a| a.ip == target_ip_clone) {
                for p in discovered_ports {
                    // Update existing or add new
                    if let Some(existing) = asset.ports.iter_mut().find(|ep| ep.port == p.port) {
                        existing.is_open = true;
                        if existing.banner.is_none() { existing.banner = p.banner; }
                        if existing.service.is_none() || existing.service.as_deref() == Some("Unknown") { existing.service = p.service; }
                    } else {
                        asset.ports.push(p);
                    }
                }
                asset.ports.sort_by_key(|p| p.port);
                asset.last_scanned = Some(Utc::now());
            }
        }

        // Update risks state
        if !discovered_risks.is_empty() {
             use shared::{Risk, RiskStatus};
             let mut risks = risks_handle.write().unwrap();
             for (port, severity, desc, sol) in discovered_risks {
                 // Avoid duplicates
                 if !risks.iter().any(|r| r.asset_ip == target_ip_clone && r.port == port && r.description == desc && r.status != RiskStatus::Resolved) {
                     risks.push(Risk {
                         id: Uuid::new_v4().to_string(),
                         asset_ip: target_ip_clone.clone(),
                         port,
                         severity: severity.to_string(),
                         description: desc.to_string(),
                         solution: Some(sol.to_string()),
                         status: RiskStatus::Open,
                         created_at: Some(Utc::now()),
                         updated_at: Some(Utc::now()),
                         assigned_to: None,
                     });
                 }
             }
        }
    });

    log_action(&state.audit_logs, &user, "TRIGGER_SCAN", &target_ip, "Triggered background scan");
    Ok(Json(format!("Scan started for {}", target_ip)))
}
