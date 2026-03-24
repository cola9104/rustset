use serde::{Deserialize, Serialize};

/// 网络区域 - 用于标记网络访问策略的来源/目标
///
/// 规则：
/// - 云网络 = 绑定 cloud_platform_id（阿里云、华为云等）
/// - 物理网络 = 绑定 machine_room_id（机房A内网、机房B内网等）
///
/// 网段绑定：
/// - cidr_blocks: CIDR 格式网段（如 "192.168.1.0/24", "10.0.0.0/16"）
/// - ip_ranges: IP 范围或单个 IP（如 "192.168.1.1-192.168.1.100", "10.0.0.1"）
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NetworkZone {
    pub id: i32,
    pub name: String,                   // 区域名称：阿里云、市政务云机房A内网等
    pub cloud_platform_id: Option<i32>, // 关联的云平台ID（云网络则有值）
    pub machine_room_id: Option<i32>,   // 关联的机房ID（物理网络则有值）
    pub cidr_blocks: Vec<String>,       // CIDR 网段列表
    pub ip_ranges: Vec<String>,         // IP 范围列表（支持单个IP或范围）
    pub description: String,            // 描述
    pub is_active: bool,
}

impl NetworkZone {
    /// 判断是否为云平台网络区域
    pub fn is_cloud_zone(&self) -> bool {
        self.cloud_platform_id.is_some()
    }

    /// 判断是否为物理机房网络区域
    pub fn is_physical_zone(&self) -> bool {
        self.machine_room_id.is_some()
    }

    /// 是否配置了网段
    pub fn has_network_config(&self) -> bool {
        !self.cidr_blocks.is_empty() || !self.ip_ranges.is_empty()
    }
}
