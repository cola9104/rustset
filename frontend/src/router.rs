use dioxus::prelude::*;
use dioxus_router::Routable;

use crate::components::layout::Layout;
use crate::components::auth::Login;
use crate::components::dashboard::Dashboard;
use crate::components::asset::AssetManagement;
use crate::components::task::TaskCenter;
use crate::components::risk::RiskCenter;
use crate::components::audit::AuditLogs;
use crate::components::user::UserManagement;
use crate::components::permission::PermissionManagement;
use crate::components::password::PasswordPolicy;
use crate::components::resource_ticket::ResourceTicket;
use crate::components::resource_ticket::cloud_service::CloudServiceRequest;
use crate::components::resource_ticket::physical_server::PhysicalServerRequest;
use crate::components::resource_ticket::network_policy::NetworkPolicyRequest;
use crate::components::cloud_platform::CloudPlatformManagement;
use crate::components::machine_room::MachineRoomManagement;
use crate::components::service_provider::ServiceProviderManagement;
use crate::components::business::BusinessApplication;
use crate::components::network_zone::NetworkZoneManagement;
use crate::components::security_product::SecurityProductManagement;

/// 路由定义
#[derive(Routable, Clone, PartialEq, Debug)]
pub enum Route {
    // 认证页面
    #[route("/login")]
    Login {},

    // 主布局包装的路由
    #[layout(Layout)]

    // 仪表板
    #[route("/")]
    Dashboard {},

    // 资产管理
    #[route("/assets")]
    AssetManagement {},

    // 任务中心
    #[route("/tasks")]
    TaskCenter {},

    // 风险中心
    #[route("/risks")]
    RiskCenter {},

    // 审计日志
    #[route("/audit")]
    AuditLogs {},

    // 业务应用管理
    #[route("/business-apps")]
    BusinessApplication {},

    // 资源工单
    #[route("/tickets")]
    ResourceTicket {},

    // 资源工单分类页面
    #[route("/tickets/cloud-service")]
    CloudServiceRequest {},

    #[route("/tickets/physical-server")]
    PhysicalServerRequest {},

    #[route("/tickets/network-policy")]
    NetworkPolicyRequest {},

    // 云平台管理
    #[route("/cloud-platforms")]
    CloudPlatformManagement {},

    // 服务商管理
    #[route("/service-providers")]
    ServiceProviderManagement {},

    // 机房管理
    #[route("/machine-rooms")]
    MachineRoomManagement {},

    // 网络区域管理
    #[route("/network-zones")]
    NetworkZoneManagement {},

    // 安全产品管理
    #[route("/security-products")]
    SecurityProductManagement {},

    // 用户管理
    #[route("/users")]
    UserManagement {},

    // 权限管理
    #[route("/permissions")]
    PermissionManagement {},

    // 密码策略
    #[route("/password-policy")]
    PasswordPolicy {},

    // 结束布局
    #[end_layout]

    // 404 页面
    #[route("/:..route")]
    NotFound { route: Vec<String> },
}

/// 404 页面组件
#[component]
fn NotFound(route: Vec<String>) -> Element {
    rsx! {
        div { class: "flex flex-col items-center justify-center min-h-screen bg-gray-100",
            h1 { class: "text-6xl font-bold text-gray-800", "404" }
            p { class: "text-xl text-gray-600 mt-4", "页面未找到" }
            p { class: "text-gray-500 mt-2", "路径: /{route.join(\"/\")}" }
        }
    }
}
