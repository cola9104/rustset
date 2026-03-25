use dioxus::prelude::*;
use dioxus_router::Routable;

use crate::components::asset::AssetManagement;
use crate::components::audit::AuditLogs;
use crate::components::auth::Login;
use crate::components::business::BusinessApplication;
use crate::components::cloud_platform::CloudPlatformManagement;
use crate::components::dashboard::Dashboard;
use crate::components::department::DepartmentManagement;
use crate::components::layout::Layout;
use crate::components::machine_room::MachineRoomManagement;
use crate::components::network_zone::NetworkZoneManagement;
use crate::components::organization::OrganizationManagement;
use crate::components::password::PasswordPolicy;
use crate::components::permission::PermissionManagement;
use crate::components::profile::ProfilePage;
use crate::components::resource_ticket::cloud_service::CloudServiceRequest;
use crate::components::resource_ticket::network_policy::NetworkPolicyRequest;
use crate::components::resource_ticket::physical_server::PhysicalServerRequest;
use crate::components::resource_ticket::ResourceTicket;
use crate::components::risk::RiskCenter;
use crate::components::security_product::SecurityProductManagement;
use crate::components::service_provider::ServiceProviderManagement;
use crate::components::task::TaskCenter;
use crate::components::user::UserManagement;
use crate::state::user_role::AuthState;

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

    #[route("/organizations")]
    OrganizationManagement {},

    #[route("/departments")]
    DepartmentManagement {},

    #[route("/profile")]
    ProfilePage {},

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

pub fn can_access_route(route: &Route, auth: &AuthState) -> bool {
    match route {
        Route::Login {} | Route::NotFound { .. } => true,
        Route::Dashboard {} => auth.can_view_dashboard(),
        Route::TaskCenter {} => auth.can_view_tasks(),
        Route::RiskCenter {} => auth.has_permission("can_view_risks"),
        Route::AuditLogs {} => auth.has_permission("can_view_audit_logs"),
        Route::BusinessApplication {} => has_any_permission(
            auth,
            &[
                "can_view_business_process",
                "can_view_business_applications",
                "can_create_business_application",
                "can_approve_business_application",
                "can_supplement_business_application",
                "can_delete_business_application",
            ],
        ),
        Route::ResourceTicket {} => can_access_resource_tickets(auth),
        Route::CloudServiceRequest {}
        | Route::PhysicalServerRequest {}
        | Route::NetworkPolicyRequest {} => auth.can_submit(),
        Route::CloudPlatformManagement {} => has_any_permission(
            auth,
            &[
                "can_access_cloud",
                "can_view_cloud_providers",
                "can_manage_cloud_providers",
            ],
        ),
        Route::AssetManagement {}
        | Route::ServiceProviderManagement {}
        | Route::MachineRoomManagement {}
        | Route::NetworkZoneManagement {}
        | Route::SecurityProductManagement {} => has_any_permission(
            auth,
            &[
                "can_access_assets_risks",
                "can_view_cloud_assets",
                "can_manage_operations",
            ],
        ),
        Route::UserManagement {} => auth.can_view_users(),
        Route::ProfilePage {} => auth.can_view_profile(),
        Route::OrganizationManagement {} | Route::DepartmentManagement {} => {
            auth.can_view_users() || auth.can_manage_permissions()
        }
        Route::PermissionManagement {} => auth.can_manage_permissions(),
        Route::PasswordPolicy {} => {
            auth.can_view_password_policy() || auth.can_manage_password_policy()
        }
    }
}

pub fn first_accessible_route(auth: &AuthState) -> Option<Route> {
    [
        Route::Dashboard {},
        Route::ResourceTicket {},
        Route::TaskCenter {},
        Route::RiskCenter {},
        Route::AssetManagement {},
        Route::BusinessApplication {},
        Route::CloudPlatformManagement {},
        Route::ServiceProviderManagement {},
        Route::MachineRoomManagement {},
        Route::NetworkZoneManagement {},
        Route::SecurityProductManagement {},
        Route::UserManagement {},
        Route::OrganizationManagement {},
        Route::DepartmentManagement {},
        Route::PermissionManagement {},
        Route::PasswordPolicy {},
        Route::AuditLogs {},
    ]
    .into_iter()
    .find(|route| can_access_route(route, auth))
}

fn can_access_resource_tickets(auth: &AuthState) -> bool {
    auth.can_view_resource_tickets()
        || auth.can_submit()
        || auth.can_approve()
        || auth.can_provision()
        || auth.can_deliver()
}

fn has_any_permission(auth: &AuthState, permissions: &[&str]) -> bool {
    permissions
        .iter()
        .any(|permission| auth.has_permission(permission))
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
