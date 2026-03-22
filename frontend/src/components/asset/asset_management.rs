use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaPenToSquare, FaTrash, FaMagnifyingGlass, FaServer, FaDesktop, FaLaptop, FaCloud,
    FaTriangleExclamation, FaCircleCheck, FaShieldHalved, FaLock,
};
use crate::app::PROVIDERS_STATE;
use crate::app::NETWORK_POLICIES_STATE;
use super::port_security::{FirewallPolicySummary, analyze_port_security, PortSecurityStatus, get_security_summary};

/// 从全局网络策略状态获取防火墙策略
fn get_firewall_policies() -> Vec<FirewallPolicySummary> {
    use crate::components::resource_ticket::network_policy::network_policy_request::NetworkPolicyStatus;

    NETWORK_POLICIES_STATE.read()
        .iter()
        .filter(|p| p.status == NetworkPolicyStatus::Active)
        .map(|p| FirewallPolicySummary {
            id: p.id,
            title: p.title.clone(),
            port_range: p.port_range.clone(),
            status: "Active".to_string(),
        })
        .collect()
}

/// 标签页类型
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssetTab {
    Hardware,  // 硬件设备
    Cloud,     // 云服务资产
}

/// 硬件资产数据模型
#[derive(Clone, Debug, PartialEq)]
pub struct HardwareAsset {
    pub id: i32,
    pub name: String,
    pub asset_type: String,
    pub ip_address: String,
    pub ports: String,  // 开放端口，如 "22, 80, 443, 3306"
    pub status: String,
    pub datacenter: String,   // 机房，如 "机房A"
    pub cabinet: String,      // 机柜，如 "机柜2"
    pub u_position: String,   // U位，如 "1U-4U"
    pub u_total: String,      // 总U数，如 "4U"
    pub provider_id: Option<i32>,  // 所属服务商ID
    pub created_at: String,
}

impl HardwareAsset {
    /// 获取服务商名称
    pub fn provider_name(&self) -> String {
        if let Some(pid) = self.provider_id {
            PROVIDERS_STATE.read()
                .iter()
                .find(|p| p.id == pid)
                .map(|p| p.short_name.clone())
                .unwrap_or_else(|| format!("服务商{}", pid))
        } else {
            "未分配".to_string()
        }
    }

    /// 获取服务商颜色
    pub fn provider_color(&self) -> &'static str {
        if let Some(pid) = self.provider_id {
            match pid {
                1 => "bg-blue-100 text-blue-800",  // 电信
                2 => "bg-orange-100 text-orange-800", // 联通
                3 => "bg-green-100 text-green-800", // 移动
                4 => "bg-purple-100 text-purple-800", // 广电
                _ => "bg-gray-100 text-gray-800",
            }
        } else {
            "bg-gray-100 text-gray-600"
        }
    }

    /// 获取自动计算的开放端口（从已生效的网络策略工单派生）
    pub fn computed_ports(&self) -> String {
        use crate::components::resource_ticket::network_policy::network_policy_request::NetworkPolicyStatus;

        let mut port_set = std::collections::HashSet::new();

        // 获取已生效的网络策略
        let policies = NETWORK_POLICIES_STATE.read();
        for policy in policies.iter() {
            if policy.status != NetworkPolicyStatus::Active {
                continue;
            }

            // 匹配逻辑：资产所在机房与策略的源/目标区域匹配
            let asset_location = &self.datacenter;
            if policy.source_zone.contains(asset_location) ||
               policy.destination_zone.contains(asset_location) ||
               asset_location.is_empty() && (policy.source_zone == "全网" || policy.destination_zone == "全网") {
                // 解析端口范围并添加到集合
                for port_str in policy.port_range.split(',') {
                    let port = port_str.trim();
                    if !port.is_empty() {
                        port_set.insert(port.to_string());
                    }
                }
            }
        }

        if port_set.is_empty() {
            "无开放端口".to_string()
        } else {
            let mut ports: Vec<String> = port_set.into_iter().collect();
            ports.sort();
            ports.join(", ")
        }
    }
}

/// 云服务资产数据模型
#[derive(Clone, Debug, PartialEq)]
pub struct CloudAsset {
    pub id: i32,
    pub name: String,                    // 实例名称
    pub provider_id: Option<i32>,        // 所属服务商ID
    pub cloud_type: String,              // 云类型（公有云、政务云）
    pub foundation: String,              // 底座（阿里云、华为云等）
    pub instance_type: String,           // 实例类型
    pub ip_address: String,
    pub ports: String,                   // 开放端口，如 "22, 80, 443"
    pub status: String,
    pub region: String,                  // 区域
    pub machine_room: String,            // 机房
    pub created_at: String,
}

impl CloudAsset {
    /// 获取服务商名称
    pub fn provider_name(&self) -> String {
        if let Some(pid) = self.provider_id {
            PROVIDERS_STATE.read()
                .iter()
                .find(|p| p.id == pid)
                .map(|p| p.short_name.clone())
                .unwrap_or_else(|| format!("服务商{}", pid))
        } else {
            "未分配".to_string()
        }
    }

    /// 获取服务商颜色
    pub fn provider_color(&self) -> &'static str {
        if let Some(pid) = self.provider_id {
            match pid {
                1 => "bg-blue-100 text-blue-800",  // 电信
                2 => "bg-orange-100 text-orange-800", // 联通
                3 => "bg-green-100 text-green-800", // 移动
                4 => "bg-purple-100 text-purple-800", // 广电
                _ => "bg-gray-100 text-gray-800",
            }
        } else {
            "bg-gray-100 text-gray-600"
        }
    }

    /// 获取自动计算的开放端口（从已生效的网络策略工单派生）
    pub fn computed_ports(&self) -> String {
        use crate::components::resource_ticket::network_policy::network_policy_request::NetworkPolicyStatus;

        let mut port_set = std::collections::HashSet::new();

        // 获取已生效的网络策略
        let policies = NETWORK_POLICIES_STATE.read();
        for policy in policies.iter() {
            if policy.status != NetworkPolicyStatus::Active {
                continue;
            }

            // 匹配逻辑：资产所在区域/机房与策略的源/目标区域匹配
            let asset_location = &self.region;
            let asset_machine_room = &self.machine_room;

            if policy.source_zone.contains(asset_location) ||
               policy.destination_zone.contains(asset_location) ||
               policy.source_zone.contains(asset_machine_room) ||
               policy.destination_zone.contains(asset_machine_room) ||
               (asset_location.is_empty() || asset_machine_room.is_empty()) &&
               (policy.source_zone == "全网" || policy.destination_zone == "全网") {
                // 解析端口范围并添加到集合
                for port_str in policy.port_range.split(',') {
                    let port = port_str.trim();
                    if !port.is_empty() {
                        port_set.insert(port.to_string());
                    }
                }
            }
        }

        if port_set.is_empty() {
            "无开放端口".to_string()
        } else {
            let mut ports: Vec<String> = port_set.into_iter().collect();
            ports.sort();
            ports.join(", ")
        }
    }
}

/// 资产管理页面
#[allow(non_snake_case)]
pub fn AssetManagement() -> Element {
    let mut active_tab = use_signal(|| AssetTab::Hardware);

    // 硬件资产数据
    let hardware_assets = use_signal(|| vec![
        HardwareAsset {
            id: 1,
            name: "服务器-01".to_string(),
            asset_type: "服务器".to_string(),
            ip_address: "192.168.1.10".to_string(),
            ports: "22, 80, 443, 3306, 8080".to_string(),
            status: "在线".to_string(),
            datacenter: "机房A".to_string(),
            cabinet: "机柜1".to_string(),
            u_position: "1U-4U".to_string(),
            u_total: "4U".to_string(),
            provider_id: Some(1),  // 电信
            created_at: "2024-01-15".to_string(),
        },
        HardwareAsset {
            id: 2,
            name: "服务器-02".to_string(),
            asset_type: "服务器".to_string(),
            ip_address: "192.168.1.11".to_string(),
            ports: "22, 80, 443".to_string(),
            status: "在线".to_string(),
            datacenter: "机房A".to_string(),
            cabinet: "机柜2".to_string(),
            u_position: "5U-8U".to_string(),
            u_total: "4U".to_string(),
            provider_id: Some(2),  // 联通
            created_at: "2024-01-16".to_string(),
        },
        HardwareAsset {
            id: 3,
            name: "工作站-01".to_string(),
            asset_type: "工作站".to_string(),
            ip_address: "192.168.2.100".to_string(),
            ports: "22, 3389".to_string(),
            status: "离线".to_string(),
            datacenter: "".to_string(),
            cabinet: "".to_string(),
            u_position: "".to_string(),
            u_total: "".to_string(),
            provider_id: None,  // 未分配
            created_at: "2024-01-20".to_string(),
        },
    ]);

    // 云服务资产数据
    let cloud_assets = use_signal(|| vec![
        CloudAsset {
            id: 1,
            name: "web-server-01".to_string(),
            provider_id: Some(1),  // 电信
            cloud_type: "公有云".to_string(),
            foundation: "阿里云".to_string(),
            instance_type: "ecs.g6.large".to_string(),
            ip_address: "47.96.123.45".to_string(),
            ports: "22, 80, 443".to_string(),
            status: "运行中".to_string(),
            region: "华东1-杭州".to_string(),
            machine_room: "市政务云机房A".to_string(),
            created_at: "2024-01-10".to_string(),
        },
        CloudAsset {
            id: 2,
            name: "api-server-01".to_string(),
            provider_id: Some(2),  // 联通
            cloud_type: "公有云".to_string(),
            foundation: "华为云".to_string(),
            instance_type: "S5.MEDIUM4".to_string(),
            ip_address: "119.29.67.89".to_string(),
            ports: "22, 8080".to_string(),
            status: "运行中".to_string(),
            region: "广州".to_string(),
            machine_room: "联通核心机房".to_string(),
            created_at: "2024-01-12".to_string(),
        },
        CloudAsset {
            id: 3,
            name: "db-server-01".to_string(),
            provider_id: Some(1),  // 电信
            cloud_type: "政务云".to_string(),
            foundation: "华为云".to_string(),
            instance_type: "s6.xlarge.4".to_string(),
            ip_address: "119.8.123.234".to_string(),
            ports: "22, 3306".to_string(),
            status: "已停止".to_string(),
            region: "华北区-北京4".to_string(),
            machine_room: "核心机房(电信)".to_string(),
            created_at: "2024-01-18".to_string(),
        },
        CloudAsset {
            id: 4,
            name: "app-server-02".to_string(),
            provider_id: Some(2),  // 联通
            cloud_type: "政务云".to_string(),
            foundation: "阿里云".to_string(),
            instance_type: "ecs.g7.xlarge".to_string(),
            ip_address: "47.97.234.56".to_string(),
            ports: "22, 80, 443, 8080".to_string(),
            status: "运行中".to_string(),
            region: "华东1-杭州".to_string(),
            machine_room: "市政务云机房B".to_string(),
            created_at: "2024-01-20".to_string(),
        },
        CloudAsset {
            id: 5,
            name: "cache-server-01".to_string(),
            provider_id: Some(3),  // 移动
            cloud_type: "公有云".to_string(),
            foundation: "阿里云".to_string(),
            instance_type: "s6.large.2".to_string(),
            ip_address: "119.9.45.123".to_string(),
            ports: "22, 6379".to_string(),
            status: "运行中".to_string(),
            region: "西南区-成都1".to_string(),
            machine_room: "市政务云机房C".to_string(),
            created_at: "2024-01-22".to_string(),
        },
        CloudAsset {
            id: 6,
            name: "file-server-01".to_string(),
            provider_id: Some(3),  // 移动
            cloud_type: "公有云".to_string(),
            foundation: "阿里云".to_string(),
            instance_type: "ecs.g6.2xlarge".to_string(),
            ip_address: "47.98.156.78".to_string(),
            ports: "22, 80, 443, 2049".to_string(),
            status: "运行中".to_string(),
            region: "华北2-北京".to_string(),
            machine_room: "移动核心机房".to_string(),
            created_at: "2024-01-25".to_string(),
        },
    ]);

    let search_query = use_signal(String::new);

    let is_hardware = *active_tab.read() == AssetTab::Hardware;

    rsx! {
        div { class: "space-y-6",
            // 页面标题
            div { class: "flex justify-between items-center",
                h1 { class: "text-2xl font-bold text-gray-800", "资产管理" }
            }

            // 标签页
            div { class: "bg-white rounded-lg shadow p-1 inline-flex",
                div { class: "flex space-x-1",
                    button {
                        class: format!(
                            "flex items-center px-6 py-2.5 rounded-md text-sm font-medium transition-all duration-200 {}",
                            if is_hardware {
                                "bg-blue-600 text-white shadow-md"
                            } else {
                                "text-gray-600 hover:bg-gray-100"
                            }
                        ),
                        onclick: move |_| active_tab.set(AssetTab::Hardware),
                        Icon { icon: FaServer, width: 16, height: 16, class: "mr-2" }
                        "硬件设备"
                    }
                    button {
                        class: format!(
                            "flex items-center px-6 py-2.5 rounded-md text-sm font-medium transition-all duration-200 {}",
                            if !is_hardware {
                                "bg-blue-600 text-white shadow-md"
                            } else {
                                "text-gray-600 hover:bg-gray-100"
                            }
                        ),
                        onclick: move |_| active_tab.set(AssetTab::Cloud),
                        Icon { icon: FaCloud, width: 16, height: 16, class: "mr-2" }
                        "云服务资产"
                    }
                }
            }

            // 标签页内容
            if is_hardware {
                HardwareAssetsTab {
                    assets: hardware_assets,
                    search_query: search_query
                }
            } else {
                CloudAssetsTab {
                    assets: cloud_assets,
                    search_query: search_query
                }
            }
        }
    }
}

/// 硬件设备标签页
#[component]
fn HardwareAssetsTab(
    assets: Signal<Vec<HardwareAsset>>,
    search_query: Signal<String>
) -> Element {
    let mut editing_asset = use_signal(|| None::<HardwareAsset>);

    // 计算统计数据
    let total_count = assets.read().len() as i32;
    let server_count = assets.read().iter().filter(|a| a.asset_type == "服务器").count() as i32;
    let workstation_count = assets.read().iter().filter(|a| a.asset_type == "工作站").count() as i32;
    let online_count = assets.read().iter().filter(|a| a.status == "在线").count() as i32;

    // 过滤资产
    let filtered_assets: Vec<HardwareAsset> = assets.read()
        .iter()
        .filter(|asset| {
            let query = search_query.read().to_lowercase();
            query.is_empty() ||
            asset.name.to_lowercase().contains(&query) ||
            asset.ip_address.contains(&query)
        })
        .cloned()
        .collect();

    let is_empty = filtered_assets.is_empty();

    // 计算端口安全统计
    let policies = get_firewall_policies();
    let mut total_ports = 0;
    let mut protected_ports = 0;
    let mut unprotected_ports = 0;

    for asset in assets.read().iter() {
        let computed_ports = asset.computed_ports();
        let port_security = analyze_port_security(&computed_ports, &policies);
        for info in port_security.iter() {
            // 计算端口数量（单个端口或端口范围）
            let port_count = if info.port.contains('-') {
                let parts: Vec<&str> = info.port.split('-').collect();
                if parts.len() == 2 {
                    parts[0].trim().parse::<u16>().ok();
                    parts[1].trim().parse::<u16>().ok();
                    // 简化计算，端口范围至少算1个端口
                    1
                } else {
                    1
                }
            } else {
                1
            };

            total_ports += port_count;

            match info.status {
                PortSecurityStatus::Protected => protected_ports += port_count,
                PortSecurityStatus::Unprotected => unprotected_ports += port_count,
                PortSecurityStatus::Partial => {
                    protected_ports += 1;
                    unprotected_ports += port_count - 1;
                }
            }
        }
    }

    let coverage_rate = if total_ports > 0 {
        (protected_ports as f32 / total_ports as f32 * 100.0) as i32
    } else {
        100
    };

    rsx! {
        // 搜索栏
        div { class: "bg-white rounded-lg shadow p-4",
            div { class: "flex items-center",
                Icon { icon: FaMagnifyingGlass, width: 20, height: 20 }
                input {
                    r#type: "text",
                    class: "ml-2 flex-1 border-0 focus:outline-none",
                    placeholder: "搜索资产名称或IP地址...",
                    value: search_query,
                    oninput: move |e| search_query.set(e.value()),
                }
            }
        }

        // 统计卡片
        div { class: "grid grid-cols-1 md:grid-cols-3 lg:grid-cols-6 gap-4",
            div { class: "bg-white rounded-lg shadow p-4",
                div { class: "flex items-center",
                    div { class: "p-2 rounded-full bg-blue-500",
                        Icon { icon: FaServer, width: 20, height: 20 }
                    }
                    div { class: "ml-3",
                        p { class: "text-sm text-gray-500", "总资产" }
                        p { class: "text-xl font-bold text-gray-800", {total_count.to_string()} }
                    }
                }
            }
            div { class: "bg-white rounded-lg shadow p-4",
                div { class: "flex items-center",
                    div { class: "p-2 rounded-full bg-green-500",
                        Icon { icon: FaDesktop, width: 20, height: 20 }
                    }
                    div { class: "ml-3",
                        p { class: "text-sm text-gray-500", "服务器" }
                        p { class: "text-xl font-bold text-gray-800", {server_count.to_string()} }
                    }
                }
            }
            div { class: "bg-white rounded-lg shadow p-4",
                div { class: "flex items-center",
                    div { class: "p-2 rounded-full bg-green-600",
                        Icon { icon: FaServer, width: 20, height: 20 }
                    }
                    div { class: "ml-3",
                        p { class: "text-sm text-gray-500", "在线" }
                        p { class: "text-xl font-bold text-gray-800", {online_count.to_string()} }
                    }
                }
            }
            div { class: "bg-white rounded-lg shadow p-4",
                div { class: "flex items-center",
                    div { class: "p-2 rounded-full bg-purple-500",
                        Icon { icon: FaShieldHalved, width: 20, height: 20 }
                    }
                    div { class: "ml-3",
                        p { class: "text-sm text-gray-500", "总端口" }
                        p { class: "text-xl font-bold text-gray-800", {total_ports.to_string()} }
                    }
                }
            }
            div { class: "bg-white rounded-lg shadow p-4",
                div { class: "flex items-center",
                    div { class: "p-2 rounded-full bg-green-500",
                        Icon { icon: FaCircleCheck, width: 20, height: 20 }
                    }
                    div { class: "ml-3",
                        p { class: "text-sm text-gray-500", "已保护" }
                        p { class: "text-xl font-bold text-green-600", {protected_ports.to_string()} }
                    }
                }
            }
            div { class: "bg-white rounded-lg shadow p-4",
                div { class: "flex items-center",
                    div { class: "p-2 rounded-full bg-red-500",
                        Icon { icon: FaTriangleExclamation, width: 20, height: 20 }
                    }
                    div { class: "ml-3",
                        p { class: "text-sm text-gray-500", "未保护" }
                        p { class: "text-xl font-bold text-red-600", {unprotected_ports.to_string()} }
                    }
                }
            }
        }

        // 防火墙覆盖率指示器
        div { class: "bg-white rounded-lg shadow p-4",
            div { class: "flex items-center justify-between",
                div { class: "flex items-center",
                    Icon { icon: FaShieldHalved, width: 20, height: 20, class: "text-purple-600 mr-3" }
                    div {
                        p { class: "text-sm font-medium text-gray-700", "防火墙策略覆盖率" }
                        p { class: "text-xs text-gray-500", "基于已生效的网络策略工单" }
                    }
                }
                div { class: "text-right",
                    p { class: "text-2xl font-bold text-gray-800", "{coverage_rate}%" }
                    p { class: "text-xs text-gray-500",
                        if coverage_rate >= 90 {
                            "安全状态良好"
                        } else if coverage_rate >= 70 {
                            "存在安全风险"
                        } else {
                            "安全风险较高"
                        }
                    }
                }
            }
            // 进度条
            div { class: "mt-3",
                div { class: "w-full bg-gray-200 rounded-full h-2",
                    div {
                        class: format!(
                            "h-2 rounded-full {}",
                            if coverage_rate >= 90 { "bg-green-500" }
                            else if coverage_rate >= 70 { "bg-yellow-500" }
                            else { "bg-red-500" }
                        ),
                        style: "width: {coverage_rate}%"
                    }
                }
            }
        }

        // 资产列表表格
        div { class: "bg-white rounded-lg shadow overflow-hidden",
            table { class: "min-w-full divide-y divide-gray-200",
                thead { class: "bg-gray-50",
                    tr {
                        th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "名称" }
                        th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "类型" }
                        th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "服务商" }
                        th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "IP地址" }
                        th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "开放端口" }
                        th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "状态" }
                        th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "机房/机柜" }
                        th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "U位" }
                        th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "操作" }
                    }
                }
                tbody { class: "bg-white divide-y divide-gray-200",
                    for asset in filtered_assets {
                        tr { class: "hover:bg-gray-50",
                            td { class: "px-6 py-4 whitespace-nowrap",
                                div { class: "flex items-center",
                                    div { class: "flex-shrink-0 h-10 w-10 bg-gray-100 rounded-full flex items-center justify-center",
                                        Icon { icon: FaServer, width: 20, height: 20 }
                                    }
                                    div { class: "ml-4",
                                        div { class: "text-sm font-medium text-gray-900", {asset.name.clone()} }
                                    }
                                }
                            }
                            td { class: "px-6 py-4 whitespace-nowrap",
                                span { class: "px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-blue-100 text-blue-800",
                                    {asset.asset_type.clone()}
                                }
                            }
                            td { class: "px-6 py-4 whitespace-nowrap",
                                span {
                                    class: "px-2 inline-flex text-xs leading-5 font-semibold rounded-full {asset.provider_color()}",
                                    {asset.provider_name()}
                                }
                            }
                            td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-500",
                                {asset.ip_address.clone()}
                            }
                            td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-500",
                                div { class: "flex items-center gap-1",
                                    Icon { icon: FaLock, width: 12, height: 12, class: "text-gray-400" }
                                    span { class: "text-xs font-mono bg-gray-100 px-2 py-1 rounded",
                                        {
                                            let computed = asset.computed_ports();
                                            computed
                                        }
                                    }
                                }
                            }
                            td { class: "px-6 py-4 whitespace-nowrap",
                                span {
                                    class: if asset.status == "在线" {
                                        "px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-green-100 text-green-800"
                                    } else {
                                        "px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-red-100 text-red-800"
                                    },
                                    {asset.status.clone()}
                                }
                            }
                            td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-500",
                                if asset.datacenter.is_empty() && asset.cabinet.is_empty() {
                                    "-"
                                } else {
                                    div {
                                        if !asset.datacenter.is_empty() {
                                            div { {asset.datacenter.clone()} }
                                        }
                                        if !asset.cabinet.is_empty() {
                                            div { class: "text-xs text-gray-400", {asset.cabinet.clone()} }
                                        }
                                    }
                                }
                            }
                            td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-500",
                                if asset.u_position.is_empty() && asset.u_total.is_empty() {
                                    "-"
                                } else {
                                    div { class: "flex items-center space-x-2",
                                        if !asset.u_position.is_empty() {
                                            span { class: "font-mono text-xs bg-gray-100 px-2 py-1 rounded", {asset.u_position.clone()} }
                                        }
                                        if !asset.u_total.is_empty() {
                                            span { class: "text-xs text-gray-400", {asset.u_total.clone()} }
                                        }
                                    }
                                }
                            }
                            td { class: "px-6 py-4 whitespace-nowrap text-sm font-medium",
                                button {
                                    class: "text-blue-600 hover:text-blue-900 mr-3",
                                    onclick: {
                                        let asset = asset.clone();
                                        move |_| editing_asset.set(Some(asset.clone()))
                                    },
                                    Icon { icon: FaPenToSquare, width: 16, height: 16 }
                                }
                                button {
                                    class: "text-red-600 hover:text-red-900",
                                    onclick: {
                                        let asset_id = asset.id;
                                        move |_| {
                                            let mut list = assets.write();
                                            list.retain(|a| a.id != asset_id);
                                        }
                                    },
                                    Icon { icon: FaTrash, width: 16, height: 16 }
                                }
                            }
                        }
                    }
                }

                if is_empty {
                    div { class: "text-center py-12 text-gray-500",
                        "没有找到匹配的资产"
                    }
                }
            }
        }

        // 编辑硬件资产模态框
        if let Some(asset) = editing_asset.read().clone() {
            EditHardwareAssetModal {
                asset: asset.clone(),
                on_close: move |_| editing_asset.set(None),
                on_save: {
                    let mut editing_asset_signal = editing_asset;
                    move |updated: HardwareAsset| {
                        let mut list = assets.write();
                        if let Some(a) = list.iter_mut().find(|a| a.id == updated.id) {
                            *a = updated;
                        }
                        editing_asset_signal.set(None);
                    }
                }
            }
        }
    }
}

/// 云服务资产标签页
#[component]
fn CloudAssetsTab(
    assets: Signal<Vec<CloudAsset>>,
    search_query: Signal<String>
) -> Element {
    let mut editing_asset = use_signal(|| None::<CloudAsset>);

    // 计算统计数据
    let total_count = assets.read().len() as i32;
    let provider_1_count = assets.read().iter().filter(|a| a.provider_id == Some(1)).count() as i32;  // 电信
    let provider_2_count = assets.read().iter().filter(|a| a.provider_id == Some(2)).count() as i32;  // 联通
    let running_count = assets.read().iter().filter(|a| a.status == "运行中").count() as i32;

    // 计算云资产端口安全统计
    let policies = get_firewall_policies();
    let mut total_ports = 0;
    let mut protected_ports = 0;
    let mut unprotected_ports = 0;

    for asset in assets.read().iter() {
        let computed_ports = asset.computed_ports();
        let port_security = analyze_port_security(&computed_ports, &policies);
        for info in port_security.iter() {
            let port_count = 1; // 简化计算
            total_ports += port_count;

            match info.status {
                PortSecurityStatus::Protected => protected_ports += port_count,
                PortSecurityStatus::Unprotected => unprotected_ports += port_count,
                PortSecurityStatus::Partial => {
                    protected_ports += 1;
                    unprotected_ports += port_count - 1;
                }
            }
        }
    }

    let coverage_rate = if total_ports > 0 {
        (protected_ports as f32 / total_ports as f32 * 100.0) as i32
    } else {
        100
    };

    // 过滤资产
    let filtered_assets: Vec<CloudAsset> = assets.read()
        .iter()
        .filter(|asset| {
            let query = search_query.read().to_lowercase();
            query.is_empty() ||
            asset.name.to_lowercase().contains(&query) ||
            asset.ip_address.contains(&query)
        })
        .cloned()
        .collect();

    let is_empty = filtered_assets.is_empty();

    rsx! {
        // 搜索栏
        div { class: "bg-white rounded-lg shadow p-4",
            div { class: "flex items-center",
                Icon { icon: FaMagnifyingGlass, width: 20, height: 20 }
                input {
                    r#type: "text",
                    class: "ml-2 flex-1 border-0 focus:outline-none",
                    placeholder: "搜索实例名称或IP地址...",
                    value: search_query,
                    oninput: move |e| search_query.set(e.value()),
                }
            }
        }

        // 统计卡片
        div { class: "grid grid-cols-1 md:grid-cols-3 lg:grid-cols-6 gap-4",
            div { class: "bg-white rounded-lg shadow p-4",
                div { class: "flex items-center",
                    div { class: "p-2 rounded-full bg-blue-500",
                        Icon { icon: FaCloud, width: 20, height: 20 }
                    }
                    div { class: "ml-3",
                        p { class: "text-sm text-gray-500", "总实例" }
                        p { class: "text-xl font-bold text-gray-800", {total_count.to_string()} }
                    }
                }
            }
            div { class: "bg-white rounded-lg shadow p-4",
                div { class: "flex items-center",
                    div { class: "p-2 rounded-full bg-orange-500",
                        Icon { icon: FaCloud, width: 20, height: 20 }
                    }
                    div { class: "ml-3",
                        p { class: "text-sm text-gray-500", "电信云" }
                        p { class: "text-xl font-bold text-gray-800", {provider_1_count.to_string()} }
                    }
                }
            }
            div { class: "bg-white rounded-lg shadow p-4",
                div { class: "flex items-center",
                    div { class: "p-2 rounded-full bg-blue-600",
                        Icon { icon: FaCloud, width: 20, height: 20 }
                    }
                    div { class: "ml-3",
                        p { class: "text-sm text-gray-500", "联通云" }
                        p { class: "text-xl font-bold text-gray-800", {provider_2_count.to_string()} }
                    }
                }
            }
            div { class: "bg-white rounded-lg shadow p-4",
                div { class: "flex items-center",
                    div { class: "p-2 rounded-full bg-green-600",
                        Icon { icon: FaServer, width: 20, height: 20 }
                    }
                    div { class: "ml-3",
                        p { class: "text-sm text-gray-500", "运行中" }
                        p { class: "text-xl font-bold text-gray-800", {running_count.to_string()} }
                    }
                }
            }
            div { class: "bg-white rounded-lg shadow p-4",
                div { class: "flex items-center",
                    div { class: "p-2 rounded-full bg-purple-500",
                        Icon { icon: FaShieldHalved, width: 20, height: 20 }
                    }
                    div { class: "ml-3",
                        p { class: "text-sm text-gray-500", "总端口" }
                        p { class: "text-xl font-bold text-gray-800", {total_ports.to_string()} }
                    }
                }
            }
            div { class: "bg-white rounded-lg shadow p-4",
                div { class: "flex items-center",
                    div { class: "p-2 rounded-full bg-red-500",
                        Icon { icon: FaTriangleExclamation, width: 20, height: 20 }
                    }
                    div { class: "ml-3",
                        p { class: "text-sm text-gray-500", "未保护" }
                        p { class: "text-xl font-bold text-red-600", {unprotected_ports.to_string()} }
                    }
                }
            }
        }

        // 防火墙覆盖率指示器
        div { class: "bg-white rounded-lg shadow p-4",
            div { class: "flex items-center justify-between",
                div { class: "flex items-center",
                    Icon { icon: FaShieldHalved, width: 20, height: 20, class: "text-purple-600 mr-3" }
                    div {
                        p { class: "text-sm font-medium text-gray-700", "云资产防火墙覆盖率" }
                        p { class: "text-xs text-gray-500", "基于已生效的网络策略工单" }
                    }
                }
                div { class: "text-right",
                    p { class: "text-2xl font-bold text-gray-800", "{coverage_rate}%" }
                    p { class: "text-xs text-gray-500",
                        if coverage_rate >= 90 {
                            "安全状态良好"
                        } else if coverage_rate >= 70 {
                            "存在安全风险"
                        } else {
                            "安全风险较高"
                        }
                    }
                }
            }
            // 进度条
            div { class: "mt-3",
                div { class: "w-full bg-gray-200 rounded-full h-2",
                    div {
                        class: format!(
                            "h-2 rounded-full {}",
                            if coverage_rate >= 90 { "bg-green-500" }
                            else if coverage_rate >= 70 { "bg-yellow-500" }
                            else { "bg-red-500" }
                        ),
                        style: "width: {coverage_rate}%"
                    }
                }
            }
        }

        // 资产列表表格
        div { class: "bg-white rounded-lg shadow overflow-hidden",
            table { class: "min-w-full divide-y divide-gray-200",
                thead { class: "bg-gray-50",
                    tr {
                        th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "实例名称" }
                        th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "服务商" }
                        th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "云类型" }
                        th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "底座" }
                        th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "实例类型" }
                        th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "IP地址" }
                        th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "开放端口" }
                        th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "防火墙状态" }
                        th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "状态" }
                        th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "区域/机房" }
                        th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "操作" }
                    }
                }
                tbody { class: "bg-white divide-y divide-gray-200",
                    for asset in filtered_assets {
                        tr { class: "hover:bg-gray-50",
                            td { class: "px-6 py-4 whitespace-nowrap",
                                div { class: "flex items-center",
                                    div { class: "flex-shrink-0 h-10 w-10 bg-blue-50 rounded-full flex items-center justify-center",
                                        Icon { icon: FaCloud, width: 20, height: 20, class: "text-blue-500" }
                                    }
                                    div { class: "ml-4",
                                        div { class: "text-sm font-medium text-gray-900", {asset.name.clone()} }
                                    }
                                }
                            }
                            td { class: "px-6 py-4 whitespace-nowrap",
                                span {
                                    class: "px-2 inline-flex text-xs leading-5 font-semibold rounded-full {asset.provider_color()}",
                                    {asset.provider_name()}
                                }
                            }
                            td { class: "px-6 py-4 whitespace-nowrap",
                                span { class: "px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-blue-100 text-blue-800",
                                    {asset.cloud_type.clone()}
                                }
                            }
                            td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-500",
                                {asset.foundation.clone()}
                            }
                            td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-500",
                                {asset.instance_type.clone()}
                            }
                            td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-500",
                                {asset.ip_address.clone()}
                            }
                            td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-500",
                                div { class: "flex items-center gap-1",
                                    Icon { icon: FaLock, width: 12, height: 12, class: "text-gray-400" }
                                    span { class: "text-xs font-mono bg-gray-100 px-2 py-1 rounded",
                                        {
                                            let computed = asset.computed_ports();
                                            computed
                                        }
                                    }
                                }
                            }
                            td { class: "px-6 py-4 whitespace-nowrap",
                                span {
                                    class: if asset.status == "运行中" {
                                        "px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-green-100 text-green-800"
                                    } else {
                                        "px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-gray-100 text-gray-800"
                                    },
                                    {asset.status.clone()}
                                }
                            }
                            td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-500",
                                div {
                                    div { {asset.region.clone()} }
                                    div { class: "text-xs text-gray-400", {asset.machine_room.clone()} }
                                }
                            }
                            td { class: "px-6 py-4 whitespace-nowrap text-sm font-medium",
                                button {
                                    class: "text-blue-600 hover:text-blue-900 mr-3",
                                    onclick: {
                                        let asset = asset.clone();
                                        move |_| editing_asset.set(Some(asset.clone()))
                                    },
                                    Icon { icon: FaPenToSquare, width: 16, height: 16 }
                                }
                                button {
                                    class: "text-red-600 hover:text-red-900",
                                    onclick: {
                                        let asset_id = asset.id;
                                        move |_| {
                                            let mut list = assets.write();
                                            list.retain(|a| a.id != asset_id);
                                        }
                                    },
                                    Icon { icon: FaTrash, width: 16, height: 16 }
                                }
                            }
                        }
                    }
                }

                if is_empty {
                    div { class: "text-center py-12 text-gray-500",
                        "没有找到匹配的云服务资产"
                    }
                }
            }
        }

        // 编辑云服务资产模态框
        if let Some(asset) = editing_asset.read().clone() {
            EditCloudAssetModal {
                asset: asset.clone(),
                on_close: move |_| editing_asset.set(None),
                on_save: {
                    let mut editing_asset_signal = editing_asset;
                    move |updated: CloudAsset| {
                        let mut list = assets.write();
                        if let Some(a) = list.iter_mut().find(|a| a.id == updated.id) {
                            *a = updated;
                        }
                        editing_asset_signal.set(None);
                    }
                }
            }
        }
    }
}

/// 编辑硬件资产模态框
#[component]
fn EditHardwareAssetModal(asset: HardwareAsset, on_close: EventHandler<()>, on_save: EventHandler<HardwareAsset>) -> Element {
    let mut name = use_signal(|| asset.name.clone());
    let mut asset_type = use_signal(|| asset.asset_type.clone());
    let mut ip_address = use_signal(|| asset.ip_address.clone());
    let mut status = use_signal(|| asset.status.clone());
    let mut datacenter = use_signal(|| asset.datacenter.clone());
    let mut cabinet = use_signal(|| asset.cabinet.clone());
    let mut u_position = use_signal(|| asset.u_position.clone());
    let mut u_total = use_signal(|| asset.u_total.clone());

    rsx! {
        div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            div { class: "bg-white rounded-lg shadow-xl w-full max-w-md mx-4",
                div { class: "flex justify-between items-center p-4 border-b",
                    h3 { class: "text-lg font-semibold", "编辑硬件资产" }
                    button {
                        class: "text-gray-400 hover:text-gray-600",
                        onclick: move |_| on_close.call(()),
                        "×"
                    }
                }

                div { class: "p-4 space-y-4",
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "名称" }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: name,
                            oninput: move |e| name.set(e.value()),
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "类型" }
                        select {
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: asset_type,
                            onchange: move |e| asset_type.set(e.value()),
                            option { value: "服务器", "服务器" }
                            option { value: "工作站", "工作站" }
                            option { value: "网络设备", "网络设备" }
                            option { value: "存储设备", "存储设备" }
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "IP地址" }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: ip_address,
                            oninput: move |e| ip_address.set(e.value()),
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1",
                            Icon { icon: FaLock, width: 14, height: 14, class: "mr-1 text-gray-400" }
                            "开放端口（自动派生）"
                        }
                        div {
                            class: "w-full px-3 py-2 bg-gray-50 border border-gray-200 rounded-md text-gray-600 text-sm",
                            style: "min-height: 40px;",
                            {
                                let computed = asset.computed_ports();
                                computed
                            }
                        }
                        p { class: "text-xs text-blue-500 mt-1",
                            "由已生效的网络策略工单自动计算，不可手动编辑"
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "状态" }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: name,
                            oninput: move |e| name.set(e.value()),
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "类型" }
                        select {
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: asset_type,
                            onchange: move |e| asset_type.set(e.value()),
                            option { value: "服务器", "服务器" }
                            option { value: "工作站", "工作站" }
                            option { value: "网络设备", "网络设备" }
                            option { value: "存储设备", "存储设备" }
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "IP地址" }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: ip_address,
                            oninput: move |e| ip_address.set(e.value()),
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1",
                            Icon { icon: FaLock, width: 14, height: 14, class: "mr-1 text-gray-400" }
                            "开放端口（自动派生）"
                        }
                        div {
                            class: "w-full px-3 py-2 bg-gray-50 border border-gray-200 rounded-md text-gray-600 text-sm",
                            style: "min-height: 40px;",
                            {
                                let computed = asset.computed_ports();
                                computed
                            }
                        }
                        p { class: "text-xs text-blue-500 mt-1",
                            "由已生效的网络策略工单自动计算，不可手动编辑"
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "状态" }
                        select {
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: status,
                            onchange: move |e| status.set(e.value()),
                            option { value: "在线", "在线" }
                            option { value: "离线", "离线" }
                        }
                    }

                    div { class: "border-t pt-4 mt-4",
                        h4 { class: "text-sm font-medium text-gray-700 mb-3", "位置信息" }
                        div { class: "grid grid-cols-2 gap-3",
                            div {
                                label { class: "block text-sm font-medium text-gray-700 mb-1", "机房" }
                                input {
                                    r#type: "text",
                                    class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                    placeholder: "如：机房A",
                                    value: datacenter,
                                    oninput: move |e| datacenter.set(e.value()),
                                }
                            }
                            div {
                                label { class: "block text-sm font-medium text-gray-700 mb-1", "机柜" }
                                input {
                                    r#type: "text",
                                    class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                    placeholder: "如：机柜2",
                                    value: cabinet,
                                    oninput: move |e| cabinet.set(e.value()),
                                }
                            }
                            div {
                                label { class: "block text-sm font-medium text-gray-700 mb-1", "U位" }
                                input {
                                    r#type: "text",
                                    class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                    placeholder: "如：1U-4U",
                                    value: u_position,
                                    oninput: move |e| u_position.set(e.value()),
                                }
                            }
                            div {
                                label { class: "block text-sm font-medium text-gray-700 mb-1", "总U数" }
                                input {
                                    r#type: "text",
                                    class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                    placeholder: "如：4U",
                                    value: u_total,
                                    oninput: move |e| u_total.set(e.value()),
                                }
                            }
                        }
                    }
                }

                div { class: "flex justify-end space-x-3 p-4 border-t",
                    button {
                        class: "px-4 py-2 text-gray-600 border border-gray-300 rounded-md hover:bg-gray-50",
                        onclick: move |_| on_close.call(()),
                        "取消"
                    }
                    button {
                        class: "px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700",
                        onclick: {
                            let asset_id = asset.id;
                            let created_at = asset.created_at.clone();
                            let provider_id = asset.provider_id;
                            let original_ports = asset.ports.clone(); // Keep original ports (computed dynamically on display)
                            move |_| {
                                let updated = HardwareAsset {
                                    id: asset_id,
                                    name: name.read().clone(),
                                    asset_type: asset_type.read().clone(),
                                    ip_address: ip_address.read().clone(),
                                    ports: original_ports.clone(),
                                    status: status.read().clone(),
                                    datacenter: datacenter.read().clone(),
                                    cabinet: cabinet.read().clone(),
                                    u_position: u_position.read().clone(),
                                    u_total: u_total.read().clone(),
                                    provider_id,
                                    created_at: created_at.clone(),
                                };
                                on_save.call(updated);
                            }
                        },
                        "保存"
                    }
                }
            }
        }
    }
}

/// 编辑云服务资产模态框
#[component]
fn EditCloudAssetModal(asset: CloudAsset, on_close: EventHandler<()>, on_save: EventHandler<CloudAsset>) -> Element {
    let mut name = use_signal(|| asset.name.clone());
    let mut provider_id = use_signal(|| asset.provider_id);
    let mut cloud_type = use_signal(|| asset.cloud_type.clone());
    let mut foundation = use_signal(|| asset.foundation.clone());
    let mut instance_type = use_signal(|| asset.instance_type.clone());
    let mut ip_address = use_signal(|| asset.ip_address.clone());
    let mut status = use_signal(|| asset.status.clone());
    let mut region = use_signal(|| asset.region.clone());
    let mut machine_room = use_signal(|| asset.machine_room.clone());
    let providers = PROVIDERS_STATE.read().clone();

    rsx! {
        div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            div { class: "bg-white rounded-lg shadow-xl w-full max-w-md mx-4",
                div { class: "flex justify-between items-center p-4 border-b",
                    h3 { class: "text-lg font-semibold", "编辑云服务资产" }
                    button {
                        class: "text-gray-400 hover:text-gray-600",
                        onclick: move |_| on_close.call(()),
                        "×"
                    }
                }

                div { class: "p-4 space-y-4",
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "实例名称" }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: name,
                            oninput: move |e| name.set(e.value()),
                        }
                    }

                    div { class: "grid grid-cols-2 gap-3",
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "服务商" }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: "{provider_id.read().unwrap_or(-1)}",
                                onchange: move |e| {
                                    let val: i32 = e.value().parse().unwrap_or(-1);
                                    provider_id.set(if val > 0 { Some(val) } else { None });
                                },
                                option { value: "-1", "未分配" }
                                for provider in providers.iter() {
                                    option {
                                        value: "{provider.id}",
                                        selected: *provider_id.read() == Some(provider.id),
                                        "{provider.short_name}"
                                    }
                                }
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "云类型" }
                            select {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: cloud_type,
                                onchange: move |e| cloud_type.set(e.value()),
                                option { value: "公有云", "公有云" }
                                option { value: "政务云", "政务云" }
                                option { value: "私有云", "私有云" }
                            }
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "底座" }
                        select {
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: foundation,
                            onchange: move |e| foundation.set(e.value()),
                            option { value: "阿里云", "阿里云" }
                            option { value: "华为云", "华为云" }
                            option { value: "腾讯云", "腾讯云" }
                            option { value: "AWS", "AWS" }
                            option { value: "Azure", "Azure" }
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "实例类型" }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            placeholder: "例如: ecs.g6.large",
                            value: instance_type,
                            oninput: move |e| instance_type.set(e.value()),
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "IP地址" }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: ip_address,
                            oninput: move |e| ip_address.set(e.value()),
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1",
                            Icon { icon: FaLock, width: 14, height: 14, class: "mr-1 text-gray-400" }
                            "开放端口（自动派生）"
                        }
                        div {
                            class: "w-full px-3 py-2 bg-gray-50 border border-gray-200 rounded-md text-gray-600 text-sm",
                            style: "min-height: 40px;",
                            {
                                let computed = asset.computed_ports();
                                computed
                            }
                        }
                        p { class: "text-xs text-blue-500 mt-1",
                            "由已生效的网络策略工单自动计算，不可手动编辑"
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "状态" }
                        select {
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: status,
                            onchange: move |e| status.set(e.value()),
                            option { value: "运行中", "运行中" }
                            option { value: "已停止", "已停止" }
                            option { value: "已释放", "已释放" }
                        }
                    }

                    div { class: "grid grid-cols-2 gap-3",
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "区域" }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                placeholder: "例如: 华东1-杭州",
                                value: region,
                                oninput: move |e| region.set(e.value()),
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "机房" }
                            input {
                                r#type: "text",
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                                placeholder: "例如: 市政务云机房A",
                                value: machine_room,
                                oninput: move |e| machine_room.set(e.value()),
                            }
                        }
                    }
                }

                div { class: "flex justify-end space-x-3 p-4 border-t",
                    button {
                        class: "px-4 py-2 text-gray-600 border border-gray-300 rounded-md hover:bg-gray-50",
                        onclick: move |_| on_close.call(()),
                        "取消"
                    }
                    button {
                        class: "px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700",
                        onclick: {
                            let asset_id = asset.id;
                            let created_at = asset.created_at.clone();
                            let original_ports = asset.ports.clone(); // Keep original ports (computed dynamically on display)
                            move |_| {
                                let updated = CloudAsset {
                                    id: asset_id,
                                    name: name.read().clone(),
                                    provider_id: *provider_id.read(),
                                    cloud_type: cloud_type.read().clone(),
                                    foundation: foundation.read().clone(),
                                    instance_type: instance_type.read().clone(),
                                    ip_address: ip_address.read().clone(),
                                    ports: original_ports.clone(),
                                    status: status.read().clone(),
                                    region: region.read().clone(),
                                    machine_room: machine_room.read().clone(),
                                    created_at: created_at.clone(),
                                };
                                on_save.call(updated);
                            }
                        },
                        "保存"
                    }
                }
            }
        }
    }
}
