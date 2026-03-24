use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use utoipa::ToSchema;

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
    Aliyun, // 阿里云
    #[serde(rename = "tencent")]
    Tencent, // 腾讯云
    #[serde(rename = "huawei")]
    Huawei, // 华为云
    #[serde(rename = "aws")]
    Aws, // AWS
    #[serde(rename = "azure")]
    Azure, // Azure
    #[serde(rename = "gcp")]
    Gcp, // Google Cloud
    #[serde(rename = "baidu")]
    Baidu, // 百度云
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

/// 计费模式 (保留用于云服务资产)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BillingMode {
    #[serde(rename = "pay_as_you_go")]
    PayAsYouGo, // 按量付费
    #[serde(rename = "subscription")]
    Subscription, // 包年包月/订阅
    #[serde(rename = "spot")]
    Spot, // 抢占式实例
}

/// 组织/单位信息
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, utoipa::ToSchema)]
pub struct Organization {
    pub id: i32,
    pub name: String,
    pub code: String,
    pub status: String,
    pub remarks: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
}

/// 部门信息
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, utoipa::ToSchema)]
pub struct Department {
    pub id: i32,
    pub organization_id: i32,
    pub name: String,
    pub code: String,
    pub parent_id: Option<i32>,
    pub level: u32,
    pub status: String,
    pub remarks: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
}

/// 项目信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub code: String, // 项目编码
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

// ============== Business Resource / 业务受理 Types ==============

// ============== Business Resource / 业务受理 Types ==============

/// 资源类型 - 支持云资源和物理机
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ResourceType {
    #[serde(rename = "cloud")]
    Cloud, // 云服务器 (ECS/云主机)
    #[serde(rename = "physical")]
    Physical, // 物理机
    #[serde(rename = "network")]
    Network, // 网络策略
}

impl ResourceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ResourceType::Cloud => "cloud",
            ResourceType::Physical => "physical",
            ResourceType::Network => "network",
        }
    }
}

impl std::str::FromStr for ResourceType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cloud" => Ok(ResourceType::Cloud),
            "physical" => Ok(ResourceType::Physical),
            "network" => Ok(ResourceType::Network),
            _ => Err(format!("Invalid resource type: {}", s)),
        }
    }
}

/// 物理机特有信息
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
pub struct PhysicalMachineInfo {
    pub id: Option<i32>,
    pub business_resource_id: Option<i32>,
    pub serial_number: Option<String>,          // 设备序列号
    pub rack_location: Option<String>,          // 机架位置 (如: A区-03机柜-U12)
    pub hardware_model: Option<String>,         // 硬件型号 (如: Dell PowerEdge R740)
    pub warranty_expiry: Option<DateTime<Utc>>, // 维保到期时间
    pub agent_status: Option<String>,           // Agent 状态 (installed/online/offline/none)
    pub ipmi_address: Option<String>,           // IPMI/iDRAC 地址
}

/// 云虚拟机特有信息
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
pub struct CloudVirtualMachineInfo {
    pub id: Option<i32>,
    pub business_resource_id: Option<i32>,
    pub billing_mode: Option<String>, // 计费模式 (按量付费/包年包月)
    pub expire_time: Option<DateTime<Utc>>, // 到期时间
    pub charge_type: Option<String>,  // 付费类型
    pub instance_charge_type: Option<String>, // 实例计费类型
    pub internet_charge_type: Option<String>, // 网络计费类型
    pub internet_max_bandwidth_out: Option<i32>, // 公网带宽出带宽最大值
    pub image_id: Option<String>,     // 镜像ID
    pub v_switch_id: Option<String>,  // 虚拟交换机ID
    pub vpc_id: Option<String>,       // VPC ID
    pub security_group_ids: Option<Vec<String>>, // 安全组ID列表
}

/// 创建物理机详情请求
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreatePhysicalMachineInfo {
    pub serial_number: Option<String>,
    pub rack_location: Option<String>,
    pub hardware_model: Option<String>,
    pub warranty_expiry: Option<DateTime<Utc>>,
    pub agent_status: Option<String>,
    pub ipmi_address: Option<String>,
}

/// 创建云虚拟机详情请求
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreateCloudVirtualMachineInfo {
    pub billing_mode: Option<String>,
    pub expire_time: Option<DateTime<Utc>>,
    pub charge_type: Option<String>,
    pub instance_charge_type: Option<String>,
    pub internet_charge_type: Option<String>,
    pub internet_max_bandwidth_out: Option<i32>,
    pub image_id: Option<String>,
    pub v_switch_id: Option<String>,
    pub vpc_id: Option<String>,
    pub security_group_ids: Option<Vec<String>>,
}

/// 更新物理机详情请求
#[derive(Debug, Clone, Serialize, Deserialize, Default, utoipa::ToSchema)]
pub struct UpdatePhysicalMachineInfo {
    pub serial_number: Option<String>,
    pub rack_location: Option<String>,
    pub hardware_model: Option<String>,
    pub warranty_expiry: Option<DateTime<Utc>>,
    pub agent_status: Option<String>,
    pub ipmi_address: Option<String>,
}

/// 更新云虚拟机详情请求
#[derive(Debug, Clone, Serialize, Deserialize, Default, utoipa::ToSchema)]
pub struct UpdateCloudVirtualMachineInfo {
    pub billing_mode: Option<String>,
    pub expire_time: Option<DateTime<Utc>>,
    pub charge_type: Option<String>,
    pub instance_charge_type: Option<String>,
    pub internet_charge_type: Option<String>,
    pub internet_max_bandwidth_out: Option<i32>,
    pub image_id: Option<String>,
    pub v_switch_id: Option<String>,
    pub vpc_id: Option<String>,
    pub security_group_ids: Option<Vec<String>>,
}

/// 业务受理单 - 云资源管理 & 物理机管理
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
pub struct BusinessResource {
    pub id: Option<i32>,

    // 资源类型
    pub resource_type: String, // 资源类型: cloud(云服务器) / physical(物理机)

    // 基本信息
    pub ecs_name: String,                      // ECS名称/物理机名称
    pub ecs_status: String,                    // ECS状态 (运行中/已停止/已释放等)
    pub resource_id: String,                   // 资源ID
    pub cloud_region: String,                  // 云区域 (如 华东1-杭州) 或 机房位置
    pub cloud_category: String,                // 云类别 (阿里云/腾讯云/华为云/AWS等) 或 机房名称
    pub cloud_provider_config_id: Option<i32>, // 关联的云区对接配置ID
    pub zone_name: Option<String>,             // 地区名称 (如 华北区、华南区、华东区)
    pub platform_name: Option<String>,         // 云平台名称 (如 公众云、政务云、内外核心云)
    pub county_city: Option<String>,           // 县市区
    pub vdc_name: Option<String>,              // VDC名称

    // 业务信息
    pub customer_name: String,            // 客户名称
    pub application_name: Option<String>, // 应用名称
    pub contract_name: Option<String>,    // 合同名称
    pub instance_id: String,              // 实例ID / 物理机序列号
    pub ecs_type: String,                 // ECS类型 (如 ecs.g6.large) 或 物理机型号

    // 配置信息
    pub ecs_os: String,      // ECS操作系统 (如 CentOS 7.9, Windows Server 2019)
    pub cpu_cores: u32,      // CPU核数
    pub memory_gb: u32,      // 内存(GB)
    pub system_disk: String, // 系统盘类型 (如 cloud_ssd, cloud_essd) 或 物理磁盘类型
    pub system_disk_size_gb: u32, // 系统盘大小(GB)
    pub data_disk: Option<String>, // 数据盘信息 (JSON字符串存储多块盘信息)

    // 时间信息
    pub completion_time: Option<DateTime<Utc>>, // 完成时间
    pub release_time: Option<DateTime<Utc>>,    // 释放时间

    // 安全产品
    pub has_security_product: bool, // 是否创建安全产品

    // 网络信息
    pub ip_address: String,                   // IP地址
    pub ecs_login_method: Option<String>,     // ECS远程登录方式 (SSH/RDP/堡垒机等)
    pub ecs_login_username: Option<String>,   // ECS登录用户名
    pub ecs_initial_password: Option<String>, // ECS初始密码

    // 堡垒机信息
    pub bastion_address: Option<String>,          // 堡垒机地址
    pub bastion_admin_account: Option<String>,    // 堡垒机管理员账号
    pub bastion_initial_password: Option<String>, // 堡垒机初始密码

    // 资源类型特有信息 (根据 resource_type 选择其一)
    pub physical_machine_info: Option<PhysicalMachineInfo>, // 物理机特有信息
    pub cloud_vm_info: Option<CloudVirtualMachineInfo>,     // 云虚拟机特有信息

    // 其他
    pub remarks: Option<String>, // 备注
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub created_by: Option<String>,
    pub updated_by: Option<String>,

    // 申请流程相关
    pub applicant: Option<String>,            // 申请人
    pub department: Option<String>,           // 申请部门
    pub approver: Option<String>,             // 审批人
    pub approval_time: Option<DateTime<Utc>>, // 审批时间
    pub approval_remarks: Option<String>,     // 审批备注
    pub rejection_reason: Option<String>,     // 拒绝原因

    // 资源配置相关
    pub bandwidth_mbps: Option<u32>,    // 带宽大小(Mbps)
    pub bandwidth_type: Option<String>, // 带宽类型(按量/包月)
    pub public_ip_count: Option<u32>,   // 公网IP数量
    pub network_type: Option<String>,   // 网络类型(VPC/经典网络)

    // 业务关联相关
    pub project_name: Option<String>,   // 项目名称
    pub project_code: Option<String>,   // 项目编号
    pub business_owner: Option<String>, // 业务负责人
    pub tech_owner: Option<String>,     // 技术负责人
    pub contact_phone: Option<String>,  // 联系电话

    // 费用相关
    pub billing_method: Option<String>, // 计费方式(包年包月/按量付费)
    pub purchase_duration: Option<u32>, // 购买时长(月)
    pub cost_center: Option<String>,    // 成本中心

    // 合规相关
    pub security_level: Option<String>,   // 等保级别(二级/三级)
    pub data_sensitivity: Option<String>, // 数据敏感级别(公开/内部/机密/绝密)

    // 其他
    pub purpose: Option<String>,                       // 用途说明
    pub expected_delivery_time: Option<DateTime<Utc>>, // 期望交付时间

    // 申请与交付状态管理
    pub application_status: Option<String>, // 申请状态: 待审核、已批准、已拒绝
    pub delivery_status: Option<String>,    // 交付状态: 待交付、交付中、已交付
    pub delivery_confirmed_at: Option<DateTime<Utc>>, // 交付确认时间
    pub delivery_confirmed_by: Option<String>, // 交付确认人
}

// 默认值函数
fn default_application_status() -> Option<String> {
    Some("待审核".to_string())
}

fn default_delivery_status() -> Option<String> {
    Some("待交付".to_string())
}

/// 创建业务资源请求
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreateBusinessResourceRequest {
    pub resource_type: String, // 资源类型: cloud / physical
    pub ecs_name: String,
    pub ecs_status: String,
    #[serde(default)] // 默认为空字符串，由运维/编排分配
    pub resource_id: Option<String>,
    pub cloud_region: String,
    pub cloud_category: String,
    pub cloud_provider_config_id: Option<i32>, // 关联的云区对接配置ID
    pub zone_name: Option<String>,             // 地区名称
    pub platform_name: Option<String>,         // 云平台名称
    pub county_city: Option<String>,
    pub vdc_name: Option<String>,
    pub customer_name: String,
    pub application_name: Option<String>,
    pub contract_name: Option<String>,
    #[serde(default)] // 默认为空字符串，由运维/编排分配
    pub instance_id: Option<String>,
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
    // 资源类型特有信息 (根据 resource_type 选择其一)
    pub physical_machine_info: Option<CreatePhysicalMachineInfo>, // 物理机特有信息
    pub cloud_vm_info: Option<CreateCloudVirtualMachineInfo>,     // 云虚拟机特有信息
    pub remarks: Option<String>,

    // 申请流程相关
    pub applicant: Option<String>,            // 申请人
    pub department: Option<String>,           // 申请部门
    pub approver: Option<String>,             // 审批人
    pub approval_time: Option<DateTime<Utc>>, // 审批时间
    pub approval_remarks: Option<String>,     // 审批备注
    pub rejection_reason: Option<String>,     // 拒绝原因

    // 资源配置相关
    pub bandwidth_mbps: Option<u32>,    // 带宽大小(Mbps)
    pub bandwidth_type: Option<String>, // 带宽类型(按量/包月)
    pub public_ip_count: Option<u32>,   // 公网IP数量
    pub network_type: Option<String>,   // 网络类型(VPC/经典网络)

    // 业务关联相关
    pub project_name: Option<String>,   // 项目名称
    pub project_code: Option<String>,   // 项目编号
    pub business_owner: Option<String>, // 业务负责人
    pub tech_owner: Option<String>,     // 技术负责人
    pub contact_phone: Option<String>,  // 联系电话

    // 费用相关
    pub billing_method: Option<String>, // 计费方式(包年包月/按量付费)
    pub purchase_duration: Option<u32>, // 购买时长(月)
    pub cost_center: Option<String>,    // 成本中心

    // 合规相关
    pub security_level: Option<String>,   // 等保级别(二级/三级)
    pub data_sensitivity: Option<String>, // 数据敏感级别(公开/内部/机密/绝密)

    // 其他
    pub purpose: Option<String>,                       // 用途说明
    pub expected_delivery_time: Option<DateTime<Utc>>, // 期望交付时间,

    // 申请与交付状态管理
    #[serde(default = "default_application_status")]
    pub application_status: Option<String>, // 申请状态: 待审核、已批准、已拒绝
    #[serde(default = "default_delivery_status")]
    pub delivery_status: Option<String>, // 交付状态: 待交付、交付中、已交付
    pub delivery_confirmed_at: Option<DateTime<Utc>>, // 交付确认时间
    pub delivery_confirmed_by: Option<String>,        // 交付确认人
}

/// 更新业务资源请求
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema, Default)]
pub struct UpdateBusinessResourceRequest {
    pub resource_type: Option<String>, // 资源类型
    pub ecs_name: Option<String>,
    pub ecs_status: Option<String>,
    pub resource_id: Option<String>, // 云平台资源ID（由运维/编排分配）
    pub cloud_region: Option<String>,
    pub cloud_category: Option<String>,
    pub cloud_provider_config_id: Option<i32>, // 关联的云区对接配置ID
    pub zone_name: Option<String>,             // 地区名称
    pub platform_name: Option<String>,         // 云平台名称
    pub county_city: Option<String>,
    pub vdc_name: Option<String>,
    pub customer_name: Option<String>,
    pub application_name: Option<String>,
    pub contract_name: Option<String>,
    pub instance_id: Option<String>, // 实例ID（由运维/编排分配）
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
    pub security_products: Option<String>, // 选中的安全产品名称，逗号分隔
    pub ip_address: Option<String>,
    pub ecs_login_method: Option<String>,
    pub ecs_login_username: Option<String>,
    pub ecs_initial_password: Option<String>,
    pub bastion_address: Option<String>,
    pub bastion_admin_account: Option<String>,
    pub bastion_initial_password: Option<String>,
    // 资源类型特有信息更新 (根据 resource_type 选择其一)
    pub physical_machine_info: Option<UpdatePhysicalMachineInfo>, // 物理机特有信息更新
    pub cloud_vm_info: Option<UpdateCloudVirtualMachineInfo>,     // 云虚拟机特有信息更新
    pub remarks: Option<String>,

    // 申请流程相关
    pub applicant: Option<String>,            // 申请人
    pub department: Option<String>,           // 申请部门
    pub approver: Option<String>,             // 审批人
    pub approval_time: Option<DateTime<Utc>>, // 审批时间
    pub approval_remarks: Option<String>,     // 审批备注
    pub rejection_reason: Option<String>,     // 拒绝原因

    // 资源配置相关
    pub bandwidth_mbps: Option<u32>,    // 带宽大小(Mbps)
    pub bandwidth_type: Option<String>, // 带宽类型(按量/包月)
    pub public_ip_count: Option<u32>,   // 公网IP数量
    pub network_type: Option<String>,   // 网络类型(VPC/经典网络)

    // 业务关联相关
    pub project_name: Option<String>,   // 项目名称
    pub project_code: Option<String>,   // 项目编号
    pub business_owner: Option<String>, // 业务负责人
    pub tech_owner: Option<String>,     // 技术负责人
    pub contact_phone: Option<String>,  // 联系电话

    // 费用相关
    pub billing_method: Option<String>, // 计费方式(包年包月/按量付费)
    pub purchase_duration: Option<u32>, // 购买时长(月)
    pub cost_center: Option<String>,    // 成本中心

    // 合规相关
    pub security_level: Option<String>,   // 等保级别(二级/三级)
    pub data_sensitivity: Option<String>, // 数据敏感级别(公开/内部/机密/绝密)

    // 其他
    pub purpose: Option<String>,                       // 用途说明
    pub expected_delivery_time: Option<DateTime<Utc>>, // 期望交付时间,

    // 申请与交付状态管理
    pub application_status: Option<String>, // 申请状态: 待审核、已批准、已拒绝
    pub delivery_status: Option<String>,    // 交付状态: 待交付、交付中、已交付
    pub delivery_confirmed_at: Option<DateTime<Utc>>, // 交付确认时间
    pub delivery_confirmed_by: Option<String>, // 交付确认人
}

/// 业务资源查询参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessResourceQuery {
    pub search_keyword: Option<String>, // 搜索关键词（ECS名称、客户名称、IP地址等）
    pub resource_type: Option<String>,  // 资源类型筛选 (cloud/physical)
    pub cloud_category: Option<String>, // 云类别/机房筛选
    pub ecs_status: Option<String>,     // 状态筛选
    pub customer_name: Option<String>,  // 客户名称筛选
    pub county_city: Option<String>,    // 县市区筛选
    pub application_name: Option<String>, // 应用名称筛选
    pub contract_name: Option<String>,  // 合同名称筛选
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
    pub by_category: Vec<(String, u32)>,    // 按云类别统计
    pub by_customer: Vec<(String, u32)>,    // 按客户统计
    pub by_county: Vec<(String, u32)>,      // 按县市区统计
    pub by_application: Vec<(String, u32)>, // 按应用统计
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
    pub owner: Option<String>,       // Responsibility
    pub weight: i32,                 // Asset Importance (1-100)
    pub labels: Vec<String>,         // Multi-level tags
    pub os: Option<String>,          // OS Fingerprint
    pub device_type: Option<String>, // Device Type Fingerprint
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PortInfo {
    pub port: u16,
    pub is_open: bool,
    pub service: Option<String>,
    pub version: Option<String>, // Service Version
    pub banner: Option<String>,  // Service Banner
    pub is_bound: bool,          // If this port is manually confirmed/bound to a service
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
    pub cloud_platform_id: Option<i32>,
    pub cloud_platform_name: Option<String>,
    pub machine_room_id: Option<i32>,
    pub machine_room_name: Option<String>,
}

/// 扫描器配置
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ScannerConfig {
    pub id: String,
    pub name: String,
    pub scanner_type: String, // rustscan, nmap, basic_tcp
    pub enabled: bool,
    pub config: serde_json::Value, // 扫描器特定配置
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

/// 创建扫描器配置请求
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateScannerRequest {
    pub name: String,
    pub scanner_type: String,
    pub enabled: Option<bool>,
    pub config: serde_json::Value,
}

/// 更新扫描器配置请求
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateScannerRequest {
    pub name: Option<String>,
    pub scanner_type: Option<String>,
    pub enabled: Option<bool>,
    pub config: Option<serde_json::Value>,
}

// --- Auth & Audit ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
pub enum Role {
    SysAdmin,       // Manage users and permissions
    SecAdmin,       // Manage assets, tasks, risks
    Auditor,        // View logs
    Custom(String), // Custom role with specific permissions
}

/// 自定义角色数据结构
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CustomRole {
    pub id: Option<i32>,
    pub name: String,                // 角色名称
    pub description: Option<String>, // 角色描述
    pub permissions: Permissions,    // 角色权限
    pub created_at: Option<String>,  // 创建时间
    pub updated_at: Option<String>,  // 更新时间
}

/// 创建自定义角色请求
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreateRoleRequest {
    pub name: String,
    pub description: Option<String>,
    pub permissions: Permissions,
}

/// 更新自定义角色请求
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct UpdateRoleRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub permissions: Option<Permissions>,
}

/// 细化权限位掩码
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default, utoipa::ToSchema)]
pub struct Permissions {
    // ========== 通用模块 ==========
    pub can_access_general: bool, // 顶级：访问通用模块

    // 仪表盘
    pub can_view_dashboard: bool, // 子级：查看仪表盘

    // 任务中心
    pub can_view_tasks: bool,  // 子级：查看任务中心
    pub can_create_task: bool, // 孙级：创建任务
    pub can_delete_task: bool, // 孙级：删除任务
    pub can_update_task: bool, // 孙级：更新任务

    // 高级扫描
    pub can_view_advanced_scan: bool, // 子级：查看高级扫描页面
    pub can_create_scan: bool,        // 孙级：创建扫描
    pub can_delete_scan: bool,        // 孙级：删除扫描
    pub can_export_scan: bool,        // 孙级：导出结果

    // ========== 资产与风险模块 ==========
    pub can_access_assets_risks: bool, // 顶级：访问资产与风险模块

    // 云服务资产
    pub can_view_cloud_assets: bool,  // 子级：查看云服务资产
    pub can_create_cloud_asset: bool, // 孙级：创建云资产
    pub can_update_cloud_asset: bool, // 孙级：更新云资产
    pub can_delete_cloud_asset: bool, // 孙级：删除云资产

    // 风险监控
    pub can_view_risks: bool,   // 子级：查看风险监控
    pub can_resolve_risk: bool, // 孙级：处置风险
    pub can_delete_risk: bool,  // 孙级：删除风险

    // 业务流程
    pub can_view_business_process: bool, // 子级：查看业务流程

    // 业务申请
    pub can_view_business_applications: bool, // 孙级：查看业务申请列表
    pub can_create_business_application: bool, // 孙级：创建业务申请
    pub can_approve_business_application: bool, // 孙级：审批业务申请
    pub can_supplement_business_application: bool, // 孙级：补充业务申请信息
    pub can_delete_business_application: bool, // 孙级：删除业务申请

    // 运维管理
    pub can_view_operations_management: bool, // 孙级：查看运维管理
    pub can_manage_operations: bool,          // 孙级：运维操作权限

    // 自动化资源编排
    pub can_view_automation_orchestration: bool, // 孙级：查看自动化编排
    pub can_execute_orchestration: bool,         // 孙级：执行编排任务
    pub can_manage_orchestration: bool,          // 孙级：管理编排任务

    // ========== Cloud模块 ==========
    pub can_access_cloud: bool, // 顶级：访问Cloud模块

    // 云厂商对接
    pub can_view_cloud_providers: bool,   // 子级：查看云厂商对接
    pub can_manage_cloud_providers: bool, // 孙级：管理云厂商对接

    // ========== 用户管理模块 ==========
    pub can_access_user_management: bool, // 顶级：访问用户管理模块

    // 用户管理
    pub can_view_users: bool,         // 子级：查看用户管理
    pub can_create_user: bool,        // 孙级：创建用户
    pub can_update_user: bool,        // 孙级：更新用户
    pub can_delete_user: bool,        // 孙级：删除用户
    pub can_manage_permissions: bool, // 孙级：管理权限

    // 密码策略管理
    pub can_view_password_policy: bool,   // 子级：查看密码策略
    pub can_manage_password_policy: bool, // 孙级：管理密码策略

    // ========== 审计模块 ==========
    pub can_access_audit: bool, // 顶级：访问审计模块

    // 审计日志
    pub can_view_audit_logs: bool, // 子级：查看审计日志
}

impl Permissions {
    /// SysAdmin 默认权限 - 拥有所有权限
    pub fn sys_admin() -> Self {
        Self {
            can_access_general: true,
            can_view_dashboard: true,
            can_view_tasks: true,
            can_create_task: true,
            can_delete_task: true,
            can_update_task: true,
            can_view_advanced_scan: true,
            can_create_scan: true,
            can_delete_scan: true,
            can_export_scan: true,
            can_access_assets_risks: true,
            can_view_cloud_assets: true,
            can_create_cloud_asset: true,
            can_update_cloud_asset: true,
            can_delete_cloud_asset: true,
            can_view_risks: true,
            can_resolve_risk: true,
            can_delete_risk: true,
            can_view_business_process: true,
            can_view_business_applications: true,
            can_create_business_application: true,
            can_approve_business_application: false,
            can_supplement_business_application: true,
            can_delete_business_application: true,
            can_view_operations_management: true,
            can_manage_operations: true,
            can_view_automation_orchestration: true,
            can_execute_orchestration: true,
            can_manage_orchestration: true,
            can_access_cloud: true,
            can_view_cloud_providers: true,
            can_manage_cloud_providers: true,
            can_access_user_management: true,
            can_view_users: true,
            can_create_user: true,
            can_update_user: true,
            can_delete_user: true,
            can_manage_permissions: true,
            can_view_password_policy: true,
            can_manage_password_policy: true,
            can_access_audit: true,
            can_view_audit_logs: true,
        }
    }

    /// SecAdmin 默认权限
    pub fn sec_admin() -> Self {
        Self {
            can_access_general: true,
            can_view_dashboard: true,
            can_view_tasks: true,
            can_create_task: true,
            can_delete_task: true,
            can_update_task: true,
            can_view_advanced_scan: true,
            can_create_scan: true,
            can_delete_scan: true,
            can_export_scan: true,
            can_access_assets_risks: true,
            can_view_cloud_assets: true,
            can_create_cloud_asset: true,
            can_update_cloud_asset: true,
            can_delete_cloud_asset: true,
            can_view_risks: true,
            can_resolve_risk: true,
            can_delete_risk: true,
            can_view_business_process: true,
            can_view_business_applications: true,
            can_create_business_application: true,
            can_approve_business_application: false,
            can_supplement_business_application: true,
            can_delete_business_application: true,
            can_view_operations_management: true,
            can_manage_operations: true,
            can_view_automation_orchestration: true,
            can_execute_orchestration: true,
            can_manage_orchestration: true,
            can_access_cloud: true,
            can_view_cloud_providers: true,
            can_manage_cloud_providers: true,
            can_access_audit: true,
            can_view_audit_logs: true,
            ..Default::default()
        }
    }

    /// Auditor 默认权限
    pub fn auditor() -> Self {
        Self {
            can_access_general: true,
            can_view_dashboard: true,
            can_view_tasks: true,
            can_view_advanced_scan: true,
            can_export_scan: true,
            can_access_assets_risks: false,
            can_view_cloud_assets: false,
            can_view_risks: false,
            can_view_business_process: false,
            can_view_business_applications: false,
            can_supplement_business_application: false,
            can_access_cloud: false,
            can_view_cloud_providers: false,
            can_access_user_management: false,
            can_view_users: false,
            can_view_password_policy: false,
            can_access_audit: true,
            can_view_audit_logs: true,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
pub struct User {
    pub id: String,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub real_name: Option<String>,
    #[serde(skip_serializing, default)]
    // Don't send password hash to frontend, allow missing on receive
    pub password: String, // Stored as a password hash on the backend.
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<i32>, // 所属组织/单位
    #[serde(skip_serializing_if = "Option::is_none")]
    pub department_id: Option<i32>, // 所属部门
    // 账户锁定相关
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed_login_attempts: Option<u32>, // 失败登录次数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locked_until: Option<DateTime<Utc>>, // 锁定到期时间
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreateUserRequest {
    pub username: String,
    pub real_name: Option<String>,
    pub password: String,
    pub role: Role,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub status: Option<String>,
    pub organization_id: Option<i32>,
    pub department_id: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct PasswordPolicy {
    pub min_length: u32,           // 最小长度
    pub require_uppercase: bool,   // 需要大写字母
    pub require_lowercase: bool,   // 需要小写字母
    pub require_number: bool,      // 需要数字
    pub require_special: bool,     // 需要特殊字符
    pub max_age_days: Option<u32>, // 密码最大有效期（天）
    pub prevent_reuse: u32,        // 防止重用最近N次密码
    pub min_strength: String,      // 最低强度要求：weak/medium/strong
    // 账户锁定配置
    pub max_login_attempts: Option<u32>, // 最大登录失败次数，None表示不限制
    pub lockout_duration_minutes: u32,   // 账户锁定时长（分钟）
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            min_length: 6,
            require_uppercase: false,
            require_lowercase: false,
            require_number: false,
            require_special: false,
            max_age_days: Some(90), // 默认90天有效期
            prevent_reuse: 3,
            min_strength: "weak".to_string(),
            max_login_attempts: Some(5),  // 默认5次失败后锁定
            lockout_duration_minutes: 30, // 默认锁定30分钟
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct UpdateUserRequest {
    pub real_name: Option<String>,
    pub password: Option<String>,
    pub role: Option<Role>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub status: Option<String>,
    pub organization_id: Option<i32>,
    pub department_id: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct LoginResponse {
    pub token: String,
    pub user: User,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct AuditLog {
    pub id: String,
    pub user_id: String,
    pub username: String,
    pub action: String,  // e.g., "CREATE_ASSET", "LOGIN", "DELETE_USER"
    pub target: String,  // e.g., "Asset: 1", "User: admin"
    pub details: String, // JSON or text description
    pub timestamp: DateTime<Utc>,
}

// ============== Advanced Scanning Types ==============

/// 扫描策略类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ScanStrategy {
    Quick,            // 快速扫描（TOP 100）
    Standard,         // 标准扫描（TOP 1000）
    Full,             // 全端口扫描（1-65535）
    Custom(Vec<u16>), // 自定义端口列表
    Cloud,            // 云平台优化扫描
}

/// 扫描引擎类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ScanEngine {
    BasicTcp, // 基础 TCP 连接（现有实现）
    RustScan, // RustScan 快速扫描
    Nmap,     // Nmap 深度扫描
    Hybrid,   // RustScan + Nmap 混合
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

/// 简单的扫描结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleScanResult {
    pub target: String,
    pub status: String,
    pub message: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// 详细扫描结果（用于 quick_scan）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickScanResult {
    pub ip: String,
    pub is_alive: bool,
    pub open_ports: Vec<PortInfo>,
    pub fingerprint: Option<String>,
    pub scanned_at: DateTime<Utc>,
}

/// 端口详细信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortDetail {
    pub id: String,
    pub asset_id: String,
    pub ip: String,
    pub port: u16,
    pub protocol: String,
    pub status: String,
    pub service_name: Option<String>,
    pub service_version: Option<String>,
    pub service_banner: Option<String>,
    pub system_name: Option<String>,
    pub system_type: Option<String>,
    pub system_url: Option<String>,
    pub middleware: Option<String>,
    pub framework: Option<String>,
    pub language: Option<String>,
    pub environment: Option<String>,
    pub department: Option<String>,
    pub owner: Option<String>,
    pub owner_contact: Option<String>,
    pub vulnerability_level: Option<String>,
    pub has_vulnerability: bool,
    pub vulnerability_count: i32,
    pub fingerprint: Option<ServiceFingerprint>,
    pub is_bound: bool,
    pub notes: Option<String>,
    pub last_scanned: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub created_by: Option<String>,
    pub updated_by: Option<String>,
}

/// 服务指纹信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceFingerprint {
    pub service: Option<String>,
    pub product: Option<String>,
    pub version: Option<String>,
    pub extra_info: Option<String>,
    pub cpe: Option<String>,
    pub os_match: Option<String>,
    pub confidence: i32,
}

/// IP扫描结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IPScanResult {
    pub id: String,
    pub ip_zone_id: String,
    pub ip: String,
    pub is_alive: bool,
    pub hostname: Option<String>,
    pub mac_address: Option<String>,
    pub open_ports: Vec<PortDetail>,
    pub os_fingerprint: Option<String>,
    pub device_type: Option<String>,
    pub confidence: i32,
    pub scan_time: DateTime<Utc>,
    pub scan_duration_ms: i64,
    pub scanned_by: String,
}

/// 高级扫描任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedScanTask {
    pub id: String,
    pub name: String,
    pub targets: Vec<String>,
    pub config: AdvancedScanConfig,
    pub status: TaskStatus,
    pub progress: f32, // 0.0 - 1.0
    pub current_target: Option<String>,
    pub scanned_count: u32,
    pub total_count: u32,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub results: Vec<QuickScanResult>,
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

// ============== Cloud Provider Integration / 云厂商对接 Types ==============

/// 云区 - Top level in cloud hierarchy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudZone {
    pub id: Option<i32>,
    pub zone_name: String, // 华北区、华南区、华东区
    pub zone_code: String, // north、south、east、west
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// 云平台 - Second level in cloud hierarchy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudPlatform {
    pub id: Option<i32>,
    pub zone_id: i32,          // 所属云区
    pub platform_name: String, // 公众云、政务云、内外核心云
    pub platform_code: String, // public、gov、internal
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// 创建云区请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCloudZoneRequest {
    pub zone_name: String,
    pub zone_code: String,
    pub description: Option<String>,
}

/// 更新云区请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCloudZoneRequest {
    pub zone_name: Option<String>,
    pub zone_code: Option<String>,
    pub description: Option<String>,
}

/// 创建云平台请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCloudPlatformRequest {
    pub zone_id: i32,
    pub platform_name: String,
    pub platform_code: String,
    pub description: Option<String>,
}

/// 更新云平台请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCloudPlatformRequest {
    pub zone_id: Option<i32>,
    pub platform_name: Option<String>,
    pub platform_code: Option<String>,
    pub description: Option<String>,
}

/// 云区对接配置状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CloudProviderConfigStatus {
    #[serde(rename = "active")]
    Active, // 已启用，可用于业务申请
    #[serde(rename = "inactive")]
    Inactive, // 已停用
    #[serde(rename = "testing")]
    Testing, // 测试中
    #[serde(rename = "error")]
    Error, // 连接错误
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
/// 现在支持云区-云平台两级层级结构。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudProviderConfig {
    pub id: Option<i32>,
    /// 所属云区ID (CloudZone)
    pub zone_id: Option<i32>,
    /// 所属云平台ID (CloudPlatform)
    pub platform_id: Option<i32>,
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
    pub zone_id: Option<i32>,
    pub platform_id: Option<i32>,
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
    pub zone_id: Option<i32>,
    pub platform_id: Option<i32>,
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
    pub id: String,          // 统一ID，格式: "{type}-{id}"
    pub asset_type: String,  // 资产类型: "physical"(物理机) / "virtual"(云虚拟机)
    pub source_type: String, // 来源类型: "business_resource"(业务受理) / "cloud_asset"(混合云同步)

    // 基本信息
    pub name: String,                  // 资产名称
    pub instance_id: String,           // 实例ID / 设备序列号
    pub status: String,                // 状态: 运行中/已停止/已释放等
    pub region: Option<String>,        // 地区 (华北/华南/华东等)
    pub cloud_platform: String,        // 云平台 (阿里云/腾讯云/华为云/AWS)
    pub cloud_zone: String,            // 运营商/厂家 (云服务商或设备厂家)
    pub cloud_service: Option<String>, // 云服务 (对外服务/内部核心服务等)

    // 实例配置
    pub instance_type: String, // 实例类型 (如 ecs.g6.large) 或 物理机型号
    pub cpu_cores: u32,        // CPU核数
    pub memory_gb: u32,        // 内存(GB)
    pub system_disk_type: String, // 系统盘类型
    pub system_disk_size_gb: u32, // 系统盘大小(GB)
    pub data_disk_info: Option<String>, // 数据盘信息

    // 操作系统
    pub os_type: String, // 操作系统类型
    pub os_name: String, // 操作系统名称

    // 网络信息
    pub ip_address: String,           // 主IP地址
    pub public_ip: Option<String>,    // 公网IP
    pub ipv6_address: Option<String>, // IPv6地址

    // 业务信息
    pub customer_name: String,            // 客户名称
    pub department: Option<String>,       // 部门
    pub project: Option<String>,          // 项目
    pub application_name: Option<String>, // 应用名称
    pub contract_name: Option<String>,    // 合同名称
    pub owner_name: Option<String>,       // 负责人

    // 访问信息
    pub login_method: Option<String>, // 登录方式 (SSH/RDP/堡垒机等)
    pub login_username: Option<String>, // 登录用户名
    pub bastion_address: Option<String>, // 堡垒机地址
    pub bastion_account: Option<String>, // 堡垒机账号
    pub bastion_initial_password: Option<String>, // 堡垒机密码

    // 物理机特有信息
    pub serial_number: Option<String>,   // 设备序列号
    pub rack_location: Option<String>,   // 机架位置
    pub hardware_model: Option<String>,  // 硬件型号
    pub warranty_expiry: Option<String>, // 维保到期时间
    pub agent_status: Option<String>,    // Agent状态
    pub ipmi_address: Option<String>,    // IPMI/iDRAC地址

    // 云虚拟机特有信息
    pub billing_mode: Option<String>, // 计费模式
    pub expire_time: Option<String>,  // 到期时间
    pub charge_type: Option<String>,  // 付费类型

    // 时间信息
    pub created_at: String,          // 创建时间
    pub updated_at: Option<String>,  // 更新时间
    pub last_synced: Option<String>, // 最后同步时间

    // 其他
    pub tags: Option<String>,    // 标签 (JSON字符串)
    pub remarks: Option<String>, // 备注

    // 关联ID
    pub business_resource_id: Option<i32>, // 关联的业务资源ID
    pub cloud_asset_id: Option<i32>,       // 关联的云资产ID
}

/// 云服务资产查询参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudServiceAssetQuery {
    pub asset_type: Option<String>,     // 资产类型筛选
    pub source_type: Option<String>,    // 来源类型筛选
    pub cloud_platform: Option<String>, // 云平台筛选
    pub status: Option<String>,         // 状态筛选
    pub customer_name: Option<String>,  // 客户筛选
    pub department: Option<String>,     // 部门筛选
    pub project: Option<String>,        // 项目筛选
    pub search_keyword: Option<String>, // 搜索关键词
}

/// 云服务资产统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudServiceAssetStats {
    pub total_count: u32,
    pub physical_count: u32, // 物理机数量
    pub virtual_count: u32,  // 云虚拟机数量
    pub running_count: u32,
    pub stopped_count: u32,
    pub total_cpu_cores: u32,
    pub total_memory_gb: u32,
    pub expiring_soon_count: u32,        // 即将到期数量
    pub by_provider: Vec<(String, u32)>, // 按厂商统计
    pub by_customer: Vec<(String, u32)>, // 按客户统计
    pub by_status: Vec<(String, u32)>,   // 按状态统计
}

/// 创建云服务资产请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCloudServiceAssetRequest {
    pub business_resource_id: i32, // 关联的业务资源ID（用于验证交付状态）
    pub instance_id: String,       // 实例ID（云厂商返回）
    pub public_ip: Option<String>, // 公网IP
    pub ipv6_address: Option<String>, // IPv6地址
    pub instance_type: String,     // 实例类型（ecs、gpu等）
    pub cpu_cores: u32,            // CPU核数
    pub memory_gb: u32,            // 内存（GB）
    pub system_disk_gb: u32,       // 系统盘（GB）
    pub data_disk_gb: u32,         // 数据盘（GB）
    pub os_type: String,           // 操作系统类型
    pub status: String,            // 初始状态：默认为"运行中"
    pub tags: Option<String>,      // 标签（JSON字符串）
    pub remarks: Option<String>,   // 备注
}

// ============== 资源工单系统 ==============

/// 工单状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TicketStatus {
    PendingApproval,  // 待审批
    Approved,         // 已批准
    Rejected,         // 已拒绝
    PendingProvision, // 待配置
    Provisioning,     // 配置中
    PendingDelivery,  // 待交付
    Delivered,        // 已交付
    Archived,         // 已归档
}

impl TicketStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TicketStatus::PendingApproval => "pending_approval",
            TicketStatus::Approved => "approved",
            TicketStatus::Rejected => "rejected",
            TicketStatus::PendingProvision => "pending_provision",
            TicketStatus::Provisioning => "provisioning",
            TicketStatus::PendingDelivery => "pending_delivery",
            TicketStatus::Delivered => "delivered",
            TicketStatus::Archived => "archived",
        }
    }
}

impl FromStr for TicketStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending_approval" => Ok(TicketStatus::PendingApproval),
            "approved" => Ok(TicketStatus::Approved),
            "rejected" => Ok(TicketStatus::Rejected),
            "pending_provision" => Ok(TicketStatus::PendingProvision),
            "provisioning" => Ok(TicketStatus::Provisioning),
            "pending_delivery" => Ok(TicketStatus::PendingDelivery),
            "delivered" => Ok(TicketStatus::Delivered),
            "archived" => Ok(TicketStatus::Archived),
            _ => Err(format!("Unknown ticket status: {}", s)),
        }
    }
}

/// 资源工单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceTicket {
    pub id: Option<i32>,
    pub resource_type: ResourceType,
    pub ecs_name: String,
    pub ticket_status: TicketStatus,

    // 关联字段
    pub provider_id: Option<i32>,
    pub provider_name: Option<String>,
    pub cloud_platform_id: Option<i32>,
    pub cloud_platform_name: Option<String>,
    pub machine_room_id: Option<i32>,
    pub machine_room_name: Option<String>,

    // 资源配置
    pub cloud_region: Option<String>,
    pub cloud_category: Option<String>,
    pub zone_name: Option<String>,
    pub zone_cabinet: Option<String>,
    pub rack_units: i32,

    // 基本信息
    pub customer_name: Option<String>,
    pub application_name: Option<String>,
    pub contract_name: Option<String>,
    pub ecs_type: Option<String>,
    pub ecs_os: Option<String>,
    pub cpu_cores: i32,
    pub memory_gb: i32,
    pub system_disk: Option<String>,
    pub system_disk_size_gb: i32,
    pub data_disk: Option<String>,
    pub has_security_product: bool,
    pub security_products: Option<String>, // 选中的安全产品名称，逗号分隔
    pub ip_address: Option<String>,
    pub delivery_status: Option<String>,
    pub remarks: Option<String>,

    // 时间信息
    pub created_at: String,
    pub updated_at: Option<String>,
    pub created_by: String,

    // 审批信息
    pub approver: Option<String>,
    pub approve_time: Option<String>,
    pub approve_comment: Option<String>,

    // 配置信息
    pub provisioner: Option<String>,
    pub provision_time: Option<String>,
    pub provision_details: Option<String>,

    // 交付信息
    pub deliverer: Option<String>,
    pub deliver_time: Option<String>,
    pub deliver_comment: Option<String>,

    // 网络策略专用字段
    pub fw_source_zone: Option<String>,
    pub fw_source_address: Option<String>,
    pub fw_dest_zone: Option<String>,
    pub fw_dest_address: Option<String>,
    pub fw_protocol: Option<String>,
    pub fw_port: Option<String>,
    pub fw_direction: Option<String>,
    pub fw_valid_until: Option<String>,
    pub fw_firewall_name: Option<String>,
}

/// 创建资源工单请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateResourceTicketRequest {
    pub resource_type: ResourceType,
    pub ecs_name: String,

    // 关联字段
    pub provider_id: Option<i32>,
    pub cloud_platform_id: Option<i32>,
    pub machine_room_id: Option<i32>,

    // 资源配置
    pub cloud_region: Option<String>,
    pub cloud_category: Option<String>,
    pub zone_name: Option<String>,
    pub zone_cabinet: Option<String>,
    pub rack_units: Option<i32>,

    // 基本信息
    pub customer_name: Option<String>,
    pub application_name: Option<String>,
    pub contract_name: Option<String>,
    pub ecs_type: Option<String>,
    pub ecs_os: Option<String>,
    pub cpu_cores: Option<i32>,
    pub memory_gb: Option<i32>,
    pub system_disk: Option<String>,
    pub system_disk_size_gb: Option<i32>,
    pub data_disk: Option<String>,
    pub has_security_product: Option<bool>,
    pub security_products: Option<String>, // 选中的安全产品名称，逗号分隔
    pub ip_address: Option<String>,
    pub remarks: Option<String>,

    // 网络策略专用字段
    pub fw_source_zone: Option<String>,
    pub fw_source_address: Option<String>,
    pub fw_dest_zone: Option<String>,
    pub fw_dest_address: Option<String>,
    pub fw_protocol: Option<String>,
    pub fw_port: Option<String>,
    pub fw_direction: Option<String>,
    pub fw_valid_until: Option<String>,
    pub fw_firewall_name: Option<String>,
}

/// 更新资源工单请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateResourceTicketRequest {
    pub ecs_name: Option<String>,
    pub provider_id: Option<i32>,
    pub cloud_platform_id: Option<i32>,
    pub machine_room_id: Option<i32>,
    pub cloud_region: Option<String>,
    pub cloud_category: Option<String>,
    pub zone_name: Option<String>,
    pub zone_cabinet: Option<String>,
    pub rack_units: Option<i32>,
    pub customer_name: Option<String>,
    pub application_name: Option<String>,
    pub contract_name: Option<String>,
    pub ecs_type: Option<String>,
    pub ecs_os: Option<String>,
    pub cpu_cores: Option<i32>,
    pub memory_gb: Option<i32>,
    pub system_disk: Option<String>,
    pub system_disk_size_gb: Option<i32>,
    pub data_disk: Option<String>,
    pub has_security_product: Option<bool>,
    pub security_products: Option<String>, // 选中的安全产品名称，逗号分隔
    pub ip_address: Option<String>,
    pub delivery_status: Option<String>,
    pub remarks: Option<String>,
}

/// 工单审批请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApproveTicketRequest {
    pub approved: bool,
    pub comment: Option<String>,
}

/// 工单配置请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisionTicketRequest {
    pub details: Option<String>,
}

/// 工单交付请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliverTicketRequest {
    pub comment: Option<String>,
}

/// 资源工单查询参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceTicketQuery {
    pub search_keyword: Option<String>,
    pub resource_type: Option<String>,
    pub ticket_status: Option<String>,
    pub provider_id: Option<i32>,
}
