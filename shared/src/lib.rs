use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NetworkZone {
    Internet,
    DMZ,
    Intranet,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub id: Option<i32>,
    pub name: String,
    pub ip: String,
    pub zone: NetworkZone,
    pub ports: Vec<PortInfo>,
    pub last_scanned: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PortInfo {
    pub port: u16,
    pub is_open: bool,
    pub service: Option<String>,
    pub is_bound: bool, // If this port is manually confirmed/bound to a service
    pub owner: Option<String>,
    pub system_name: Option<String>,
    pub middleware: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortBindingRequest {
    pub owner: String,
    pub system_name: String,
    pub middleware: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub target: String,
    pub status: TaskStatus,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub found_assets: usize,
    pub found_risks: usize,
    // Config fields preserved for edit/display
    pub port_policy: String,
    pub domain_brute: bool,
    pub service_detection: bool,
    pub os_detection: bool,
    pub site_identify: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskStatus {
    Open,
    Resolved,
    Ignored,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Risk {
    pub id: String,
    pub asset_ip: String,
    pub port: u16,
    pub severity: String, // Critical, High, Medium, Low
    pub description: String,
    pub solution: Option<String>,
    pub status: RiskStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskRequest {
    pub name: String,
    pub target: String,
    pub port_policy: String, // "ALL", "TOP1000", "TOP100", "TEST"
    pub domain_brute: bool,
    pub service_detection: bool,
    pub os_detection: bool,
    pub site_identify: bool, // Web fingerprint
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanRequest {
    pub target_ip: String,
    pub ports: Vec<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ZoneConfig {
    pub id: String,
    pub name: String,
    pub cidr: String,
    pub priority: i32,
}
