use std::collections::HashSet;

/// 端口安全状态
#[derive(Clone, Debug, PartialEq)]
pub enum PortSecurityStatus {
    Protected,   // 已受防火墙策略保护
    Unprotected, // 未受保护（存在风险）
    Partial,     // 部分端口受保护
}

impl PortSecurityStatus {
    pub fn display_name(&self) -> &'static str {
        match self {
            PortSecurityStatus::Protected => "已保护",
            PortSecurityStatus::Unprotected => "未保护",
            PortSecurityStatus::Partial => "部分保护",
        }
    }

    pub fn color_class(&self) -> &'static str {
        match self {
            PortSecurityStatus::Protected => "text-green-600 bg-green-50",
            PortSecurityStatus::Unprotected => "text-red-600 bg-red-50",
            PortSecurityStatus::Partial => "text-yellow-600 bg-yellow-50",
        }
    }

    pub fn icon_color(&self) -> &'static str {
        match self {
            PortSecurityStatus::Protected => "text-green-500",
            PortSecurityStatus::Unprotected => "text-red-500",
            PortSecurityStatus::Partial => "text-yellow-500",
        }
    }
}

/// 单个端口的安全状态
#[derive(Clone, Debug)]
pub struct PortSecurityInfo {
    pub port: String,
    pub status: PortSecurityStatus,
    pub policies: Vec<String>, // 关联的策略名称
}

impl PortSecurityInfo {
    pub fn new(port: String) -> Self {
        Self {
            port,
            status: PortSecurityStatus::Unprotected,
            policies: Vec::new(),
        }
    }
}

/// 防火墙策略摘要（用于端口匹配）
#[derive(Clone, Debug)]
pub struct FirewallPolicySummary {
    pub id: i32,
    pub title: String,
    pub port_range: String,
    pub status: String, // "Active" 表示已生效
}

impl FirewallPolicySummary {
    /// 解析端口范围，返回包含的所有端口
    pub fn parse_ports(&self) -> HashSet<u16> {
        let mut ports = HashSet::new();

        for part in self.port_range.split(',') {
            let part = part.trim();
            if let Ok(port) = part.parse::<u16>() {
                ports.insert(port);
            } else if part.contains('-') {
                // 处理端口范围，如 "8080-8090"
                let range_parts: Vec<&str> = part.split('-').collect();
                if range_parts.len() == 2 {
                    if let (Ok(start), Ok(end)) = (
                        range_parts[0].trim().parse::<u16>(),
                        range_parts[1].trim().parse::<u16>(),
                    ) {
                        for port in start..=end {
                            ports.insert(port);
                        }
                    }
                }
            }
        }

        ports
    }

    /// 检查是否保护指定端口
    pub fn protects_port(&self, port: u16) -> bool {
        self.parse_ports().contains(&port)
    }
}

/// 分析端口的安全状态
pub fn analyze_port_security(
    asset_ports: &str,
    firewall_policies: &[FirewallPolicySummary],
) -> Vec<PortSecurityInfo> {
    let mut result = Vec::new();

    // 解析资产端口
    let asset_ports_set = parse_port_string(asset_ports);

    // 收集所有已生效策略保护的端口
    let mut protected_ports: HashSet<u16> = HashSet::new();
    let mut port_to_policies: std::collections::HashMap<u16, Vec<String>> =
        std::collections::HashMap::new();

    for policy in firewall_policies {
        if policy.status != "Active" && policy.status != "已生效" {
            continue;
        }

        for port in policy.parse_ports() {
            if asset_ports_set.contains(&port) {
                protected_ports.insert(port);
                port_to_policies
                    .entry(port)
                    .or_insert_with(Vec::new)
                    .push(policy.title.clone());
            }
        }
    }

    // 构建端口安全信息
    for port_str in asset_ports.split(',') {
        let port_str = port_str.trim();
        if port_str.is_empty() {
            continue;
        }

        let mut info = PortSecurityInfo::new(port_str.to_string());

        // 处理单个端口
        if let Ok(port) = port_str.parse::<u16>() {
            if protected_ports.contains(&port) {
                info.status = PortSecurityStatus::Protected;
                if let Some(policies) = port_to_policies.get(&port) {
                    info.policies = policies.clone();
                }
            }
        }
        // 处理端口范围
        else if port_str.contains('-') {
            let range_parts: Vec<&str> = port_str.split('-').collect();
            if range_parts.len() == 2 {
                if let (Ok(start), Ok(end)) = (
                    range_parts[0].trim().parse::<u16>(),
                    range_parts[1].trim().parse::<u16>(),
                ) {
                    let mut all_protected = true;
                    let mut any_protected = false;
                    let mut all_policies: Vec<String> = Vec::new();

                    for port in start..=end {
                        if protected_ports.contains(&port) {
                            any_protected = true;
                            if let Some(policies) = port_to_policies.get(&port) {
                                for policy in policies {
                                    if !all_policies.contains(policy) {
                                        all_policies.push(policy.clone());
                                    }
                                }
                            }
                        } else {
                            all_protected = false;
                        }
                    }

                    if all_protected {
                        info.status = PortSecurityStatus::Protected;
                    } else if any_protected {
                        info.status = PortSecurityStatus::Partial;
                    }
                    info.policies = all_policies;
                }
            }
        }

        result.push(info);
    }

    result
}

/// 解析端口字符串为端口集合
pub fn parse_port_string(port_str: &str) -> HashSet<u16> {
    let mut ports = HashSet::new();

    for part in port_str.split(',') {
        let part = part.trim();
        if let Ok(port) = part.parse::<u16>() {
            ports.insert(port);
        } else if part.contains('-') {
            let range_parts: Vec<&str> = part.split('-').collect();
            if range_parts.len() == 2 {
                if let (Ok(start), Ok(end)) = (
                    range_parts[0].trim().parse::<u16>(),
                    range_parts[1].trim().parse::<u16>(),
                ) {
                    for port in start..=end {
                        ports.insert(port);
                    }
                }
            }
        }
    }

    ports
}

/// 获取整体安全状态摘要
pub fn get_security_summary(port_infos: &[PortSecurityInfo]) -> (usize, usize, usize) {
    let protected = port_infos
        .iter()
        .filter(|p| p.status == PortSecurityStatus::Protected)
        .count();
    let unprotected = port_infos
        .iter()
        .filter(|p| p.status == PortSecurityStatus::Unprotected)
        .count();
    let partial = port_infos
        .iter()
        .filter(|p| p.status == PortSecurityStatus::Partial)
        .count();

    (protected, unprotected, partial)
}
