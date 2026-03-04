use dioxus::prelude::*;

/// 用户角色枚举
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UserRole {
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
            UserRole::Applicant => vec![
                ApplicationTab::MyApplications,
            ],
            UserRole::Approver => vec![
                ApplicationTab::PendingApproval,
            ],
            UserRole::Operator => vec![
                ApplicationTab::PendingProvision,
            ],
            UserRole::Deliverer => vec![
                ApplicationTab::PendingDelivery,
                ApplicationTab::Delivered,
            ],
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
    pub token: String,
}

impl AuthState {
    /// 创建新的认证状态
    pub fn new(username: String, role: UserRole, token: String) -> Self {
        Self {
            username,
            role,
            token,
        }
    }

    /// 创建默认认证状态（开发测试用）
    pub fn default_with_role(role: UserRole) -> Self {
        Self {
            username: "测试用户".to_string(),
            role,
            token: "test-token".to_string(),
        }
    }
}

/// 全局认证状态 Hook
pub fn use_auth() -> Signal<AuthState> {
    // 在实际应用中，这里应该从 localStorage 或后端获取
    // 目前返回一个默认的管理员状态用于开发
    try_use_context::<Signal<AuthState>>()
        .unwrap_or_else(|| use_hook(|| Signal::new(AuthState::default_with_role(UserRole::Admin))))
}
