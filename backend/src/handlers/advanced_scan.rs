//! Advanced scanning API handlers
//!
//! Provides REST API endpoints for advanced scanning operations using RustScan.

#![allow(dead_code)]

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::{
    db_advanced_scan_task_to_shared, db_quick_scan_result_to_shared, delete_advanced_scan_task,
    get_advanced_scan_tasks, get_quick_scan_results, insert_advanced_scan_task_wrapper,
    insert_quick_scan_result_wrapper, update_advanced_scan_task,
};
use crate::middleware::AuthUser;
use crate::state::AppState;
use crate::utils::{get_current_user_from_auth, log_action};
use shared::{AdvancedScanConfig, AdvancedScanTask, CreateAdvancedScanRequest, TaskStatus};

/// Execute advanced scan
pub async fn execute_advanced_scan(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<CreateAdvancedScanRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    // Get current user for audit logging
    let current_user = get_current_user_from_auth(&auth_user, &state.users)
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Validate request
    if req.targets.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let task_id = Uuid::new_v4().to_string();
    let config = AdvancedScanConfig {
        strategy: req.strategy,
        engine: req.engine,
        concurrency: req.concurrency.unwrap_or(500),
        timeout_ms: req.timeout_ms.unwrap_or(5000),
        service_detection: req.service_detection.unwrap_or(true),
        os_detection: req.os_detection.unwrap_or(false),
        web_fingerprint: req.web_fingerprint.unwrap_or(false),
        vulnerability_scan: false,
        cloud_tag_sync: req.cloud_tag_sync.unwrap_or(true),
        rate_limit: None,
    };

    // Create initial task
    let task = AdvancedScanTask {
        id: task_id.clone(),
        name: req.name.clone(),
        targets: req.targets.clone(),
        config: config.clone(),
        status: TaskStatus::Running,
        progress: 0.0,
        current_target: req.targets.first().cloned(),
        scanned_count: 0,
        total_count: req.targets.len() as u32,
        start_time: Some(Utc::now()),
        end_time: None,
        results: vec![],
        cloud_mappings: vec![],
        error_message: None,
        created_by: None, // Will be set from auth context in production
    };

    // Persist to database
    let _ = insert_advanced_scan_task_wrapper(&task).await;

    // Store task in memory
    state
        .advanced_tasks
        .write()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .push(task.clone());

    // Audit log
    log_action(
        &state.audit_logs,
        &current_user,
        "SCAN_TASK_CREATED",
        &req.name,
        &format!("Created advanced scan task with ID {}", task_id),
    );

    // Spawn background scan task using spawn_blocking for scan operations
    let state_clone = state.clone();
    let task_id_clone = task_id.clone();
    tokio::task::spawn_blocking(move || {
        // Get scan manager inside blocking context
        let rt = match tokio::runtime::Runtime::new() {
            Ok(runtime) => runtime,
            Err(error) => {
                tracing::error!(
                    "Failed to create Tokio runtime for advanced scan: {}",
                    error
                );
                return;
            }
        };

        let result = rt.block_on(async {
            let scan_manager_guard = state_clone.scan_manager.write().await;
            if let Some(ref scan_manager) = *scan_manager_guard {
                scan_manager
                    .execute_advanced_scan(task_id_clone.clone(), req.targets, config)
                    .await
            } else {
                Err("Scan manager not available".into())
            }
        });

        // Handle result
        if let Ok(completed_task) = result {
            let rt2 = match tokio::runtime::Runtime::new() {
                Ok(runtime) => runtime,
                Err(error) => {
                    tracing::error!(
                        "Failed to create Tokio runtime for advanced scan persistence: {}",
                        error
                    );
                    return;
                }
            };
            rt2.block_on(async {
                if let Ok(mut tasks) = state_clone.advanced_tasks.write() {
                    if let Some(t) = tasks.iter_mut().find(|t| t.id == task_id_clone) {
                        *t = completed_task.clone();
                    }
                } else {
                    tracing::error!("Failed to acquire advanced task write lock");
                }

                let _ = update_advanced_scan_task(&completed_task).await;

                for scan_result in &completed_task.results {
                    let _ = insert_quick_scan_result_wrapper(scan_result, &task_id_clone).await;
                }
            })
        }
    });

    Ok(Json(task))
}

/// Get all advanced scan tasks
pub async fn get_advanced_tasks(State(state): State<AppState>) -> impl IntoResponse {
    // Try to load from database first
    let tasks = match get_advanced_scan_tasks().await {
        Ok(db_tasks) => {
            let mut tasks_with_results = Vec::new();
            for db_task in db_tasks {
                let mut task = db_advanced_scan_task_to_shared(db_task);
                // Load results for each task
                if let Ok(results) = get_quick_scan_results(&task.id).await {
                    task.results = results
                        .into_iter()
                        .map(db_quick_scan_result_to_shared)
                        .collect();
                }
                tasks_with_results.push(task);
            }
            // Update in-memory cache
            if let Ok(mut tasks) = state.advanced_tasks.write() {
                *tasks = tasks_with_results.clone();
            } else {
                tracing::error!("Failed to update advanced task cache");
            }
            tasks_with_results
        }
        Err(e) => {
            tracing::warn!("Error loading advanced scan tasks from database: {}", e);
            // Fallback to memory cache
            match state.advanced_tasks.read() {
                Ok(tasks) => tasks.clone(),
                Err(error) => {
                    tracing::error!("Failed to read advanced task cache: {}", error);
                    Vec::new()
                }
            }
        }
    };

    Json(tasks)
}

/// Get specific advanced scan task
pub async fn get_advanced_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    // Try to load from database first
    if let Ok(db_tasks) = get_advanced_scan_tasks().await {
        if let Some(db_task) = db_tasks.iter().find(|t| t.id == id) {
            let mut task = db_advanced_scan_task_to_shared(db_task.clone());
            // Load results
            if let Ok(results) = get_quick_scan_results(&id).await {
                task.results = results
                    .into_iter()
                    .map(db_quick_scan_result_to_shared)
                    .collect();
            }
            return Ok(Json(task));
        }
    }

    // Fallback to memory cache
    let tasks = state
        .advanced_tasks
        .read()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let task = tasks
        .iter()
        .find(|t| t.id == id)
        .cloned()
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(task))
}

/// Delete advanced scan task
pub async fn delete_advanced_scan(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    // Get current user for audit logging
    let current_user = get_current_user_from_auth(&auth_user, &state.users)
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Get task name for audit log before deletion
    let task_name = {
        let tasks = state
            .advanced_tasks
            .read()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        tasks.iter().find(|t| t.id == id).map(|t| t.name.clone())
    };

    // Remove from in-memory storage
    {
        let mut tasks = state
            .advanced_tasks
            .write()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let idx = tasks
            .iter()
            .position(|t| t.id == id)
            .ok_or(StatusCode::NOT_FOUND)?;
        tasks.remove(idx);
    }

    // Persist to database
    let _ = delete_advanced_scan_task(&id).await;

    // Audit log
    let target = task_name.unwrap_or_else(|| id.clone());
    log_action(
        &state.audit_logs,
        &current_user,
        "SCAN_TASK_DELETED",
        &target,
        &format!("Deleted advanced scan task with ID {}", id),
    );

    Ok(StatusCode::NO_CONTENT)
}

/// Cancel running advanced scan
pub async fn cancel_advanced_scan(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    // Get current user for audit logging
    let current_user = get_current_user_from_auth(&auth_user, &state.users)
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Update task status
    let (found, task_to_update) = {
        let mut tasks = state
            .advanced_tasks
            .write()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        if let Some(task) = tasks.iter_mut().find(|t| t.id == id) {
            if task.status == TaskStatus::Running {
                task.status = TaskStatus::Failed;
                task.error_message = Some("Cancelled by user".to_string());
                task.end_time = Some(Utc::now());
                (true, task.clone())
            } else {
                (true, task.clone())
            }
        } else {
            (
                false,
                shared::AdvancedScanTask {
                    id: String::new(),
                    name: String::new(),
                    targets: vec![],
                    config: shared::AdvancedScanConfig::default(),
                    status: TaskStatus::Failed,
                    progress: 0.0,
                    current_target: None,
                    scanned_count: 0,
                    total_count: 0,
                    start_time: None,
                    end_time: None,
                    results: vec![],
                    cloud_mappings: vec![],
                    error_message: None,
                    created_by: None,
                },
            )
        }
    };

    if !found {
        return Err(StatusCode::NOT_FOUND);
    }

    // Persist to database (after releasing lock)
    let _ = update_advanced_scan_task(&task_to_update).await;

    // Audit log
    log_action(
        &state.audit_logs,
        &current_user,
        "SCAN_TASK_UPDATED",
        &task_to_update.name,
        "Cancelled advanced scan task",
    );

    Ok(StatusCode::OK)
}

/// Export scan results
pub async fn export_scan_results(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    // Try to get results from database first
    let results = match get_quick_scan_results(&id).await {
        Ok(db_results) => db_results
            .into_iter()
            .map(db_quick_scan_result_to_shared)
            .collect(),
        Err(_) => {
            // Fallback to memory cache
            let tasks = state
                .advanced_tasks
                .read()
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let task = tasks
                .iter()
                .find(|t| t.id == id)
                .ok_or(StatusCode::NOT_FOUND)?;
            task.results.clone()
        }
    };

    // Generate CSV report
    let mut csv = String::from("IP,Is Alive,Open Ports,Scan Time\n");

    for result in &results {
        let ports: Vec<String> = result
            .open_ports
            .iter()
            .filter(|p| p.is_open)
            .map(|p| p.port.to_string())
            .collect();

        csv.push_str(&format!(
            "{},{},{},{}\n",
            result.ip,
            result.is_alive,
            ports.join(";"),
            result.scanned_at.format("%Y-%m-%d %H:%M:%S")
        ));
    }

    let headers = [("content-type", "text/csv")];
    Ok((headers, csv))
}

/// Get scan engines status
pub async fn get_scan_engines_status(State(state): State<AppState>) -> impl IntoResponse {
    let scan_manager_guard = state.scan_manager.read().await;

    let status = if let Some(ref scan_manager) = *scan_manager_guard {
        ScanEnginesStatus {
            rustscan_enabled: true,
            rustscan_status: "available".to_string(),
            nmap_enabled: false,
            nmap_status: "not_configured".to_string(),
            basic_tcp_enabled: true,
            basic_tcp_status: "available".to_string(),
            active_scans: scan_manager.get_active_jobs().await.len(),
        }
    } else {
        ScanEnginesStatus {
            rustscan_enabled: false,
            rustscan_status: "not_initialized".to_string(),
            nmap_enabled: false,
            nmap_status: "not_configured".to_string(),
            basic_tcp_enabled: true,
            basic_tcp_status: "available".to_string(),
            active_scans: 0,
        }
    };

    Json(status)
}

/// Scan progress stream (SSE endpoint)
pub async fn scan_progress_stream(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let Ok(tasks) = state.advanced_tasks.read() else {
        return Json(serde_json::json!({
            "error": "Failed to read task state"
        }));
    };
    let task = tasks.iter().find(|t| t.id == id);

    if let Some(task) = task {
        Json(serde_json::json!({
            "id": task.id,
            "progress": task.progress,
            "status": task.status,
            "scanned_count": task.scanned_count,
            "total_count": task.total_count,
            "current_target": task.current_target,
        }))
    } else {
        Json(serde_json::json!({
            "error": "Task not found"
        }))
    }
}

/// Scan engines status response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ScanEnginesStatus {
    rustscan_enabled: bool,
    rustscan_status: String,
    nmap_enabled: bool,
    nmap_status: String,
    basic_tcp_enabled: bool,
    basic_tcp_status: String,
    active_scans: usize,
}

/// Quick scan endpoint (single target, common ports)
#[derive(Debug, Deserialize)]
pub struct QuickScanRequest {
    pub target: String,
    pub ports: Option<Vec<u16>>,
    pub service_detection: Option<bool>,
}

pub async fn quick_scan(
    State(state): State<AppState>,
    Json(req): Json<QuickScanRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let scan_manager_guard = state.scan_manager.read().await;

    let scan_manager = scan_manager_guard
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let ports = req
        .ports
        .unwrap_or_else(|| crate::scanners::rustscan::RustScan::get_common_ports("TOP1000"));

    let result = if req.service_detection.unwrap_or(false) {
        let detailed = scan_manager.detailed_scan(&req.target, &ports).await;

        serde_json::json!({
            "ip": detailed.ip,
            "is_alive": detailed.is_alive,
            "open_ports": detailed.open_ports,
            "scan_duration_ms": detailed.scan_duration_ms,
            "scan_time": detailed.scan_time,
        })
    } else {
        let quick = scan_manager.quick_scan(&req.target, &ports).await;

        serde_json::json!({
            "ip": quick.ip,
            "is_alive": quick.is_alive,
            "open_ports": quick.open_ports,
            "scanned_at": quick.scanned_at,
        })
    };

    Ok(Json(result))
}

#[derive(Debug, Deserialize)]
pub struct BatchScanRequest {
    pub targets: Vec<String>,
    pub ports: Option<Vec<u16>>,
}

pub async fn batch_scan(
    State(state): State<AppState>,
    Json(req): Json<BatchScanRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let scan_manager_guard = state.scan_manager.read().await;

    let scan_manager = scan_manager_guard
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let ports = req
        .ports
        .unwrap_or_else(|| crate::scanners::rustscan::RustScan::get_common_ports("TOP100"));

    let results = scan_manager.scan_multiple(&req.targets, &ports).await;

    Ok(Json(serde_json::json!({
        "results": results,
        "count": results.len(),
    })))
}
