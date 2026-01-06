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
    pub contact_person: Option<String>,
    pub contact_phone: Option<String>,
    // New: Track creator/modifier
    pub created_by: Option<String>, // username
    pub updated_by: Option<String>, // username
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PortInfo {
    pub port: u16,
    pub is_open: bool,
    pub service: Option<String>,
    pub is_bound: bool, // If this port is manually confirmed/bound to a service
    pub system_name: Option<String>,
    pub middleware: Option<String>,
    // New: Track creator/modifier for ports if needed, but usually Asset level is enough or tracked via logs.
    // User asked to "bind corresponding operation account" when adding ports.
    pub created_by: Option<String>,
    pub updated_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortBindingRequest {
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
    pub created_by: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneConfig {
    pub id: String,
    pub name: String,
    pub cidr: String,
    pub priority: i32,
}

// --- Auth & Audit ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Role {
    SysAdmin, // Manage users
    SecAdmin, // Manage assets, tasks, risks
    Auditor,  // View logs
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: String,
    pub username: String,
    #[serde(skip_serializing, default)] // Don't send password hash to frontend, allow missing on receive
    pub password: String, // In real app, this is a hash. For demo, we might store plain or simple hash.
    pub role: Role,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub role: Role,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub password: Option<String>,
    pub role: Option<Role>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: String,
    pub user_id: String,
    pub username: String,
    pub action: String, // e.g., "CREATE_ASSET", "LOGIN", "DELETE_USER"
    pub target: String, // e.g., "Asset: 1", "User: admin"
    pub details: String, // JSON or text description
    pub timestamp: DateTime<Utc>,
}
