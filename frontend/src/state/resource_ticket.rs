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

    /// 转换为后端API字符串
    pub fn to_api_str(self) -> &'static str {
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
    pub fn is_pending_provision_stage(&self) -> bool {
        matches!(
            self,
            TicketStatus::PendingProvision | TicketStatus::Approved | TicketStatus::Provisioning
        )
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            TicketStatus::Draft => "草稿",
            TicketStatus::Submitted => "已提交",
            TicketStatus::PendingApproval => "待审批",
            TicketStatus::Approved
            | TicketStatus::PendingProvision
            | TicketStatus::Provisioning => "待配置",
            TicketStatus::Rejected => "已拒绝",
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
            TicketStatus::Approved
            | TicketStatus::PendingProvision
            | TicketStatus::Provisioning => "bg-purple-100 text-purple-800",
            TicketStatus::Rejected => "bg-red-100 text-red-800",
            TicketStatus::PendingDelivery => "bg-orange-100 text-orange-800",
            TicketStatus::Delivered => "bg-teal-100 text-teal-800",
            TicketStatus::Archived => "bg-gray-100 text-gray-600",
        }
    }

    /// 转换为后端API字符串
    pub fn to_api_str(self) -> &'static str {
        match self {
            TicketStatus::Draft => "draft",
            TicketStatus::Submitted => "submitted",
            TicketStatus::PendingApproval => "pending_approval",
            TicketStatus::Approved
            | TicketStatus::PendingProvision
            | TicketStatus::Provisioning => "pending_provision",
            TicketStatus::Rejected => "rejected",
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
            "PendingApproval" => TicketStatus::PendingApproval,
            "approved" | "Approved" => TicketStatus::PendingProvision,
            "rejected" => TicketStatus::Rejected,
            "Rejected" => TicketStatus::Rejected,
            "pending_provision" | "PendingProvision" => TicketStatus::PendingProvision,
            "provisioning" | "Provisioning" => TicketStatus::PendingProvision,
            "pending_delivery" => TicketStatus::PendingDelivery,
            "PendingDelivery" => TicketStatus::PendingDelivery,
            "delivered" => TicketStatus::Delivered,
            "Delivered" => TicketStatus::Delivered,
            "archived" => TicketStatus::Archived,
            "Archived" => TicketStatus::Archived,
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
    pub provider_id: Option<i32>,       // 关联的服务商ID
    pub provider_name: String,          // 冗余字段，便于显示
    pub cloud_platform_id: Option<i32>, // 关联的云平台ID
    pub cloud_platform_name: String,    // 冗余字段，便于显示
    pub machine_room_id: Option<i32>,   // 关联的机房ID
    pub machine_room_name: String,      // 冗余字段，便于显示

    pub cloud_region: String,
    pub cloud_category: String,
    pub zone_name: String,    // 可用区（云资源）/ 区域（物理机）
    pub zone_cabinet: String, // 机柜（物理机）
    pub rack_units: i32,      // 机位(U数)（物理机）
    pub customer_name: String,
    pub application_name: String,
    pub application_endpoint_id: Option<i32>,
    pub application_domain: Option<String>,
    pub contract_name: String,
    pub ecs_type: String,
    pub ecs_os: String,
    pub resource_count: i32,
    pub cpu_cores: i32,
    pub memory_gb: i32,
    pub system_disk: String,
    pub system_disk_size_gb: i32,
    pub data_disk: String,
    pub expire_at: Option<String>,
    pub has_security_product: bool,
    pub security_products: String, // 选中的安全产品名称，逗号分隔
    pub ip_address: String,
    pub delivery_status: String,
    pub remarks: String,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: String,
    pub applicant_name: String,
    pub organization_id: Option<i32>,
    pub organization_name: String,
    pub department_id: Option<i32>,
    pub department_name: String,

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
    pub fw_source_zone: Option<String>,    // 源区域
    pub fw_source_address: Option<String>, // 源地址/IP段
    pub fw_source_port: Option<String>,    // 源端口
    pub fw_dest_zone: Option<String>,      // 目标区域
    pub fw_dest_address: Option<String>,   // 目标地址/IP
    pub fw_dest_port: Option<String>,      // 目标端口
    pub fw_protocol: Option<String>,       // 协议类型 (TCP/UDP/ICMP/ANY)
    pub fw_port: Option<String>,           // 端口 (单个端口或范围，如 "80" 或 "8080-8090")
    pub fw_direction: Option<String>,      // 访问方向 (入站/出站/双向)
    pub fw_valid_until: Option<String>,    // 有效期限
    pub fw_firewall_name: Option<String>,  // 防火墙设备名称
}

#[cfg(test)]
mod tests {
    use super::TicketStatus;

    #[test]
    fn legacy_ticket_statuses_are_normalized_to_pending_provision() {
        assert_eq!(
            TicketStatus::from_api_str("approved"),
            TicketStatus::PendingProvision
        );
        assert_eq!(
            TicketStatus::from_api_str("provisioning"),
            TicketStatus::PendingProvision
        );
        assert_eq!(
            TicketStatus::PendingProvision.to_api_str(),
            "pending_provision"
        );
        assert_eq!(TicketStatus::Approved.to_api_str(), "pending_provision");
    }

    #[test]
    fn pending_provision_stage_helper_covers_legacy_variants() {
        assert!(TicketStatus::PendingProvision.is_pending_provision_stage());
        assert!(TicketStatus::Approved.is_pending_provision_stage());
        assert!(TicketStatus::Provisioning.is_pending_provision_stage());
        assert!(!TicketStatus::PendingDelivery.is_pending_provision_stage());
    }
}
