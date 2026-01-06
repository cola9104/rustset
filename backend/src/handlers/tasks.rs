use axum::{
    extract::{State, Json, Path},
    http::{StatusCode, HeaderMap},
};
use shared::{Task, CreateTaskRequest, TaskStatus, ScanRequest, PortInfo, Role};
use crate::state::AppState;
use crate::utils::{get_current_user, log_action};
use uuid::Uuid;
use chrono::Utc;

pub async fn get_tasks(State(state): State<AppState>, headers: HeaderMap) -> Result<Json<Vec<Task>>, (StatusCode, String)> {
    let _user = get_current_user(&headers, &state.users).ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;
    let tasks = state.tasks.lock().unwrap();
    Ok(Json(tasks.clone()))
}

pub async fn create_task(State(state): State<AppState>, headers: HeaderMap, Json(req): Json<CreateTaskRequest>) -> Result<Json<Task>, (StatusCode, String)> {
    let user = get_current_user(&headers, &state.users).ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;
    if user.role != Role::SecAdmin {
        return Err((StatusCode::FORBIDDEN, "Access denied: SecAdmin only".to_string()));
    }

    let mut tasks = state.tasks.lock().unwrap();
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
    tasks.push(new_task.clone());
    log_action(&state.audit_logs, &user, "CREATE_TASK", &req.name, "Created new scan task");
    
    Ok(Json(new_task))
}

pub async fn update_task(State(state): State<AppState>, headers: HeaderMap, Path(id): Path<String>, Json(req): Json<CreateTaskRequest>) -> Result<Json<Option<Task>>, (StatusCode, String)> {
    let user = get_current_user(&headers, &state.users).ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;
    if user.role != Role::SecAdmin {
        return Err((StatusCode::FORBIDDEN, "Access denied: SecAdmin only".to_string()));
    }
    
    let mut tasks = state.tasks.lock().unwrap();
    if let Some(task) = tasks.iter_mut().find(|t| t.id == id) {
        task.name = req.name;
        task.target = req.target;
        task.port_policy = req.port_policy;
        task.domain_brute = req.domain_brute;
        task.service_detection = req.service_detection;
        task.os_detection = req.os_detection;
        task.site_identify = req.site_identify;
        
        log_action(&state.audit_logs, &user, "UPDATE_TASK", &task.name, "Updated task config");
        return Ok(Json(Some(task.clone())));
    }
    Ok(Json(None))
}

pub async fn delete_task(State(state): State<AppState>, headers: HeaderMap, Path(id): Path<String>) -> Result<Json<String>, (StatusCode, String)> {
    let user = get_current_user(&headers, &state.users).ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;
    if user.role != Role::SecAdmin {
        return Err((StatusCode::FORBIDDEN, "Access denied: SecAdmin only".to_string()));
    }

    let mut tasks = state.tasks.lock().unwrap();
    tasks.retain(|t| t.id != id);
    log_action(&state.audit_logs, &user, "DELETE_TASK", &id, "Deleted task");
    Ok(Json("Deleted".to_string()))
}

pub async fn trigger_scan(State(state): State<AppState>, headers: HeaderMap, Json(req): Json<ScanRequest>) -> Result<Json<String>, (StatusCode, String)> {
    let user = get_current_user(&headers, &state.users).ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;
    // Scan might be allowed for SecAdmin or maybe even SysAdmin? Let's say SecAdmin.
    if user.role != Role::SecAdmin {
        return Err((StatusCode::FORBIDDEN, "Access denied: SecAdmin only".to_string()));
    }

    let target_ip = req.target_ip.clone();
    let mut assets = state.assets.lock().unwrap();
    if let Some(asset) = assets.iter_mut().find(|a| a.ip == target_ip) {
        for port in req.ports {
            if !asset.ports.iter().any(|p| p.port == port) {
                asset.ports.push(PortInfo {
                    port,
                    is_open: true,
                    service: Some("Unknown".to_string()),
                    is_bound: false,
                    system_name: None,
                    middleware: None,
                    created_by: Some("scanner".to_string()),
                    updated_by: None,
                });
            }
        }
        asset.last_scanned = Some(Utc::now());
    }
    log_action(&state.audit_logs, &user, "TRIGGER_SCAN", &target_ip, "Triggered manual scan");
    Ok(Json(format!("Scan initiated for {}", target_ip)))
}
