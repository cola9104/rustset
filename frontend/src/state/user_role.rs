use crate::app::AuthUser;
use dioxus::prelude::*;

/// 用户角色枚举
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UserRole {
    /// 未登录
    Guest,
    /// 申请人员
    Applicant,
    /// 审批人员
    Approver,
    /// 运维人员
    Operator,
    /// 交付人员
    Deliverer,
    /// 管理员（拥有所有权限）
    Admin,
}

impl UserRole {
    /// 获取角色显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            UserRole::Guest => "未登录",
            UserRole::Applicant => "申请人员",
            UserRole::Approver => "审批人员",
            UserRole::Operator => "运维人员",
            UserRole::Deliverer => "交付人员",
            UserRole::Admin => "管理员",
        }
    }

    /// 检查是否可以提交申请
    pub fn can_submit(&self) -> bool {
        matches!(self, UserRole::Applicant | UserRole::Admin)
    }

    /// 检查是否可以审批
    pub fn can_approve(&self) -> bool {
        matches!(self, UserRole::Approver | UserRole::Admin)
    }

    /// 检查是否可以配置
    pub fn can_provision(&self) -> bool {
        matches!(self, UserRole::Operator | UserRole::Admin)
    }

    /// 检查是否可以交付
    pub fn can_deliver(&self) -> bool {
        matches!(self, UserRole::Deliverer | UserRole::Admin)
    }

    /// 获取所有可访问的标签页
    pub fn accessible_tabs(&self) -> Vec<ApplicationTab> {
        match self {
            UserRole::Guest => vec![],
            UserRole::Applicant => vec![ApplicationTab::MyApplications],
            UserRole::Approver => vec![ApplicationTab::PendingApproval],
            UserRole::Operator => vec![ApplicationTab::PendingProvision],
            UserRole::Deliverer => vec![ApplicationTab::PendingDelivery, ApplicationTab::Delivered],
            UserRole::Admin => vec![
                ApplicationTab::MyApplications,
                ApplicationTab::PendingApproval,
                ApplicationTab::PendingProvision,
                ApplicationTab::PendingDelivery,
                ApplicationTab::Delivered,
                ApplicationTab::Archived,
            ],
        }
    }
}

/// 应用状态标签页
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApplicationTab {
    /// 我的申请
    MyApplications,
    /// 待审批
    PendingApproval,
    /// 待配置
    PendingProvision,
    /// 待交付
    PendingDelivery,
    /// 已交付
    Delivered,
    /// 已归档
    Archived,
}

impl ApplicationTab {
    pub fn display_name(&self) -> &'static str {
        match self {
            ApplicationTab::MyApplications => "我的申请",
            ApplicationTab::PendingApproval => "待审批",
            ApplicationTab::PendingProvision => "待配置",
            ApplicationTab::PendingDelivery => "待交付",
            ApplicationTab::Delivered => "已交付",
            ApplicationTab::Archived => "已归档",
        }
    }

    pub fn icon_name(&self) -> &'static str {
        match self {
            ApplicationTab::MyApplications => "fa-list",
            ApplicationTab::PendingApproval => "fa-clock",
            ApplicationTab::PendingProvision => "fa-cog",
            ApplicationTab::PendingDelivery => "fa-truck",
            ApplicationTab::Delivered => "fa-box-open",
            ApplicationTab::Archived => "fa-archive",
        }
    }
}

/// 认证状态
#[derive(Clone, Debug, PartialEq)]
pub struct AuthState {
    pub username: String,
    pub role: UserRole,
    pub role_label: String,
}

impl AuthState {
    /// 创建新的认证状态
    #[allow(dead_code)]
    pub fn new(username: String, role: UserRole, role_label: String) -> Self {
        Self {
            username,
            role,
            role_label,
        }
    }

    /// 创建未登录认证状态
    pub fn guest() -> Self {
        Self {
            username: String::new(),
            role: UserRole::Guest,
            role_label: UserRole::Guest.display_name().to_string(),
        }
    }

    pub fn display_role_label(&self) -> &str {
        &self.role_label
    }
}

fn workflow_role_from_backend(role: &str) -> UserRole {
    match role {
        "Operator" => UserRole::Operator,
        "SysAdmin" | "SecAdmin" | "Auditor" | "Custom" => UserRole::Admin,
        _ => UserRole::Admin,
    }
}

fn backend_role_label(role: &str) -> String {
    match role {
        "SysAdmin" => "系统管理员".to_string(),
        "SecAdmin" => "安全管理员".to_string(),
        "Auditor" => "审计员".to_string(),
        "Operator" => "运维人员".to_string(),
        "Custom" => "自定义角色".to_string(),
        _ => role.to_string(),
    }
}

impl From<&AuthUser> for AuthState {
    fn from(user: &AuthUser) -> Self {
        Self {
            username: user.username.clone(),
            role: workflow_role_from_backend(&user.role),
            role_label: backend_role_label(&user.role),
        }
    }
}

/// 全局认证状态 Hook
pub fn use_auth() -> Signal<AuthState> {
    // 在实际应用中，这里应该从 localStorage 或后端获取
    try_use_context::<Signal<AuthState>>()
        .unwrap_or_else(|| use_hook(|| Signal::new(AuthState::guest())))
}
