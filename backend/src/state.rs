use std::sync::{Arc, RwLock as StdRwLock};
use tokio::sync::RwLock as TokioRwLock;
use chrono::{DateTime, Utc};
use shared::{Asset, Task, Risk, ZoneConfig, User, AuditLog, AdvancedScanTask, PasswordPolicy, CustomRole, CloudZone, CloudPlatform};
use crate::scanners::engine::ScanManager;

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
    pub password_history: Arc<StdRwLock<Vec<(String, String, DateTime<Utc>)>>>,
    pub cloud_zones: Arc<StdRwLock<Vec<CloudZone>>>,
    pub cloud_platforms: Arc<StdRwLock<Vec<CloudPlatform>>>,
}
