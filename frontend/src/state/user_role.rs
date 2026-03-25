use crate::app::AuthUser;
use dioxus::prelude::*;

/// 用户角色枚举
/// 当前后端登录态尚未直接下发全部工单工作流角色，但前端流程仍保留这些语义。
#[allow(dead_code)]
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
    pub permissions: Vec<String>,
}

impl AuthState {
    /// 创建未登录认证状态
    pub fn guest() -> Self {
        Self {
            username: String::new(),
            role: UserRole::Guest,
            role_label: UserRole::Guest.display_name().to_string(),
            permissions: Vec::new(),
        }
    }

    pub fn display_role_label(&self) -> &str {
        &self.role_label
    }

    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.iter().any(|item| item == permission)
    }

    pub fn can_view_dashboard(&self) -> bool {
        self.has_permission("can_view_dashboard")
    }

    pub fn can_view_tasks(&self) -> bool {
        self.has_permission("can_view_tasks")
    }

    pub fn can_create_task(&self) -> bool {
        self.has_permission("can_create_task")
    }

    pub fn can_delete_task(&self) -> bool {
        self.has_permission("can_delete_task")
    }

    pub fn can_manage_operations(&self) -> bool {
        self.has_permission("can_manage_operations")
    }

    pub fn can_manage_cloud_providers(&self) -> bool {
        self.has_permission("can_manage_cloud_providers")
    }

    pub fn can_view_users(&self) -> bool {
        self.has_permission("can_view_users")
    }

    pub fn can_create_user(&self) -> bool {
        self.has_permission("can_create_user")
    }

    pub fn can_update_user(&self) -> bool {
        self.has_permission("can_update_user")
    }

    pub fn can_delete_user(&self) -> bool {
        self.has_permission("can_delete_user")
    }

    pub fn can_manage_permissions(&self) -> bool {
        self.has_permission("can_manage_permissions")
    }

    pub fn can_view_password_policy(&self) -> bool {
        self.has_permission("can_view_password_policy")
    }

    pub fn can_manage_password_policy(&self) -> bool {
        self.has_permission("can_manage_password_policy")
    }

    pub fn scope_value(&self, key: &str) -> Option<&str> {
        let prefix = format!("{key}:");
        self.permissions
            .iter()
            .find_map(|item| item.strip_prefix(&prefix))
    }

    pub fn can_submit(&self) -> bool {
        self.has_permission("can_create_resource_tickets")
    }

    pub fn can_approve(&self) -> bool {
        self.has_permission("can_approve_resource_tickets")
    }

    pub fn can_provision(&self) -> bool {
        self.has_permission("can_provision_resource_tickets")
    }

    pub fn can_deliver(&self) -> bool {
        self.has_permission("can_deliver_resource_tickets")
    }

    pub fn can_view_resource_tickets(&self) -> bool {
        self.has_permission("can_view_resource_tickets")
    }

    pub fn accessible_tabs(&self) -> Vec<ApplicationTab> {
        let mut tabs = Vec::new();

        if self.can_submit() {
            tabs.push(ApplicationTab::MyApplications);
        }
        if self.can_approve() {
            tabs.push(ApplicationTab::PendingApproval);
        }
        if self.can_provision() {
            tabs.push(ApplicationTab::PendingProvision);
        }
        if self.can_deliver() {
            tabs.push(ApplicationTab::PendingDelivery);
            tabs.push(ApplicationTab::Delivered);
        }
        if self.has_permission("can_view_resource_tickets")
            && self.scope_value("resource_ticket_scope") == Some("all")
        {
            tabs.push(ApplicationTab::Archived);
        }

        if tabs.is_empty() && self.can_view_resource_tickets() {
            tabs.push(ApplicationTab::MyApplications);
        }

        tabs
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
            permissions: user.permissions.clone(),
        }
    }
}

/// 全局认证状态 Hook
pub fn use_auth() -> Signal<AuthState> {
    // 在实际应用中，这里应该从 localStorage 或后端获取
    try_use_context::<Signal<AuthState>>()
        .unwrap_or_else(|| use_hook(|| Signal::new(AuthState::guest())))
}
