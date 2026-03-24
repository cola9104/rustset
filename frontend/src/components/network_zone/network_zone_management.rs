use crate::components::ip_zones::ip_zones_page::IpZonesPage;
use dioxus::prelude::*;

/// 网络区域管理
///
/// 当前后端已有真实可用的数据源是 IP Zones（`/api/ip-zones`）。
/// 先复用该页面，避免继续展示仅存在于前端内存中的静态区域数据。
#[allow(non_snake_case)]
pub fn NetworkZoneManagement() -> Element {
    rsx! {
        IpZonesPage {}
    }
}
