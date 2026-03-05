use serde::{Deserialize, Serialize};

/// 资源类型
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceType {
    Physical,
    Cloud,
    Network,
}

impl ResourceType {
    pub fn display_name(&self) -> &'static str {
        match self {
            ResourceType::Physical => "物理机",
            ResourceType::Cloud => "云服务",
            ResourceType::Network => "网络策略",
        }
    }

    pub fn icon_class(&self) -> &'static str {
        match self {
            ResourceType::Physical => "fa-server",
            ResourceType::Cloud => "fa-cloud",
            ResourceType::Network => "fa-shield-halved",
        }
    }

    /// 转换为后端API字符串
    pub fn to_api_str(&self) -> &'static str {
        match self {
            ResourceType::Physical => "physical",
            ResourceType::Cloud => "cloud",
            ResourceType::Network => "network",
        }
    }

    /// 从后端API字符串解析
    pub fn from_api_str(s: &str) -> Self {
        match s {
            "physical" => ResourceType::Physical,
            "cloud" => ResourceType::Cloud,
            "network" => ResourceType::Network,
            _ => ResourceType::Cloud,
        }
    }
}

/// 网络区域
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum NetworkZone {
    InternetDmz,      // 互联网DMZ
    GovDmz,           // 政务网DMZ
    OfficeNetwork,    // 办公网
    DataCenter,       // 数据中心
    TrustedZone,      // 可信区
    Other(String),    // 其他
}

impl NetworkZone {
    pub fn display_name(&self) -> String {
        match self {
            NetworkZone::InternetDmz => "互联网DMZ".to_string(),
            NetworkZone::GovDmz => "政务网DMZ".to_string(),
            NetworkZone::OfficeNetwork => "办公网".to_string(),
            NetworkZone::DataCenter => "数据中心".to_string(),
            NetworkZone::TrustedZone => "可信区".to_string(),
            NetworkZone::Other(s) => s.clone(),
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "互联网DMZ" => NetworkZone::InternetDmz,
            "政务网DMZ" => NetworkZone::GovDmz,
            "办公网" => NetworkZone::OfficeNetwork,
            "数据中心" => NetworkZone::DataCenter,
            "可信区" => NetworkZone::TrustedZone,
            _ => NetworkZone::Other(s.to_string()),
        }
    }
}

/// 协议类型
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FirewallProtocol {
    Tcp,
    Udp,
    Icmp,
    Any,
}

impl FirewallProtocol {
    pub fn display_name(&self) -> &'static str {
        match self {
            FirewallProtocol::Tcp => "TCP",
            FirewallProtocol::Udp => "UDP",
            FirewallProtocol::Icmp => "ICMP",
            FirewallProtocol::Any => "ANY",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "TCP" => FirewallProtocol::Tcp,
            "UDP" => FirewallProtocol::Udp,
            "ICMP" => FirewallProtocol::Icmp,
            _ => FirewallProtocol::Any,
        }
    }
}

/// 访问方向
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessDirection {
    Inbound,   // 入站
    Outbound,  // 出站
    Bidirectional, // 双向
}

impl AccessDirection {
    pub fn display_name(&self) -> &'static str {
        match self {
            AccessDirection::Inbound => "入站",
            AccessDirection::Outbound => "出站",
            AccessDirection::Bidirectional => "双向",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "入站" => AccessDirection::Inbound,
            "出站" => AccessDirection::Outbound,
            _ => AccessDirection::Bidirectional,
        }
    }
}

/// 工单状态
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TicketStatus {
    Draft,
    Submitted,
    PendingApproval,
    Approved,
    Rejected,
    PendingProvision,
    Provisioning,
    PendingDelivery,
    Delivered,
    Archived,
}

impl TicketStatus {
    pub fn display_name(&self) -> &'static str {
        match self {
            TicketStatus::Draft => "草稿",
            TicketStatus::Submitted => "已提交",
            TicketStatus::PendingApproval => "待审批",
            TicketStatus::Approved => "已通过",
            TicketStatus::Rejected => "已拒绝",
            TicketStatus::PendingProvision => "待配置",
            TicketStatus::Provisioning => "配置中",
            TicketStatus::PendingDelivery => "待交付",
            TicketStatus::Delivered => "已交付",
            TicketStatus::Archived => "已归档",
        }
    }

    pub fn color_class(&self) -> &'static str {
        match self {
            TicketStatus::Draft => "bg-gray-100 text-gray-800",
            TicketStatus::Submitted => "bg-blue-100 text-blue-800",
            TicketStatus::PendingApproval => "bg-yellow-100 text-yellow-800",
            TicketStatus::Approved => "bg-green-100 text-green-800",
            TicketStatus::Rejected => "bg-red-100 text-red-800",
            TicketStatus::PendingProvision => "bg-purple-100 text-purple-800",
            TicketStatus::Provisioning => "bg-indigo-100 text-indigo-800",
            TicketStatus::PendingDelivery => "bg-orange-100 text-orange-800",
            TicketStatus::Delivered => "bg-teal-100 text-teal-800",
            TicketStatus::Archived => "bg-gray-100 text-gray-600",
        }
    }

    /// 转换为后端API字符串
    pub fn to_api_str(&self) -> &'static str {
        match self {
            TicketStatus::Draft => "draft",
            TicketStatus::Submitted => "submitted",
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

    /// 从后端API字符串解析
    pub fn from_api_str(s: &str) -> Self {
        match s {
            "draft" => TicketStatus::Draft,
            "submitted" => TicketStatus::Submitted,
            "pending_approval" => TicketStatus::PendingApproval,
            "approved" => TicketStatus::Approved,
            "rejected" => TicketStatus::Rejected,
            "pending_provision" => TicketStatus::PendingProvision,
            "provisioning" => TicketStatus::Provisioning,
            "pending_delivery" => TicketStatus::PendingDelivery,
            "delivered" => TicketStatus::Delivered,
            "archived" => TicketStatus::Archived,
            _ => TicketStatus::Draft,
        }
    }
}

/// 资源工单数据模型
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ResourceTicket {
    pub id: i32,
    pub resource_type: ResourceType,
    pub ecs_name: String,
    pub ticket_status: TicketStatus,

    // 关联字段
    pub provider_id: Option<i32>,            // 关联的服务商ID
    pub provider_name: String,               // 冗余字段，便于显示
    pub cloud_platform_id: Option<i32>,      // 关联的云平台ID
    pub cloud_platform_name: String,        // 冗余字段，便于显示
    pub machine_room_id: Option<i32>,         // 关联的机房ID
    pub machine_room_name: String,           // 冗余字段，便于显示

    pub cloud_region: String,
    pub cloud_category: String,
    pub zone_name: String,          // 可用区（云资源）/ 区域（物理机）
    pub zone_cabinet: String,       // 机柜（物理机）
    pub rack_units: i32,            // 机位(U数)（物理机）
    pub customer_name: String,
    pub application_name: String,
    pub contract_name: String,
    pub ecs_type: String,
    pub ecs_os: String,
    pub cpu_cores: i32,
    pub memory_gb: i32,
    pub system_disk: String,
    pub system_disk_size_gb: i32,
    pub data_disk: String,
    pub has_security_product: bool,
    pub ip_address: String,
    pub delivery_status: String,
    pub remarks: String,
    pub created_at: String,
    pub updated_at: String,
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

    // 网络策略工单专用字段（防火墙端口开放申请）
    pub fw_source_zone: Option<String>,         // 源区域
    pub fw_source_address: Option<String>,      // 源地址/IP段
    pub fw_dest_zone: Option<String>,           // 目标区域
    pub fw_dest_address: Option<String>,        // 目标地址/IP
    pub fw_protocol: Option<String>,            // 协议类型 (TCP/UDP/ICMP/ANY)
    pub fw_port: Option<String>,                // 端口 (单个端口或范围，如 "80" 或 "8080-8090")
    pub fw_direction: Option<String>,           // 访问方向 (入站/出站/双向)
    pub fw_valid_until: Option<String>,         // 有效期限
    pub fw_firewall_name: Option<String>,       // 防火墙设备名称
}

/// 创建测试数据（与云平台和机房关联）
pub fn init_test_tickets() -> Vec<ResourceTicket> {
    vec![
        ResourceTicket {
            id: 1,
            resource_type: ResourceType::Cloud,
            ecs_name: "Web服务器-01".to_string(),
            ticket_status: TicketStatus::PendingApproval,
            provider_id: Some(1),
            provider_name: "电信".to_string(),
            cloud_platform_id: Some(1),
            cloud_platform_name: "电信-政务公有云".to_string(),
            machine_room_id: Some(1),
            machine_room_name: "市政务云机房A".to_string(),
            cloud_region: "华东".to_string(),
            cloud_category: "云主机".to_string(),
            zone_name: "杭州可用区A".to_string(),
            zone_cabinet: String::new(),
            rack_units: 0,
            customer_name: "某某公司".to_string(),
            application_name: "业务系统A".to_string(),
            contract_name: "合同-2024-001".to_string(),
            ecs_type: "ecs.g6.xlarge".to_string(),
            ecs_os: "CentOS 7.9".to_string(),
            cpu_cores: 4,
            memory_gb: 16,
            system_disk: "SSD".to_string(),
            system_disk_size_gb: 100,
            data_disk: "500GB SSD".to_string(),
            has_security_product: true,
            ip_address: "".to_string(),
            delivery_status: "未交付".to_string(),
            remarks: "用于部署Web服务".to_string(),
            created_at: "2024-03-01 09:30".to_string(),
            updated_at: "2024-03-01 09:30".to_string(),
            created_by: "张三".to_string(),
            approver: None,
            approve_time: None,
            approve_comment: None,
            provisioner: None,
            provision_time: None,
            provision_details: None,
            deliverer: None,
            deliver_time: None,
            deliver_comment: None,
            fw_source_zone: None,
            fw_source_address: None,
            fw_dest_zone: None,
            fw_dest_address: None,
            fw_protocol: None,
            fw_port: None,
            fw_direction: None,
            fw_valid_until: None,
            fw_firewall_name: None,
        },
        ResourceTicket {
            id: 2,
            resource_type: ResourceType::Network,
            ecs_name: "互联网DMZ访问政务网DMZ".to_string(),
            ticket_status: TicketStatus::PendingApproval,
            provider_id: Some(1),
            provider_name: "电信".to_string(),
            cloud_platform_id: Some(1),
            cloud_platform_name: "电信-政务公有云".to_string(),
            machine_room_id: Some(1),
            machine_room_name: "市政务云机房A".to_string(),
            cloud_region: "华东".to_string(),
            cloud_category: "网络策略".to_string(),
            zone_name: "".to_string(),
            zone_cabinet: String::new(),
            rack_units: 0,
            customer_name: "某某局".to_string(),
            application_name: "门户网站访问".to_string(),
            contract_name: "合同-2024-001".to_string(),
            ecs_type: "".to_string(),
            ecs_os: "".to_string(),
            cpu_cores: 0,
            memory_gb: 0,
            system_disk: "".to_string(),
            system_disk_size_gb: 0,
            data_disk: "".to_string(),
            has_security_product: false,
            ip_address: "".to_string(),
            delivery_status: "未交付".to_string(),
            remarks: "门户网站需要从互联网DMZ访问政务网DMZ的数据库服务器".to_string(),
            created_at: "2024-03-04 10:00".to_string(),
            updated_at: "2024-03-04 10:00".to_string(),
            created_by: "张三".to_string(),
            approver: None,
            approve_time: None,
            approve_comment: None,
            provisioner: None,
            provision_time: None,
            provision_details: None,
            deliverer: None,
            deliver_time: None,
            deliver_comment: None,
            fw_source_zone: Some("互联网DMZ".to_string()),
            fw_source_address: Some("203.0.113.0/24".to_string()),
            fw_dest_zone: Some("政务网DMZ".to_string()),
            fw_dest_address: Some("10.1.1.100".to_string()),
            fw_protocol: Some("TCP".to_string()),
            fw_port: Some("443".to_string()),
            fw_direction: Some("入站".to_string()),
            fw_valid_until: Some("2025-03-04".to_string()),
            fw_firewall_name: Some("核心防火墙-01".to_string()),
        },
        ResourceTicket {
            id: 3,
            resource_type: ResourceType::Network,
            ecs_name: "办公网访问数据中心".to_string(),
            ticket_status: TicketStatus::Approved,
            provider_id: Some(1),
            provider_name: "电信".to_string(),
            cloud_platform_id: Some(1),
            cloud_platform_name: "电信-政务公有云".to_string(),
            machine_room_id: Some(1),
            machine_room_name: "市政务云机房A".to_string(),
            cloud_region: "华东".to_string(),
            cloud_category: "网络策略".to_string(),
            zone_name: "".to_string(),
            zone_cabinet: String::new(),
            rack_units: 0,
            customer_name: "某某局".to_string(),
            application_name: "管理后台访问".to_string(),
            contract_name: "合同-2024-002".to_string(),
            ecs_type: "".to_string(),
            ecs_os: "".to_string(),
            cpu_cores: 0,
            memory_gb: 0,
            system_disk: "".to_string(),
            system_disk_size_gb: 0,
            data_disk: "".to_string(),
            has_security_product: false,
            ip_address: "".to_string(),
            delivery_status: "未交付".to_string(),
            remarks: "办公网终端需要访问数据中心的管理后台".to_string(),
            created_at: "2024-03-03 14:20".to_string(),
            updated_at: "2024-03-03 16:00".to_string(),
            created_by: "李四".to_string(),
            approver: Some("王审批".to_string()),
            approve_time: Some("2024-03-03 16:00".to_string()),
            approve_comment: Some("合理需求，批准开通".to_string()),
            provisioner: None,
            provision_time: None,
            provision_details: None,
            deliverer: None,
            deliver_time: None,
            deliver_comment: None,
            fw_source_zone: Some("办公网".to_string()),
            fw_source_address: Some("192.168.10.0/24".to_string()),
            fw_dest_zone: Some("数据中心".to_string()),
            fw_dest_address: Some("10.2.1.50".to_string()),
            fw_protocol: Some("TCP".to_string()),
            fw_port: Some("8080".to_string()),
            fw_direction: Some("出站".to_string()),
            fw_valid_until: Some("2025-06-30".to_string()),
            fw_firewall_name: Some("核心防火墙-02".to_string()),
        },
    ]
}
