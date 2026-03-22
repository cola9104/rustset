use dioxus::prelude::*;
use dioxus_router::Router;

use crate::router::Route;
use crate::state::{
    service_provider::ServiceProviderConfig,
    machine_room::MachineRoomConfig,
    cloud_platform::CloudPlatformConfig,
    network_zone::NetworkZone,
    security_product::SecurityProduct,
    network_policy::NetworkPolicyConfig,
    init_service_providers,
    init_machine_rooms,
    init_cloud_platforms,
    init_network_zones,
    init_security_products,
    init_network_policies,
};

/// 全局安全产品数据状态
pub static SECURITY_PRODUCTS_STATE: GlobalSignal<Vec<SecurityProduct>> = Signal::global(init_security_products);

/// 主应用组件
#[allow(non_snake_case)]
pub fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }
}

/// 认证用户信息
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AuthUser {
    pub id: i32,
    pub username: String,
    pub role: String,
    pub permissions: Vec<String>,
}

/// 全局认证状态
pub static AUTH_STATE: GlobalSignal<Option<AuthUser>> = Signal::global(|| None);

/// 全局服务商数据状态
pub static PROVIDERS_STATE: GlobalSignal<Vec<ServiceProviderConfig>> = Signal::global(init_service_providers);

/// 全局机房数据状态
pub static MACHINE_ROOMS_STATE: GlobalSignal<Vec<MachineRoomConfig>> = Signal::global(init_machine_rooms);

/// 全局云平台数据状态
pub static CLOUD_PLATFORMS_STATE: GlobalSignal<Vec<CloudPlatformConfig>> = Signal::global(init_cloud_platforms);

/// 全局网络区域数据状态
/// 从云平台和机房数据动态生成网络区域
pub static NETWORK_ZONES_STATE: GlobalSignal<Vec<NetworkZone>> = Signal::global(|| {
    let cloud_platforms = init_cloud_platforms();
    let machine_rooms = init_machine_rooms();
    init_network_zones(&cloud_platforms, &machine_rooms)
});

/// 全局网络策略数据状态
pub static NETWORK_POLICIES_STATE: GlobalSignal<Vec<NetworkPolicyConfig>> = Signal::global(init_network_policies);

/// 检查是否已认证
#[allow(dead_code)]
pub fn is_authenticated() -> bool {
    AUTH_STATE.read().is_some()
}

/// 登出
#[allow(dead_code)]
pub fn logout() {
    *AUTH_STATE.write() = None;
}

/// 检查是否是管理员
#[allow(dead_code)]
pub fn is_admin() -> bool {
    AUTH_STATE.read()
        .as_ref()
        .map(|u| u.role == "admin")
        .unwrap_or(false)
}

/// 检查是否有权限
#[allow(dead_code)]
pub fn has_permission(permission: &str) -> bool {
    AUTH_STATE.read()
        .as_ref()
        .map(|u| u.permissions.contains(&permission.to_string()))
        .unwrap_or(false)
}
