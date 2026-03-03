use serde::{Deserialize, Serialize};

/// 资源类型
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceType {
    Physical,
    Cloud,
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
}

/// 资源工单数据模型
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ResourceTicket {
    pub id: i32,
    pub resource_type: ResourceType,
    pub ecs_name: String,
    pub ticket_status: TicketStatus,

    // 关联字段
    pub cloud_platform_id: Option<i32>,      // 关联的云平台ID
    pub cloud_platform_name: String,        // 冗余字段，便于显示
    pub machine_room_id: Option<i32>,         // 关联的机房ID
    pub machine_room_name: String,           // 冗余字段，便于显示

    pub cloud_region: String,
    pub cloud_category: String,
    pub zone_name: String,
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
}

/// 创建测试数据（与云平台和机房关联）
pub fn init_test_tickets() -> Vec<ResourceTicket> {
    vec![
        ResourceTicket {
            id: 1,
            resource_type: ResourceType::Cloud,
            ecs_name: "Web服务器-01".to_string(),
            ticket_status: TicketStatus::PendingApproval,
            cloud_platform_id: Some(1),  // 关联：电信-政务公有云
            cloud_platform_name: "电信-政务公有云".to_string(),
            machine_room_id: Some(1),     // 关联：市政务云机房A
            machine_room_name: "市政务云机房A".to_string(),
            cloud_region: "华东".to_string(),
            cloud_category: "云主机".to_string(),
            zone_name: "杭州可用区A".to_string(),
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
        },
        ResourceTicket {
            id: 2,
            resource_type: ResourceType::Physical,
            ecs_name: "数据库服务器-01".to_string(),
            ticket_status: TicketStatus::Approved,
            cloud_platform_id: Some(2),  // 关联：电信-政务私有云
            cloud_platform_name: "电信-政务私有云".to_string(),
            machine_room_id: Some(2),     // 关联：核心机房
            machine_room_name: "核心机房".to_string(),
            cloud_region: "华东".to_string(),
            cloud_category: "物理机".to_string(),
            zone_name: "杭州机房".to_string(),
            customer_name: "某某局".to_string(),
            application_name: "核心数据库".to_string(),
            contract_name: "合同-2024-002".to_string(),
            ecs_type: "Dell R740".to_string(),
            ecs_os: "Ubuntu 20.04".to_string(),
            cpu_cores: 64,
            memory_gb: 256,
            system_disk: "NVMe".to_string(),
            system_disk_size_gb: 2000,
            data_disk: "4TB SATA".to_string(),
            has_security_product: true,
            ip_address: "".to_string(),
            delivery_status: "未交付".to_string(),
            remarks: "核心数据库服务器".to_string(),
            created_at: "2024-02-28 14:20".to_string(),
            updated_at: "2024-03-01 10:15".to_string(),
            created_by: "李四".to_string(),
            approver: Some("王审批".to_string()),
            approve_time: Some("2024-03-01 10:15".to_string()),
            approve_comment: Some("配置合理，批准通过".to_string()),
            provisioner: None,
            provision_time: None,
            provision_details: None,
            deliverer: None,
            deliver_time: None,
            deliver_comment: None,
        },
        ResourceTicket {
            id: 3,
            resource_type: ResourceType::Cloud,
            ecs_name: "应用服务器-01".to_string(),
            ticket_status: TicketStatus::PendingProvision,
            cloud_platform_id: Some(3),  // 关联：联通-政务公有云
            cloud_platform_name: "联通-政务公有云".to_string(),
            machine_room_id: Some(3),     // 关联：市政务云机房B
            machine_room_name: "市政务云机房B".to_string(),
            cloud_region: "华北".to_string(),
            cloud_category: "云主机".to_string(),
            zone_name: "北京可用区B".to_string(),
            customer_name: "测试公司".to_string(),
            application_name: "测试环境".to_string(),
            contract_name: "合同-2024-003".to_string(),
            ecs_type: "ecs.g6.2xlarge".to_string(),
            ecs_os: "CentOS 8.4".to_string(),
            cpu_cores: 8,
            memory_gb: 32,
            system_disk: "SSD".to_string(),
            system_disk_size_gb: 200,
            data_disk: "1TB SSD".to_string(),
            has_security_product: false,
            ip_address: "".to_string(),
            delivery_status: "未交付".to_string(),
            remarks: "测试环境应用服务器".to_string(),
            created_at: "2024-03-02 08:45".to_string(),
            updated_at: "2024-03-02 11:30".to_string(),
            created_by: "王五".to_string(),
            approver: Some("王审批".to_string()),
            approve_time: Some("2024-03-02 11:30".to_string()),
            approve_comment: None,
            provisioner: None,
            provision_time: None,
            provision_details: None,
            deliverer: None,
            deliver_time: None,
            deliver_comment: None,
        },
        ResourceTicket {
            id: 4,
            resource_type: ResourceType::Cloud,
            ecs_name: "文件服务器-01".to_string(),
            ticket_status: TicketStatus::PendingDelivery,
            cloud_platform_id: Some(1),  // 关联：电信-政务公有云
            cloud_platform_name: "电信-政务公有云".to_string(),
            machine_room_id: Some(1),     // 关联：市政务云机房A
            machine_room_name: "市政务云机房A".to_string(),
            cloud_region: "华东".to_string(),
            cloud_category: "云主机".to_string(),
            zone_name: "杭州可用区C".to_string(),
            customer_name: "某某公司".to_string(),
            application_name: "文件共享服务".to_string(),
            contract_name: "合同-2024-001".to_string(),
            ecs_type: "ecs.g6.large".to_string(),
            ecs_os: "Ubuntu 22.04".to_string(),
            cpu_cores: 2,
            memory_gb: 8,
            system_disk: "SSD".to_string(),
            system_disk_size_gb: 50,
            data_disk: "2TB SSD".to_string(),
            has_security_product: true,
            ip_address: "192.168.1.200".to_string(),
            delivery_status: "待交付".to_string(),
            remarks: "文件共享服务器".to_string(),
            created_at: "2024-03-01 16:20".to_string(),
            updated_at: "2024-03-03 09:00".to_string(),
            created_by: "赵六".to_string(),
            approver: Some("王审批".to_string()),
            approve_time: Some("2024-03-01 17:00".to_string()),
            approve_comment: Some("同意".to_string()),
            provisioner: Some("运维张三".to_string()),
            provision_time: Some("2024-03-03 09:00".to_string()),
            provision_details: Some("已创建实例，IP: 192.168.1.200，已配置安全组".to_string()),
            deliverer: None,
            deliver_time: None,
            deliver_comment: None,
        },
        ResourceTicket {
            id: 5,
            resource_type: ResourceType::Physical,
            ecs_name: "备份服务器-01".to_string(),
            ticket_status: TicketStatus::Delivered,
            cloud_platform_id: Some(2),  // 关联：电信-政务私有云
            cloud_platform_name: "电信-政务私有云".to_string(),
            machine_room_id: Some(2),     // 关联：核心机房
            machine_room_name: "核心机房".to_string(),
            cloud_region: "华东".to_string(),
            cloud_category: "物理机".to_string(),
            zone_name: "杭州机房".to_string(),
            customer_name: "某某局".to_string(),
            application_name: "备份系统".to_string(),
            contract_name: "合同-2024-004".to_string(),
            ecs_type: "Dell R740".to_string(),
            ecs_os: "CentOS 7.9".to_string(),
            cpu_cores: 32,
            memory_gb: 128,
            system_disk: "SSD".to_string(),
            system_disk_size_gb: 500,
            data_disk: "8TB SATA".to_string(),
            has_security_product: true,
            ip_address: "192.168.2.50".to_string(),
            delivery_status: "已交付".to_string(),
            remarks: "数据备份服务器".to_string(),
            created_at: "2024-02-25 10:00".to_string(),
            updated_at: "2024-02-28 15:30".to_string(),
            created_by: "李四".to_string(),
            approver: Some("王审批".to_string()),
            approve_time: Some("2024-02-25 14:00".to_string()),
            approve_comment: Some("批准".to_string()),
            provisioner: Some("运维李四".to_string()),
            provision_time: Some("2024-02-26 10:00".to_string()),
            provision_details: Some("服务器已上架，IP: 192.168.2.50，RAID配置完成".to_string()),
            deliverer: Some("交付王五".to_string()),
            deliver_time: Some("2024-02-28 15:30".to_string()),
            deliver_comment: Some("已复核无误，正式交付".to_string()),
        },
    ]
}
