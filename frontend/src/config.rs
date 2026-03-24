/// API 配置模块
///
/// 集中管理后端 API 的 URL 配置

/// 获取后端 API 基础 URL
pub fn api_base() -> String {
    // 优先从环境变量读取
    if let Ok(url) = std::env::var("VITE_API_BASE") {
        return url;
    }

    // 开发环境默认值
    #[cfg(debug_assertions)]
    {
        return "http://localhost:3003/api".to_string();
    }

    // 生产环境默认值（相对路径，使用代理）
    #[cfg(not(debug_assertions))]
    {
        return "/api".to_string();
    }
}

/// 登录 API URL
pub fn login_url() -> String {
    format!("{}/login", api_base())
}

/// 登出 API URL
pub fn logout_url() -> String {
    format!("{}/logout", api_base())
}

/// Token 刷新 API URL
pub fn refresh_token_url() -> String {
    format!("{}/refresh-token", api_base())
}

/// IP Zones API URL
pub fn ip_zones_url() -> String {
    format!("{}/ip-zones", api_base())
}

/// IP 查找 Zone API URL
pub fn ip_find_zone_url(ip: &str) -> String {
    format!("{}/ip-zones/find?ip={}", api_base(), ip)
}

/// 扫描 API URL
pub fn scan_ip_url() -> String {
    format!("{}/scan-ip", api_base())
}

/// 批量扫描 API URL
pub fn batch_scan_url() -> String {
    format!("{}/batch-scan-ips", api_base())
}

/// 扫描结果 API URL
pub fn scan_results_url() -> String {
    format!("{}/scan-results", api_base())
}

/// 用户列表 API URL
pub fn users_url() -> String {
    format!("{}/users", api_base())
}

/// 任务列表 API URL
pub fn tasks_url() -> String {
    format!("{}/tasks", api_base())
}

/// 当前用户信息 API URL
pub fn current_user_url() -> String {
    format!("{}/users/me", api_base())
}

/// 仪表板汇总 API URL
pub fn dashboard_summary_url() -> String {
    format!("{}/dashboard-summary", api_base())
}

/// 审计日志 API URL
pub fn audit_logs_url() -> String {
    format!("{}/logs", api_base())
}

/// 健康检查 API URL
pub fn health_url() -> String {
    format!("{}/health", api_base())
}
