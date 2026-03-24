use super::{CloudPlatformConfig, MachineRoomConfig};
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

    /// 获取区域类型显示名称
    pub fn zone_type_name(&self) -> &'static str {
        if self.is_cloud_zone() {
            "云平台"
        } else {
            "物理机房"
        }
    }

    /// 获取所有网段/IP的显示文本
    pub fn network_display(&self) -> String {
        let mut parts = Vec::new();

        // 添加 CIDR 网段
        for cidr in &self.cidr_blocks {
            parts.push(format!("CIDR: {}", cidr));
        }

        // 添加 IP 范围
        for range in &self.ip_ranges {
            parts.push(format!("IP: {}", range));
        }

        if parts.is_empty() {
            "未配置网段".to_string()
        } else {
            parts.join(", ")
        }
    }

    /// 是否配置了网段
    pub fn has_network_config(&self) -> bool {
        !self.cidr_blocks.is_empty() || !self.ip_ranges.is_empty()
    }
}

/// 从云平台配置生成云网络区域
pub fn create_cloud_zone(id: i32, platform: &CloudPlatformConfig) -> NetworkZone {
    NetworkZone {
        id,
        name: platform.foundation.clone(), // 阿里云、华为云
        cloud_platform_id: Some(platform.id),
        machine_room_id: None,
        cidr_blocks: Vec::new(), // 默认为空，需要手动配置
        ip_ranges: Vec::new(),   // 默认为空，需要手动配置
        description: format!("{} ({})", platform.cloud_type, platform.platform_name),
        is_active: platform.status == "active",
    }
}

/// 从机房配置生成物理网络区域
pub fn create_physical_zone(id: i32, room: &MachineRoomConfig) -> NetworkZone {
    NetworkZone {
        id,
        name: format!("{}内网", room.room_name), // 市政务云机房A内网、核心机房内网
        cloud_platform_id: None,
        machine_room_id: Some(room.id),
        cidr_blocks: Vec::new(), // 默认为空，需要手动配置
        ip_ranges: Vec::new(),   // 默认为空，需要手动配置
        description: format!("{} - {}", room.facility_type, room.room_type),
        is_active: room.status == "active",
    }
}

/// 初始化默认网络区域（从云平台和机房数据生成）
pub fn init_network_zones(
    cloud_platforms: &[CloudPlatformConfig],
    machine_rooms: &[MachineRoomConfig],
) -> Vec<NetworkZone> {
    let mut zones = Vec::new();
    let mut id = 1;

    // 1. 从云平台生成云网络区域
    // 注意：使用 foundation 去重，避免重复的"阿里云"、"华为云"等
    let mut seen_foundations = std::collections::HashSet::new();

    for platform in cloud_platforms {
        if !seen_foundations.contains(&platform.foundation) {
            let mut zone = create_cloud_zone(id, platform);
            // 为云平台添加默认示例网段
            match platform.foundation.as_str() {
                "阿里云" => {
                    zone.cidr_blocks = vec!["10.0.0.0/16".to_string(), "172.16.0.0/16".to_string()];
                }
                "华为云" => {
                    zone.cidr_blocks = vec!["192.168.0.0/16".to_string()];
                }
                _ => {}
            }
            zones.push(zone);
            seen_foundations.insert(platform.foundation.clone());
            id += 1;
        }
    }

    // 2. 从机房生成物理网络区域
    for room in machine_rooms {
        let mut zone = create_physical_zone(id, room);
        // 为机房添加默认示例网段（根据机房类型）
        if room.room_type.contains("核心机房") {
            zone.cidr_blocks = vec![format!("{}.0.0.0/16", 100 + room.id)];
        } else if room.room_type.contains("DMZ") {
            zone.ip_ranges = vec![format!("{}.{}.{}.1", 192, room.id, 1)];
        }
        zones.push(zone);
        id += 1;
    }

    zones
}
