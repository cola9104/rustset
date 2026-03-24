use dioxus::prelude::*;
use dioxus_router::Router;

use crate::router::Route;
use crate::state::user_role::AuthState as WorkflowAuthState;
use crate::state::{
    cloud_platform::CloudPlatformConfig, machine_room::MachineRoomConfig,
    network_policy::NetworkPolicyConfig, network_zone::NetworkZone,
    security_product::SecurityProduct, service_provider::ServiceProviderConfig,
};
use crate::utils::storage::clear_token;

/// 全局安全产品数据状态
pub static SECURITY_PRODUCTS_STATE: GlobalSignal<Vec<SecurityProduct>> =
    Signal::global(|| Vec::new());

/// 主应用组件
#[allow(non_snake_case)]
pub fn App() -> Element {
    use_context_provider(|| Signal::new(WorkflowAuthState::guest()));

    rsx! {
        Router::<Route> {}
    }
}

/// 认证用户信息
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AuthUser {
    pub id: String,
    pub username: String,
    pub role: String,
    pub permissions: Vec<String>,
}

/// 全局认证状态
pub static AUTH_STATE: GlobalSignal<Option<AuthUser>> = Signal::global(|| None);

/// 全局服务商数据状态
pub static PROVIDERS_STATE: GlobalSignal<Vec<ServiceProviderConfig>> =
    Signal::global(|| Vec::new());

/// 全局机房数据状态
pub static MACHINE_ROOMS_STATE: GlobalSignal<Vec<MachineRoomConfig>> =
    Signal::global(|| Vec::new());

/// 全局云平台数据状态
pub static CLOUD_PLATFORMS_STATE: GlobalSignal<Vec<CloudPlatformConfig>> =
    Signal::global(|| Vec::new());

/// 全局网络区域数据状态
pub static NETWORK_ZONES_STATE: GlobalSignal<Vec<NetworkZone>> = Signal::global(|| Vec::new());

/// 全局网络策略数据状态
pub static NETWORK_POLICIES_STATE: GlobalSignal<Vec<NetworkPolicyConfig>> =
    Signal::global(|| Vec::new());

/// 检查是否已认证
#[allow(dead_code)]
pub fn is_authenticated() -> bool {
    AUTH_STATE.read().is_some()
}

/// 登出
#[allow(dead_code)]
pub fn logout() {
    clear_token();
    *AUTH_STATE.write() = None;
}

/// 检查是否是管理员
#[allow(dead_code)]
pub fn is_admin() -> bool {
    AUTH_STATE
        .read()
        .as_ref()
        .map(|u| matches!(u.role.as_str(), "SysAdmin" | "SecAdmin"))
        .unwrap_or(false)
}

/// 检查是否有权限
#[allow(dead_code)]
pub fn has_permission(permission: &str) -> bool {
    AUTH_STATE
        .read()
        .as_ref()
        .map(|u| u.permissions.contains(&permission.to_string()))
        .unwrap_or(false)
}
