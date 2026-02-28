use dioxus::prelude::*;
use dioxus_router::Routable;

use crate::components::layout::Layout;
use crate::components::auth::Login;
use crate::components::dashboard::Dashboard;
use crate::components::asset::AssetManagement;
use crate::components::task::TaskCenter;
use crate::components::risk::RiskCenter;
use crate::components::audit::AuditLogs;

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

    // 高级扫描
    #[route("/scanning")]
    AdvancedScanning {},

    // 云服务资产
    #[route("/cloud-assets")]
    CloudServiceAsset {},

    // 运维管理
    #[route("/operations")]
    OperationsManagement {},

    // 业务申请
    #[route("/business")]
    BusinessApplication {},

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

/// 高级扫描
#[component]
fn AdvancedScanning() -> Element {
    rsx! {
        div { class: "p-8",
            h1 { class: "text-2xl font-bold text-gray-800", "高级扫描" }
            p { class: "text-gray-600 mt-2", "配置和执行高级扫描任务" }
        }
    }
}

/// 云服务资产
#[component]
fn CloudServiceAsset() -> Element {
    rsx! {
        div { class: "p-8",
            h1 { class: "text-2xl font-bold text-gray-800", "云服务资产" }
            p { class: "text-gray-600 mt-2", "管理云平台资产" }
        }
    }
}

/// 运维管理
#[component]
fn OperationsManagement() -> Element {
    rsx! {
        div { class: "p-8",
            h1 { class: "text-2xl font-bold text-gray-800", "运维管理" }
            p { class: "text-gray-600 mt-2", "系统运维配置" }
        }
    }
}

/// 业务申请
#[component]
fn BusinessApplication() -> Element {
    rsx! {
        div { class: "p-8",
            h1 { class: "text-2xl font-bold text-gray-800", "业务申请" }
            p { class: "text-gray-600 mt-2", "管理业务资源申请" }
        }
    }
}

/// 用户管理
#[component]
fn UserManagement() -> Element {
    rsx! {
        div { class: "p-8",
            h1 { class: "text-2xl font-bold text-gray-800", "用户管理" }
            p { class: "text-gray-600 mt-2", "管理系统用户" }
        }
    }
}

/// 权限管理
#[component]
fn PermissionManagement() -> Element {
    rsx! {
        div { class: "p-8",
            h1 { class: "text-2xl font-bold text-gray-800", "权限管理" }
            p { class: "text-gray-600 mt-2", "配置用户权限和角色" }
        }
    }
}

/// 密码策略
#[component]
fn PasswordPolicy() -> Element {
    rsx! {
        div { class: "p-8",
            h1 { class: "text-2xl font-bold text-gray-800", "密码策略" }
            p { class: "text-gray-600 mt-2", "配置密码安全策略" }
        }
    }
}
