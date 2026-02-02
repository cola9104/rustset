use std::sync::{Arc, Mutex as StdMutex};
use tokio::sync::Mutex as TokioMutex;
use chrono::{DateTime, Utc};
use shared::{Asset, Task, Risk, ZoneConfig, User, AuditLog, AdvancedScanTask, PasswordPolicy, CustomRole};
use crate::scanners::engine::ScanManager;

#[derive(Clone)]
pub struct AppState {
    pub assets: Arc<StdMutex<Vec<Asset>>>,
    pub tasks: Arc<StdMutex<Vec<Task>>>,
    pub risks: Arc<StdMutex<Vec<Risk>>>,
    pub zones: Arc<StdMutex<Vec<ZoneConfig>>>,
    pub users: Arc<StdMutex<Vec<User>>>,
    pub audit_logs: Arc<StdMutex<Vec<AuditLog>>>,
    pub advanced_tasks: Arc<StdMutex<Vec<AdvancedScanTask>>>,
    pub custom_roles: Arc<StdMutex<Vec<CustomRole>>>,
    pub scan_manager: Arc<TokioMutex<Option<ScanManager>>>,
    pub password_policy: Arc<StdMutex<PasswordPolicy>>,
    pub password_history: Arc<StdMutex<Vec<(String, String, DateTime<Utc>)>>>,
}
