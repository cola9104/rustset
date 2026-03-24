use crate::components::resource_ticket::network_policy::network_policy_request::{
    AccessDirection, NetworkPolicyRequest, NetworkPolicyStatus, PolicyProtocol,
};
use dioxus::prelude::ReadableExt;

/// 网络策略配置（用于全局状态管理）
#[derive(Clone, Debug, PartialEq)]
pub struct NetworkPolicyConfig {
    pub id: i32,
    pub title: String,
    pub applicant: String,
    pub department: String,
    pub source_zone: String,
    pub destination_zone: String,
    pub direction: AccessDirection,
    pub protocol: PolicyProtocol,
    pub port_range: String,
    pub description: String,
    pub valid_until: String,
    pub status: NetworkPolicyStatus,
    pub created_at: String,
}

impl From<NetworkPolicyRequest> for NetworkPolicyConfig {
    fn from(req: NetworkPolicyRequest) -> Self {
        Self {
            id: req.id,
            title: req.title,
            applicant: req.applicant,
            department: req.department,
            source_zone: req.source_zone,
            destination_zone: req.destination_zone,
            direction: req.direction,
            protocol: req.protocol,
            port_range: req.port_range,
            description: req.description,
            valid_until: req.valid_until,
            status: req.status,
            created_at: req.created_at,
        }
    }
}

impl From<NetworkPolicyConfig> for NetworkPolicyRequest {
    fn from(config: NetworkPolicyConfig) -> Self {
        Self {
            id: config.id,
            title: config.title,
            applicant: config.applicant,
            department: config.department,
            source_zone: config.source_zone,
            destination_zone: config.destination_zone,
            direction: config.direction,
            protocol: config.protocol,
            port_range: config.port_range,
            description: config.description,
            valid_until: config.valid_until,
            status: config.status,
            created_at: config.created_at,
        }
    }
}

/// 初始化网络策略数据（已废弃，仅供兼容性保留）
pub fn init_network_policies() -> Vec<NetworkPolicyConfig> {
    Vec::new()
}

/// 获取已生效的网络策略（用于端口安全检查）
pub fn get_active_network_policies() -> Vec<NetworkPolicyConfig> {
    // 从全局状态获取已生效的网络策略
    use crate::app::NETWORK_POLICIES_STATE;
    NETWORK_POLICIES_STATE
        .read()
        .iter()
        .filter(|p| p.status == NetworkPolicyStatus::Active)
        .cloned()
        .collect()
}
