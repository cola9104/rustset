use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum SecurityProductCategory {
    Firewall,
    Waf,
    Ips,
    Ids,
    AntiDdos,
    Vpn,
    Bastion,
    Siem,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityProductStatus {
    Active,
    Inactive,
    Maintenance,
    Decommissioned,
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SecurityProduct {
    pub id: i32,
    pub name: String,
    pub category: SecurityProductCategory,
    pub vendor: String,
    pub model: String,
    pub version: String,
    pub serial_number: Option<String>,
    pub license_type: String,
    pub license_expiry: Option<String>,
    pub management_ip: Option<String>,
    pub deployment_mode: String,
    pub cloud_platform_id: Option<i32>,
    pub machine_room_id: Option<i32>,
    pub provider_id: Option<i32>,
    pub status: SecurityProductStatus,
    pub features: Vec<String>,
    pub throughput: Option<String>,
    pub contact_person: String,
    pub contact_phone: String,
    pub remarks: Option<String>,
    pub created_at: String,
}

impl SecurityProduct {
    pub fn is_cloud_deployment(&self) -> bool {
        self.cloud_platform_id.is_some()
    }

    pub fn is_physical_deployment(&self) -> bool {
        self.machine_room_id.is_some()
    }
}
