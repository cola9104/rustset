use crate::router::Route;
use crate::state::user_role::use_auth;
use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaBriefcase, FaBuilding, FaCloud, FaEarthAmericas, FaFileLines, FaHouse, FaKey, FaList, FaLock,
    FaNetworkWired, FaServer, FaShieldHalved, FaTriangleExclamation, FaUsers,
};
use dioxus_free_icons::Icon;
use dioxus_router::Link;

/// 侧边栏组件
#[component]
pub fn Sidebar(collapsed: Signal<bool>) -> Element {
    let is_collapsed = *collapsed.read();
    let auth = use_auth();
    let current_auth = auth.read().clone();
    let can_access_tickets = current_auth.can_view_resource_tickets()
        || current_auth.can_submit()
        || current_auth.can_approve()
        || current_auth.can_provision()
        || current_auth.can_deliver();
    let can_access_business = current_auth.has_permission("can_view_business_process")
        || current_auth.has_permission("can_view_business_applications")
        || current_auth.has_permission("can_create_business_application")
        || current_auth.has_permission("can_approve_business_application")
        || current_auth.has_permission("can_supplement_business_application");
    let can_access_asset_ops = current_auth.has_permission("can_access_assets_risks")
        || current_auth.has_permission("can_view_cloud_assets")
        || current_auth.has_permission("can_manage_operations");
    let can_access_cloud_config = current_auth.has_permission("can_access_cloud")
        || current_auth.has_permission("can_view_cloud_providers")
        || current_auth.has_permission("can_manage_cloud_providers");
    let can_access_user_admin = current_auth.has_permission("can_view_users");
    let can_access_role_admin = current_auth.has_permission("can_manage_permissions");
    let can_access_org_admin = can_access_user_admin || can_access_role_admin;

    rsx! {
        aside {
            class: if is_collapsed {
                "w-16 bg-slate-800 text-white transition-all duration-300"
            } else {
                "w-64 bg-slate-800 text-white transition-all duration-300"
            },

            // Logo
            div { class: "h-14 flex items-center justify-center border-b border-slate-700",
                if !is_collapsed {
                    span { class: "text-xl font-bold", "RustSet" }
                } else {
                    span { class: "text-xl font-bold", "R" }
                }
            }

            // 导航菜单
            nav { class: "mt-4 flex-1",
                // 仪表板
                if current_auth.has_permission("can_view_dashboard") {
                    Link {
                        to: Route::Dashboard {},
                        class: "flex items-center px-4 py-3 text-slate-300 hover:bg-slate-700 hover:text-white transition-colors",
                        Icon { icon: FaHouse, width: 20, height: 20 }
                        if !is_collapsed {
                            span { class: "ml-3", "仪表板" }
                        }
                    }
                }
                // 任务中心
                if current_auth.has_permission("can_view_tasks") {
                    Link {
                        to: Route::TaskCenter {},
                        class: "flex items-center px-4 py-3 text-slate-300 hover:bg-slate-700 hover:text-white transition-colors",
                        Icon { icon: FaList, width: 20, height: 20 }
                        if !is_collapsed {
                            span { class: "ml-3", "任务中心" }
                        }
                    }
                }
                // 风险中心
                if current_auth.has_permission("can_view_risks") {
                    Link {
                        to: Route::RiskCenter {},
                        class: "flex items-center px-4 py-3 text-slate-300 hover:bg-slate-700 hover:text-white transition-colors",
                        Icon { icon: FaTriangleExclamation, width: 20, height: 20 }
                        if !is_collapsed {
                            span { class: "ml-3", "风险中心" }
                        }
                    }
                }
                // 资源工单
                if can_access_tickets {
                    Link {
                        to: Route::ResourceTicket {},
                        class: "flex items-center px-4 py-3 text-slate-300 hover:bg-slate-700 hover:text-white transition-colors",
                        Icon { icon: FaBriefcase, width: 20, height: 20 }
                        if !is_collapsed {
                            span { class: "ml-3", "资源工单" }
                        }
                    }
                }
                // 资产管理
                if can_access_asset_ops {
                    Link {
                        to: Route::AssetManagement {},
                        class: "flex items-center px-4 py-3 text-slate-300 hover:bg-slate-700 hover:text-white transition-colors",
                        Icon { icon: FaServer, width: 20, height: 20 }
                        if !is_collapsed {
                            span { class: "ml-3", "资产管理" }
                        }
                    }
                }
                // 业务应用管理
                if can_access_business {
                    Link {
                        to: Route::BusinessApplication {},
                        class: "flex items-center px-4 py-3 text-slate-300 hover:bg-slate-700 hover:text-white transition-colors",
                        Icon { icon: FaEarthAmericas, width: 20, height: 20 }
                        if !is_collapsed {
                            span { class: "ml-3", "业务应用" }
                        }
                    }
                }
                // 云平台管理
                if can_access_cloud_config {
                    Link {
                        to: Route::CloudPlatformManagement {},
                        class: "flex items-center px-4 py-3 text-slate-300 hover:bg-slate-700 hover:text-white transition-colors",
                        Icon { icon: FaCloud, width: 20, height: 20 }
                        if !is_collapsed {
                            span { class: "ml-3", "云平台管理" }
                        }
                    }
                }
                // 服务商管理
                if can_access_asset_ops {
                    Link {
                        to: Route::ServiceProviderManagement {},
                        class: "flex items-center px-4 py-3 text-slate-300 hover:bg-slate-700 hover:text-white transition-colors",
                        Icon { icon: FaNetworkWired, width: 20, height: 20 }
                        if !is_collapsed {
                            span { class: "ml-3", "服务商管理" }
                        }
                    }
                }
                // 机房管理
                if can_access_asset_ops {
                    Link {
                        to: Route::MachineRoomManagement {},
                        class: "flex items-center px-4 py-3 text-slate-300 hover:bg-slate-700 hover:text-white transition-colors",
                        Icon { icon: FaBuilding, width: 20, height: 20 }
                        if !is_collapsed {
                            span { class: "ml-3", "机房管理" }
                        }
                    }
                }
                // 网络区域管理
                if can_access_asset_ops {
                    Link {
                        to: Route::NetworkZoneManagement {},
                        class: "flex items-center px-4 py-3 text-slate-300 hover:bg-slate-700 hover:text-white transition-colors",
                        Icon { icon: FaNetworkWired, width: 20, height: 20 }
                        if !is_collapsed {
                            span { class: "ml-3", "网络区域" }
                        }
                    }
                }
                // 安全产品管理
                if can_access_asset_ops {
                    Link {
                        to: Route::SecurityProductManagement {},
                        class: "flex items-center px-4 py-3 text-slate-300 hover:bg-slate-700 hover:text-white transition-colors",
                        Icon { icon: FaShieldHalved, width: 20, height: 20 }
                        if !is_collapsed {
                            span { class: "ml-3", "安全产品" }
                        }
                    }
                }
                // 用户管理
                if can_access_user_admin {
                    Link {
                        to: Route::UserManagement {},
                        class: "flex items-center px-4 py-3 text-slate-300 hover:bg-slate-700 hover:text-white transition-colors",
                        Icon { icon: FaUsers, width: 20, height: 20 }
                        if !is_collapsed {
                            span { class: "ml-3", "用户管理" }
                        }
                    }
                }
                if can_access_org_admin {
                    Link {
                        to: Route::OrganizationManagement {},
                        class: "flex items-center px-4 py-3 text-slate-300 hover:bg-slate-700 hover:text-white transition-colors",
                        Icon { icon: FaBuilding, width: 20, height: 20 }
                        if !is_collapsed {
                            span { class: "ml-3", "组织管理" }
                        }
                    }
                }
                if can_access_org_admin {
                    Link {
                        to: Route::DepartmentManagement {},
                        class: "flex items-center px-4 py-3 text-slate-300 hover:bg-slate-700 hover:text-white transition-colors",
                        Icon { icon: FaBriefcase, width: 20, height: 20 }
                        if !is_collapsed {
                            span { class: "ml-3", "部门管理" }
                        }
                    }
                }
                // 权限管理
                if can_access_role_admin {
                    Link {
                        to: Route::PermissionManagement {},
                        class: "flex items-center px-4 py-3 text-slate-300 hover:bg-slate-700 hover:text-white transition-colors",
                        Icon { icon: FaKey, width: 20, height: 20 }
                        if !is_collapsed {
                            span { class: "ml-3", "权限管理" }
                        }
                    }
                }
                // 密码策略
                if current_auth.has_permission("can_view_password_policy") {
                    Link {
                        to: Route::PasswordPolicy {},
                        class: "flex items-center px-4 py-3 text-slate-300 hover:bg-slate-700 hover:text-white transition-colors",
                        Icon { icon: FaLock, width: 20, height: 20 }
                        if !is_collapsed {
                            span { class: "ml-3", "密码策略" }
                        }
                    }
                }
                // 审计日志
                if current_auth.has_permission("can_view_audit_logs") {
                    Link {
                        to: Route::AuditLogs {},
                        class: "flex items-center px-4 py-3 text-slate-300 hover:bg-slate-700 hover:text-white transition-colors",
                        Icon { icon: FaFileLines, width: 20, height: 20 }
                        if !is_collapsed {
                            span { class: "ml-3", "审计日志" }
                        }
                    }
                }
            }
        }
    }
}
