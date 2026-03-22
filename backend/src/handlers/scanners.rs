//! Scanners Handler - 扫描器接口
//!
//! 功能：
//! - 单 IP 扫描
//! - 批量 IP 扫描
//! - 扫描结果查询

use axum::{
    extract::{State, Path, Query, Json},
    http::HeaderMap,
};
use serde::{Deserialize, Serialize};
use chrono::Utc;

use crate::state::AppState;
use crate::middleware::ApiError;
use crate::utils::{get_current_user, log_action};
use shared::{ScannerConfig, CreateScannerRequest, UpdateScannerRequest};

/// 扫描请求
#[derive(Debug, Deserialize)]
pub struct ScanRequest {
    pub target: String,
    pub ports: Option<Vec<u16>>,
}

/// 批量扫描请求
#[derive(Debug, Deserialize)]
pub struct BatchScanRequest {
    pub targets: Vec<String>,
    pub ports: Option<Vec<u16>>,
}

/// 扫描结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub id: String,
    pub target: String,
    pub status: String, // pending, running, completed, failed
    pub start_time: String,
    pub end_time: Option<String>,
    pub ports: Vec<PortScanResult>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortScanResult {
    pub port: u16,
    pub is_open: bool,
    pub service: Option<String>,
}

/// 扫描结果查询参数
#[derive(Debug, Deserialize)]
pub struct ScanResultsQuery {
    pub target: Option<String>,
    pub status: Option<String>,
    pub limit: Option<usize>,
}

/// 执行单 IP 扫描
pub async fn scan_ip(
    State(state): State<AppState>,
    _headers: HeaderMap,
    Json(req): Json<ScanRequest>,
) -> Result<Json<ScanResult>, ApiError> {
    // 验证目标 IP
    let target = req.target;
    if target.parse::<std::net::IpAddr>().is_err() {
        // 可能是主机名，跳过验证
    }

    let ports = req.ports.unwrap_or_else(|| vec![
        22, 23, 80, 443, 445, 3389, 3306, 5432, 6379, 8080, 8443, 27017
    ]);

    let scan_id = uuid::Uuid::new_v4().to_string();
    let start_time = Utc::now();

    // 模拟扫描（实际应调用 RustScan）
    let port_results: Vec<PortScanResult> = ports.into_iter()
        .map(|port| PortScanResult {
            port,
            is_open: false, // 默认关闭
            service: get_service_name(port),
        })
        .collect();

    let result = ScanResult {
        id: scan_id,
        target,
        status: "completed".to_string(),
        start_time: start_time.to_rfc3339(),
        end_time: Some(Utc::now().to_rfc3339()),
        ports: port_results,
        error: None,
    };

    // 保存结果
    let mut results = state.scan_results.write()
        .map_err(|e| ApiError::internal(format!("Failed to save results: {}", e)))?;
    results.push(result.clone());

    Ok(Json(result))
}

/// 执行批量扫描
pub async fn batch_scan_ips(
    State(state): State<AppState>,
    _headers: HeaderMap,
    Json(req): Json<BatchScanRequest>,
) -> Result<Json<Vec<ScanResult>>, ApiError> {
    let ports = req.ports.unwrap_or_else(|| vec![
        22, 80, 443, 445, 3389, 3306
    ]);

    let mut results = Vec::new();

    for target in req.targets {
        let scan_id = uuid::Uuid::new_v4().to_string();
        let start_time = Utc::now();

        let port_results: Vec<PortScanResult> = ports.iter()
            .map(|&port| PortScanResult {
                port,
                is_open: false,
                service: get_service_name(port),
            })
            .collect();

        let result = ScanResult {
            id: scan_id,
            target,
            status: "completed".to_string(),
            start_time: start_time.to_rfc3339(),
            end_time: Some(Utc::now().to_rfc3339()),
            ports: port_results,
            error: None,
        };

        results.push(result);
    }

    // 保存结果
    let mut scan_results = state.scan_results.write()
        .map_err(|e| ApiError::internal(format!("Failed to save results: {}", e)))?;
    scan_results.extend(results.clone());

    Ok(Json(results))
}

/// 获取扫描结果
pub async fn get_scan_results(
    State(state): State<AppState>,
    _headers: HeaderMap,
    Query(query): Query<ScanResultsQuery>,
) -> Result<Json<Vec<ScanResult>>, ApiError> {
    let results = state.scan_results.read()
        .map_err(|e| ApiError::internal(format!("Failed to read results: {}", e)))?;

    let filtered: Vec<ScanResult> = results.iter()
        .filter(|r| {
            if let Some(ref target) = query.target {
                if !r.target.contains(target) {
                    return false;
                }
            }
            if let Some(ref status) = query.status {
                if &r.status != status {
                    return false;
                }
            }
            true
        })
        .take(query.limit.unwrap_or(100)).cloned()
        .collect();

    Ok(Json(filtered))
}

/// 获取单个扫描结果
pub async fn get_scan_result(
    State(state): State<AppState>,
    _headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<ScanResult>, ApiError> {
    let results = state.scan_results.read()
        .map_err(|e| ApiError::internal(format!("Failed to read results: {}", e)))?;

    results.iter()
        .find(|r| r.id == id)
        .cloned()
        .map(Json)
        .ok_or_else(|| ApiError::not_found("Scan result not found"))
}

/// 根据端口号获取服务名
fn get_service_name(port: u16) -> Option<String> {
    match port {
        21 => Some("FTP".to_string()),
        22 => Some("SSH".to_string()),
        23 => Some("Telnet".to_string()),
        25 => Some("SMTP".to_string()),
        53 => Some("DNS".to_string()),
        80 => Some("HTTP".to_string()),
        110 => Some("POP3".to_string()),
        143 => Some("IMAP".to_string()),
        443 => Some("HTTPS".to_string()),
        445 => Some("SMB".to_string()),
        993 => Some("IMAPS".to_string()),
        995 => Some("POP3S".to_string()),
        3306 => Some("MySQL".to_string()),
        3389 => Some("RDP".to_string()),
        5432 => Some("PostgreSQL".to_string()),
        6379 => Some("Redis".to_string()),
        8080 => Some("HTTP-Alt".to_string()),
        8443 => Some("HTTPS-Alt".to_string()),
        27017 => Some("MongoDB".to_string()),
        _ => None,
    }
}

// ============== Scanner Configuration Management ==============

/// 获取所有扫描器配置
pub async fn get_scanners(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<ScannerConfig>>, ApiError> {
    let _current_user = get_current_user(&headers, &state.users)
        .ok_or_else(|| ApiError::unauthorized("Unauthorized"))?;

    let scanners = state.scanners.read()
        .map_err(|e| ApiError::internal(format!("Failed to read scanner configs: {}", e)))?;

    Ok(Json(scanners.clone()))
}

/// 创建扫描器配置
pub async fn create_scanner(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateScannerRequest>
) -> Result<Json<ScannerConfig>, ApiError> {
    let current_user = get_current_user(&headers, &state.users)
        .ok_or_else(|| ApiError::unauthorized("Unauthorized"))?;

    // 只有安全管理员和系统管理员可以创建扫描器配置
    if current_user.role != shared::Role::SysAdmin && current_user.role != shared::Role::SecAdmin {
        return Err(ApiError::forbidden("只有管理员可以创建扫描器配置"));
    }

    let now = Utc::now().to_rfc3339();
    let scanner_name = req.name.clone();
    let scanner_type = req.scanner_type.clone();

    let new_scanner = ScannerConfig {
        id: uuid::Uuid::new_v4().to_string(),
        name: scanner_name.clone(),
        scanner_type: scanner_type.clone(),
        enabled: req.enabled.unwrap_or(true),
        config: req.config,
        created_at: Some(now.clone()),
        updated_at: Some(now),
    };

    let mut scanners = state.scanners.write()
        .map_err(|e| ApiError::internal(format!("Failed to write scanner configs: {}", e)))?;
    scanners.push(new_scanner.clone());
    drop(scanners);

    // Audit log
    log_action(&state.audit_logs, &current_user, "SCANNER_CREATED", &scanner_name,
              &format!("Created scanner config with type {}", scanner_type));

    Ok(Json(new_scanner))
}

/// 更新扫描器配置
pub async fn update_scanner(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(req): Json<UpdateScannerRequest>,
) -> Result<Json<ScannerConfig>, ApiError> {
    let current_user = get_current_user(&headers, &state.users)
        .ok_or_else(|| ApiError::unauthorized("Unauthorized"))?;

    // 只有安全管理员和系统管理员可以修改扫描器配置
    if current_user.role != shared::Role::SysAdmin && current_user.role != shared::Role::SecAdmin {
        return Err(ApiError::forbidden("只有管理员可以修改扫描器配置"));
    }

    let now = Utc::now().to_rfc3339();

    let updated_scanner = {
        let mut scanners = state.scanners.write()
            .map_err(|e| ApiError::internal(format!("Failed to write scanner configs: {}", e)))?;

        let scanner = scanners.iter_mut()
            .find(|s| s.id == id)
            .ok_or_else(|| ApiError::not_found("Scanner config not found"))?;

        if let Some(name) = req.name {
            scanner.name = name;
        }
        if let Some(scanner_type) = req.scanner_type {
            scanner.scanner_type = scanner_type;
        }
        if let Some(enabled) = req.enabled {
            scanner.enabled = enabled;
        }
        if let Some(config) = req.config {
            scanner.config = config;
        }
        scanner.updated_at = Some(now);
        scanner.clone()
    };

    // Audit log
    log_action(&state.audit_logs, &current_user, "SCANNER_UPDATED", &updated_scanner.name,
              &format!("Updated scanner config with type {}", updated_scanner.scanner_type));

    Ok(Json(updated_scanner))
}

/// 删除扫描器配置
pub async fn delete_scanner(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<String>, ApiError> {
    let current_user = get_current_user(&headers, &state.users)
        .ok_or_else(|| ApiError::unauthorized("Unauthorized"))?;

    // 只有安全管理员和系统管理员可以删除扫描器配置
    if current_user.role != shared::Role::SysAdmin && current_user.role != shared::Role::SecAdmin {
        return Err(ApiError::forbidden("只有管理员可以删除扫描器配置"));
    }

    let removed_scanner = {
        let mut scanners = state.scanners.write()
            .map_err(|e| ApiError::internal(format!("Failed to write scanner configs: {}", e)))?;

        scanners.iter()
            .position(|s| s.id == id)
            .map(|idx| scanners.remove(idx))
    };

    let removed_scanner = removed_scanner
        .ok_or_else(|| ApiError::not_found("Scanner config not found"))?;

    // Audit log
    log_action(&state.audit_logs, &current_user, "SCANNER_DELETED", &removed_scanner.name,
              &format!("Deleted scanner config with type {}", removed_scanner.scanner_type));

    Ok(Json("Deleted".to_string()))
}
