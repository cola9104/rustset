use std::sync::{Arc, Mutex as StdMutex};
use tokio::sync::Mutex as TokioMutex;
use chrono::{DateTime, Utc};
use shared::{Asset, Task, Risk, ZoneConfig, User, AuditLog, IPZone, ScanResult, PortDetail, IPScanResult, AdvancedScanTask, CloudAsset, PasswordPolicy, BusinessResource, CloudProviderConfig};
use crate::scanners::engine::ScanManager;

#[derive(Clone)]
pub struct AppState {
    pub assets: Arc<StdMutex<Vec<Asset>>>,
    pub tasks: Arc<StdMutex<Vec<Task>>>,
    pub risks: Arc<StdMutex<Vec<Risk>>>,
    pub zones: Arc<StdMutex<Vec<ZoneConfig>>>,
    pub ip_zones: Arc<StdMutex<Vec<IPZone>>>,
    pub scan_results: Arc<StdMutex<Vec<ScanResult>>>,
    pub port_details: Arc<StdMutex<Vec<PortDetail>>>, // 端口详细信息表
    pub ip_scan_results: Arc<StdMutex<Vec<IPScanResult>>>, // IP扫描结果
    pub users: Arc<StdMutex<Vec<User>>>,
    pub audit_logs: Arc<StdMutex<Vec<AuditLog>>>,
    pub advanced_tasks: Arc<StdMutex<Vec<AdvancedScanTask>>>, // 高级扫描任务
    pub cloud_assets: Arc<StdMutex<Vec<CloudAsset>>>, // 云资产列表
    pub business_resources: Arc<StdMutex<Vec<BusinessResource>>>, // 业务资源（业务受理）
    pub cloud_provider_configs: Arc<StdMutex<Vec<CloudProviderConfig>>>, // 云区对接配置
    pub scan_manager: Arc<TokioMutex<Option<ScanManager>>>, // 扫描管理器（使用 Tokio Mutex）
    pub password_policy: Arc<StdMutex<PasswordPolicy>>, // 密码策略
    pub password_history: Arc<StdMutex<Vec<(String, String, DateTime<Utc>)>>>, // (user_id, old_password_hash, changed_at)
}
