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
    /// 关联的业务资源ID（从业务申请录入时使用）
    pub business_resource_id: Option<i32>,
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
    pub business_resource_id: Option<i32>, // 按业务资源ID筛选
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

// ============== Business Resource / 业务受理 Types ==============

/// 资源类型 - 支持云资源和物理机
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ResourceType {
    #[serde(rename = "cloud")]
    Cloud,           // 云服务器 (ECS/云主机)
    #[serde(rename = "physical")]
    Physical,        // 物理机
}

impl ResourceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ResourceType::Cloud => "cloud",
            ResourceType::Physical => "physical",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "cloud" => Some(ResourceType::Cloud),
            "physical" => Some(ResourceType::Physical),
            _ => None,
        }
    }
}

/// 物理机特有信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalMachineInfo {
    pub serial_number: String,           // 设备序列号
    pub rack_location: Option<String>,    // 机架位置 (如: A区-03机柜-U12)
    pub hardware_model: Option<String>,   // 硬件型号 (如: Dell PowerEdge R740)
    pub warranty_expiry: Option<DateTime<Utc>>, // 维保到期时间
    pub agent_status: Option<String>,     // Agent 状态 (installed/online/offline/none)
    pub ipmi_address: Option<String>,     // IPMI/iDRAC 地址
    pub ipmi_username: Option<String>,    // IPMI 用户名
    pub ipmi_password: Option<String>,    // IPMI 密码
}

/// 业务受理单 - 云资源管理 & 物理机管理
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessResource {
    pub id: Option<i32>,

    // 资源类型
    pub resource_type: String,          // 资源类型: cloud(云服务器) / physical(物理机)

    // 基本信息
    pub ecs_name: String,                // ECS名称/物理机名称
    pub ecs_status: String,              // ECS状态 (运行中/已停止/已释放等)
    pub resource_id: String,             // 资源ID
    pub cloud_region: String,            // 云区域 (如 华东1-杭州) 或 机房位置
    pub cloud_category: String,          // 云类别 (阿里云/腾讯云/华为云/AWS等) 或 机房名称
    pub cloud_provider_config_id: Option<i32>,  // 关联的云区对接配置ID
    pub county_city: Option<String>,     // 县市区
    pub vdc_name: Option<String>,        // VDC名称

    // 业务信息
    pub customer_name: String,           // 客户名称
    pub application_name: Option<String>, // 应用名称
    pub contract_name: Option<String>,  // 合同名称
    pub instance_id: String,             // 实例ID / 物理机序列号
    pub ecs_type: String,               // ECS类型 (如 ecs.g6.large) 或 物理机型号

    // 配置信息
    pub ecs_os: String,                 // ECS操作系统 (如 CentOS 7.9, Windows Server 2019)
    pub cpu_cores: u32,                 // CPU核数
    pub memory_gb: u32,                 // 内存(GB)
    pub system_disk: String,            // 系统盘类型 (如 cloud_ssd, cloud_essd) 或 物理磁盘类型
    pub system_disk_size_gb: u32,       // 系统盘大小(GB)
    pub data_disk: Option<String>,      // 数据盘信息 (JSON字符串存储多块盘信息)

    // 时间信息
    pub completion_time: Option<DateTime<Utc>>, // 完成时间
    pub release_time: Option<DateTime<Utc>>,   // 释放时间

    // 安全产品
    pub has_security_product: bool,    // 是否创建安全产品

    // 网络信息
    pub ip_address: String,             // IP地址
    pub ecs_login_method: Option<String>, // ECS远程登录方式 (SSH/RDP/堡垒机等)
    pub ecs_login_username: Option<String>, // ECS登录用户名
    pub ecs_initial_password: Option<String>, // ECS初始密码

    // 堡垒机信息
    pub bastion_address: Option<String>,     // 堡垒机地址
    pub bastion_admin_account: Option<String>, // 堡垒机管理员账号
    pub bastion_initial_password: Option<String>, // 堡垒机初始密码

    // 物理机特有信息 (仅当 resource_type = physical 时使用)
    pub serial_number: Option<String>,         // 设备序列号
    pub rack_location: Option<String>,         // 机架位置
    pub hardware_model: Option<String>,        // 硬件型号
    pub warranty_expiry: Option<DateTime<Utc>>, // 维保到期时间
    pub agent_status: Option<String>,          // Agent 状态
    pub ipmi_address: Option<String>,          // IPMI/iDRAC 地址

    // 其他
    pub remarks: Option<String>,         // 备注
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub created_by: Option<String>,
    pub updated_by: Option<String>,
}

/// 创建业务资源请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBusinessResourceRequest {
    pub resource_type: String,         // 资源类型: cloud / physical
    pub ecs_name: String,
    pub ecs_status: String,
    pub resource_id: String,
    pub cloud_region: String,
    pub cloud_category: String,
    pub cloud_provider_config_id: Option<i32>,  // 关联的云区对接配置ID
    pub county_city: Option<String>,
    pub vdc_name: Option<String>,
    pub customer_name: String,
    pub application_name: Option<String>,
    pub contract_name: Option<String>,
    pub instance_id: String,
    pub ecs_type: String,
    pub ecs_os: String,
    pub cpu_cores: u32,
    pub memory_gb: u32,
    pub system_disk: String,
    pub system_disk_size_gb: u32,
    pub data_disk: Option<String>,
    pub completion_time: Option<DateTime<Utc>>,
    pub release_time: Option<DateTime<Utc>>,
    pub has_security_product: bool,
    pub ip_address: String,
    pub ecs_login_method: Option<String>,
    pub ecs_login_username: Option<String>,
    pub ecs_initial_password: Option<String>,
    pub bastion_address: Option<String>,
    pub bastion_admin_account: Option<String>,
    pub bastion_initial_password: Option<String>,
    // 物理机特有字段
    pub serial_number: Option<String>,         // 设备序列号
    pub rack_location: Option<String>,         // 机架位置
    pub hardware_model: Option<String>,        // 硬件型号
    pub warranty_expiry: Option<DateTime<Utc>>, // 维保到期时间
    pub ipmi_address: Option<String>,          // IPMI/iDRAC 地址
    pub remarks: Option<String>,
}

/// 更新业务资源请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateBusinessResourceRequest {
    pub resource_type: Option<String>,         // 资源类型
    pub ecs_name: Option<String>,
    pub ecs_status: Option<String>,
    pub cloud_region: Option<String>,
    pub cloud_category: Option<String>,
    pub cloud_provider_config_id: Option<i32>,  // 关联的云区对接配置ID
    pub county_city: Option<String>,
    pub vdc_name: Option<String>,
    pub customer_name: Option<String>,
    pub application_name: Option<String>,
    pub contract_name: Option<String>,
    pub ecs_type: Option<String>,
    pub ecs_os: Option<String>,
    pub cpu_cores: Option<u32>,
    pub memory_gb: Option<u32>,
    pub system_disk: Option<String>,
    pub system_disk_size_gb: Option<u32>,
    pub data_disk: Option<String>,
    pub completion_time: Option<DateTime<Utc>>,
    pub release_time: Option<DateTime<Utc>>,
    pub has_security_product: Option<bool>,
    pub ip_address: Option<String>,
    pub ecs_login_method: Option<String>,
    pub ecs_login_username: Option<String>,
    pub ecs_initial_password: Option<String>,
    pub bastion_address: Option<String>,
    pub bastion_admin_account: Option<String>,
    pub bastion_initial_password: Option<String>,
    // 物理机特有字段
    pub serial_number: Option<String>,         // 设备序列号
    pub rack_location: Option<String>,         // 机架位置
    pub hardware_model: Option<String>,        // 硬件型号
    pub warranty_expiry: Option<DateTime<Utc>>, // 维保到期时间
    pub agent_status: Option<String>,          // Agent 状态
    pub ipmi_address: Option<String>,          // IPMI/iDRAC 地址
    pub remarks: Option<String>,
}

impl Default for UpdateBusinessResourceRequest {
    fn default() -> Self {
        Self {
            resource_type: None,
            ecs_name: None,
            ecs_status: None,
            cloud_region: None,
            cloud_category: None,
            cloud_provider_config_id: None,
            county_city: None,
            vdc_name: None,
            customer_name: None,
            application_name: None,
            contract_name: None,
            ecs_type: None,
            ecs_os: None,
            cpu_cores: None,
            memory_gb: None,
            system_disk: None,
            system_disk_size_gb: None,
            data_disk: None,
            completion_time: None,
            release_time: None,
            has_security_product: None,
            ip_address: None,
            ecs_login_method: None,
            ecs_login_username: None,
            ecs_initial_password: None,
            bastion_address: None,
            bastion_admin_account: None,
            bastion_initial_password: None,
            serial_number: None,
            rack_location: None,
            hardware_model: None,
            warranty_expiry: None,
            agent_status: None,
            ipmi_address: None,
            remarks: None,
        }
    }
}

/// 业务资源查询参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessResourceQuery {
    pub search_keyword: Option<String>,     // 搜索关键词（ECS名称、客户名称、IP地址等）
    pub resource_type: Option<String>,      // 资源类型筛选 (cloud/physical)
    pub cloud_category: Option<String>,     // 云类别/机房筛选
    pub ecs_status: Option<String>,         // 状态筛选
    pub customer_name: Option<String>,      // 客户名称筛选
    pub county_city: Option<String>,        // 县市区筛选
    pub application_name: Option<String>,   // 应用名称筛选
    pub contract_name: Option<String>,      // 合同名称筛选
}

/// 业务资源统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessResourceStats {
    pub total_count: u32,
    pub running_count: u32,
    pub stopped_count: u32,
    pub released_count: u32,
    pub total_cpu_cores: u32,
    pub total_memory_gb: u32,
    pub with_security_product_count: u32,
    pub cloud_count: u32,                   // 云资源数量
    pub physical_count: u32,                // 物理机数量
    pub by_category: Vec<(String, u32)>,     // 按云类别统计
    pub by_customer: Vec<(String, u32)>,     // 按客户统计
    pub by_county: Vec<(String, u32)>,       // 按县市区统计
    pub by_application: Vec<(String, u32)>,  // 按应用统计
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

/// IP区域配置 - 支持自定义网段划分
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IPZone {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub cidr: String,
    pub cloud_region: Option<CloudRegion>,
    pub is_cloud: bool,
    pub priority: i32,
    pub scan_enabled: bool,           // 是否启用扫描
    pub auto_discover: bool,          // 是否自动发现
    pub port_scan_policy: PortScanPolicy, // 端口扫描策略
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub created_by: Option<String>,
}

/// 端口扫描策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortScanPolicy {
    pub policy_type: String, // "TOP100", "TOP1000", "COMMON", "CUSTOM", "ALL"
    pub custom_ports: Option<Vec<u16>>, // 自定义端口列表
    pub scan_timeout_seconds: u32, // 扫描超时时间
    pub max_concurrent: u32, // 最大并发数
}

/// 端口详细信息 - 可编辑的web系统等信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortDetail {
    pub id: String,
    pub asset_id: String,           // 关联的资产ID
    pub ip: String,                 // IP地址
    pub port: u16,                  // 端口号
    pub protocol: String,           // TCP/UDP
    pub status: String,             // open/closed/filtered
    pub service_name: Option<String>, // 服务名称
    pub service_version: Option<String>, // 服务版本
    pub service_banner: Option<String>, // 服务Banner

    // 资产管理字段 - 可编辑
    pub system_name: Option<String>,    // 系统名称
    pub system_type: Option<String>,    // 系统类型 (Web系统/数据库/中间件等)
    pub system_url: Option<String>,     // 访问URL (for web systems)
    pub middleware: Option<String>,     // 中间件信息
    pub framework: Option<String>,      // 框架信息
    pub language: Option<String>,       // 开发语言

    // 环境信息
    pub environment: Option<String>,    // 环境 (生产/测试/开发)
    pub department: Option<String>,     // 部门
    pub owner: Option<String>,          // 负责人
    pub owner_contact: Option<String>,  // 负责人联系方式

    // 安全信息
    pub vulnerability_level: Option<String>, // 风险等级
    pub has_vulnerability: bool,         // 是否存在漏洞
    pub vulnerability_count: i32,        // 漏洞数量

    // 指纹信息
    pub fingerprint: Option<ServiceFingerprint>, // 服务指纹

    // 元数据
    pub is_bound: bool,              // 是否已绑定/确认
    pub notes: Option<String>,       // 备注
    pub last_scanned: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub created_by: Option<String>,
    pub updated_by: Option<String>,
}

/// 服务指纹信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceFingerprint {
    pub service: Option<String>,      // 服务名称
    pub product: Option<String>,      // 产品名称
    pub version: Option<String>,      // 版本
    pub extra_info: Option<String>,   // 额外信息
    pub cpe: Option<String>,         // CPE标识
    pub os_match: Option<String>,    // 操作系统匹配
    pub confidence: i32,             // 置信度
}

/// IP扫描结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IPScanResult {
    pub id: String,
    pub ip_zone_id: String,          // 所属IP区域
    pub ip: String,
    pub is_alive: bool,
    pub hostname: Option<String>,    // 主机名
    pub mac_address: Option<String>, // MAC地址
    pub open_ports: Vec<PortDetail>,
    pub os_fingerprint: Option<String>, // 操作系统指纹
    pub device_type: Option<String>,  // 设备类型
    pub confidence: i32,              // 扫描置信度
    pub scan_time: DateTime<Utc>,
    pub scan_duration_ms: i64,       // 扫描耗时
    pub scanned_by: String,          // 扫描人
}

/// 批量IP扫描请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchIPScanRequest {
    pub ip_zone_id: String,          // IP区域ID
    pub ip_ranges: Vec<String>,      // IP范围列表，如 ["192.168.1.1-192.168.1.100", "192.168.2.0/24"]
    pub port_policy: PortScanPolicy,
    pub ping_check: bool,            // 是否先ping检查
    pub max_concurrent: u32,         // 最大并发数
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub ip: String,
    pub is_alive: bool,
    pub open_ports: Vec<PortInfo>,
    pub fingerprint: Option<String>,
    pub scanned_at: DateTime<Utc>,
}

// --- Auth & Audit ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Role {
    SysAdmin, // Manage users and permissions
    SecAdmin, // Manage assets, tasks, risks
    Auditor,  // View logs
    Custom(String), // Custom role with specific permissions
}

/// 细化权限位掩码
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Permissions {
    // 扫描权限
    pub can_create_scan: bool,
    pub can_delete_scan: bool,
    pub can_export_scan: bool,

    // 资产权限
    pub can_view_assets: bool,
    pub can_create_asset: bool,
    pub can_update_asset: bool,
    pub can_delete_asset: bool,

    // 云资产权限
    pub can_view_cloud: bool,
    pub can_manage_cloud: bool,
    pub can_delete_cloud: bool,
    pub can_sync_cloud: bool,

    // 风险权限
    pub can_view_risks: bool,
    pub can_resolve_risk: bool,
    pub can_delete_risk: bool,

    // 用户管理权限
    pub can_view_users: bool,
    pub can_create_user: bool,
    pub can_update_user: bool,
    pub can_delete_user: bool,
    pub can_manage_permissions: bool,

    // 审计权限
    pub can_view_audit_logs: bool,

    // 区域管理权限
    pub can_view_zones: bool,
    pub can_manage_zones: bool,
}

impl Default for Permissions {
    fn default() -> Self {
        Self {
            can_create_scan: false,
            can_delete_scan: false,
            can_export_scan: false,
            can_view_assets: false,
            can_create_asset: false,
            can_update_asset: false,
            can_delete_asset: false,
            can_view_cloud: false,
            can_manage_cloud: false,
            can_delete_cloud: false,
            can_sync_cloud: false,
            can_view_risks: false,
            can_resolve_risk: false,
            can_delete_risk: false,
            can_view_users: false,
            can_create_user: false,
            can_update_user: false,
            can_delete_user: false,
            can_manage_permissions: false,
            can_view_audit_logs: false,
            can_view_zones: false,
            can_manage_zones: false,
        }
    }
}

impl Permissions {
    /// SysAdmin 默认权限
    pub fn sys_admin() -> Self {
        Self {
            can_view_users: true,
            can_create_user: true,
            can_update_user: true,
            can_delete_user: true,
            can_manage_permissions: true,
            can_view_audit_logs: true,
            can_view_cloud: true,
            can_view_assets: true,
            can_view_zones: true,
            can_view_risks: true,
            ..Default::default()
        }
    }

    /// SecAdmin 默认权限
    pub fn sec_admin() -> Self {
        Self {
            can_create_scan: true,
            can_delete_scan: true,
            can_export_scan: true,
            can_view_assets: true,
            can_create_asset: true,
            can_update_asset: true,
            can_delete_asset: true,
            can_view_cloud: true,
            can_manage_cloud: true,
            can_delete_cloud: true,
            can_sync_cloud: true,
            can_view_risks: true,
            can_resolve_risk: true,
            can_delete_risk: true,
            can_view_zones: true,
            can_manage_zones: true,
            can_view_audit_logs: true,
            ..Default::default()
        }
    }

    /// Auditor 默认权限
    pub fn auditor() -> Self {
        Self {
            can_view_assets: true,
            can_view_cloud: true,
            can_view_risks: true,
            can_view_audit_logs: true,
            can_export_scan: true,
            can_view_users: true,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: String,
    pub username: String,
    #[serde(skip_serializing, default)] // Don't send password hash to frontend, allow missing on receive
    pub password: String, // In real app, this is a hash. For demo, we might store plain or simple hash.
    pub role: Role,
    pub permissions: Option<Permissions>, // 细化权限（如果 role 是 Custom）
    pub created_at: DateTime<Utc>,
    // 密码策略相关
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_changed_at: Option<DateTime<Utc>>, // 最后修改密码时间
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_strength: Option<String>, // 密码强度：weak/medium/strong
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_password_change: Option<bool>, // 是否强制修改密码
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_login_at: Option<DateTime<Utc>>, // 最后登录时间
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>, // 邮箱
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>, // 手机号
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>, // 账户状态：active/disabled/locked
    // 账户锁定相关
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed_login_attempts: Option<u32>, // 失败登录次数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locked_until: Option<DateTime<Utc>>, // 锁定到期时间
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub role: Role,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordPolicy {
    pub min_length: u32,           // 最小长度
    pub require_uppercase: bool,    // 需要大写字母
    pub require_lowercase: bool,    // 需要小写字母
    pub require_number: bool,       // 需要数字
    pub require_special: bool,      // 需要特殊字符
    pub max_age_days: Option<u32>,  // 密码最大有效期（天）
    pub prevent_reuse: u32,         // 防止重用最近N次密码
    pub min_strength: String,       // 最低强度要求：weak/medium/strong
    // 账户锁定配置
    pub max_login_attempts: Option<u32>,  // 最大登录失败次数，None表示不限制
    pub lockout_duration_minutes: u32,    // 账户锁定时长（分钟）
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            min_length: 6,
            require_uppercase: false,
            require_lowercase: false,
            require_number: false,
            require_special: false,
            max_age_days: Some(90),  // 默认90天有效期
            prevent_reuse: 3,
            min_strength: "weak".to_string(),
            max_login_attempts: Some(5),  // 默认5次失败后锁定
            lockout_duration_minutes: 30,  // 默认锁定30分钟
        }
    }
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

// ============== Advanced Scanning Types ==============

/// 扫描策略类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ScanStrategy {
    Quick,              // 快速扫描（TOP 100）
    Standard,           // 标准扫描（TOP 1000）
    Full,               // 全端口扫描（1-65535）
    Custom(Vec<u16>),   // 自定义端口列表
    Cloud,              // 云平台优化扫描
}

/// 扫描引擎类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ScanEngine {
    BasicTcp,           // 基础 TCP 连接（现有实现）
    RustScan,           // RustScan 快速扫描
    Nmap,               // Nmap 深度扫描
    Hybrid,             // RustScan + Nmap 混合
}

/// 高级扫描配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedScanConfig {
    /// 扫描策略
    pub strategy: ScanStrategy,

    /// 扫描引擎
    pub engine: ScanEngine,

    /// 并发级别
    pub concurrency: u32,

    /// 超时设置（毫秒）
    pub timeout_ms: u64,

    /// 服务指纹识别
    pub service_detection: bool,

    /// 操作系统识别
    pub os_detection: bool,

    /// Web 应用指纹识别
    pub web_fingerprint: bool,

    /// 漏洞扫描
    pub vulnerability_scan: bool,

    /// 云平台标签同步
    pub cloud_tag_sync: bool,

    /// 速率限制（每秒包数）
    pub rate_limit: Option<u32>,
}

impl Default for AdvancedScanConfig {
    fn default() -> Self {
        Self {
            strategy: ScanStrategy::Standard,
            engine: ScanEngine::Hybrid,
            concurrency: 1000,
            timeout_ms: 5000,
            service_detection: true,
            os_detection: false,
            web_fingerprint: true,
            vulnerability_scan: false,
            cloud_tag_sync: true,
            rate_limit: None,
        }
    }
}

/// 服务指纹规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceFingerprintRule {
    /// 服务名称
    pub service: String,

    /// 匹配规则（正则表达式）
    pub patterns: Vec<String>,

    /// 端口号（可选）
    pub port: Option<u16>,

    /// 协议（TCP/UDP）
    pub protocol: String,

    /// CPE 标识
    pub cpe: Option<String>,

    /// 置信度（0-100）
    pub confidence: i32,

    /// 版本提取正则
    pub version_regex: Option<String>,
}

/// 指纹匹配结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FingerprintMatch {
    pub service: String,
    pub version: Option<String>,
    pub confidence: i32,
    pub cpe: Option<String>,
    pub matched_pattern: String,
}

/// 扫描结果与云资产关联
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanCloudAssetMapping {
    pub id: String,
    pub ip: String,
    pub cloud_asset_id: Option<i32>,
    pub cloud_provider: Option<CloudProvider>,
    pub instance_id: Option<String>,
    pub matched_tags: Vec<String>,
    pub confidence: f32,
    pub last_matched: DateTime<Utc>,
}

/// 高级扫描任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedScanTask {
    pub id: String,
    pub name: String,
    pub targets: Vec<String>,
    pub config: AdvancedScanConfig,
    pub status: TaskStatus,
    pub progress: f32,  // 0.0 - 1.0
    pub current_target: Option<String>,
    pub scanned_count: u32,
    pub total_count: u32,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub results: Vec<ScanResult>,
    pub cloud_mappings: Vec<ScanCloudAssetMapping>,
    pub error_message: Option<String>,
    pub created_by: Option<String>,
}

/// 高级扫描请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAdvancedScanRequest {
    pub name: String,
    pub targets: Vec<String>,
    pub strategy: ScanStrategy,
    pub engine: ScanEngine,
    pub concurrency: Option<u32>,
    pub timeout_ms: Option<u64>,
    pub service_detection: Option<bool>,
    pub os_detection: Option<bool>,
    pub web_fingerprint: Option<bool>,
    pub cloud_tag_sync: Option<bool>,
}

// ============== Cloud Provider Integration / 云区对接管理 Types ==============

/// 云区对接配置状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CloudProviderConfigStatus {
    #[serde(rename = "active")]
    Active,     // 已启用，可用于业务申请
    #[serde(rename = "inactive")]
    Inactive,   // 已停用
    #[serde(rename = "testing")]
    Testing,    // 测试中
    #[serde(rename = "error")]
    Error,      // 连接错误
}

impl CloudProviderConfigStatus {
    pub fn as_str(&self) -> &str {
        match self {
            CloudProviderConfigStatus::Active => "active",
            CloudProviderConfigStatus::Inactive => "inactive",
            CloudProviderConfigStatus::Testing => "testing",
            CloudProviderConfigStatus::Error => "error",
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            CloudProviderConfigStatus::Active => "已启用",
            CloudProviderConfigStatus::Inactive => "已停用",
            CloudProviderConfigStatus::Testing => "测试中",
            CloudProviderConfigStatus::Error => "连接错误",
        }
    }
}

/// 云区对接配置
///
/// 用于管理已对接的云平台账户和区域信息，
/// 业务申请时只能选择已配置且启用的云区。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudProviderConfig {
    pub id: Option<i32>,
    /// 云厂商
    pub provider: CloudProvider,
    /// 区域ID (如 cn-hangzhou, ap-guangzhou)
    pub region_id: String,
    /// 区域名称 (如 华东1(杭州), 华南-广州)
    pub region_name: String,
    /// 可用区列表 (可选，如 ["cn-hangzhou-i", "cn-hangzhou-j"])
    pub available_zones: Vec<String>,
    /// 账户/AK名称 (用于标识不同账户)
    pub account_name: String,
    /// Access Key ID (加密存储)
    pub access_key_id: String,
    /// Access Key Secret (加密存储)
    pub access_key_secret: String,
    /// 状态
    pub status: CloudProviderConfigStatus,
    /// 备注
    pub remarks: Option<String>,
    /// 最后连接测试时间
    pub last_test_time: Option<DateTime<Utc>>,
    /// 最后连接测试结果
    pub last_test_result: Option<String>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: Option<DateTime<Utc>>,
}

/// 创建云区对接配置请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCloudProviderConfigRequest {
    pub provider: CloudProvider,
    pub region_id: String,
    pub region_name: String,
    pub available_zones: Vec<String>,
    pub account_name: String,
    pub access_key_id: String,
    pub access_key_secret: String,
    pub remarks: Option<String>,
}

/// 更新云区对接配置请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCloudProviderConfigRequest {
    pub region_name: Option<String>,
    pub available_zones: Option<Vec<String>>,
    pub account_name: Option<String>,
    pub access_key_id: Option<String>,
    pub access_key_secret: Option<String>,
    pub status: Option<CloudProviderConfigStatus>,
    pub remarks: Option<String>,
}

/// 云区对接配置查询参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudProviderConfigQuery {
    pub provider: Option<CloudProvider>,
    pub region_id: Option<String>,
    pub status: Option<CloudProviderConfigStatus>,
    pub account_name: Option<String>,
}

/// 连接测试结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionTestResult {
    pub success: bool,
    pub message: String,
    pub response_time_ms: Option<u64>,
    pub tested_at: DateTime<Utc>,
}

// ============== Cloud Service Asset / 云服务资产管理 Types ==============

/// 统一的云服务资产（包含物理机和云虚拟机）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudServiceAsset {
    // 基础标识
    pub id: String,                      // 统一ID，格式: "{type}-{id}"
    pub asset_type: String,              // 资产类型: "physical"(物理机) / "virtual"(云虚拟机)
    pub source_type: String,             // 来源类型: "business_resource"(业务受理) / "cloud_asset"(混合云同步)

    // 基本信息
    pub name: String,                    // 资产名称
    pub instance_id: String,             // 实例ID / 设备序列号
    pub status: String,                  // 状态: 运行中/已停止/已释放等
    pub cloud_provider: String,          // 云厂商 (阿里云/腾讯云/华为云/AWS) 或 机房名称
    pub region: String,                  // 区域/机房位置

    // 实例配置
    pub instance_type: String,           // 实例类型 (如 ecs.g6.large) 或 物理机型号
    pub cpu_cores: u32,                  // CPU核数
    pub memory_gb: u32,                  // 内存(GB)
    pub system_disk_type: String,        // 系统盘类型
    pub system_disk_size_gb: u32,        // 系统盘大小(GB)
    pub data_disk_info: Option<String>,  // 数据盘信息

    // 操作系统
    pub os_type: String,                 // 操作系统类型
    pub os_name: String,                 // 操作系统名称

    // 网络信息
    pub ip_address: String,              // 主IP地址
    pub public_ip: Option<String>,       // 公网IP
    pub ipv6_address: Option<String>,    // IPv6地址

    // 业务信息
    pub customer_name: String,           // 客户名称
    pub department: Option<String>,      // 部门
    pub project: Option<String>,         // 项目
    pub application_name: Option<String>, // 应用名称
    pub contract_name: Option<String>,   // 合同名称
    pub owner_name: Option<String>,      // 负责人

    // 访问信息
    pub login_method: Option<String>,    // 登录方式 (SSH/RDP/堡垒机等)
    pub login_username: Option<String>,  // 登录用户名
    pub bastion_address: Option<String>, // 堡垒机地址
    pub bastion_account: Option<String>, // 堡垒机账号

    // 物理机特有信息
    pub serial_number: Option<String>,   // 设备序列号
    pub rack_location: Option<String>,   // 机架位置
    pub hardware_model: Option<String>,  // 硬件型号
    pub warranty_expiry: Option<String>, // 维保到期时间
    pub agent_status: Option<String>,    // Agent状态
    pub ipmi_address: Option<String>,    // IPMI/iDRAC地址

    // 云虚拟机特有信息
    pub billing_mode: Option<String>,    // 计费模式
    pub expire_time: Option<String>,     // 到期时间
    pub charge_type: Option<String>,     // 付费类型

    // 时间信息
    pub created_at: String,              // 创建时间
    pub updated_at: Option<String>,      // 更新时间
    pub last_synced: Option<String>,     // 最后同步时间

    // 其他
    pub tags: Option<String>,            // 标签 (JSON字符串)
    pub remarks: Option<String>,         // 备注

    // 关联ID
    pub business_resource_id: Option<i32>, // 关联的业务资源ID
    pub cloud_asset_id: Option<i32>,       // 关联的云资产ID
}

/// 云服务资产查询参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudServiceAssetQuery {
    pub asset_type: Option<String>,      // 资产类型筛选
    pub source_type: Option<String>,     // 来源类型筛选
    pub cloud_provider: Option<String>,  // 云厂商筛选
    pub status: Option<String>,          // 状态筛选
    pub customer_name: Option<String>,   // 客户筛选
    pub department: Option<String>,      // 部门筛选
    pub project: Option<String>,         // 项目筛选
    pub search_keyword: Option<String>,  // 搜索关键词
}

/// 云服务资产统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudServiceAssetStats {
    pub total_count: u32,
    pub physical_count: u32,             // 物理机数量
    pub virtual_count: u32,              // 云虚拟机数量
    pub running_count: u32,
    pub stopped_count: u32,
    pub total_cpu_cores: u32,
    pub total_memory_gb: u32,
    pub expiring_soon_count: u32,        // 即将到期数量
    pub by_provider: Vec<(String, u32)>, // 按厂商统计
    pub by_customer: Vec<(String, u32)>, // 按客户统计
    pub by_status: Vec<(String, u32)>,   // 按状态统计
}
