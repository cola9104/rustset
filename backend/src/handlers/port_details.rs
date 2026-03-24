//! Port Details Handler - 端口详细信息管理
//!
//! 功能：
//! - 端口服务识别
//! - 端口风险评分
//! - 批量端口绑定

use axum::{
    extract::{Json, Path, State},
    http::HeaderMap,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::middleware::ApiError;
use crate::state::AppState;
use crate::utils::get_current_user;

/// 端口详细信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortDetail {
    pub id: Option<i32>,
    pub port: u16,
    pub protocol: String, // tcp, udp
    pub service: Option<String>,
    pub version: Option<String>,
    pub banner: Option<String>,
    pub risk_level: Option<String>, // low, medium, high, critical
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
}

/// 创建端口详情请求
#[derive(Debug, Deserialize)]
pub struct CreatePortDetailRequest {
    pub port: u16,
    pub protocol: String,
    pub service: Option<String>,
    pub version: Option<String>,
    pub banner: Option<String>,
    pub risk_level: Option<String>,
    pub description: Option<String>,
}

/// 批量绑定请求
#[derive(Debug, Deserialize)]
pub struct BatchBindPortsRequest {
    pub asset_id: i32,
    pub ports: Vec<PortBinding>,
}

#[derive(Debug, Deserialize)]
pub struct PortBinding {
    pub port: u16,
    pub protocol: String,
    pub service: Option<String>,
}

/// 端口服务识别
pub fn identify_service(port: u16, protocol: &str) -> Option<(String, String)> {
    let services = [
        (21, "tcp", "FTP", "File Transfer Protocol"),
        (22, "tcp", "SSH", "Secure Shell"),
        (23, "tcp", "Telnet", "Telnet Protocol"),
        (25, "tcp", "SMTP", "Mail Transfer"),
        (53, "udp", "DNS", "Domain Name System"),
        (80, "tcp", "HTTP", "Web Server"),
        (110, "tcp", "POP3", "Post Office Protocol"),
        (143, "tcp", "IMAP", "Internet Message Access Protocol"),
        (443, "tcp", "HTTPS", "Secure Web Server"),
        (445, "tcp", "SMB", "Server Message Block"),
        (993, "tcp", "IMAPS", "Secure IMAP"),
        (995, "tcp", "POP3S", "Secure POP3"),
        (3306, "tcp", "MySQL", "MySQL Database"),
        (3389, "tcp", "RDP", "Remote Desktop Protocol"),
        (5432, "tcp", "PostgreSQL", "PostgreSQL Database"),
        (6379, "tcp", "Redis", "Redis Cache"),
        (8080, "tcp", "HTTP-Alt", "Alternative HTTP"),
        (8443, "tcp", "HTTPS-Alt", "Alternative HTTPS"),
        (27017, "tcp", "MongoDB", "MongoDB Database"),
    ];

    for (p, proto, name, desc) in services {
        if p == port && proto == protocol {
            return Some((name.to_string(), desc.to_string()));
        }
    }
    None
}

/// 获取端口风险等级
pub fn get_port_risk(port: u16, service: Option<&str>) -> String {
    // 高风险端口
    let high_risk = [21, 23, 445, 135, 139, 3389];
    // 中风险端口
    let medium_risk = [22, 25, 80, 8080, 8443];

    if high_risk.contains(&port) {
        "high".to_string()
    } else if medium_risk.contains(&port) {
        "medium".to_string()
    } else if let Some(svc) = service {
        if svc.contains("Database") || svc.contains("DB") {
            "medium".to_string()
        } else {
            "low".to_string()
        }
    } else {
        "low".to_string()
    }
}

/// 获取所有端口详情
pub async fn get_port_details(
    State(state): State<AppState>,
    _headers: HeaderMap,
) -> Result<Json<Vec<PortDetail>>, ApiError> {
    // 从内存中获取（实际应从数据库）
    let ports = state
        .port_details
        .read()
        .map_err(|e| ApiError::internal(format!("Failed to read ports: {}", e)))?;
    Ok(Json(ports.clone()))
}

/// 获取单个端口详情
pub async fn get_port_detail(
    State(state): State<AppState>,
    _headers: HeaderMap,
    Path(id): Path<i32>,
) -> Result<Json<PortDetail>, ApiError> {
    let ports = state
        .port_details
        .read()
        .map_err(|e| ApiError::internal(format!("Failed to read ports: {}", e)))?;

    ports
        .iter()
        .find(|p| p.id == Some(id))
        .cloned()
        .map(Json)
        .ok_or_else(|| ApiError::not_found("Port detail not found"))
}

/// 创建端口详情
pub async fn create_port_detail(
    State(state): State<AppState>,
    _headers: HeaderMap,
    Json(req): Json<CreatePortDetailRequest>,
) -> Result<Json<PortDetail>, ApiError> {
    let mut ports = state
        .port_details
        .write()
        .map_err(|e| ApiError::internal(format!("Failed to write ports: {}", e)))?;

    // 自动识别服务
    let (service, description) = if req.service.is_none() {
        identify_service(req.port, &req.protocol)
            .map(|(s, d)| (Some(s), Some(d)))
            .unwrap_or((None, req.description.clone()))
    } else {
        (req.service.clone(), req.description.clone())
    };

    // 自动评估风险
    let risk_level = req
        .risk_level
        .clone()
        .unwrap_or_else(|| get_port_risk(req.port, service.as_deref()));

    let new_port = PortDetail {
        id: Some(ports.len() as i32 + 1),
        port: req.port,
        protocol: req.protocol,
        service,
        version: req.version,
        banner: req.banner,
        risk_level: Some(risk_level),
        description,
        created_at: Utc::now().to_rfc3339(),
        updated_at: None,
    };

    ports.push(new_port.clone());
    Ok(Json(new_port))
}

/// 更新端口详情
pub async fn update_port_detail(
    State(state): State<AppState>,
    _headers: HeaderMap,
    Path(id): Path<i32>,
    Json(req): Json<CreatePortDetailRequest>,
) -> Result<Json<PortDetail>, ApiError> {
    let mut ports = state
        .port_details
        .write()
        .map_err(|e| ApiError::internal(format!("Failed to write ports: {}", e)))?;

    let port = ports
        .iter_mut()
        .find(|p| p.id == Some(id))
        .ok_or_else(|| ApiError::not_found("Port detail not found"))?;

    port.port = req.port;
    port.protocol = req.protocol;
    port.service = req.service;
    port.version = req.version;
    port.banner = req.banner;
    port.risk_level = req.risk_level;
    port.description = req.description;
    port.updated_at = Some(Utc::now().to_rfc3339());

    Ok(Json(port.clone()))
}

/// 删除端口详情
pub async fn delete_port_detail(
    State(state): State<AppState>,
    _headers: HeaderMap,
    Path(id): Path<i32>,
) -> Result<Json<String>, ApiError> {
    let mut ports = state
        .port_details
        .write()
        .map_err(|e| ApiError::internal(format!("Failed to write ports: {}", e)))?;

    let initial_len = ports.len();
    ports.retain(|p| p.id != Some(id));

    if ports.len() == initial_len {
        return Err(ApiError::not_found("Port detail not found"));
    }

    Ok(Json("Deleted".to_string()))
}

/// 批量绑定端口到资产
pub async fn batch_bind_ports(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<BatchBindPortsRequest>,
) -> Result<Json<Vec<shared::PortInfo>>, ApiError> {
    let _user = get_current_user(&headers, &state.users)
        .ok_or_else(|| ApiError::unauthorized("Unauthorized"))?;

    let mut assets = state
        .assets
        .write()
        .map_err(|e| ApiError::internal(format!("Failed to write assets: {}", e)))?;

    let asset = assets
        .iter_mut()
        .find(|a| a.id == Some(req.asset_id))
        .ok_or_else(|| ApiError::not_found("Asset not found"))?;

    let port_infos: Vec<shared::PortInfo> = req
        .ports
        .into_iter()
        .map(|p| {
            let service = identify_service(p.port, &p.protocol)
                .map(|(s, _)| Some(s.clone()))
                .unwrap_or(p.service.clone());

            shared::PortInfo {
                port: p.port,
                is_open: true,
                service,
                version: None,
                banner: None,
                is_bound: true,
                system_name: None,
                middleware: None,
                created_by: Some(_user.username.clone()),
                updated_by: None,
            }
        })
        .collect();

    asset.ports = port_infos.clone();
    Ok(Json(port_infos))
}
