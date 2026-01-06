use std::sync::{Arc, Mutex};
use shared::{Asset, Task, Risk, ZoneConfig, User, AuditLog};

#[derive(Clone)]
pub struct AppState {
    pub assets: Arc<Mutex<Vec<Asset>>>,
    pub tasks: Arc<Mutex<Vec<Task>>>,
    pub risks: Arc<Mutex<Vec<Risk>>>,
    pub zones: Arc<Mutex<Vec<ZoneConfig>>>,
    pub users: Arc<Mutex<Vec<User>>>,
    pub audit_logs: Arc<Mutex<Vec<AuditLog>>>,
}
