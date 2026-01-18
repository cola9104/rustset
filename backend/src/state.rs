use std::sync::{Arc, Mutex as StdMutex};
use tokio::sync::Mutex as TokioMutex;
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use shared::{Asset, Task, Risk, ZoneConfig, User, AuditLog, AdvancedScanTask, CloudAsset, PasswordPolicy};
use crate::scanners::engine::ScanManager;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<SqlitePool>,  // SQLite数据库连接池
    pub scan_manager: Arc<TokioMutex<Option<ScanManager>>>, // 扫描管理器（使用 Tokio Mutex）
}
