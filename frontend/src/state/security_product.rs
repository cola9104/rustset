use serde::{Deserialize, Serialize};

/// 安全产品分类枚举
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum SecurityProductCategory {
    Firewall,   // 防火墙
    Waf,        // Web应用防火墙
    Ips,        // 入侵防御系统
    Ids,        // 入侵检测系统
    AntiDdos,   // 抗DDoS
    Vpn,        // VPN网关
    Bastion,    // 堡垒机
    Siem,       // 安全信息和事件管理
}

impl SecurityProductCategory {
    pub fn display_name(&self) -> &'static str {
        match self {
            SecurityProductCategory::Firewall => "防火墙",
            SecurityProductCategory::Waf => "WAF",
            SecurityProductCategory::Ips => "IPS",
            SecurityProductCategory::Ids => "IDS",
            SecurityProductCategory::AntiDdos => "抗DDoS",
            SecurityProductCategory::Vpn => "VPN网关",
            SecurityProductCategory::Bastion => "堡垒机",
            SecurityProductCategory::Siem => "SIEM",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            SecurityProductCategory::Firewall => "网络边界访问控制",
            SecurityProductCategory::Waf => "Web应用层攻击防护",
            SecurityProductCategory::Ips => "主动入侵防御与阻断",
            SecurityProductCategory::Ids => "入侵行为检测与告警",
            SecurityProductCategory::AntiDdos => "DDoS攻击流量清洗",
            SecurityProductCategory::Vpn => "安全远程访问网关",
            SecurityProductCategory::Bastion => "运维审计与访问控制",
            SecurityProductCategory::Siem => "安全事件收集与分析",
        }
    }

    pub fn all_categories() -> Vec<Self> {
        vec![
            SecurityProductCategory::Firewall,
            SecurityProductCategory::Waf,
            SecurityProductCategory::Ips,
            SecurityProductCategory::Ids,
            SecurityProductCategory::AntiDdos,
            SecurityProductCategory::Vpn,
            SecurityProductCategory::Bastion,
            SecurityProductCategory::Siem,
        ]
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "防火墙" | "Firewall" => SecurityProductCategory::Firewall,
            "WAF" | "Waf" => SecurityProductCategory::Waf,
            "IPS" | "Ips" => SecurityProductCategory::Ips,
            "IDS" | "Ids" => SecurityProductCategory::Ids,
            "抗DDoS" | "AntiDdos" => SecurityProductCategory::AntiDdos,
            "VPN网关" | "Vpn" => SecurityProductCategory::Vpn,
            "堡垒机" | "Bastion" => SecurityProductCategory::Bastion,
            "SIEM" | "Siem" => SecurityProductCategory::Siem,
            _ => SecurityProductCategory::Firewall,
        }
    }
}

/// 安全产品状态
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityProductStatus {
    Active,         // 运行中
    Inactive,       // 已停用
    Maintenance,    // 维护中
    Decommissioned, // 已下线
}

impl SecurityProductStatus {
    pub fn display_name(&self) -> &'static str {
        match self {
            SecurityProductStatus::Active => "运行中",
            SecurityProductStatus::Inactive => "已停用",
            SecurityProductStatus::Maintenance => "维护中",
            SecurityProductStatus::Decommissioned => "已下线",
        }
    }

    pub fn color_class(&self) -> &'static str {
        match self {
            SecurityProductStatus::Active => "bg-green-100 text-green-800",
            SecurityProductStatus::Inactive => "bg-gray-100 text-gray-800",
            SecurityProductStatus::Maintenance => "bg-yellow-100 text-yellow-800",
            SecurityProductStatus::Decommissioned => "bg-red-100 text-red-800",
        }
    }

    pub fn all_statuses() -> Vec<Self> {
        vec![
            SecurityProductStatus::Active,
            SecurityProductStatus::Inactive,
            SecurityProductStatus::Maintenance,
            SecurityProductStatus::Decommissioned,
        ]
    }
}

/// 安全产品数据模型
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SecurityProduct {
    pub id: i32,
    pub name: String,                         // 产品名称
    pub category: SecurityProductCategory,    // 产品分类
    pub vendor: String,                       // 厂商
    pub model: String,                        // 型号
    pub version: String,                      // 版本
    pub serial_number: Option<String>,        // 序列号
    pub license_type: String,                 // 授权类型
    pub license_expiry: Option<String>,       // 授权到期日
    pub management_ip: Option<String>,        // 管理IP
    pub deployment_mode: String,              // 部署模式
    pub cloud_platform_id: Option<i32>,       // 云平台ID
    pub machine_room_id: Option<i32>,         // 机房ID
    pub provider_id: Option<i32>,             // 服务商ID
    pub status: SecurityProductStatus,        // 状态
    pub features: Vec<String>,                // 功能特性
    pub throughput: Option<String>,           // 吞吐量
    pub contact_person: String,               // 负责人
    pub contact_phone: String,                // 联系电话
    pub remarks: Option<String>,              // 备注
    pub created_at: String,
}

impl SecurityProduct {
    pub fn display_name(&self) -> String {
        format!("{} - {} {}", self.vendor, self.model, self.name)
    }

    pub fn short_name(&self) -> String {
        format!("{} ({})", self.name, self.category.display_name())
    }

    /// 是否云部署
    pub fn is_cloud_deployment(&self) -> bool {
        self.cloud_platform_id.is_some()
    }

    /// 是否物理部署
    pub fn is_physical_deployment(&self) -> bool {
        self.machine_room_id.is_some()
    }
}

/// 初始化安全产品数据
pub fn init_security_products() -> Vec<SecurityProduct> {
    vec![
        // 防火墙产品
        SecurityProduct {
            id: 1,
            name: "下一代防火墙-核心".to_string(),
            category: SecurityProductCategory::Firewall,
            vendor: "深信服".to_string(),
            model: "NGAF-8000".to_string(),
            version: "V8.0.5".to_string(),
            serial_number: Some("SXF-NGAF-8000-001".to_string()),
            license_type: "订阅".to_string(),
            license_expiry: Some("2025-12-31".to_string()),
            management_ip: Some("10.1.1.1".to_string()),
            deployment_mode: "路由模式".to_string(),
            cloud_platform_id: None,
            machine_room_id: Some(1),
            provider_id: Some(1),
            status: SecurityProductStatus::Active,
            features: vec!["应用识别".to_string(), "入侵防御".to_string(), "病毒过滤".to_string()],
            throughput: Some("40 Gbps".to_string()),
            contact_person: "张三".to_string(),
            contact_phone: "13800138001".to_string(),
            remarks: Some("核心边界防火墙".to_string()),
            created_at: "2024-01-01".to_string(),
        },
        SecurityProduct {
            id: 2,
            name: "边界防火墙-互联网".to_string(),
            category: SecurityProductCategory::Firewall,
            vendor: "华为".to_string(),
            model: "USG6650".to_string(),
            version: "V500R005C20".to_string(),
            serial_number: Some("HW-USG6650-001".to_string()),
            license_type: "永久".to_string(),
            license_expiry: None,
            management_ip: Some("10.1.1.2".to_string()),
            deployment_mode: "路由模式".to_string(),
            cloud_platform_id: None,
            machine_room_id: Some(2),
            provider_id: Some(1),
            status: SecurityProductStatus::Active,
            features: vec!["NAT".to_string(), "VPN".to_string(), "IPS".to_string()],
            throughput: Some("20 Gbps".to_string()),
            contact_person: "李四".to_string(),
            contact_phone: "13800138002".to_string(),
            remarks: Some("互联网出口防火墙".to_string()),
            created_at: "2024-01-15".to_string(),
        },
        // WAF产品
        SecurityProduct {
            id: 3,
            name: "Web应用防火墙".to_string(),
            category: SecurityProductCategory::Waf,
            vendor: "绿盟".to_string(),
            model: "WAF-6000".to_string(),
            version: "V3.2.1".to_string(),
            serial_number: Some("NS-WAF-6000-001".to_string()),
            license_type: "订阅".to_string(),
            license_expiry: Some("2025-06-30".to_string()),
            management_ip: Some("10.1.2.1".to_string()),
            deployment_mode: "单臂模式".to_string(),
            cloud_platform_id: Some(1),
            machine_room_id: None,
            provider_id: Some(1),
            status: SecurityProductStatus::Active,
            features: vec!["OWASP防护".to_string(), "CC防护".to_string(), "Bot检测".to_string()],
            throughput: Some("10 Gbps".to_string()),
            contact_person: "王五".to_string(),
            contact_phone: "13800138003".to_string(),
            remarks: Some("政务网站WAF防护".to_string()),
            created_at: "2024-02-01".to_string(),
        },
        // IPS产品
        SecurityProduct {
            id: 4,
            name: "入侵防御系统".to_string(),
            category: SecurityProductCategory::Ips,
            vendor: "启明星辰".to_string(),
            model: "NIPS-6000".to_string(),
            version: "V5.0".to_string(),
            serial_number: Some("VEN-NIPS-6000-001".to_string()),
            license_type: "订阅".to_string(),
            license_expiry: Some("2025-09-30".to_string()),
            management_ip: Some("10.1.3.1".to_string()),
            deployment_mode: "桥接模式".to_string(),
            cloud_platform_id: None,
            machine_room_id: Some(1),
            provider_id: Some(1),
            status: SecurityProductStatus::Active,
            features: vec!["实时阻断".to_string(), "漏洞防护".to_string(), "威胁情报".to_string()],
            throughput: Some("15 Gbps".to_string()),
            contact_person: "赵六".to_string(),
            contact_phone: "13800138004".to_string(),
            remarks: None,
            created_at: "2024-02-10".to_string(),
        },
        // IDS产品
        SecurityProduct {
            id: 5,
            name: "入侵检测系统".to_string(),
            category: SecurityProductCategory::Ids,
            vendor: "绿盟".to_string(),
            model: "NIDS-5000".to_string(),
            version: "V4.5".to_string(),
            serial_number: Some("NS-NIDS-5000-001".to_string()),
            license_type: "永久".to_string(),
            license_expiry: None,
            management_ip: Some("10.1.4.1".to_string()),
            deployment_mode: "旁路模式".to_string(),
            cloud_platform_id: None,
            machine_room_id: Some(2),
            provider_id: Some(2),
            status: SecurityProductStatus::Active,
            features: vec!["流量分析".to_string(), "异常检测".to_string(), "日志审计".to_string()],
            throughput: Some("20 Gbps".to_string()),
            contact_person: "孙七".to_string(),
            contact_phone: "13800138005".to_string(),
            remarks: Some("联通区域IDS检测".to_string()),
            created_at: "2024-02-15".to_string(),
        },
        // 抗DDoS产品
        SecurityProduct {
            id: 6,
            name: "抗DDoS设备".to_string(),
            category: SecurityProductCategory::AntiDdos,
            vendor: "深信服".to_string(),
            model: "AD-10000".to_string(),
            version: "V2.1".to_string(),
            serial_number: Some("SXF-AD-10000-001".to_string()),
            license_type: "订阅".to_string(),
            license_expiry: Some("2025-12-31".to_string()),
            management_ip: Some("10.1.5.1".to_string()),
            deployment_mode: "路由模式".to_string(),
            cloud_platform_id: None,
            machine_room_id: Some(1),
            provider_id: Some(1),
            status: SecurityProductStatus::Active,
            features: vec!["流量清洗".to_string(), "CC防护".to_string(), "黑洞引流".to_string()],
            throughput: Some("100 Gbps".to_string()),
            contact_person: "周八".to_string(),
            contact_phone: "13800138006".to_string(),
            remarks: Some("互联网入口DDoS防护".to_string()),
            created_at: "2024-03-01".to_string(),
        },
        // VPN网关产品
        SecurityProduct {
            id: 7,
            name: "SSL VPN网关".to_string(),
            category: SecurityProductCategory::Vpn,
            vendor: "深信服".to_string(),
            model: "SSL VPN-5000".to_string(),
            version: "V7.0".to_string(),
            serial_number: Some("SXF-VPN-5000-001".to_string()),
            license_type: "订阅".to_string(),
            license_expiry: Some("2025-06-30".to_string()),
            management_ip: Some("10.1.6.1".to_string()),
            deployment_mode: "单臂模式".to_string(),
            cloud_platform_id: None,
            machine_room_id: Some(1),
            provider_id: Some(1),
            status: SecurityProductStatus::Active,
            features: vec!["SSL加密".to_string(), "双因素认证".to_string(), "访问控制".to_string()],
            throughput: Some("5 Gbps".to_string()),
            contact_person: "吴九".to_string(),
            contact_phone: "13800138007".to_string(),
            remarks: Some("远程办公VPN接入".to_string()),
            created_at: "2024-03-10".to_string(),
        },
        // 堡垒机产品
        SecurityProduct {
            id: 8,
            name: "运维堡垒机".to_string(),
            category: SecurityProductCategory::Bastion,
            vendor: "启明星辰".to_string(),
            model: "USM-3000".to_string(),
            version: "V3.5".to_string(),
            serial_number: Some("VEN-USM-3000-001".to_string()),
            license_type: "永久".to_string(),
            license_expiry: None,
            management_ip: Some("10.1.7.1".to_string()),
            deployment_mode: "单机部署".to_string(),
            cloud_platform_id: None,
            machine_room_id: Some(1),
            provider_id: Some(1),
            status: SecurityProductStatus::Active,
            features: vec!["账号管理".to_string(), "操作审计".to_string(), "权限控制".to_string()],
            throughput: None,
            contact_person: "郑十".to_string(),
            contact_phone: "13800138008".to_string(),
            remarks: Some("运维审计堡垒机".to_string()),
            created_at: "2024-03-15".to_string(),
        },
        // SIEM产品
        SecurityProduct {
            id: 9,
            name: "安全态势感知平台".to_string(),
            category: SecurityProductCategory::Siem,
            vendor: "奇安信".to_string(),
            model: "NGSOC-5000".to_string(),
            version: "V2.0".to_string(),
            serial_number: Some("QAX-NGSOC-5000-001".to_string()),
            license_type: "订阅".to_string(),
            license_expiry: Some("2025-12-31".to_string()),
            management_ip: Some("10.1.8.1".to_string()),
            deployment_mode: "分布式部署".to_string(),
            cloud_platform_id: Some(1),
            machine_room_id: None,
            provider_id: Some(1),
            status: SecurityProductStatus::Active,
            features: vec!["日志采集".to_string(), "威胁分析".to_string(), "态势大屏".to_string()],
            throughput: Some("100000 EPS".to_string()),
            contact_person: "钱十一".to_string(),
            contact_phone: "13800138009".to_string(),
            remarks: Some("安全运营中心".to_string()),
            created_at: "2024-03-20".to_string(),
        },
    ]
}
