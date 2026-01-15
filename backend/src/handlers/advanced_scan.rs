//! Advanced scanning API handlers
//!
//! Provides REST API endpoints for advanced scanning operations using RustScan.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;

use crate::state::AppState;
use shared::{
    AdvancedScanTask, AdvancedScanConfig, CreateAdvancedScanRequest, TaskStatus,
};

/// Execute advanced scan
pub async fn execute_advanced_scan(
    State(state): State<AppState>,
    Json(req): Json<CreateAdvancedScanRequest>,
) -> Result<impl IntoResponse, StatusCode> {
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
        name: req.name,
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

    // Store task
    state.advanced_tasks.lock().unwrap().push(task.clone());

    // Spawn background scan task
    let state_clone = state.clone();
    let task_id_clone = task_id.clone();
    tokio::spawn(async move {
        // Get scan manager
        let scan_manager_guard = state_clone.scan_manager.lock().await;
        if let Some(ref scan_manager) = *scan_manager_guard {
            // Execute scan
            let result = scan_manager.execute_advanced_scan(
                task_id_clone.clone(),
                req.targets,
                config,
            ).await;

            if let Ok(completed_task) = result {
                // Update task in state
                let mut tasks = state_clone.advanced_tasks.lock().unwrap();
                if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id_clone) {
                    *task = completed_task;
                }
            }
        }
    });

    Ok(Json(task))
}

/// Get all advanced scan tasks
pub async fn get_advanced_tasks(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let tasks = state.advanced_tasks.lock().unwrap().clone();
    Json(tasks)
}

/// Get specific advanced scan task
pub async fn get_advanced_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    let tasks = state.advanced_tasks.lock().unwrap();
    let task = tasks.iter().find(|t| t.id == id)
        .cloned()
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(task))
}

/// Delete advanced scan task
pub async fn delete_advanced_scan(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut tasks = state.advanced_tasks.lock().unwrap();
    let idx = tasks.iter().position(|t| t.id == id)
        .ok_or(StatusCode::NOT_FOUND)?;
    tasks.remove(idx);
    Ok(StatusCode::NO_CONTENT)
}

/// Cancel running advanced scan
pub async fn cancel_advanced_scan(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    // Update task status
    let mut tasks = state.advanced_tasks.lock().unwrap();
    if let Some(task) = tasks.iter_mut().find(|t| t.id == id) {
        if task.status == TaskStatus::Running {
            task.status = TaskStatus::Failed;
            task.error_message = Some("Cancelled by user".to_string());
            task.end_time = Some(Utc::now());
        }
        Ok(StatusCode::OK)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// Export scan results
pub async fn export_scan_results(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    let tasks = state.advanced_tasks.lock().unwrap();
    let task = tasks.iter().find(|t| t.id == id)
        .ok_or(StatusCode::NOT_FOUND)?;

    // Generate CSV report
    let mut csv = String::from("IP,Is Alive,Open Ports,Scan Time\n");

    for result in &task.results {
        let ports: Vec<String> = result.open_ports.iter()
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

    let headers = [( "content-type", "text/csv" )];
    Ok((headers, csv))
}

/// Get scan engines status
pub async fn get_scan_engines_status(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let scan_manager_guard = state.scan_manager.lock().await;

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
    let tasks = state.advanced_tasks.lock().unwrap();
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
    let scan_manager_guard = state.scan_manager.lock().await;

    let scan_manager = scan_manager_guard.as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let ports = req.ports.unwrap_or_else(|| {
        crate::scanners::rustscan::RustScan::get_common_ports("TOP1000")
    });

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
    let scan_manager_guard = state.scan_manager.lock().await;

    let scan_manager = scan_manager_guard.as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let ports = req.ports.unwrap_or_else(|| {
        crate::scanners::rustscan::RustScan::get_common_ports("TOP100")
    });

    let results = scan_manager.scan_multiple(&req.targets, &ports).await;

    Ok(Json(serde_json::json!({
        "results": results,
        "count": results.len(),
    })))
}
