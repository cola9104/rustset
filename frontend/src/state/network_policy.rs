use crate::components::resource_ticket::network_policy::network_policy_request::{
    AccessDirection, NetworkPolicyRequest, NetworkPolicyStatus, PolicyProtocol,
};

/// 网络策略配置（用于全局状态管理）
#[derive(Clone, Debug, PartialEq)]
pub struct NetworkPolicyConfig {
    pub id: i32,
    pub title: String,
    pub organization: String,
    pub applicant: String,
    pub applicant_account: String,
    pub department: String,
    pub source_zone: String,
    pub source_address: String,
    pub source_port: String,
    pub destination_zone: String,
    pub destination_address: String,
    pub destination_port: String,
    pub direction: AccessDirection,
    pub protocol: PolicyProtocol,
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
            organization: req.organization,
            applicant: req.applicant,
            applicant_account: req.applicant_account,
            department: req.department,
            source_zone: req.source_zone,
            source_address: req.source_address,
            source_port: req.source_port,
            destination_zone: req.destination_zone,
            destination_address: req.destination_address,
            destination_port: req.destination_port,
            direction: req.direction,
            protocol: req.protocol,
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
            organization: config.organization,
            applicant: config.applicant,
            applicant_account: config.applicant_account,
            department: config.department,
            source_zone: config.source_zone,
            source_address: config.source_address,
            source_port: config.source_port,
            destination_zone: config.destination_zone,
            destination_address: config.destination_address,
            destination_port: config.destination_port,
            direction: config.direction,
            protocol: config.protocol,
            description: config.description,
            valid_until: config.valid_until,
            status: config.status,
            created_at: config.created_at,
        }
    }
}
