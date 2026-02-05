// API 基础 URL - 使用 window.location 自动检测
// 在开发环境可以使用 localhost，在生产环境使用实际访问的地址
pub fn api_base_url() -> String {
    // 尝试从当前页面 URL 获取主机和端口
    if let Some(window) = web_sys::window() {
        if let Ok(href) = window.location().href() {
            // 如果是通过 HTTP 访问的，使用同源的 API 地址（默认端口 3003）
            if let Ok(host) = window.location().host() {
                // 将 8080 替换为 3003
                if host.contains(":8080") {
                    return host.replace(":8080", ":3003");
                }
                // 如果没有端口，添加 3003
                if !host.contains(':') {
                    return format!("{}:3003", host);
                }
                return host;
            }
        }
    }
    // 默认回退到 localhost
    "localhost:3003".to_string()
}

// 构建 API URL 的辅助函数
pub fn api_url(path: &str) -> String {
    format!("http://{}/api/{}", api_base_url(), path)
}
