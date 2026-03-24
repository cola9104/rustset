use crate::handlers::port_details::PortDetail;
use crate::handlers::scanners::ScanResult;
use crate::scanners::engine::ScanManager;
use chrono::{DateTime, Utc};
use shared::{
    AdvancedScanTask, Asset, AuditLog, CloudPlatform, CloudZone, CustomRole, PasswordPolicy, Risk,
    ScannerConfig, Task, User, ZoneConfig,
};
use std::sync::{Arc, RwLock as StdRwLock};
use tokio::sync::RwLock as TokioRwLock;

type PasswordHistoryEntry = (String, String, DateTime<Utc>);

#[derive(Clone)]
pub struct AppState {
    /// Use RwLock for better read performance (multiple concurrent readers)
    pub assets: Arc<StdRwLock<Vec<Asset>>>,
    pub tasks: Arc<StdRwLock<Vec<Task>>>,
    pub risks: Arc<StdRwLock<Vec<Risk>>>,
    pub zones: Arc<StdRwLock<Vec<ZoneConfig>>>,
    pub users: Arc<StdRwLock<Vec<User>>>,
    pub audit_logs: Arc<StdRwLock<Vec<AuditLog>>>,
    pub advanced_tasks: Arc<StdRwLock<Vec<AdvancedScanTask>>>,
    pub custom_roles: Arc<StdRwLock<Vec<CustomRole>>>,
    pub scan_manager: Arc<TokioRwLock<Option<ScanManager>>>,
    pub password_policy: Arc<StdRwLock<PasswordPolicy>>,
    pub password_history: Arc<StdRwLock<Vec<PasswordHistoryEntry>>>,
    #[allow(dead_code)]
    pub cloud_zones: Arc<StdRwLock<Vec<CloudZone>>>,
    #[allow(dead_code)]
    pub cloud_platforms: Arc<StdRwLock<Vec<CloudPlatform>>>,
    pub port_details: Arc<StdRwLock<Vec<PortDetail>>>,
    /// Scanner configs are lightweight admin state, kept in memory like zones/port details.
    pub scanners: Arc<StdRwLock<Vec<ScannerConfig>>>,
    pub scan_results: Arc<StdRwLock<Vec<ScanResult>>>,
}
