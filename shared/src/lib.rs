use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NetworkZone {
    Internet,
    DMZ,
    Intranet,
    Custom(String),
}

// ============== Multi-Cloud Management Types ==============

/// 云服务提供商类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CloudProvider {
    #[serde(rename = "aliyun")]
    Aliyun,        // 阿里云
    #[serde(rename = "tencent")]
    Tencent,       // 腾讯云
    #[serde(rename = "huawei")]
    Huawei,        // 华为云
    #[serde(rename = "aws")]
    Aws,           // AWS
    #[serde(rename = "azure")]
    Azure,         // Azure
    #[serde(rename = "gcp")]
    Gcp,           // Google Cloud
    #[serde(rename = "baidu")]
    Baidu,         // 百度云
    #[serde(rename = "custom")]
    Custom(String), // 自定义/其他
}

impl CloudProvider {
    pub fn as_str(&self) -> &str {
        match self {
            CloudProvider::Aliyun => "aliyun",
            CloudProvider::Tencent => "tencent",
            CloudProvider::Huawei => "huawei",
            CloudProvider::Aws => "aws",
            CloudProvider::Azure => "azure",
            CloudProvider::Gcp => "gcp",
            CloudProvider::Baidu => "baidu",
            CloudProvider::Custom(s) => s,
        }
    }
}

/// 云区域/可用区
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudRegion {
    pub provider: CloudProvider,
    pub region_id: String,     // 如 cn-hangzhou, ap-guangzhou
    pub zone_id: Option<String>, // 可用区，如 cn-hangzhou-i
    pub region_name: String,   // 如 华东1(杭州)
}

/// 计费模式
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BillingMode {
    #[serde(rename = "pay_as_you_go")]
    PayAsYouGo,     // 按量付费
    #[serde(rename = "subscription")]
    Subscription,   // 包年包月/订阅
    #[serde(rename = "spot")]
    Spot,          // 抢占式实例
}

/// 实例状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VMStatus {
    #[serde(rename = "running")]
    Running,        // 运行中
    #[serde(rename = "stopped")]
    Stopped,        // 已停止
    #[serde(rename = "starting")]
    Starting,       // 启动中
    #[serde(rename = "stopping")]
    Stopping,       // 停止中
    #[serde(rename = "rebooting")]
    Rebooting,      // 重启中
    #[serde(rename = "deleted")]
    Deleted,        // 已释放
    #[serde(rename = "error")]
    Error,          // 异常
}

/// 实例规格信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceSpec {
    pub instance_type: String,   // 如 ecs.g6.large
    pub cpu_cores: u32,          // CPU核数
    pub memory_gb: u32,          // 内存GB
    pub cpu_arch: Option<String>, // CPU架构，如 x86, arm
    pub gpu_spec: Option<String>, // GPU规格（如果有）
    pub bandwidth_mbps: Option<u32>, // 公网带宽Mbps
}

/// 系统盘信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemDisk {
    pub disk_type: String,       // 如 cloud_ssd, cloud_essd
    pub size_gb: u32,            // 容量GB
    pub category: Option<String>, // 如 IOPS, throughput
    pub performance_level: Option<String>, // ESSD性能级别 PL0/PL1/PL2/PL3
}

/// 云盘信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudDisk {
    pub disk_id: String,
    pub disk_name: String,
    pub disk_type: String,       // cloud, cloud_ssd, cloud_essd, cloud_efficiency
    pub size_gb: u32,
    pub status: String,          // in_use, available, attaching
    pub category: Option<String>,
    pub iops: Option<u32>,
    pub throughput_mb: Option<f32>,
    pub is_snapshot: bool,       // 是否从快照创建
}

/// 快照信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotInfo {
    pub has_snapshot: bool,
    pub snapshot_count: u32,
    pub latest_snapshot_time: Option<DateTime<Utc>>,
    pub total_snapshot_size_gb: u32,
}

/// 部门信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Department {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub level: u32,
}

/// 项目信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub code: String,            // 项目编码
    pub department_id: Option<String>,
}

/// 负责人信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactPerson {
    pub id: String,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub department: Option<String>,
}

/// 混合云虚拟机资产
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudAsset {
    // 基础信息
    pub id: Option<i32>,
    pub asset_name: String,          // 资产名称

    // 实例规格
    pub spec: InstanceSpec,

    // 系统盘
    pub system_disk: SystemDisk,

    // 云区信息
    pub cloud_region: CloudRegion,

    // IP地址
    pub public_ip: Option<String>,   // 公网IP
    pub private_ip: String,          // 内网IP
    pub ipv6_address: Option<String>, // IPv6地址

    // 计费与到期
    pub billing_mode: BillingMode,
    pub expire_time: Option<DateTime<Utc>>, // 到期时间

    // 状态
    pub status: VMStatus,

    // 操作系统
    pub os_type: String,             // 操作系统类型，如 Linux, Windows
    pub os_name: String,             // 如 CentOS 7.9, Windows Server 2019
    pub os_arch: Option<String>,     // x86_64, arm64
    pub image_id: String,            // 镜像ID
    pub image_name: Option<String>,  // 镜像名称

    // 组织信息
    pub department: Department,      // 部门
    pub project: Project,            // 项目
    pub owner: ContactPerson,        // 负责人

    // 时间信息
    pub created_at: DateTime<Utc>,   // 虚拟机创建时间
    pub last_synced: Option<DateTime<Utc>>, // 最后同步时间

    // 云盘信息
    pub cloud_disks: Vec<CloudDisk>,
    pub cloud_disk_count: u32,       // 云盘数量
    pub cloud_disk_total_size_gb: u32, // 云盘数据总量GB

    // 快照信息
    pub snapshot_info: SnapshotInfo, // 是否有快照等

    // 实例ID（云厂商返回的）
    pub instance_id: String,         // 实例唯一标识

    // 标签
    pub tags: Vec<String>,
    pub charge_type: Option<String>, // 付费类型 PostPaid/PrePaid
}

/// 创建云资产请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCloudAssetRequest {
    pub asset_name: String,
    pub instance_id: String,
    pub cloud_provider: CloudProvider,
    pub region_id: String,
    pub zone_id: Option<String>,
    pub instance_type: String,
    pub cpu_cores: u32,
    pub memory_gb: u32,
    pub system_disk_type: String,
    pub system_disk_size_gb: u32,
    pub public_ip: Option<String>,
    pub private_ip: String,
    pub ipv6_address: Option<String>,
    pub billing_mode: BillingMode,
    pub expire_time: Option<DateTime<Utc>>,
    pub os_type: String,
    pub os_name: String,
    pub image_id: String,
    pub image_name: Option<String>,
    pub department_id: String,
    pub department_name: String,
    pub project_id: String,
    pub project_name: String,
    pub project_code: String,
    pub owner_id: String,
    pub owner_name: String,
    pub owner_email: Option<String>,
    pub owner_phone: Option<String>,
    pub cloud_disks: Vec<CloudDisk>,
    pub has_snapshot: bool,
    pub snapshot_count: u32,
    pub tags: Vec<String>,
    pub bandwidth_mbps: Option<u32>,
}

/// 更新云资产请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCloudAssetRequest {
    pub asset_name: Option<String>,
    pub status: Option<VMStatus>,
    pub expire_time: Option<DateTime<Utc>>,
    pub owner_id: Option<String>,
    pub owner_name: Option<String>,
    pub department_id: Option<String>,
    pub project_id: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// 云资产列表查询参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudAssetQuery {
    pub provider: Option<CloudProvider>,
    pub region_id: Option<String>,
    pub status: Option<VMStatus>,
    pub department_id: Option<String>,
    pub project_id: Option<String>,
    pub owner_id: Option<String>,
    pub search_keyword: Option<String>, // 搜索资产名称或实例ID
    pub expire_soon_days: Option<u32>,  // 即将到期天数
}

/// 云资产统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudAssetStats {
    pub total_count: u32,
    pub running_count: u32,
    pub stopped_count: u32,
    pub total_cpu_cores: u32,
    pub total_memory_gb: u32,
    pub total_disk_size_gb: u32,
    pub expiring_soon_count: u32,      // 即将到期数量
    pub by_provider: Vec<(CloudProvider, u32)>, // 按厂商统计
    pub by_department: Vec<(String, u32)>,      // 按部门统计
    pub by_project: Vec<(String, u32)>,         // 按项目统计
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
    // Asset Mapping features
    pub owner: Option<String>,      // Responsibility
    pub weight: i32,                // Asset Importance (1-100)
    pub labels: Vec<String>,        // Multi-level tags
    pub os: Option<String>,         // OS Fingerprint
    pub device_type: Option<String>,// Device Type Fingerprint
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PortInfo {
    pub port: u16,
    pub is_open: bool,
    pub service: Option<String>,
    pub version: Option<String>,    // Service Version
    pub banner: Option<String>,     // Service Banner
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
    Verified,
    Resolved,
    Ignored,
    FalsePositive,
    PendingReview,
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
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub assigned_to: Option<String>, // Responsibility
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
