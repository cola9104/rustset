//! RustSet Frontend - Yew 0.21 Version
//! Security Asset Management Platform

use serde::{Serialize, Deserialize};
use gloo_net::http::Request;

// API 基础 URL - 使用 window.location 自动检测
// 在开发环境可以使用 localhost，在生产环境使用实际访问的地址
fn api_base_url() -> String {
    // 尝试从当前页面 URL 获取主机和端口
    if let Some(window) = web_sys::window() {
        if let Ok(href) = window.location().href() {
            // 如果是通过 HTTP 访问的，使用同源的 API 地址（默认端口 3003）
            if let Ok(host) = window.location().host() {
                // 将 8080 替换为 3003
                if host.contains(":8080") {
                    return host.replace(":8080", ":3003");
                }
                // 如果没有端口，添加 3003
                if !host.contains(':') {
                    return format!("{}:3003", host);
                }
                return host;
            }
        }
    }
    // 默认回退到 localhost
    "localhost:3003".to_string()
}

// 构建 API URL 的辅助函数
fn api_url(path: &str) -> String {
    format!("http://{}/api/{}", api_base_url(), path)
}

use shared::{
    Asset, NetworkZone, Task, TaskStatus,
    Risk, ZoneConfig, User, Role, LoginRequest, LoginResponse, AuditLog, CreateUserRequest,
    // Multi-Cloud types
    CloudAsset, CloudAssetStats, CloudProvider, VMStatus, BillingMode,
    CloudProviderConfigStatus, CloudProviderConfig,
    // Advanced Scanning types
    ScanStrategy, ScanEngine, AdvancedScanConfig, AdvancedScanTask,
    CreateAdvancedScanRequest, QuickScanResult, Permissions, PasswordPolicy,
    // Business Resource types
    BusinessResource, CreateBusinessResourceRequest, UpdateBusinessResourceRequest,
    // Cloud Service Asset types
    CloudServiceAsset, CloudServiceAssetQuery, CloudServiceAssetStats,
};
use wasm_bindgen_futures::spawn_local;
use web_sys::{InputEvent, Event, HtmlSelectElement, HtmlTextAreaElement, HtmlInputElement, Url, KeyboardEvent};
use yew::prelude::*;
use gloo_timers::callback::Timeout;
use js_sys::Array;

// ============== Page State ==============

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Page {
    Login,
    Dashboard,
    TaskCenter,
    AdvancedScanning, // 新增高级扫描页面
    RiskCenter,
    UserManagement,
    PermissionManagement, // 权限管理
    AuditLogs,
    CloudManagement, // 混合云管理
    CloudProviderManagement, // 云区对接管理
    UserProfile, // 个人中心
    PasswordPolicyManagement, // 密码策略管理
    BusinessAcceptance, // 业务受理（保留用于兼容性，实际使用子页面）
    BusinessApplication, // 业务申请
    OperationsManagement, // 运维管理（补充信息、审批）
    AutomationOrchestration, // 自动化资源编排（原运维交付）
    CloudServiceAssetManagement, // 云服务资产管理（统一纳管物理机+云虚拟机）
}

// ============== Auth State ==============

#[derive(Clone, Debug, PartialEq)]
pub struct AuthState {
    pub token: Option<String>,
    pub user: Option<User>,
}

// ============== Language ==============

#[derive(Clone, PartialEq, Eq)]
pub enum Language {
    Zh,
    En,
}

impl Language {
    fn t(&self, key: &str) -> String {
        match (self, key) {
            (Language::Zh, "login") => "登录".to_string(),
            (Language::En, "login") => "Login".to_string(),
            (Language::Zh, "username") => "用户名".to_string(),
            (Language::En, "username") => "Username".to_string(),
            (Language::Zh, "password") => "密码".to_string(),
            (Language::En, "password") => "Password".to_string(),
            (Language::Zh, "default_account_hint") => "默认管理员账户".to_string(),
            (Language::En, "default_account_hint") => "Default Administrator Account".to_string(),
            (Language::Zh, "change_password_hint") => "首次登录后请修改密码".to_string(),
            (Language::En, "change_password_hint") => "Please change password after first login".to_string(),
            (Language::Zh, "logout") => "退出登录".to_string(),
            (Language::En, "logout") => "Logout".to_string(),
            (Language::Zh, "user_management") => "👥 用户管理".to_string(),
            (Language::En, "user_management") => "👥 User Management".to_string(),
            (Language::Zh, "permission_management") => "🔑 权限管理".to_string(),
            (Language::En, "permission_management") => "🔑 Permission Management".to_string(),
            (Language::Zh, "password_policy_management") => "🔐 密码策略管理".to_string(),
            (Language::En, "password_policy_management") => "🔐 Password Policy".to_string(),
            (Language::Zh, "audit_logs") => "📜 审计日志".to_string(),
            (Language::En, "audit_logs") => "📜 Audit Logs".to_string(),
            (Language::Zh, "role") => "角色".to_string(),
            (Language::En, "role") => "Role".to_string(),
            (Language::Zh, "create_user") => "创建用户".to_string(),
            (Language::En, "create_user") => "Create User".to_string(),
            (Language::Zh, "general") => "通用".to_string(),
            (Language::En, "general") => "General".to_string(),
            (Language::Zh, "dashboard") => "📊 仪表盘".to_string(),
            (Language::En, "dashboard") => "📊 Dashboard".to_string(),
            (Language::Zh, "task_center") => "🚀 任务中心".to_string(),
            (Language::En, "task_center") => "🚀 Task Center".to_string(),
            (Language::Zh, "advanced_scanning") => "🔍 高级扫描".to_string(),
            (Language::En, "advanced_scanning") => "🔍 Advanced Scanning".to_string(),
            (Language::Zh, "create_scan") => "创建扫描".to_string(),
            (Language::En, "create_scan") => "Create Scan".to_string(),
            (Language::Zh, "scan_tasks") => "扫描任务".to_string(),
            (Language::En, "scan_tasks") => "Scan Tasks".to_string(),
            (Language::Zh, "scan_name") => "扫描名称".to_string(),
            (Language::En, "scan_name") => "Scan Name".to_string(),
            (Language::Zh, "targets") => "扫描目标".to_string(),
            (Language::En, "targets") => "Targets".to_string(),
            (Language::Zh, "strategy") => "扫描策略".to_string(),
            (Language::En, "strategy") => "Strategy".to_string(),
            (Language::Zh, "engine") => "扫描引擎".to_string(),
            (Language::En, "engine") => "Engine".to_string(),
            (Language::Zh, "start_scan") => "开始扫描".to_string(),
            (Language::En, "start_scan") => "Start Scan".to_string(),
            (Language::Zh, "scan_progress") => "扫描进度".to_string(),
            (Language::En, "scan_progress") => "Scan Progress".to_string(),
            (Language::Zh, "scan_results") => "扫描结果".to_string(),
            (Language::En, "scan_results") => "Scan Results".to_string(),
            (Language::Zh, "export_results") => "导出结果".to_string(),
            (Language::En, "export_results") => "Export Results".to_string(),
            (Language::Zh, "assets_risks") => "资产与风险".to_string(),
            (Language::En, "assets_risks") => "Assets & Risks".to_string(),
            (Language::Zh, "risk_monitoring") => "⚠️ 风险监控".to_string(),
            (Language::En, "risk_monitoring") => "⚠️ Risk Monitoring".to_string(),
            (Language::Zh, "verify") => "验证".to_string(),
            (Language::En, "verify") => "Verify".to_string(),
            (Language::Zh, "verified") => "已验证".to_string(),
            (Language::En, "verified") => "Verified".to_string(),
            (Language::Zh, "ignored") => "已忽略".to_string(),
            (Language::En, "ignored") => "Ignored".to_string(),
            (Language::Zh, "resolved") => "已解决".to_string(),
            (Language::En, "resolved") => "Resolved".to_string(),
            (Language::Zh, "all") => "全部".to_string(),
            (Language::En, "all") => "All".to_string(),
            (Language::Zh, "open") => "待处理".to_string(),
            (Language::En, "open") => "Open".to_string(),
            (Language::Zh, "total_assets") => "资产总数".to_string(),
            (Language::En, "total_assets") => "Total Assets".to_string(),
            (Language::Zh, "total_ports") => "端口总数".to_string(),
            (Language::En, "total_ports") => "Total Ports".to_string(),
            (Language::Zh, "unbound_ports") => "未绑定端口".to_string(),
            (Language::En, "unbound_ports") => "Unbound Ports".to_string(),
            (Language::Zh, "active_tasks") => "活跃任务".to_string(),
            (Language::En, "active_tasks") => "Active Tasks".to_string(),
            (Language::Zh, "system_status") => "系统状态".to_string(),
            (Language::En, "system_status") => "System Status".to_string(),
            (Language::Zh, "system_running_msg") => "系统运行正常".to_string(),
            (Language::En, "system_running_msg") => "System is running normally".to_string(),
            (Language::Zh, "create_new_task") => "创建新任务".to_string(),
            (Language::En, "create_new_task") => "Create New Task".to_string(),
            (Language::Zh, "task_name") => "任务名称".to_string(),
            (Language::En, "task_name") => "Task Name".to_string(),
            (Language::Zh, "target_ip_domain") => "目标 (IP/域名)".to_string(),
            (Language::En, "target_ip_domain") => "Target (IP/Domain)".to_string(),
            (Language::Zh, "start_task") => "开始任务".to_string(),
            (Language::En, "start_task") => "Start Task".to_string(),
            (Language::Zh, "task_list") => "任务列表".to_string(),
            (Language::En, "task_list") => "Task List".to_string(),
            (Language::Zh, "name") => "名称".to_string(),
            (Language::En, "name") => "Name".to_string(),
            (Language::Zh, "target") => "目标".to_string(),
            (Language::En, "target") => "Target".to_string(),
            (Language::Zh, "owner") => "责任人".to_string(),
            (Language::En, "owner") => "Owner".to_string(),
            (Language::Zh, "weight") => "权重".to_string(),
            (Language::En, "weight") => "Weight".to_string(),
            (Language::Zh, "labels") => "标签".to_string(),
            (Language::En, "labels") => "Labels".to_string(),
            (Language::Zh, "status") => "状态".to_string(),
            (Language::En, "status") => "Status".to_string(),
            (Language::Zh, "assets_found") => "发现资产".to_string(),
            (Language::En, "assets_found") => "Assets Found".to_string(),
            (Language::Zh, "risks_found") => "发现风险".to_string(),
            (Language::En, "risks_found") => "Risks Found".to_string(),
            (Language::Zh, "add_asset") => "添加资产".to_string(),
            (Language::En, "add_asset") => "Add Asset".to_string(),
            (Language::Zh, "ip") => "IP地址".to_string(),
            (Language::En, "ip") => "IP".to_string(),
            (Language::Zh, "zone") => "区域".to_string(),
            (Language::En, "zone") => "Zone".to_string(),
            (Language::Zh, "add") => "添加".to_string(),
            (Language::En, "add") => "Add".to_string(),
            (Language::Zh, "save") => "保存".to_string(),
            (Language::En, "save") => "Save".to_string(),
            (Language::Zh, "cancel") => "取消".to_string(),
            (Language::En, "cancel") => "Cancel".to_string(),
            (Language::Zh, "severity") => "严重程度".to_string(),
            (Language::En, "severity") => "Severity".to_string(),
            (Language::Zh, "asset") => "资产".to_string(),
            (Language::En, "asset") => "Asset".to_string(),
            (Language::Zh, "port") => "端口".to_string(),
            (Language::En, "port") => "Port".to_string(),
            (Language::Zh, "description") => "描述".to_string(),
            (Language::En, "description") => "Description".to_string(),
            (Language::Zh, "action") => "操作".to_string(),
            (Language::En, "action") => "Action".to_string(),
            (Language::Zh, "resolve") => "处置".to_string(),
            (Language::En, "resolve") => "Resolve".to_string(),
            (Language::Zh, "no_risks") => "暂无风险".to_string(),
            (Language::En, "no_risks") => "No risks detected yet".to_string(),
            (Language::Zh, "port_policy") => "端口扫描类型".to_string(),
            (Language::En, "port_policy") => "Port Policy".to_string(),
            (Language::Zh, "domain_brute") => "域名爆破".to_string(),
            (Language::En, "domain_brute") => "Domain Brute".to_string(),
            (Language::Zh, "service_detection") => "服务识别".to_string(),
            (Language::En, "service_detection") => "Service Detection".to_string(),
            (Language::Zh, "os_detection") => "操作系统识别".to_string(),
            (Language::En, "os_detection") => "OS Detection".to_string(),
            (Language::Zh, "site_identify") => "站点指纹识别".to_string(),
            (Language::En, "site_identify") => "Web Fingerprint".to_string(),
            (Language::Zh, "cidr") => "CIDR".to_string(),
            (Language::En, "cidr") => "CIDR".to_string(),
            (Language::Zh, "priority") => "优先级".to_string(),
            (Language::En, "priority") => "Priority".to_string(),
            (Language::Zh, "add_zone") => "添加区域".to_string(),
            (Language::En, "add_zone") => "Add Zone".to_string(),
            (Language::Zh, "delete") => "删除".to_string(),
            (Language::En, "delete") => "Delete".to_string(),
            (Language::Zh, "edit") => "编辑".to_string(),
            (Language::En, "edit") => "Edit".to_string(),
            (Language::Zh, "edit_asset") => "编辑资产".to_string(),
            (Language::En, "edit_asset") => "Edit Asset".to_string(),
            (Language::Zh, "edit_task") => "编辑任务".to_string(),
            (Language::En, "edit_task") => "Edit Task".to_string(),
            (Language::Zh, "edit_zone") => "编辑区域".to_string(),
            (Language::En, "edit_zone") => "Edit Zone".to_string(),
            (Language::Zh, "ports") => "端口".to_string(),
            (Language::En, "ports") => "Ports".to_string(),
            (Language::Zh, "service") => "服务".to_string(),
            (Language::En, "service") => "Service".to_string(),
            (Language::Zh, "add_port") => "添加端口".to_string(),
            (Language::En, "add_port") => "Add Port".to_string(),
            (Language::Zh, "save_port") => "保存端口".to_string(),
            (Language::En, "save_port") => "Save Port".to_string(),
            (Language::Zh, "timestamp") => "时间戳".to_string(),
            (Language::En, "timestamp") => "Timestamp".to_string(),
            (Language::Zh, "details") => "详情".to_string(),
            (Language::En, "details") => "Details".to_string(),
            (Language::Zh, "login_failed") => "登录失败".to_string(),
            (Language::En, "login_failed") => "Login Failed".to_string(),
            (Language::Zh, "access_denied") => "访问拒绝".to_string(),
            (Language::En, "access_denied") => "Access Denied".to_string(),

            // User Profile
            (Language::Zh, "user_profile") => "👤 个人中心".to_string(),
            (Language::En, "user_profile") => "👤 Profile".to_string(),
            (Language::Zh, "profile_info") => "个人信息".to_string(),
            (Language::En, "profile_info") => "Profile Information".to_string(),
            (Language::Zh, "change_password") => "修改密码".to_string(),
            (Language::En, "change_password") => "Change Password".to_string(),
            (Language::Zh, "current_password") => "当前密码".to_string(),
            (Language::En, "current_password") => "Current Password".to_string(),
            (Language::Zh, "new_password") => "新密码".to_string(),
            (Language::En, "new_password") => "New Password".to_string(),
            (Language::Zh, "confirm_password") => "确认密码".to_string(),
            (Language::En, "confirm_password") => "Confirm Password".to_string(),
            (Language::Zh, "password_updated") => "密码修改成功".to_string(),
            (Language::En, "password_updated") => "Password updated successfully".to_string(),
            (Language::Zh, "password_mismatch") => "两次密码不一致".to_string(),
            (Language::En, "password_mismatch") => "Passwords do not match".to_string(),
            (Language::Zh, "wrong_current_password") => "当前密码错误".to_string(),
            (Language::En, "wrong_current_password") => "Current password is incorrect".to_string(),
            (Language::Zh, "created_at") => "创建时间".to_string(),
            (Language::En, "created_at") => "Created At".to_string(),
            (Language::Zh, "last_login") => "最后登录".to_string(),
            (Language::En, "last_login") => "Last Login".to_string(),

            // Password Policy Management
            (Language::Zh, "min_length") => "最小长度".to_string(),
            (Language::En, "min_length") => "Minimum Length".to_string(),
            (Language::Zh, "require_uppercase") => "需要大写字母".to_string(),
            (Language::En, "require_uppercase") => "Require Uppercase".to_string(),
            (Language::Zh, "require_lowercase") => "需要小写字母".to_string(),
            (Language::En, "require_lowercase") => "Require Lowercase".to_string(),
            (Language::Zh, "require_number") => "需要数字".to_string(),
            (Language::En, "require_number") => "Require Number".to_string(),
            (Language::Zh, "require_special") => "需要特殊字符".to_string(),
            (Language::En, "require_special") => "Require Special Character".to_string(),
            (Language::Zh, "max_age_days") => "密码最大有效期（天）".to_string(),
            (Language::En, "max_age_days") => "Password Max Age (Days)".to_string(),
            (Language::Zh, "prevent_reuse") => "防止重用最近N次密码".to_string(),
            (Language::En, "prevent_reuse") => "Prevent Reusing Last N Passwords".to_string(),
            (Language::Zh, "min_strength") => "最低强度要求".to_string(),
            (Language::En, "min_strength") => "Minimum Strength".to_string(),
            (Language::Zh, "weak") => "弱".to_string(),
            (Language::En, "weak") => "Weak".to_string(),
            (Language::Zh, "medium") => "中".to_string(),
            (Language::En, "medium") => "Medium".to_string(),
            (Language::Zh, "strong") => "强".to_string(),
            (Language::En, "strong") => "Strong".to_string(),
            (Language::Zh, "save_policy") => "保存策略".to_string(),
            (Language::En, "save_policy") => "Save Policy".to_string(),
            (Language::Zh, "cancel") => "取消".to_string(),
            (Language::En, "cancel") => "Cancel".to_string(),
            (Language::Zh, "policy_saved") => "策略保存成功".to_string(),
            (Language::En, "policy_saved") => "Policy saved successfully".to_string(),
            (Language::Zh, "policy_error") => "策略保存失败".to_string(),
            (Language::En, "policy_error") => "Failed to save policy".to_string(),

            // Account Lockout Policy
            (Language::Zh, "account_lockout") => "账户锁定策略".to_string(),
            (Language::En, "account_lockout") => "Account Lockout".to_string(),
            (Language::Zh, "max_login_attempts") => "最大登录失败次数".to_string(),
            (Language::En, "max_login_attempts") => "Max Login Attempts".to_string(),
            (Language::Zh, "lockout_duration_minutes") => "锁定时长（分钟）".to_string(),
            (Language::En, "lockout_duration_minutes") => "Lockout Duration (Minutes)".to_string(),
            (Language::Zh, "unlimited") => "不限制".to_string(),
            (Language::En, "unlimited") => "Unlimited".to_string(),
            (Language::Zh, "no_limit") => "无限制".to_string(),
            (Language::En, "no_limit") => "No Limit".to_string(),

            // Multi-Cloud Management
            (Language::Zh, "cloud_management") => "☁️ 混合云管理".to_string(),
            (Language::En, "cloud_management") => "☁️ Multi-Cloud".to_string(),
            (Language::Zh, "cloud_provider_management") => "🌐 云区对接管理".to_string(),
            (Language::En, "cloud_provider_management") => "🌐 Cloud Providers".to_string(),
            (Language::Zh, "asset_name") => "资产名称".to_string(),
            (Language::En, "asset_name") => "Asset Name".to_string(),
            (Language::Zh, "spec") => "规格".to_string(),
            (Language::En, "spec") => "Spec".to_string(),
            (Language::Zh, "system_disk") => "系统盘".to_string(),
            (Language::En, "system_disk") => "System Disk".to_string(),
            (Language::Zh, "cloud_region") => "云区".to_string(),
            (Language::En, "cloud_region") => "Cloud Region".to_string(),
            (Language::Zh, "ip_address") => "IP地址".to_string(),
            (Language::En, "ip_address") => "IP Address".to_string(),
            (Language::Zh, "billing_mode") => "计费模式".to_string(),
            (Language::En, "billing_mode") => "Billing Mode".to_string(),
            (Language::Zh, "expire_time") => "到期时间".to_string(),
            (Language::En, "expire_time") => "Expire Time".to_string(),
            (Language::Zh, "vm_status") => "状态".to_string(),
            (Language::En, "vm_status") => "Status".to_string(),
            (Language::Zh, "os") => "操作系统".to_string(),
            (Language::En, "os") => "OS".to_string(),
            (Language::Zh, "department") => "部门".to_string(),
            (Language::En, "department") => "Department".to_string(),
            (Language::Zh, "project") => "项目".to_string(),
            (Language::En, "project") => "Project".to_string(),
            (Language::Zh, "cloud_owner") => "负责人".to_string(),
            (Language::En, "cloud_owner") => "Owner".to_string(),
            (Language::Zh, "vm_created_at") => "创建时间".to_string(),
            (Language::En, "vm_created_at") => "Created".to_string(),
            (Language::Zh, "image_id") => "镜像ID".to_string(),
            (Language::En, "image_id") => "Image ID".to_string(),
            (Language::Zh, "disk_total") => "云盘总量".to_string(),
            (Language::En, "disk_total") => "Disk Total".to_string(),
            (Language::Zh, "disk_count") => "云盘数量".to_string(),
            (Language::En, "disk_count") => "Disks".to_string(),
            (Language::Zh, "has_snapshot") => "快照".to_string(),
            (Language::En, "has_snapshot") => "Snapshot".to_string(),
            (Language::Zh, "public_ip") => "公网IP".to_string(),
            (Language::En, "public_ip") => "Public IP".to_string(),
            (Language::Zh, "private_ip") => "内网IP".to_string(),
            (Language::En, "private_ip") => "Private IP".to_string(),
            (Language::Zh, "cpu_memory") => "CPU/内存".to_string(),
            (Language::En, "cpu_memory") => "CPU/Memory".to_string(),
            (Language::Zh, "running") => "运行中".to_string(),
            (Language::En, "running") => "Running".to_string(),
            (Language::Zh, "stopped") => "已停止".to_string(),
            (Language::En, "stopped") => "Stopped".to_string(),
            (Language::Zh, "pay_as_you_go") => "按量付费".to_string(),
            (Language::En, "pay_as_you_go") => "Pay-As-You-Go".to_string(),
            (Language::Zh, "subscription") => "包年包月".to_string(),
            (Language::En, "subscription") => "Subscription".to_string(),
            (Language::Zh, "yes") => "是".to_string(),
            (Language::En, "yes") => "Yes".to_string(),
            (Language::Zh, "no") => "否".to_string(),
            (Language::En, "no") => "No".to_string(),
            (Language::Zh, "no_cloud_assets") => "暂无云资产".to_string(),
            (Language::En, "no_cloud_assets") => "No cloud assets yet".to_string(),
            (Language::Zh, "add_cloud_asset") => "添加云资产".to_string(),
            (Language::En, "add_cloud_asset") => "Add Cloud Asset".to_string(),
            (Language::Zh, "sync_cloud_assets") => "同步云资产".to_string(),
            (Language::En, "sync_cloud_assets") => "Sync Assets".to_string(),
            (Language::Zh, "expiring_soon") => "即将到期".to_string(),
            (Language::En, "expiring_soon") => "Expiring Soon".to_string(),
            // Business Acceptance translations
            (Language::Zh, "business_acceptance") => "📋 业务受理".to_string(),
            (Language::En, "business_acceptance") => "📋 Business Acceptance".to_string(),
            (Language::Zh, "business_application") => "📝 业务申请".to_string(),
            (Language::En, "business_application") => "📝 Business Application".to_string(),
            (Language::Zh, "operations_management") => "🔧 运维管理".to_string(),
            (Language::En, "operations_management") => "🔧 Operations".to_string(),
            (Language::Zh, "automation_orchestration") => "⚙️ 自动化资源编排".to_string(),
            (Language::En, "automation_orchestration") => "⚙️ Automation Orchestration".to_string(),
            (Language::Zh, "cloud_service_asset_management") => "🖥️ 云服务资产".to_string(),
            (Language::En, "cloud_service_asset_management") => "🖥️ Cloud Assets".to_string(),
            (Language::Zh, "ecs_name") => "ECS名称".to_string(),
            (Language::En, "ecs_name") => "ECS Name".to_string(),
            (Language::Zh, "ecs_status") => "ECS状态".to_string(),
            (Language::En, "ecs_status") => "ECS Status".to_string(),
            (Language::Zh, "resource_id") => "资源ID".to_string(),
            (Language::En, "resource_id") => "Resource ID".to_string(),
            (Language::Zh, "cloud_region") => "云区域".to_string(),
            (Language::En, "cloud_region") => "Cloud Region".to_string(),
            (Language::Zh, "cloud_category") => "云类别".to_string(),
            (Language::En, "cloud_category") => "Cloud Category".to_string(),
            (Language::Zh, "county_city") => "县市区".to_string(),
            (Language::En, "county_city") => "County/City".to_string(),
            (Language::Zh, "vdc_name") => "VDC名称".to_string(),
            (Language::En, "vdc_name") => "VDC Name".to_string(),
            (Language::Zh, "customer_name") => "客户名称".to_string(),
            (Language::En, "customer_name") => "Customer Name".to_string(),
            (Language::Zh, "application_name") => "应用名称".to_string(),
            (Language::En, "application_name") => "Application Name".to_string(),
            (Language::Zh, "contract_name") => "合同名称".to_string(),
            (Language::En, "contract_name") => "Contract Name".to_string(),
            (Language::Zh, "instance_id") => "实例ID".to_string(),
            (Language::En, "instance_id") => "Instance ID".to_string(),
            (Language::Zh, "ecs_type") => "ECS类型".to_string(),
            (Language::En, "ecs_type") => "ECS Type".to_string(),
            (Language::Zh, "ecs_os") => "ECS操作系统".to_string(),
            (Language::En, "ecs_os") => "ECS OS".to_string(),
            (Language::Zh, "cpu_cores") => "CPU核数".to_string(),
            (Language::En, "cpu_cores") => "CPU Cores".to_string(),
            (Language::Zh, "memory_gb") => "内存(GB)".to_string(),
            (Language::En, "memory_gb") => "Memory (GB)".to_string(),
            (Language::Zh, "system_disk") => "系统盘".to_string(),
            (Language::En, "system_disk") => "System Disk".to_string(),
            (Language::Zh, "system_disk_size_gb") => "系统盘大小(GB)".to_string(),
            (Language::En, "system_disk_size_gb") => "System Disk Size (GB)".to_string(),
            (Language::Zh, "data_disk") => "数据盘".to_string(),
            (Language::En, "data_disk") => "Data Disk".to_string(),
            (Language::Zh, "completion_time") => "完成时间".to_string(),
            (Language::En, "completion_time") => "Completion Time".to_string(),
            (Language::Zh, "release_time") => "释放时间".to_string(),
            (Language::En, "release_time") => "Release Time".to_string(),
            (Language::Zh, "has_security_product") => "安全产品".to_string(),
            (Language::En, "has_security_product") => "Security Product".to_string(),
            (Language::Zh, "ip_address") => "IP地址".to_string(),
            (Language::En, "ip_address") => "IP Address".to_string(),
            (Language::Zh, "ecs_login_method") => "登录方式".to_string(),
            (Language::En, "ecs_login_method") => "Login Method".to_string(),
            (Language::Zh, "ecs_login_username") => "登录用户名".to_string(),
            (Language::En, "ecs_login_username") => "Login Username".to_string(),
            (Language::Zh, "ecs_initial_password") => "初始密码".to_string(),
            (Language::En, "ecs_initial_password") => "Initial Password".to_string(),
            (Language::Zh, "bastion_address") => "堡垒机地址".to_string(),
            (Language::En, "bastion_address") => "Bastion Address".to_string(),
            (Language::Zh, "bastion_admin_account") => "堡垒机账号".to_string(),
            (Language::En, "bastion_admin_account") => "Bastion Account".to_string(),
            (Language::Zh, "bastion_initial_password") => "堡垒机密码".to_string(),
            (Language::En, "bastion_initial_password") => "Bastion Password".to_string(),
            (Language::Zh, "remarks") => "备注".to_string(),
            (Language::En, "remarks") => "Remarks".to_string(),
            (Language::Zh, "running") => "运行中".to_string(),
            (Language::En, "running") => "Running".to_string(),
            (Language::Zh, "stopped") => "已停止".to_string(),
            (Language::En, "stopped") => "Stopped".to_string(),
            (Language::Zh, "released") => "已释放".to_string(),
            (Language::En, "released") => "Released".to_string(),
            (Language::Zh, "add_business_resource") => "添加业务资源".to_string(),
            (Language::En, "add_business_resource") => "Add Business Resource".to_string(),
            (Language::Zh, "edit") => "编辑".to_string(),
            (Language::En, "edit") => "Edit".to_string(),
            (Language::Zh, "delete") => "删除".to_string(),
            (Language::En, "delete") => "Delete".to_string(),
            (Language::Zh, "save") => "保存".to_string(),
            (Language::En, "save") => "Save".to_string(),
            (Language::Zh, "cancel") => "取消".to_string(),
            (Language::En, "cancel") => "Cancel".to_string(),
            (Language::Zh, "export") => "导出".to_string(),
            (Language::En, "export") => "Export".to_string(),
            (Language::Zh, "public_cloud") => "公有云".to_string(),
            (Language::En, "public_cloud") => "Public Cloud".to_string(),
            (Language::Zh, "private_cloud") => "私有云".to_string(),
            (Language::En, "private_cloud") => "Private Cloud".to_string(),
            (Language::Zh, "hybrid_cloud") => "混合云".to_string(),
            (Language::En, "hybrid_cloud") => "Hybrid Cloud".to_string(),
            (Language::Zh, "actions") => "操作".to_string(),
            (Language::En, "actions") => "Actions".to_string(),

            _ => key.to_string(),
        }
    }
}

// ============== Context Providers ==============

// We'll use a simpler approach with callback props for Yew

// Helper function to get token from localStorage
fn get_auth_token() -> String {
    match web_sys::window() {
        Some(window) => match window.local_storage() {
            Ok(Some(storage)) => match storage.get_item("auth_token") {
                Ok(token) => token.unwrap_or_default(),
                _ => String::new(),
            },
            _ => String::new(),
        },
        _ => String::new(),
    }
}

// Helper function to get user from localStorage
fn get_auth_user() -> Option<User> {
    match web_sys::window() {
        Some(window) => match window.local_storage() {
            Ok(Some(storage)) => match storage.get_item("auth_user") {
                Ok(Some(user_str)) => serde_json::from_str::<LoginResponse>(&user_str).ok().map(|r| r.user),
                _ => None,
            },
            _ => None,
        },
        _ => None,
    }
}

// Helper function to get user role
fn get_user_role() -> Option<Role> {
    get_auth_user().map(|user| user.role)
}

// Helper function to get user permissions
fn get_user_permissions() -> Option<Permissions> {
    get_auth_user().and_then(|user| user.permissions)
}

// Helper function to set auth in localStorage
fn set_auth(token: &str, user_str: &str) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.set_item("auth_token", token);
            let _ = storage.set_item("auth_user", user_str);
        }
    }
}

// Helper function to clear auth from localStorage
fn clear_auth() {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.remove_item("auth_token");
            let _ = storage.remove_item("auth_user");
        }
    }
}

// ============== Login Component ==============

#[derive(Properties, PartialEq)]
pub struct LoginProps {
    pub current_page: UseStateHandle<Page>,
}

#[function_component]
fn Login(props: &LoginProps) -> Html {
    let current_page = props.current_page.clone();
    let username = use_state(|| String::new());
    let password = use_state(|| String::new());
    let error_msg = use_state(|| None::<String>);
    let loading = use_state(|| false);
    let lang = use_state(|| Language::Zh);

    // We need a way to manage auth state - in Yew we'll use a callback
    let on_submit = {
        let username = username.clone();
        let password = password.clone();
        let error_msg = error_msg.clone();
        let loading = loading.clone();
        let current_page = current_page.clone();

        Callback::from(move |_| {
            let username_val = (*username).clone();
            let password_val = (*password).clone();
            let error_msg = error_msg.clone();
            let loading = loading.clone();
            let current_page = current_page.clone();

            spawn_local(async move {
                loading.set(true);
                error_msg.set(None);

                let req = LoginRequest {
                    username: username_val,
                    password: password_val,
                };

                let json = serde_json::to_string(&req).unwrap();
                let http_req = Request::post(&api_url("login"))
                    .header("Content-Type", "application/json")
                    .body(json)
                    .unwrap();
                let resp = http_req.send().await;

                match resp {
                    Ok(response) if response.ok() => {
                        let text = response.text().await.unwrap_or_default();
                        gloo_console::log!("Login response text:", &text);
                        if let Ok(login_resp) = serde_json::from_str::<LoginResponse>(&text) {
                            // Store token in localStorage for simplicity
                            set_auth(&login_resp.token, &text);
                            current_page.set(Page::Dashboard);
                        } else {
                            error_msg.set(Some("JSON Parse Error".to_string()));
                            loading.set(false);
                        }
                    }
                    Ok(response) => {
                        let status = response.status();
                        gloo_console::error!("Login failed with status:", status as i32);
                        error_msg.set(Some(format!("Login failed: {}", status)));
                        loading.set(false);
                    }
                    Err(e) => {
                        gloo_console::error!("Network Error:", e.to_string());
                        error_msg.set(Some(format!("Network Error: {}", e)));
                        loading.set(false);
                    }
                }
            });
        })
    };

    let on_username_input = {
        let username = username.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            username.set(input.value());
        })
    };

    let on_password_input = {
        let password = password.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            password.set(input.value());
        })
    };

    let toggle_lang = {
        let lang = lang.clone();
        Callback::from(move |_| {
            lang.set(if *lang == Language::Zh {
                Language::En
            } else {
                Language::Zh
            });
        })
    };

    let loading_class = if *loading { " is-loading" } else { "" };

    html! {
        <section class="hero is-fullheight is-light">
            <div class="hero-body">
                <div class="container">
                    <div class="columns is-centered">
                        <div class="column is-5-tablet is-4-desktop is-3-widescreen">
                            <div class="has-text-centered mb-5">
                                <h1 class="title is-2">{ "RustSet" }</h1>
                                <button class="button is-small is-white" onclick={toggle_lang}>
                                    { if *lang == Language::Zh { "中文" } else { "English" } }
                                </button>
                            </div>
                            <div class="box">
                                <h3 class="title has-text-centered">{ lang.t("login") }</h3>
                                <div>
                                    <div class="field">
                                        <label class="label">{ lang.t("username") }</label>
                                        <div class="control">
                                            <input
                                                class="input"
                                                type="text"
                                                placeholder="e.g. admin"
                                                value={(*username).clone()}
                                                oninput={on_username_input}
                                            />
                                        </div>
                                    </div>
                                    <div class="field">
                                        <label class="label">{ lang.t("password") }</label>
                                        <div class="control">
                                            <input
                                                class="input"
                                                type="password"
                                                placeholder="*******"
                                                value={(*password).clone()}
                                                oninput={on_password_input}
                                            />
                                        </div>
                                    </div>
                                    if let Some(msg) = (*error_msg).as_ref() {
                                        <div class="notification is-danger is-light">
                                            { msg.clone() }
                                        </div>
                                    }
                                    <div class="field">
                                        <button
                                            class={format!("button is-primary is-fullwidth{}", loading_class)}
                                            onclick={on_submit}
                                            disabled={*loading}
                                        >
                                            { lang.t("login") }
                                        </button>
                                    </div>
                                    <div class="content mt-4 has-text-centered has-text-grey is-size-7">
                                        <p>{ lang.t("default_account_hint") }</p>
                                        <p class="is-family-monospace">{ "admin / admin" }</p>
                                        <p class="has-text-warning-dark mt-2">{ lang.t("change_password_hint") }</p>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </section>
    }
}

// ============== Sidebar Component ==============

#[derive(Properties, PartialEq)]
pub struct SidebarProps {
    pub current_page: UseStateHandle<Page>,
}

#[function_component]
fn Sidebar(props: &SidebarProps) -> Html {
    let current_page = props.current_page.clone();
    let lang = use_state(|| Language::Zh);

    // Get user from localStorage
    let user_role = get_auth_user().map(|u| u.role);
    let user_permissions = get_auth_user().and_then(|u| u.permissions);

    // 实时从后端获取最新权限
    let latest_permissions = use_state(|| None as Option<Permissions>);
    let loading_permissions = use_state(|| false);

    // 获取 token
    let token = get_auth_token();

    // 使用 effect 在组件挂载时获取最新权限
    let latest_permissions_clone = latest_permissions.clone();
    let loading_permissions_clone = loading_permissions.clone();
    let token_clone = token.clone();

    use_effect_with((), move |_| {
        let token = token_clone.clone();
        if !token.is_empty() {
            loading_permissions_clone.set(true);
            spawn_local(async move {
                match Request::get(&api_url("users/me"))
                    .header("Authorization", &token)
                    .send()
                    .await
                {
                    Ok(resp) if resp.ok() => {
                        if let Ok(updated_user) = resp.json::<User>().await {
                            // 更新 localStorage
                            if let Ok(user_str) = serde_json::to_string(&updated_user) {
                                let login_response = LoginResponse {
                                    token: token.clone(),
                                    user: updated_user.clone(),
                                };
                                if let Ok(login_str) = serde_json::to_string(&login_response) {
                                    set_auth(&token, &login_str);
                                }
                            }
                            // 更新权限状态
                            latest_permissions_clone.set(updated_user.permissions);
                        }
                    }
                    _ => {}
                }
                loading_permissions_clone.set(false);
            });
        }
        || ()
    });

    // 使用实时获取的权限，如果没有则使用缓存的
    let effective_permissions = latest_permissions.as_ref().or(user_permissions.as_ref());

    // 调试：输出用户角色和权限信息
    web_sys::console::log_1(&format!("🔍 侧边栏调试 - user_role: {:?}", user_role).into());
    if let Some(perms) = effective_permissions {
        web_sys::console::log_1(&format!("🔍 侧边栏调试 - 权限: can_access_general={}, can_access_assets_risks={}, can_access_cloud={}, can_access_audit={}",
            perms.can_access_general, perms.can_access_assets_risks, perms.can_access_cloud, perms.can_access_audit).into());
    } else {
        web_sys::console::log_1(&"⚠️ 侧边栏调试 - user_permissions is None!".into());
    }

    let toggle_lang = {
        let lang = lang.clone();
        Callback::from(move |_| {
            lang.set(if *lang == Language::Zh {
                Language::En
            } else {
                Language::Zh
            });
        })
    };

    let on_logout: Callback<Event> = {
        let current_page = current_page.clone();
        Callback::from(move |_: Event| {
            clear_auth();
            current_page.set(Page::Login);
        })
    };

    let navigate = |page: Page| -> Callback<MouseEvent> {
        let current_page = current_page.clone();
        Callback::from(move |_| current_page.set(page))
    };

    html! {
        <aside class="menu p-4" style="height: 100vh; background-color: #f5f5f5; overflow-y: auto; position: sticky; top: 0;">
            <div class="level is-mobile mb-4">
                <div class="level-left">
                    <h1 class="title is-4">{ "RustSet" }</h1>
                </div>
                <div class="level-right">
                    <button class="button is-small is-white" onclick={toggle_lang}>
                        { if *lang == Language::Zh { "中文" } else { "English" } }
                    </button>
                </div>
            </div>
            <div class="level is-mobile">
                <div class="level-left">
                    <p class="menu-label">{ lang.t("general") }</p>
                </div>
            </div>
            <ul class="menu-list">
                <li><a onclick={navigate(Page::Dashboard)}>{ lang.t("dashboard") }</a></li>
                // 通用模块 - 使用顶级权限 can_access_general
                if user_role == Some(Role::SecAdmin) || user_role == Some(Role::Auditor) || effective_permissions.map(|p| p.can_access_general).unwrap_or(false) {
                    <li><a onclick={navigate(Page::TaskCenter)}>{ lang.t("task_center") }</a></li>
                    <li><a onclick={navigate(Page::AdvancedScanning)}>{ lang.t("advanced_scanning") }</a></li>
                }
                // 风险监控 - 使用权限 can_view_risks
                if user_role == Some(Role::SecAdmin) || effective_permissions.map(|p| p.can_view_risks).unwrap_or(false) {
                    <li><a onclick={navigate(Page::RiskCenter)}>{ lang.t("risk_monitoring") }</a></li>
                }
            </ul>
            // 业务流程 - 使用顶级权限 can_access_assets_risks
            if user_role == Some(Role::SecAdmin) || effective_permissions.map(|p| p.can_view_business_process).unwrap_or(false) {
                <p class="menu-label">{ "业务流程" }</p>
                <ul class="menu-list">
                    <li><a onclick={navigate(Page::BusinessApplication)}>{ lang.t("business_application") }</a></li>
                    <li><a onclick={navigate(Page::OperationsManagement)}>{ lang.t("operations_management") }</a></li>
                    <li><a onclick={navigate(Page::AutomationOrchestration)}>{ lang.t("automation_orchestration") }</a></li>
                </ul>
            }
            // 云管理模块 - 使用顶级权限 can_access_cloud
            if user_role == Some(Role::SecAdmin) || effective_permissions.map(|p| p.can_access_cloud).unwrap_or(false) {
                <p class="menu-label">{ "云管理" }</p>
                <ul class="menu-list">
                    if user_role == Some(Role::SecAdmin) || effective_permissions.map(|p| p.can_view_cloud_providers).unwrap_or(false) {
                        <li><a onclick={navigate(Page::CloudProviderManagement)}>{ lang.t("cloud_provider_management") }</a></li>
                    }
                    if user_role == Some(Role::SecAdmin) || effective_permissions.map(|p| p.can_view_cloud_management).unwrap_or(false) {
                        <li><a onclick={navigate(Page::CloudManagement)}>{ lang.t("cloud_management") }</a></li>
                    }
                    if user_role == Some(Role::SecAdmin) || effective_permissions.map(|p| p.can_view_cloud_assets).unwrap_or(false) {
                        <li><a onclick={navigate(Page::CloudServiceAssetManagement)}>{ lang.t("cloud_service_asset_management") }</a></li>
                    }

                </ul>
            }
            // 用户管理模块 - 使用顶级权限 can_access_user_management
            if user_role == Some(Role::SecAdmin) || effective_permissions.map(|p| p.can_access_user_management).unwrap_or(false) {
                <p class="menu-label">{ lang.t("user_management") }</p>
                <ul class="menu-list">
                    if user_role == Some(Role::SecAdmin) || effective_permissions.map(|p| p.can_view_users).unwrap_or(false) {
                        <li><a onclick={navigate(Page::UserManagement)}>{ lang.t("user_management") }</a></li>
                    }
                    if user_role == Some(Role::SecAdmin) || effective_permissions.map(|p| p.can_manage_permissions).unwrap_or(false) {
                        <li><a onclick={navigate(Page::PermissionManagement)}>{ lang.t("permission_management") }</a></li>
                    }
                    if user_role == Some(Role::SecAdmin) || effective_permissions.map(|p| p.can_view_password_policy).unwrap_or(false) {
                        <li><a onclick={navigate(Page::PasswordPolicyManagement)}>{ lang.t("password_policy_management") }</a></li>
                    }
                </ul>
            }
            // 审计模块 - 使用顶级权限 can_access_audit
            if user_role == Some(Role::Auditor) || effective_permissions.map(|p| p.can_access_audit).unwrap_or(false) {
                <p class="menu-label">{ lang.t("audit_logs") }</p>
                <ul class="menu-list">
                    if user_role == Some(Role::Auditor) || effective_permissions.map(|p| p.can_view_audit_logs).unwrap_or(false) {
                        <li><a onclick={navigate(Page::AuditLogs)}>{ lang.t("audit_logs") }</a></li>
                    }
                </ul>
            }
            <p class="menu-label">{ "Account" }</p>
            <ul class="menu-list">
                <li><a onclick={navigate(Page::UserProfile)}>{ lang.t("user_profile") }</a></li>
            </ul>
        </aside>
    }
}

// ============== Dashboard Component ==============

#[function_component]
fn Dashboard() -> Html {
    let lang = use_state(|| Language::Zh);
    let stats = use_state(|| (0, 0, 0, 0));
    let loading = use_state(|| true);

    // Get token from localStorage
    let token = get_auth_token();

    // Use effect to fetch data only once on mount
    use_effect_with((), {
        let stats = stats.clone();
        let loading = loading.clone();
        let token = token.clone();

        move |_| {
            spawn_local(async move {
                loading.set(true);

                let assets_req = Request::get(&api_url("assets"))
                    .header("Authorization", &token)
                    .send()
                    .await;
                let tasks_req = Request::get(&api_url("tasks"))
                    .header("Authorization", &token)
                    .send()
                    .await;

                let mut asset_count = 0;
                let mut port_count = 0;
                let mut unbound_count = 0;
                let mut task_count = 0;

                if let Ok(resp) = assets_req {
                    if let Ok(assets) = resp.json::<Vec<Asset>>().await {
                        asset_count = assets.len();
                        port_count = assets.iter().map(|a| a.ports.len()).sum();
                        unbound_count = assets.iter().map(|a| a.ports.iter().filter(|p| !p.is_bound).count()).sum();
                    }
                }

                if let Ok(resp) = tasks_req {
                    if let Ok(tasks) = resp.json::<Vec<Task>>().await {
                        task_count = tasks.len();
                    }
                }

                stats.set((asset_count, port_count, unbound_count, task_count));
                loading.set(false);
            });
            || ()
        }
    });

    let (total_assets, total_ports, unbound_ports, total_tasks) = *stats;

    html! {
        <div class="container p-4">
            <h1 class="title">{ lang.t("dashboard") }</h1>
            <div class="columns is-multiline">
                <div class="column is-3">
                    <div class="box has-background-info-light">
                        <div class="heading">{ lang.t("total_assets") }</div>
                        <div class="title">{ total_assets }</div>
                    </div>
                </div>
                <div class="column is-3">
                    <div class="box has-background-primary-light">
                        <div class="heading">{ lang.t("total_ports") }</div>
                        <div class="title">{ total_ports }</div>
                    </div>
                </div>
                <div class="column is-3">
                    <div class="box has-background-danger-light">
                        <div class="heading">{ lang.t("unbound_ports") }</div>
                        <div class="title has-text-danger">{ unbound_ports }</div>
                    </div>
                </div>
                <div class="column is-3">
                    <div class="box has-background-warning-light">
                        <div class="heading">{ lang.t("active_tasks") }</div>
                        <div class="title">{ total_tasks }</div>
                    </div>
                </div>
            </div>
            <div class="box">
                <h2 class="subtitle">{ lang.t("system_status") }</h2>
                <p>{ lang.t("system_running_msg") }</p>
            </div>
        </div>
    }
}

// ============== Task Center Component ==============

#[function_component]
fn TaskCenter() -> Html {
    let lang = use_state(|| Language::Zh);
    let tasks = use_state(|| Vec::new());
    let loading = use_state(|| true);

    // Get token from localStorage
    let token = get_auth_token();

    // Use effect to fetch data only once on mount
    use_effect_with((), {
        let tasks = tasks.clone();
        let loading = loading.clone();
        let token = token.clone();

        move |_| {
            spawn_local(async move {
                if let Ok(resp) = Request::get(&api_url("tasks")).header("Authorization", &token).send().await {
                    if let Ok(data) = resp.json::<Vec<Task>>().await {
                        tasks.set(data);
                    }
                }
                loading.set(false);
            });
            || ()
        }
    });

    let status_class = |status: &TaskStatus| -> &'static str {
        match status {
            TaskStatus::Pending => "is-warning",
            TaskStatus::Running => "is-info",
            TaskStatus::Completed => "is-success",
            TaskStatus::Failed => "is-danger",
        }
    };

    let status_to_lowercase = |status: &TaskStatus| -> String {
        format!("{:?}", status).to_lowercase()
    };

    html! {
        <div class="container p-4">
            <h1 class="title">{ lang.t("task_center") }</h1>
            <div class="box">
                if (*tasks).is_empty() && *loading {
                    <p>{ "Loading..." }</p>
                } else if (*tasks).is_empty() {
                    <p>{ "No tasks yet" }</p>
                } else {
                    <table class="table is-fullwidth is-hoverable">
                        <thead>
                            <tr>
                                <th>{ lang.t("name") }</th>
                                <th>{ lang.t("target") }</th>
                                <th>{ lang.t("status") }</th>
                                <th>{ lang.t("assets_found") }</th>
                                <th>{ lang.t("risks_found") }</th>
                            </tr>
                        </thead>
                        <tbody>
                            { for tasks.iter().map(|task| {
                                let status_str = format!("{:?}", task.status);
                                html! {
                                    <tr>
                                        <td>{ &task.name }</td>
                                        <td><code>{ &task.target }</code></td>
                                        <td>
                                            <span class={classes!(status_class(&task.status), format!("status-{}", status_to_lowercase(&task.status)))}>
                                                { status_str }
                                            </span>
                                        </td>
                                        <td>{ task.found_assets }</td>
                                        <td>{ task.found_risks }</td>
                                    </tr>
                                }
                            })}
                        </tbody>
                    </table>
                }
            </div>
        </div>
    }
}

// ============== Risk Center Component ==============

#[function_component]
fn RiskCenter() -> Html {
    let lang = use_state(|| Language::Zh);
    let risks = use_state(|| Vec::new());
    let loading = use_state(|| true);

    // Get token from localStorage
    let token = get_auth_token();

    // Use effect to fetch data only once on mount
    use_effect_with((), {
        let risks = risks.clone();
        let loading = loading.clone();
        let token = token.clone();

        move |_| {
            spawn_local(async move {
                if let Ok(resp) = Request::get(&api_url("risks")).header("Authorization", &token).send().await {
                    if let Ok(data) = resp.json::<Vec<Risk>>().await {
                        risks.set(data);
                    }
                }
                loading.set(false);
            });
            || ()
        }
    });

    let severity_class = |sev: &str| -> &'static str {
        match sev {
            "Critical" => "is-danger",
            "High" => "is-warning",
            "Medium" => "is-info",
            _ => "is-light",
        }
    };

    html! {
        <div class="container p-4">
            <h1 class="title">{ lang.t("risk_monitoring") }</h1>
            if (*risks).is_empty() && *loading {
                <div class="box has-text-centered">
                    <p class="has-text-grey">{ "Loading..." }</p>
                </div>
            } else if (*risks).is_empty() {
                <div class="box has-text-centered">
                    <p class="has-text-grey">{ lang.t("no_risks") }</p>
                </div>
            } else {
                <div class="box">
                    <table class="table is-fullwidth is-hoverable">
                        <thead>
                            <tr>
                                <th>{ lang.t("severity") }</th>
                                <th>{ lang.t("asset") }</th>
                                <th>{ lang.t("port") }</th>
                                <th>{ lang.t("description") }</th>
                                <th>{ lang.t("status") }</th>
                            </tr>
                        </thead>
                        <tbody>
                            { for risks.iter().map(|risk| {
                                let status_str = format!("{:?}", risk.status);
                                html! {
                                    <tr>
                                        <td>
                                            <span class={classes!("tag", severity_class(&risk.severity))}>
                                                { &risk.severity }
                                            </span>
                                        </td>
                                        <td>{ &risk.asset_ip }</td>
                                        <td>{ risk.port }</td>
                                        <td>{ &risk.description }</td>
                                        <td>{ status_str }</td>
                                    </tr>
                                }
                            })}
                        </tbody>
                    </table>
                </div>
            }
        </div>
    }
}

// ============== Business Application Component (业务申请) ==============

#[function_component]
fn BusinessApplication() -> Html {
    let lang = use_state(|| Language::Zh);
    let resources = use_state(|| Vec::new());
    let loading = use_state(|| true);

    // Active cloud provider configs (from cloud-provider-configs/active API)
    let active_configs = use_state(|| Vec::<CloudProviderConfig>::new());

    // Form state for adding/editing
    let show_form = use_state(|| false);
    let editing_id = use_state(|| None as Option<i32>);
    let form_data = use_state(|| CreateBusinessResourceRequest {
        resource_type: "cloud".to_string(),
        ecs_name: String::new(),
        ecs_status: "待审批".to_string(),
        resource_id: String::new(),
        cloud_region: String::new(),
        cloud_category: String::new(),
        cloud_provider_config_id: None,
        county_city: None,
        vdc_name: None,
        customer_name: String::new(),
        application_name: None,
        contract_name: None,
        instance_id: String::new(),
        ecs_type: String::new(),
        ecs_os: String::new(),
        cpu_cores: 2,
        memory_gb: 4,
        system_disk: "ESSD".to_string(),
        system_disk_size_gb: 40,
        data_disk: None,
        completion_time: None,
        release_time: None,
        has_security_product: false,
        ip_address: String::new(),
        ecs_login_method: None,
        ecs_login_username: None,
        ecs_initial_password: None,
        bastion_address: None,
        bastion_admin_account: None,
        bastion_initial_password: None,
        // 物理机特有字段
        serial_number: None,
        rack_location: None,
        hardware_model: None,
        warranty_expiry: None,
        ipmi_address: None,
        remarks: None,
    });

    // Get token from localStorage
    let token = get_auth_token();

    // Fetch active cloud provider configs
    let fetch_active_configs = {
        let active_configs = active_configs.clone();
        let token = token.clone();

        Callback::from(move |_| {
            let active_configs = active_configs.clone();
            let token = token.clone();

            spawn_local(async move {
                if let Ok(resp) = Request::get(&api_url("cloud-provider-configs/active"))
                    .header("Authorization", &token)
                    .send()
                    .await
                {
                    if let Ok(data) = resp.json::<Vec<CloudProviderConfig>>().await {
                        active_configs.set(data);
                    }
                }
            });
        })
    };

    // Fetch business resources (filter by status: 待审批, 审批中)
    let fetch_resources = {
        let resources = resources.clone();
        let loading = loading.clone();
        let token = token.clone();

        Callback::from(move |_| {
            let resources = resources.clone();
            let loading = loading.clone();
            let token = token.clone();

            spawn_local(async move {
                if let Ok(resp) = Request::get(&api_url("business-resources"))
                    .header("Authorization", &token)
                    .send()
                    .await
                {
                    if let Ok(data) = resp.json::<Vec<BusinessResource>>().await {
                        // Filter for application status (待审批, 审批中)
                        let filtered: Vec<BusinessResource> = data
                            .into_iter()
                            .filter(|r| r.ecs_status == "待审批" || r.ecs_status == "审批中" || r.ecs_status == "已驳回")
                            .collect();
                        resources.set(filtered);
                    }
                }
                loading.set(false);
            });
        })
    };

    // Use effect to fetch data on mount
    use_effect_with((), {
        let fetch_resources = fetch_resources.clone();
        let fetch_active_configs = fetch_active_configs.clone();
        move |_| {
            fetch_active_configs.emit(());
            fetch_resources.emit(());
            || ()
        }
    });

    // Status badge class
    let status_class = |status: &str| -> &'static str {
        match status {
            "待审批" => "is-warning",
            "审批中" => "is-info",
            "已驳回" => "is-danger",
            "已通过" => "is-success",
            _ => "is-light",
        }
    };

    // Helper function to get provider name in Chinese
    let provider_name = |provider: &CloudProvider| -> String {
        match provider {
            CloudProvider::Aliyun => "阿里云".to_string(),
            CloudProvider::Tencent => "腾讯云".to_string(),
            CloudProvider::Huawei => "华为云".to_string(),
            CloudProvider::Aws => "AWS".to_string(),
            CloudProvider::Azure => "Azure".to_string(),
            CloudProvider::Gcp => "GCP".to_string(),
            CloudProvider::Baidu => "百度云".to_string(),
            CloudProvider::Custom(s) => s.clone(),
        }
    };

    // Handle form submission
    let on_submit = {
        let form_data = form_data.clone();
        let editing_id = editing_id.clone();
        let show_form = show_form.clone();
        let token = token.clone();
        let fetch_resources = fetch_resources.clone();

        Callback::from(move |_| {
            let data = form_data.clone();
            let edit_id = *editing_id;
            let token = token.clone();
            let fetch_resources = fetch_resources.clone();
            let show_form = show_form.clone();

            spawn_local(async move {
                let result = if let Some(id) = edit_id {
                    // For update, we need to convert to UpdateRequest
                    let update_req = UpdateBusinessResourceRequest {
                        resource_type: Some(data.resource_type.clone()),
                        ecs_name: Some(data.ecs_name.clone()),
                        ecs_status: Some(data.ecs_status.clone()),
                        cloud_region: Some(data.cloud_region.clone()),
                        cloud_category: Some(data.cloud_category.clone()),
                        cloud_provider_config_id: data.cloud_provider_config_id,
                        county_city: data.county_city.clone(),
                        vdc_name: data.vdc_name.clone(),
                        customer_name: Some(data.customer_name.clone()),
                        application_name: data.application_name.clone(),
                        contract_name: data.contract_name.clone(),
                        ecs_type: Some(data.ecs_type.clone()),
                        ecs_os: Some(data.ecs_os.clone()),
                        cpu_cores: Some(data.cpu_cores),
                        memory_gb: Some(data.memory_gb),
                        system_disk: Some(data.system_disk.clone()),
                        system_disk_size_gb: Some(data.system_disk_size_gb),
                        data_disk: data.data_disk.clone(),
                        completion_time: data.completion_time,
                        release_time: data.release_time,
                        has_security_product: Some(data.has_security_product),
                        ip_address: Some(data.ip_address.clone()),
                        ecs_login_method: data.ecs_login_method.clone(),
                        ecs_login_username: data.ecs_login_username.clone(),
                        ecs_initial_password: data.ecs_initial_password.clone(),
                        bastion_address: data.bastion_address.clone(),
                        bastion_admin_account: data.bastion_admin_account.clone(),
                        bastion_initial_password: data.bastion_initial_password.clone(),
                        // 物理机特有字段
                        serial_number: data.serial_number.clone(),
                        rack_location: data.rack_location.clone(),
                        hardware_model: data.hardware_model.clone(),
                        warranty_expiry: data.warranty_expiry,
                        agent_status: None,
                        ipmi_address: data.ipmi_address.clone(),
                        remarks: data.remarks.clone(),
                    };
                    let url = format!("{}/{}", api_url("business-resources"), id);
                    let json_body = serde_json::to_string(&update_req).unwrap_or_default();
                    Request::put(&url)
                        .header("Authorization", &token)
                        .header("Content-Type", "application/json")
                        .body(json_body)
                        .unwrap()
                        .send()
                        .await
                } else {
                    let json_body = serde_json::to_string(&*data).unwrap_or_default();
                    Request::post(&api_url("business-resources"))
                        .header("Authorization", &token)
                        .header("Content-Type", "application/json")
                        .body(json_body)
                        .unwrap()
                        .send()
                        .await
                };

                if result.is_ok() {
                    show_form.set(false);
                    fetch_resources.emit(());
                }
            });
        })
    };

    // Handle edit
    let on_edit = {
        let editing_id = editing_id.clone();
        let show_form = show_form.clone();
        let form_data = form_data.clone();

        Callback::from(move |resource: BusinessResource| {
            let edit_data = CreateBusinessResourceRequest {
                resource_type: resource.resource_type.clone(),
                ecs_name: resource.ecs_name.clone(),
                ecs_status: resource.ecs_status.clone(),
                resource_id: resource.resource_id.clone(),
                cloud_region: resource.cloud_region.clone(),
                cloud_category: resource.cloud_category.clone(),
                cloud_provider_config_id: resource.cloud_provider_config_id,
                county_city: resource.county_city.clone(),
                vdc_name: resource.vdc_name.clone(),
                customer_name: resource.customer_name.clone(),
                application_name: resource.application_name.clone(),
                contract_name: resource.contract_name.clone(),
                instance_id: resource.instance_id.clone(),
                ecs_type: resource.ecs_type.clone(),
                ecs_os: resource.ecs_os.clone(),
                cpu_cores: resource.cpu_cores,
                memory_gb: resource.memory_gb,
                system_disk: resource.system_disk.clone(),
                system_disk_size_gb: resource.system_disk_size_gb,
                data_disk: resource.data_disk.clone(),
                completion_time: resource.completion_time,
                release_time: resource.release_time,
                has_security_product: resource.has_security_product,
                ip_address: resource.ip_address.clone(),
                ecs_login_method: resource.ecs_login_method.clone(),
                ecs_login_username: resource.ecs_login_username.clone(),
                ecs_initial_password: resource.ecs_initial_password.clone(),
                bastion_address: resource.bastion_address.clone(),
                bastion_admin_account: resource.bastion_admin_account.clone(),
                bastion_initial_password: resource.bastion_initial_password.clone(),
                // 物理机特有字段
                serial_number: resource.serial_number.clone(),
                rack_location: resource.rack_location.clone(),
                hardware_model: resource.hardware_model.clone(),
                warranty_expiry: resource.warranty_expiry,
                ipmi_address: resource.ipmi_address.clone(),
                remarks: resource.remarks.clone(),
            };

            form_data.set(edit_data);
            editing_id.set(resource.id);
            show_form.set(true);
        })
    };

    // Handle delete
    let on_delete = {
        let token = token.clone();
        let fetch_resources = fetch_resources.clone();

        Callback::from(move |id: i32| {
            let token = token.clone();
            let fetch_resources = fetch_resources.clone();

            spawn_local(async move {
                let url = format!("{}/{}", api_url("business-resources"), id);
                if Request::delete(&url)
                    .header("Authorization", &token)
                    .send()
                    .await
                    .is_ok()
                {
                    fetch_resources.emit(());
                }
            });
        })
    };

    // Handle approve/reject
    let on_approve = {
        let token = token.clone();
        let fetch_resources = fetch_resources.clone();

        Callback::from(move |(id, status): (i32, String)| {
            let token = token.clone();
            let fetch_resources = fetch_resources.clone();

            spawn_local(async move {
                if status == "已通过" {
                    // 调用审批API，自动创建云资源
                    let url = format!("{}/{}/approve", api_url("business-resources"), id);
                    if Request::post(&url)
                        .header("Authorization", &token)
                        .send()
                        .await
                        .is_ok()
                    {
                        fetch_resources.emit(());
                    }
                } else {
                    // 驳回，直接更新状态
                    let update_req = UpdateBusinessResourceRequest {
                        ecs_status: Some(status),
                        ..Default::default()
                    };
                    let url = format!("{}/{}", api_url("business-resources"), id);
                    let json_body = serde_json::to_string(&update_req).unwrap_or_default();
                    if Request::put(&url)
                        .header("Authorization", &token)
                        .header("Content-Type", "application/json")
                        .body(json_body)
                        .unwrap()
                        .send()
                        .await
                        .is_ok()
                    {
                        fetch_resources.emit(());
                    }
                }
            });
        })
    };

    // Reset form
    let reset_form = {
        let form_data = form_data.clone();
        let editing_id = editing_id.clone();
        let show_form = show_form.clone();

        Callback::from(move |_| {
            form_data.set(CreateBusinessResourceRequest {
                resource_type: "cloud".to_string(),
                ecs_name: String::new(),
                ecs_status: "待审批".to_string(),
                resource_id: String::new(),
                cloud_region: String::new(),
                cloud_category: String::new(),
                cloud_provider_config_id: None,
                county_city: None,
                vdc_name: None,
                customer_name: String::new(),
                application_name: None,
                contract_name: None,
                instance_id: String::new(),
                ecs_type: String::new(),
                ecs_os: String::new(),
                cpu_cores: 2,
                memory_gb: 4,
                system_disk: "ESSD".to_string(),
                system_disk_size_gb: 40,
                data_disk: None,
                completion_time: None,
                release_time: None,
                has_security_product: false,
                ip_address: String::new(),
                ecs_login_method: None,
                ecs_login_username: None,
                ecs_initial_password: None,
                bastion_address: None,
                bastion_admin_account: None,
                bastion_initial_password: None,
                // 物理机特有字段
                serial_number: None,
                rack_location: None,
                hardware_model: None,
                warranty_expiry: None,
                ipmi_address: None,
                remarks: None,
            });
            editing_id.set(None);
            show_form.set(false);
        })
    };

    // Handle input change
    let on_input_change = {
        let form_data = form_data.clone();

        Callback::from(move |(field, value): (String, String)| {
            let mut data = (*form_data).clone();

            match field.as_str() {
                "ecs_name" => data.ecs_name = value,
                "ecs_status" => data.ecs_status = value,
                "resource_id" => data.resource_id = value,
                "cloud_region" => data.cloud_region = value,
                "cloud_category" => data.cloud_category = value,
                "county_city" => data.county_city = if value.is_empty() { None } else { Some(value) },
                "vdc_name" => data.vdc_name = if value.is_empty() { None } else { Some(value) },
                "customer_name" => data.customer_name = value,
                "application_name" => data.application_name = if value.is_empty() { None } else { Some(value) },
                "contract_name" => data.contract_name = if value.is_empty() { None } else { Some(value) },
                "instance_id" => data.instance_id = value,
                "ecs_type" => data.ecs_type = value,
                "ecs_os" => data.ecs_os = value,
                "system_disk" => data.system_disk = value,
                "data_disk" => data.data_disk = if value.is_empty() { None } else { Some(value) },
                "ip_address" => data.ip_address = value,
                "ecs_login_method" => data.ecs_login_method = if value.is_empty() { None } else { Some(value) },
                "ecs_login_username" => data.ecs_login_username = if value.is_empty() { None } else { Some(value) },
                "ecs_initial_password" => data.ecs_initial_password = if value.is_empty() { None } else { Some(value) },
                "bastion_address" => data.bastion_address = if value.is_empty() { None } else { Some(value) },
                "bastion_admin_account" => data.bastion_admin_account = if value.is_empty() { None } else { Some(value) },
                "bastion_initial_password" => data.bastion_initial_password = if value.is_empty() { None } else { Some(value) },
                "remarks" => data.remarks = if value.is_empty() { None } else { Some(value) },
                _ => {}
            }

            form_data.set(data);
        })
    };

    // Handle numeric input
    let on_number_input = {
        let form_data = form_data.clone();

        Callback::from(move |(field, value): (String, u32)| {
            let mut data = (*form_data).clone();

            match field.as_str() {
                "cpu_cores" => data.cpu_cores = value,
                "memory_gb" => data.memory_gb = value,
                "system_disk_size_gb" => data.system_disk_size_gb = value,
                _ => {}
            }

            form_data.set(data);
        })
    };

    // Handle checkbox
    let on_checkbox_change = {
        let form_data = form_data.clone();

        Callback::from(move |value: bool| {
            let mut data = (*form_data).clone();
            data.has_security_product = value;
            form_data.set(data);
        })
    };

    // Handle cloud provider config selection
    let on_config_select = {
        let form_data = form_data.clone();
        let active_configs = active_configs.clone();
        let provider_name_for_callback = provider_name;

        Callback::from(move |config_id_str: String| {
            let mut data = (*form_data).clone();

            if config_id_str.is_empty() {
                // No selection
                data.cloud_provider_config_id = None;
                // 不清空cloud_category，因为用户可能已经选择了云类别
                data.cloud_region = String::new();
            } else if let Ok(config_id) = config_id_str.parse::<i32>() {
                // Find the selected config
                data.cloud_provider_config_id = Some(config_id);
                if let Some(config) = active_configs.iter().find(|c| c.id == Some(config_id)) {
                    // Auto-fill cloud_category (使用中文名称) and cloud_region from the selected config
                    data.cloud_category = provider_name_for_callback(&config.provider);
                    data.cloud_region = config.region_name.clone();
                    // 云资源类型
                    data.resource_type = "cloud".to_string();
                }
            }

            form_data.set(data);
        })
    };

    // 计算是否应该显示云平台配置选择（不是物理机且已选择云类别）
    let show_cloud_config_select = {
        let is_physical = (*form_data).cloud_category == "物理机";
        !is_physical && !(*form_data).cloud_category.is_empty()
    };

    // 从已启用的云平台配置中提取唯一的云厂商列表（用于云类别下拉）
    let unique_providers: Vec<(CloudProvider, String)> = {
        use std::collections::HashSet;
        let mut providers = HashSet::new();
        for config in (*active_configs).iter() {
            providers.insert(config.provider.clone());
        }
        let mut provider_list: Vec<(CloudProvider, String)> = providers
            .into_iter()
            .map(|p| (p.clone(), provider_name(&p)))
            .collect();
        provider_list.sort_by(|a, b| a.1.cmp(&b.1)); // 按中文名称排序
        provider_list
    };

    html! {
        <div class="container p-4">
            <h1 class="title">{ lang.t("business_application") }</h1>

            <div class="box">
                <div class="level">
                    <div class="level-left">
                        <button class="button is-primary" onclick={
                            let show_form = show_form.clone();
                            Callback::from(move |_| show_form.set(true))
                        }>
                            <span class="icon"><i class="fas fa-plus"></i></span>
                            <span>{ lang.t("add_business_resource") }</span>
                        </button>
                        <button class="button is-info ml-2" onclick={
                            let token = token.clone();
                            Callback::from(move |_| {
                                let token = token.clone();
                                spawn_local(async move {
                                    let _ = Request::get(&api_url("business-resources/export"))
                                        .header("Authorization", &token)
                                        .send()
                                        .await;
                                });
                            })
                        }>
                            <span class="icon"><i class="fas fa-download"></i></span>
                            <span>{ lang.t("export") }</span>
                        </button>
                    </div>
                </div>

                if *show_form {
                    <div class="modal is-active">
                        <div class="modal-background" onclick={reset_form.clone()}></div>
                        <div class="modal-card" style="width: 800px;">
                            <header class="modal-card-head">
                                <p class="modal-card-title">
                                    { if (*editing_id).is_some() { lang.t("edit") } else { lang.t("add_business_resource") } }
                                </p>
                                <button class="delete" onclick={reset_form.clone()}></button>
                            </header>
                            <section class="modal-card-body" style="max-height: calc(100vh - 200px); overflow-y: auto;">
                                <div class="columns is-multiline">
                                    // Basic Info
                                    <div class="column is-6">
                                        <label class="label">{ lang.t("ecs_name") }</label>
                                        <input
                                            type="text"
                                            class="input"
                                            value={(*form_data).ecs_name.clone()}
                                            onchange={
                                                let on_input_change = on_input_change.clone();
                                                Callback::from(move |e: Event| {
                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                    on_input_change.emit(("ecs_name".to_string(), input.value()));
                                                })
                                            }
                                        />
                                    </div>
                                    <div class="column is-6">
                                        <label class="label">{ lang.t("ecs_status") }</label>
                                        <div class="select is-fullwidth">
                                            <select
                                                onchange={
                                                    let on_input_change = on_input_change.clone();
                                                    Callback::from(move |e: Event| {
                                                        let select: HtmlSelectElement = e.target_unchecked_into();
                                                        on_input_change.emit(("ecs_status".to_string(), select.value()));
                                                    })
                                                }
                                            >
                                                <option selected={(*form_data).ecs_status == "待审批"}>{ "待审批" }</option>
                                                <option selected={(*form_data).ecs_status == "审批中"}>{ "审批中" }</option>
                                                <option selected={(*form_data).ecs_status == "已驳回"}>{ "已驳回" }</option>
                                                <option selected={(*form_data).ecs_status == "已通过"}>{ "已通过" }</option>
                                            </select>
                                        </div>
                                    </div>
                                    <div class="column is-6">
                                        <label class="label">{ lang.t("resource_id") }</label>
                                        <input
                                            type="text"
                                            class="input"
                                            value={(*form_data).resource_id.clone()}
                                            onchange={
                                                let on_input_change = on_input_change.clone();
                                                Callback::from(move |e: Event| {
                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                    on_input_change.emit(("resource_id".to_string(), input.value()));
                                                })
                                            }
                                        />
                                    </div>
                                    <div class="column is-6">
                                        <label class="label">{ lang.t("instance_id") }</label>
                                        <input
                                            type="text"
                                            class="input"
                                            value={(*form_data).instance_id.clone()}
                                            onchange={
                                                let on_input_change = on_input_change.clone();
                                                Callback::from(move |e: Event| {
                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                    on_input_change.emit(("instance_id".to_string(), input.value()));
                                                })
                                            }
                                        />
                                    </div>

                                    // Cloud Info
                                    <div class="column is-6">
                                        <label class="label">{ lang.t("cloud_region") }</label>
                                        <input
                                            type="text"
                                            class="input"
                                            value={(*form_data).cloud_region.clone()}
                                            placeholder="华东1"
                                            onchange={
                                                let on_input_change = on_input_change.clone();
                                                Callback::from(move |e: Event| {
                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                    on_input_change.emit(("cloud_region".to_string(), input.value()));
                                                })
                                            }
                                        />
                                    </div>
                                    <div class="column is-6">
                                        <label class="label">{ lang.t("cloud_category") }</label>
                                        <div class="select is-fullwidth">
                                            <select
                                                onchange={
                                                    let on_input_change = on_input_change.clone();
                                                    let form_data = form_data.clone();
                                                    let active_configs = active_configs.clone();
                                                    let provider_name_clone = provider_name;
                                                    Callback::from(move |e: Event| {
                                                        let select: HtmlSelectElement = e.target_unchecked_into();
                                                        let value = select.value();

                                                        // 根据云类别设置resource_type
                                                        let mut data = (*form_data).clone();
                                                        if value == "物理机" {
                                                            data.resource_type = "physical".to_string();
                                                            data.cloud_provider_config_id = None;
                                                        } else {
                                                            data.resource_type = "cloud".to_string();
                                                        }
                                                        data.cloud_category = value.clone();

                                                        // 如果不是物理机且有云平台配置，自动填充云平台信息
                                                        if value != "物理机" && !value.is_empty() {
                                                            // 查找匹配的云平台配置（通过中文名称匹配）
                                                            if let Some(config) = (*active_configs).iter().find(|c| {
                                                                provider_name_clone(&c.provider) == value
                                                            }) {
                                                                data.cloud_provider_config_id = config.id;
                                                                data.cloud_region = config.region_name.clone();
                                                            }
                                                        } else if value == "物理机" {
                                                            data.cloud_region = "物理机房".to_string();
                                                        }

                                                        form_data.set(data);
                                                        on_input_change.emit(("cloud_category".to_string(), value));
                                                    })
                                                }
                                            >
                                                <option value="">{ "选择云类别..." }</option>
                                                <option value="物理机" selected={(*form_data).cloud_category == "物理机"}>{ "物理机" }</option>
                                                {
                                                    unique_providers.iter().map(|(provider, provider_name_cn)| {
                                                        let provider_str = format!("{:?}", provider);
                                                        let is_selected = (*form_data).cloud_category == *provider_name_cn;
                                                        html! {
                                                            <option
                                                                value={provider_name_cn.clone()}
                                                                selected={is_selected}
                                                            >
                                                                { provider_name_cn }
                                                            </option>
                                                        }
                                                    }).collect::<Vec<_>>()
                                                }
                                            </select>
                                        </div>
                                        // 显示已选中的云平台信息
                                        if !(*form_data).cloud_category.is_empty() {
                                            <p class="help is-info">
                                                { format!("已选择: {}", (*form_data).cloud_category) }
                                                { if (*form_data).resource_type == "physical" {
                                                    html! { <span>{ " - 物理机不需要云平台配置" }</span> }
                                                } else {
                                                    html! {}
                                                }}
                                            </p>
                                        }
                                    </div>

                                    // 云平台配置选择 - 只有选择云资源时才显示
                                    if show_cloud_config_select {
                                        <div class="column is-6">
                                            <label class="label">{ "云平台配置" }</label>
                                            <div class="select is-fullwidth">
                                                <select
                                                    onchange={
                                                        let on_config_select = on_config_select.clone();
                                                        Callback::from(move |e: Event| {
                                                            let select: HtmlSelectElement = e.target_unchecked_into();
                                                            on_config_select.emit(select.value());
                                                        })
                                                    }
                                                >
                                                    <option value="">{ "选择云平台配置..." }</option>
                                                    {
                                                        (*active_configs).iter().filter(|c| {
                                                            // 只显示已选择的云类别的配置（通过中文名称匹配）
                                                            let provider_name_cn = provider_name(&c.provider);
                                                            provider_name_cn == (*form_data).cloud_category || (*form_data).cloud_category.is_empty()
                                                        }).map(|config| {
                                                            let selected = (*form_data).cloud_provider_config_id == config.id;
                                                            let provider_name_cn = provider_name(&config.provider);
                                                            let label = format!("{} - {}", provider_name_cn, config.region_name);
                                                            html! {
                                                                <option
                                                                    value={config.id.unwrap_or(0).to_string()}
                                                                    selected={selected}
                                                                >
                                                                    { label }
                                                                </option>
                                                            }
                                                        }).collect::<Vec<_>>()
                                                    }
                                                </select>
                                            </div>
                                            <p class="help">
                                                { "选择已对接的云平台账户和区域" }
                                            </p>
                                        </div>
                                    } else {
                                        <div class="column is-6"></div>
                                    }

                                    <div class="column is-6">
                                        <label class="label">{ lang.t("county_city") }</label>
                                        <input
                                            type="text"
                                            class="input"
                                            value={(*form_data).county_city.clone().unwrap_or_default()}
                                            placeholder={ if (*form_data).cloud_category == "物理机" { "如：杭州机房" } else { "杭州市" } }
                                            onchange={
                                                let on_input_change = on_input_change.clone();
                                                Callback::from(move |e: Event| {
                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                    on_input_change.emit(("county_city".to_string(), input.value()));
                                                })
                                            }
                                        />
                                    </div>
                                    <div class="column is-6">
                                        <label class="label">{ lang.t("vdc_name") }</label>
                                        <input
                                            type="text"
                                            class="input"
                                            value={(*form_data).vdc_name.clone().unwrap_or_default()}
                                            placeholder="VDC-01"
                                            onchange={
                                                let on_input_change = on_input_change.clone();
                                                Callback::from(move |e: Event| {
                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                    on_input_change.emit(("vdc_name".to_string(), input.value()));
                                                })
                                            }
                                        />
                                    </div>

                                    // Customer Info
                                    <div class="column is-6">
                                        <label class="label">{ lang.t("customer_name") }</label>
                                        <input
                                            type="text"
                                            class="input"
                                            value={(*form_data).customer_name.clone()}
                                            onchange={
                                                let on_input_change = on_input_change.clone();
                                                Callback::from(move |e: Event| {
                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                    on_input_change.emit(("customer_name".to_string(), input.value()));
                                                })
                                            }
                                        />
                                    </div>
                                    <div class="column is-6">
                                        <label class="label">{ lang.t("application_name") }</label>
                                        <input
                                            type="text"
                                            class="input"
                                            value={(*form_data).application_name.clone().unwrap_or_default()}
                                            onchange={
                                                let on_input_change = on_input_change.clone();
                                                Callback::from(move |e: Event| {
                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                    on_input_change.emit(("application_name".to_string(), input.value()));
                                                })
                                            }
                                        />
                                    </div>
                                    <div class="column is-6">
                                        <label class="label">{ lang.t("contract_name") }</label>
                                        <input
                                            type="text"
                                            class="input"
                                            value={(*form_data).contract_name.clone().unwrap_or_default()}
                                            onchange={
                                                let on_input_change = on_input_change.clone();
                                                Callback::from(move |e: Event| {
                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                    on_input_change.emit(("contract_name".to_string(), input.value()));
                                                })
                                            }
                                        />
                                    </div>

                                    // ECS Specs
                                    <div class="column is-6">
                                        <label class="label">{ lang.t("ecs_type") }</label>
                                        <input
                                            type="text"
                                            class="input"
                                            value={(*form_data).ecs_type.clone()}
                                            placeholder="ecs.g6.large"
                                            onchange={
                                                let on_input_change = on_input_change.clone();
                                                Callback::from(move |e: Event| {
                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                    on_input_change.emit(("ecs_type".to_string(), input.value()));
                                                })
                                            }
                                        />
                                    </div>
                                    <div class="column is-6">
                                        <label class="label">{ lang.t("ecs_os") }</label>
                                        <input
                                            type="text"
                                            class="input"
                                            value={(*form_data).ecs_os.clone()}
                                            placeholder="CentOS 7.9"
                                            onchange={
                                                let on_input_change = on_input_change.clone();
                                                Callback::from(move |e: Event| {
                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                    on_input_change.emit(("ecs_os".to_string(), input.value()));
                                                })
                                            }
                                        />
                                    </div>
                                    <div class="column is-4">
                                        <label class="label">{ lang.t("cpu_cores") }</label>
                                        <input
                                            type="number"
                                            class="input"
                                            value={(*form_data).cpu_cores.to_string()}
                                            onchange={
                                                let on_number_input = on_number_input.clone();
                                                Callback::from(move |e: Event| {
                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                    if let Ok(val) = input.value().parse::<u32>() {
                                                        on_number_input.emit(("cpu_cores".to_string(), val));
                                                    }
                                                })
                                            }
                                        />
                                    </div>
                                    <div class="column is-4">
                                        <label class="label">{ lang.t("memory_gb") }</label>
                                        <input
                                            type="number"
                                            class="input"
                                            value={(*form_data).memory_gb.to_string()}
                                            onchange={
                                                let on_number_input = on_number_input.clone();
                                                Callback::from(move |e: Event| {
                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                    if let Ok(val) = input.value().parse::<u32>() {
                                                        on_number_input.emit(("memory_gb".to_string(), val));
                                                    }
                                                })
                                            }
                                        />
                                    </div>
                                    <div class="column is-4">
                                        <label class="label">{ lang.t("system_disk_size_gb") }</label>
                                        <input
                                            type="number"
                                            class="input"
                                            value={(*form_data).system_disk_size_gb.to_string()}
                                            onchange={
                                                let on_number_input = on_number_input.clone();
                                                Callback::from(move |e: Event| {
                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                    if let Ok(val) = input.value().parse::<u32>() {
                                                        on_number_input.emit(("system_disk_size_gb".to_string(), val));
                                                    }
                                                })
                                            }
                                        />
                                    </div>
                                    <div class="column is-6">
                                        <label class="label">{ lang.t("system_disk") }</label>
                                        <input
                                            type="text"
                                            class="input"
                                            value={(*form_data).system_disk.clone()}
                                            placeholder="ESSD"
                                            onchange={
                                                let on_input_change = on_input_change.clone();
                                                Callback::from(move |e: Event| {
                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                    on_input_change.emit(("system_disk".to_string(), input.value()));
                                                })
                                            }
                                        />
                                    </div>
                                    <div class="column is-6">
                                        <label class="label">{ lang.t("data_disk") }</label>
                                        <input
                                            type="text"
                                            class="input"
                                            value={(*form_data).data_disk.clone().unwrap_or_default()}
                                            placeholder="ESSD 100GB"
                                            onchange={
                                                let on_input_change = on_input_change.clone();
                                                Callback::from(move |e: Event| {
                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                    on_input_change.emit(("data_disk".to_string(), input.value()));
                                                })
                                            }
                                        />
                                    </div>

                                    // Network Info
                                    <div class="column is-6">
                                        <label class="label">{ lang.t("ip_address") }</label>
                                        <input
                                            type="text"
                                            class="input"
                                            value={(*form_data).ip_address.clone()}
                                            onchange={
                                                let on_input_change = on_input_change.clone();
                                                Callback::from(move |e: Event| {
                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                    on_input_change.emit(("ip_address".to_string(), input.value()));
                                                })
                                            }
                                        />
                                    </div>
                                    <div class="column is-6">
                                        <label class="label">{ lang.t("ecs_login_method") }</label>
                                        <input
                                            type="text"
                                            class="input"
                                            value={(*form_data).ecs_login_method.clone().unwrap_or_default()}
                                            placeholder="SSH/密码"
                                            onchange={
                                                let on_input_change = on_input_change.clone();
                                                Callback::from(move |e: Event| {
                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                    on_input_change.emit(("ecs_login_method".to_string(), input.value()));
                                                })
                                            }
                                        />
                                    </div>
                                    <div class="column is-6">
                                        <label class="label">{ lang.t("ecs_login_username") }</label>
                                        <input
                                            type="text"
                                            class="input"
                                            value={(*form_data).ecs_login_username.clone().unwrap_or_default()}
                                            onchange={
                                                let on_input_change = on_input_change.clone();
                                                Callback::from(move |e: Event| {
                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                    on_input_change.emit(("ecs_login_username".to_string(), input.value()));
                                                })
                                            }
                                        />
                                    </div>
                                    <div class="column is-6">
                                        <label class="label">{ lang.t("ecs_initial_password") }</label>
                                        <input
                                            type="password"
                                            class="input"
                                            value={(*form_data).ecs_initial_password.clone().unwrap_or_default()}
                                            onchange={
                                                let on_input_change = on_input_change.clone();
                                                Callback::from(move |e: Event| {
                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                    on_input_change.emit(("ecs_initial_password".to_string(), input.value()));
                                                })
                                            }
                                        />
                                    </div>

                                    // Bastion Info
                                    <div class="column is-6">
                                        <label class="label">{ lang.t("bastion_address") }</label>
                                        <input
                                            type="text"
                                            class="input"
                                            value={(*form_data).bastion_address.clone().unwrap_or_default()}
                                            onchange={
                                                let on_input_change = on_input_change.clone();
                                                Callback::from(move |e: Event| {
                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                    on_input_change.emit(("bastion_address".to_string(), input.value()));
                                                })
                                            }
                                        />
                                    </div>
                                    <div class="column is-6">
                                        <label class="label">{ lang.t("bastion_admin_account") }</label>
                                        <input
                                            type="text"
                                            class="input"
                                            value={(*form_data).bastion_admin_account.clone().unwrap_or_default()}
                                            onchange={
                                                let on_input_change = on_input_change.clone();
                                                Callback::from(move |e: Event| {
                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                    on_input_change.emit(("bastion_admin_account".to_string(), input.value()));
                                                })
                                            }
                                        />
                                    </div>
                                    <div class="column is-6">
                                        <label class="label">{ lang.t("bastion_initial_password") }</label>
                                        <input
                                            type="password"
                                            class="input"
                                            value={(*form_data).bastion_initial_password.clone().unwrap_or_default()}
                                            onchange={
                                                let on_input_change = on_input_change.clone();
                                                Callback::from(move |e: Event| {
                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                    on_input_change.emit(("bastion_initial_password".to_string(), input.value()));
                                                })
                                            }
                                        />
                                    </div>

                                    // Security Product
                                    <div class="column is-12">
                                        <label class="checkbox">
                                            <input
                                                type="checkbox"
                                                checked={(*form_data).has_security_product}
                                                onchange={
                                                    let on_checkbox_change = on_checkbox_change.clone();
                                                    Callback::from(move |e: Event| {
                                                        let input: HtmlInputElement = e.target_unchecked_into();
                                                        on_checkbox_change.emit(input.checked());
                                                    })
                                                }
                                            />
                                            { format!(" {} {}", lang.t("has_security_product"), if (*form_data).has_security_product { "(是)" } else { "(否)" }) }
                                        </label>
                                    </div>

                                    // Remarks
                                    <div class="column is-12">
                                        <label class="label">{ lang.t("remarks") }</label>
                                        <textarea
                                            class="textarea"
                                            rows="3"
                                            value={(*form_data).remarks.clone().unwrap_or_default()}
                                            onchange={
                                                let on_input_change = on_input_change.clone();
                                                Callback::from(move |e: Event| {
                                                    let input: HtmlTextAreaElement = e.target_unchecked_into();
                                                    on_input_change.emit(("remarks".to_string(), input.value()));
                                                })
                                            }
                                        ></textarea>
                                    </div>
                                </div>
                            </section>
                            <footer class="modal-card-foot">
                                <button class="button is-success" onclick={on_submit.clone()}>{ lang.t("save") }</button>
                                <button class="button" onclick={reset_form.clone()}>{ lang.t("cancel") }</button>
                            </footer>
                        </div>
                    </div>
                }

                if (*resources).is_empty() && *loading {
                    <p class="has-text-centered has-text-grey">{ "Loading..." }</p>
                } else if (*resources).is_empty() {
                    <p class="has-text-centered has-text-grey">{ "暂无业务资源" }</p>
                } else {
                    <div class="table-container" style="overflow-x: auto;">
                        <table class="table is-fullwidth is-hoverable is-striped" style="min-width: max-content;">
                            <thead>
                                <tr>
                                    <th>{ lang.t("ecs_name") }</th>
                                    <th>{ lang.t("ecs_status") }</th>
                                    <th>{ lang.t("resource_id") }</th>
                                    <th>{ lang.t("cloud_region") }</th>
                                    <th>{ lang.t("cloud_category") }</th>
                                    <th>{ lang.t("county_city") }</th>
                                    <th>{ lang.t("vdc_name") }</th>
                                    <th>{ lang.t("customer_name") }</th>
                                    <th>{ lang.t("application_name") }</th>
                                    <th>{ lang.t("contract_name") }</th>
                                    <th>{ lang.t("instance_id") }</th>
                                    <th>{ lang.t("ecs_type") }</th>
                                    <th>{ lang.t("ecs_os") }</th>
                                    <th>{ lang.t("cpu_cores") }</th>
                                    <th>{ lang.t("memory_gb") }</th>
                                    <th>{ lang.t("system_disk") }</th>
                                    <th>{ lang.t("system_disk_size_gb") }</th>
                                    <th>{ lang.t("data_disk") }</th>
                                    <th>{ lang.t("completion_time") }</th>
                                    <th>{ lang.t("release_time") }</th>
                                    <th>{ lang.t("has_security_product") }</th>
                                    <th>{ lang.t("ip_address") }</th>
                                    <th>{ lang.t("ecs_login_method") }</th>
                                    <th>{ lang.t("ecs_login_username") }</th>
                                    <th>{ lang.t("ecs_initial_password") }</th>
                                    <th>{ lang.t("bastion_address") }</th>
                                    <th>{ lang.t("bastion_admin_account") }</th>
                                    <th>{ lang.t("bastion_initial_password") }</th>
                                    <th>{ lang.t("remarks") }</th>
                                    <th>{ lang.t("actions") }</th>
                                </tr>
                            </thead>
                            <tbody>
                                { for resources.iter().map(|resource| {
                                    let resource_clone = resource.clone();
                                    let resource_for_delete = resource.clone();
                                    let resource_for_approve = resource.clone();
                                    let on_edit = on_edit.clone();
                                    let on_delete = on_delete.clone();
                                    let on_approve = on_approve.clone();

                                    html! {
                                        <tr>
                                            <td><strong>{ &resource_clone.ecs_name }</strong></td>
                                            <td>
                                                <span class={classes!("tag", status_class(&resource_clone.ecs_status))}>
                                                    { &resource_clone.ecs_status }
                                                </span>
                                            </td>
                                            <td><code>{ &resource_clone.resource_id }</code></td>
                                            <td>{ &resource_clone.cloud_region }</td>
                                            <td>{ &resource_clone.cloud_category }</td>
                                            <td>{ resource_clone.county_city.as_ref().unwrap_or(&String::new()) }</td>
                                            <td>{ resource_clone.vdc_name.as_ref().unwrap_or(&String::new()) }</td>
                                            <td>{ &resource_clone.customer_name }</td>
                                            <td>{ resource_clone.application_name.as_ref().unwrap_or(&String::new()) }</td>
                                            <td>{ resource_clone.contract_name.as_ref().unwrap_or(&String::new()) }</td>
                                            <td><code>{ &resource_clone.instance_id }</code></td>
                                            <td>{ &resource_clone.ecs_type }</td>
                                            <td>{ &resource_clone.ecs_os }</td>
                                            <td>{ resource_clone.cpu_cores }</td>
                                            <td>{ resource_clone.memory_gb }</td>
                                            <td>{ &resource_clone.system_disk }</td>
                                            <td>{ resource_clone.system_disk_size_gb }</td>
                                            <td>{ resource_clone.data_disk.as_ref().unwrap_or(&String::new()) }</td>
                                            <td>{ resource_clone.completion_time.map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_default() }</td>
                                            <td>{ resource_clone.release_time.map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_default() }</td>
                                            <td>
                                                if resource_clone.has_security_product {
                                                    <span class="tag is-success">{ "是" }</span>
                                                } else {
                                                    <span class="tag is-light">{ "否" }</span>
                                                }
                                            </td>
                                            <td><code>{ &resource_clone.ip_address }</code></td>
                                            <td>{ resource_clone.ecs_login_method.as_ref().unwrap_or(&String::new()) }</td>
                                            <td>{ resource_clone.ecs_login_username.as_ref().unwrap_or(&String::new()) }</td>
                                            <td>{ resource_clone.ecs_initial_password.as_ref().map(|_| "***").unwrap_or_default() }</td>
                                            <td>{ resource_clone.bastion_address.as_ref().unwrap_or(&String::new()) }</td>
                                            <td>{ resource_clone.bastion_admin_account.as_ref().unwrap_or(&String::new()) }</td>
                                            <td>{ resource_clone.bastion_initial_password.as_ref().map(|_| "***").unwrap_or_default() }</td>
                                            <td>{ resource_clone.remarks.as_ref().unwrap_or(&String::new()) }</td>
                                            <td>
                                                <div class="buttons are-small">
                                                    // Approve button
                                                    if resource_clone.ecs_status == "待审批" || resource_clone.ecs_status == "审批中" {
                                                        <button
                                                            class="button is-success is-outlined"
                                                            onclick={
                                                                let on_approve = on_approve.clone();
                                                                let id = resource_for_approve.id;
                                                                Callback::from(move |_| {
                                                                    if let Some(id_val) = id {
                                                                        on_approve.emit((id_val, "已通过".to_string()));
                                                                    }
                                                                })
                                                            }
                                                        >
                                                            <span class="icon"><i class="fas fa-check"></i></span>
                                                            <span>{ "通过" }</span>
                                                        </button>
                                                        <button
                                                            class="button is-warning is-outlined"
                                                            onclick={
                                                                let on_approve = on_approve.clone();
                                                                let id = resource_for_approve.id;
                                                                Callback::from(move |_| {
                                                                    if let Some(id_val) = id {
                                                                        on_approve.emit((id_val, "已驳回".to_string()));
                                                                    }
                                                                })
                                                            }
                                                        >
                                                            <span class="icon"><i class="fas fa-times"></i></span>
                                                            <span>{ "驳回" }</span>
                                                        </button>
                                                    }
                                                    <button
                                                        class="button is-info is-outlined"
                                                        onclick={
                                                            let on_edit = on_edit.clone();
                                                            let r = resource_clone.clone();
                                                            Callback::from(move |_| on_edit.emit(r.clone()))
                                                        }
                                                    >
                                                        <span class="icon"><i class="fas fa-edit"></i></span>
                                                        <span>{ lang.t("edit") }</span>
                                                    </button>
                                                    <button
                                                        class="button is-danger is-outlined"
                                                        onclick={
                                                            let on_delete = on_delete.clone();
                                                            let id = resource_for_delete.id;
                                                            Callback::from(move |_| {
                                                                if let Some(id_val) = id {
                                                                    on_delete.emit(id_val);
                                                                }
                                                            })
                                                        }
                                                    >
                                                        <span class="icon"><i class="fas fa-trash"></i></span>
                                                        <span>{ lang.t("delete") }</span>
                                                    </button>
                                                </div>
                                            </td>
                                        </tr>
                                    }
                                })}
                            </tbody>
                        </table>
                    </div>
                }
            </div>
        </div>
    }
}

// ============== Operations Management Component (运维管理) ==============
// 运维人员使用：补充信息、审批业务申请

#[function_component]
fn OperationsManagement() -> Html {
    let lang = use_state(|| Language::Zh);
    let resources = use_state(|| Vec::new());
    let loading = use_state(|| true);

    // Modal states
    let show_detail_modal = use_state(|| false);
    let show_approve_modal = use_state(|| false);
    let show_supplement_modal = use_state(|| false);
    let selected_resource = use_state(|| None as Option<BusinessResource>);

    // Form state for supplementing info
    let supplement_form = use_state(|| UpdateBusinessResourceRequest::default());

    // Get token from localStorage
    let token = get_auth_token();

    // Fetch all pending resources (待审批, 审批中, 待交付)
    let fetch_resources = {
        let resources = resources.clone();
        let loading = loading.clone();
        let token = token.clone();

        Callback::from(move |_| {
            let resources = resources.clone();
            let loading = loading.clone();
            let token = token.clone();

            spawn_local(async move {
                if let Ok(resp) = Request::get(&api_url("business-resources"))
                    .header("Authorization", &token)
                    .send()
                    .await
                {
                    if let Ok(data) = resp.json::<Vec<BusinessResource>>().await {
                        // Filter for operations statuses (待审批, 审批中, 待交付)
                        let filtered: Vec<BusinessResource> = data
                            .into_iter()
                            .filter(|r| {
                                r.ecs_status == "待审批" ||
                                r.ecs_status == "审批中" ||
                                r.ecs_status == "待交付" ||
                                r.ecs_status == "已驳回"
                            })
                            .collect();
                        resources.set(filtered);
                    }
                }
                loading.set(false);
            });
        })
    };

    // Use effect to fetch data on mount
    use_effect_with((), {
        let fetch_resources = fetch_resources.clone();
        move |_| {
            fetch_resources.emit(());
            || ()
        }
    });

    // Status badge class
    let status_class = |status: &str| -> &'static str {
        match status {
            "待审批" => "is-warning",
            "审批中" => "is-info",
            "待交付" => "is-primary",
            "已驳回" => "is-danger",
            "已交付" => "is-success",
            _ => "is-light",
        }
    };

    // Handle view detail
    let on_view_detail = {
        let selected_resource = selected_resource.clone();
        let show_detail_modal = show_detail_modal.clone();

        Callback::from(move |resource: BusinessResource| {
            selected_resource.set(Some(resource));
            show_detail_modal.set(true);
        })
    };

    // Handle approve/reject
    let on_approve_click = {
        let selected_resource = selected_resource.clone();
        let show_approve_modal = show_approve_modal.clone();

        Callback::from(move |resource: BusinessResource| {
            selected_resource.set(Some(resource));
            show_approve_modal.set(true);
        })
    };

    // Handle supplement info
    let on_supplement_click = {
        let selected_resource = selected_resource.clone();
        let show_supplement_modal = show_supplement_modal.clone();
        let supplement_form = supplement_form.clone();

        Callback::from(move |resource: BusinessResource| {
            let resource_clone = resource.clone();
            selected_resource.set(Some(resource));

            // Pre-fill form with current resource data
            supplement_form.set(UpdateBusinessResourceRequest {
                ip_address: if resource_clone.ip_address.is_empty() { None } else { Some(resource_clone.ip_address.clone()) },
                ecs_login_method: resource_clone.ecs_login_method.clone(),
                ecs_login_username: resource_clone.ecs_login_username.clone(),
                ecs_initial_password: resource_clone.ecs_initial_password.clone(),
                bastion_address: resource_clone.bastion_address.clone(),
                bastion_admin_account: resource_clone.bastion_admin_account.clone(),
                bastion_initial_password: resource_clone.bastion_initial_password.clone(),
                remarks: resource_clone.remarks.clone(),
                ..Default::default()
            });

            show_supplement_modal.set(true);
        })
    };

    // Submit approval/reject
    let on_submit_approve = {
        let token = token.clone();
        let selected_resource = selected_resource.clone();
        let show_approve_modal = show_approve_modal.clone();
        let fetch_resources = fetch_resources.clone();

        Callback::from(move |(id, status, note): (i32, String, String)| {
            let token = token.clone();
            let fetch_resources = fetch_resources.clone();
            let selected_resource = selected_resource.clone();
            let show_approve_modal = show_approve_modal.clone();

            spawn_local(async move {
                if status == "approved" {
                    // 调用审批API
                    let url = format!("{}/{}/approve", api_url("business-resources"), id);
                    if Request::post(&url)
                        .header("Authorization", &token)
                        .send()
                        .await
                        .is_ok()
                    {
                        fetch_resources.emit(());
                    }
                } else {
                    // 驳回，更新状态为已驳回
                    let update_req = UpdateBusinessResourceRequest {
                        ecs_status: Some("已驳回".to_string()),
                        remarks: Some(note),
                        ..Default::default()
                    };
                    let url = format!("{}/{}", api_url("business-resources"), id);
                    let json_body = serde_json::to_string(&update_req).unwrap_or_default();
                    if Request::put(&url)
                        .header("Authorization", &token)
                        .header("Content-Type", "application/json")
                        .body(json_body)
                        .unwrap()
                        .send()
                        .await
                        .is_ok()
                    {
                        fetch_resources.emit(());
                    }
                }
                selected_resource.set(None);
                show_approve_modal.set(false);
            });
        })
    };

    // Submit supplement
    let on_submit_supplement = {
        let token = token.clone();
        let selected_resource = selected_resource.clone();
        let show_supplement_modal = show_supplement_modal.clone();
        let supplement_form = supplement_form.clone();
        let fetch_resources = fetch_resources.clone();

        Callback::from(move |_| {
            let token = token.clone();
            let selected_resource = selected_resource.clone();
            let show_supplement_modal = show_supplement_modal.clone();
            let supplement_form = supplement_form.clone();
            let fetch_resources = fetch_resources.clone();

            spawn_local(async move {
                if let Some(resource) = &*selected_resource {
                    if let Some(id) = resource.id {
                        let url = format!("{}/{}", api_url("business-resources"), id);
                        let json_body = serde_json::to_string(&*supplement_form).unwrap_or_default();
                        if Request::put(&url)
                            .header("Authorization", &token)
                            .header("Content-Type", "application/json")
                            .body(json_body)
                            .unwrap()
                            .send()
                            .await
                            .is_ok()
                        {
                            fetch_resources.emit(());
                        }
                    }
                }
                selected_resource.set(None);
                show_supplement_modal.set(false);
            });
        })
    };

    // Handle form input change
    let on_form_input = {
        let supplement_form = supplement_form.clone();
        Callback::from(move |(field, value): (String, String)| {
            let mut data = (*supplement_form).clone();
            match field.as_str() {
                "ip_address" => data.ip_address = Some(value),
                "ecs_login_method" => data.ecs_login_method = if value.is_empty() { None } else { Some(value) },
                "ecs_login_username" => data.ecs_login_username = if value.is_empty() { None } else { Some(value) },
                "ecs_initial_password" => data.ecs_initial_password = if value.is_empty() { None } else { Some(value) },
                "bastion_address" => data.bastion_address = if value.is_empty() { None } else { Some(value) },
                "bastion_admin_account" => data.bastion_admin_account = if value.is_empty() { None } else { Some(value) },
                "bastion_initial_password" => data.bastion_initial_password = if value.is_empty() { None } else { Some(value) },
                "remarks" => data.remarks = if value.is_empty() { None } else { Some(value) },
                _ => {}
            }
            supplement_form.set(data);
        })
    };

    // Stats
    let total_count = (*resources).len();
    let pending_count = (*resources).iter().filter(|r| r.ecs_status == "待审批").count();
    let processing_count = (*resources).iter().filter(|r| r.ecs_status == "审批中" || r.ecs_status == "待交付").count();
    let rejected_count = (*resources).iter().filter(|r| r.ecs_status == "已驳回").count();

    html! {
        <div class="container p-4">
            <h1 class="title">{ lang.t("operations_management") }</h1>

            // Stats cards
            <div class="columns is-multiline">
                <div class="column is-3">
                    <div class="box has-background-white-bis">
                        <p class="heading">{"总数"}</p>
                        <p class="title is-4">{ total_count }</p>
                    </div>
                </div>
                <div class="column is-3">
                    <div class="box has-background-warning-light">
                        <p class="heading">{"待审批"}</p>
                        <p class="title is-4 has-text-warning">{ pending_count }</p>
                    </div>
                </div>
                <div class="column is-3">
                    <div class="box has-background-info-light">
                        <p class="heading">{"处理中"}</p>
                        <p class="title is-4 has-text-info">{ processing_count }</p>
                    </div>
                </div>
                <div class="column is-3">
                    <div class="box has-background-danger-light">
                        <p class="heading">{"已驳回"}</p>
                        <p class="title is-4 has-text-danger">{ rejected_count }</p>
                    </div>
                </div>
            </div>

            // Filter tabs
            <div class="tabs is-boxed mb-4">
                <ul>
                    <li class={Classes::from(&*active_tab_filter(|_| true, &resources))}>
                        <a onclick={
                            let resources = resources.clone();
                            Callback::from(move |_| {
                                // Show all
                            })
                        }>{"全部"}</a>
                    </li>
                    <li><a>{"待审批"}</a></li>
                    <li><a>{"处理中"}</a></li>
                    <li><a>{"已驳回"}</a></li>
                </ul>
            </div>

            // Resources table
            if *loading {
                <div class="has-text-centered py-6">
                    <span class="icon is-large">
                        <i class="fas fa-spinner fa-spin"></i>
                    </span>
                    <p>{"加载中..."}</p>
                </div>
            } else if (*resources).is_empty() {
                <div class="box has-background-white-bis has-text-centered py-6">
                    <p class="is-size-5 has-text-grey">{"暂无待处理业务申请"}</p>
                </div>
            } else {
                <div class="box">
                    <table class="table is-fullwidth is-hoverable">
                        <thead>
                            <tr>
                                <th>{"ID"}</th>
                                <th>{"ECS名称"}</th>
                                <th>{"云平台"}</th>
                                <th>{"区域"}</th>
                                <th>{"客户"}</th>
                                <th>{"状态"}</th>
                                <th>{"申请时间"}</th>
                                <th>{"操作"}</th>
                            </tr>
                        </thead>
                        <tbody>
                            {
                                (*resources).iter().map(|resource| {
                                    let on_view_detail = on_view_detail.clone();
                                    let on_approve_click = on_approve_click.clone();
                                    let on_supplement_click = on_supplement_click.clone();
                                    let resource_clone = resource.clone();

                                    html! {
                                        <tr key={resource.id.unwrap_or(0)}>
                                            <td>{ resource.id.unwrap_or(0) }</td>
                                            <td>{ &resource_clone.ecs_name }</td>
                                            <td>{ &resource_clone.cloud_category }</td>
                                            <td>{ &resource_clone.cloud_region }</td>
                                            <td>{ &resource_clone.customer_name }</td>
                                            <td>
                                                <span class={classes!("tag", status_class(&resource_clone.ecs_status))}>
                                                    { &resource_clone.ecs_status }
                                                </span>
                                            </td>
                                            <td>{
                                                resource_clone.created_at
                                                    .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
                                                    .unwrap_or_default()
                                            }</td>
                                            <td>
                                                <div class="buttons are-small">
                                                    <button class="button is-info is-light"
                                                        onclick={
                                                            let resource_clone = resource_clone.clone();
                                                            Callback::from(move |_| on_view_detail.emit(resource_clone.clone()))
                                                        }>
                                                        <span class="icon"><i class="fas fa-eye"></i></span>
                                                        <span>{"查看"}</span>
                                                    </button>
                                                    {
                                                        if resource_clone.ecs_status == "待审批" || resource_clone.ecs_status == "审批中" {
                                                            html! {
                                                                <>
                                                                    <button class="button is-success is-light"
                                                                        onclick={
                                                                            let resource_clone = resource_clone.clone();
                                                                            Callback::from(move |_| on_approve_click.emit(resource_clone.clone()))
                                                                        }>
                                                                        <span class="icon"><i class="fas fa-check"></i></span>
                                                                        <span>{"审批"}</span>
                                                                    </button>
                                                                    <button class="button is-warning is-light"
                                                                        onclick={
                                                                            let resource_clone = resource_clone.clone();
                                                                            Callback::from(move |_| on_supplement_click.emit(resource_clone.clone()))
                                                                        }>
                                                                        <span class="icon"><i class="fas fa-edit"></i></span>
                                                                        <span>{"补充信息"}</span>
                                                                    </button>
                                                                </>
                                                            }
                                                        } else if resource_clone.ecs_status == "待交付" {
                                                            html! {
                                                                <button class="button is-primary is-light"
                                                                    onclick={
                                                                        let resource_clone = resource_clone.clone();
                                                                        Callback::from(move |_| on_supplement_click.emit(resource_clone.clone()))
                                                                    }>
                                                                    <span class="icon"><i class="fas fa-plus"></i></span>
                                                                    <span>{"录入云资源"}</span>
                                                                </button>
                                                            }
                                                        } else {
                                                            html! {}
                                                        }
                                                    }
                                                </div>
                                            </td>
                                        </tr>
                                    }
                                }).collect::<Html>()
                            }
                        </tbody>
                    </table>
                </div>
            }

            // Detail Modal
            {
                if *show_detail_modal {
                    if let Some(resource) = &*selected_resource {
                        html! {
                            <div class="modal is-active">
                                <div class="modal-background" onclick={
                                    let show_detail_modal = show_detail_modal.clone();
                                    Callback::from(move |_| show_detail_modal.set(false))
                                }></div>
                                <div class="modal-card">
                                    <header class="modal-card-head">
                                        <p class="modal-card-title">{"业务申请详情"}</p>
                                        <button class="delete" onclick={
                                            let show_detail_modal = show_detail_modal.clone();
                                            Callback::from(move |_| show_detail_modal.set(false))
                                        }></button>
                                    </header>
                                    <section class="modal-card-body">
                                        <div class="content">
                                            <table class="table is-fullwidth">
                                                <tr><td><strong>{"ID"}</strong></td><td>{ resource.id.unwrap_or(0) }</td></tr>
                                                <tr><td><strong>{"资源类型"}</strong></td><td>{ &resource.resource_type }</td></tr>
                                                <tr><td><strong>{"ECS名称"}</strong></td><td>{ &resource.ecs_name }</td></tr>
                                                <tr><td><strong>{"云平台"}</strong></td><td>{ &resource.cloud_category }</td></tr>
                                                <tr><td><strong>{"区域"}</strong></td><td>{ &resource.cloud_region }</td></tr>
                                                <tr><td><strong>{"客户"}</strong></td><td>{ &resource.customer_name }</td></tr>
                                                <tr><td><strong>{"应用"}</strong></td><td>{ resource.application_name.as_ref().map(|s| s.as_str()).unwrap_or("-") }</td></tr>
                                                <tr><td><strong>{"合同"}</strong></td><td>{ resource.contract_name.as_ref().map(|s| s.as_str()).unwrap_or("-") }</td></tr>
                                                <tr><td><strong>{"实例类型"}</strong></td><td>{ &resource.ecs_type }</td></tr>
                                                <tr><td><strong>{"操作系统"}</strong></td><td>{ &resource.ecs_os }</td></tr>
                                                <tr><td><strong>{"CPU"}</strong></td><td>{ resource.cpu_cores }</td></tr>
                                                <tr><td><strong>{"内存"}</strong></td><td>{ resource.memory_gb }{" GB"}</td></tr>
                                                <tr><td><strong>{"系统盘"}</strong></td><td>{ format!("{} GB {}", resource.system_disk_size_gb, resource.system_disk) }</td></tr>
                                                <tr><td><strong>{"安全产品"}</strong></td><td>{ if resource.has_security_product { "是" } else { "否" } }</td></tr>
                                                <tr><td><strong>{"状态"}</strong></td><td>
                                                    <span class={classes!("tag", status_class(&resource.ecs_status))}>
                                                        { &resource.ecs_status }
                                                    </span>
                                                </td></tr>
                                                <tr><td><strong>{"IP地址"}</strong></td><td>{ if resource.ip_address.is_empty() { "-" } else { &resource.ip_address } }</td></tr>
                                                <tr><td><strong>{"创建时间"}</strong></td><td>{
                                                    resource.created_at.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string()).unwrap_or_default()
                                                }</td></tr>
                                            </table>
                                        </div>
                                    </section>
                                    <footer class="modal-card-foot">
                                        <button class="button" onclick={
                                            let show_detail_modal = show_detail_modal.clone();
                                            Callback::from(move |_| show_detail_modal.set(false))
                                        }>{"关闭"}</button>
                                    </footer>
                                </div>
                            </div>
                        }
                    } else { html! {} }
                } else { html! {} }
            }

            // Approve/Reject Modal
            {
                if *show_approve_modal {
                    if let Some(resource) = &*selected_resource {
                        html! {
                            <div class="modal is-active">
                                <div class="modal-background" onclick={
                                    let show_approve_modal = show_approve_modal.clone();
                                    Callback::from(move |_| show_approve_modal.set(false))
                                }></div>
                                <div class="modal-card">
                                    <header class="modal-card-head">
                                        <p class="modal-card-title">{"审批业务申请"}</p>
                                        <button class="delete" onclick={
                                            let show_approve_modal = show_approve_modal.clone();
                                            Callback::from(move |_| show_approve_modal.set(false))
                                        }></button>
                                    </header>
                                    <section class="modal-card-body">
                                        <p><strong>{"ECS名称: "}</strong>{ &resource.ecs_name }</p>
                                        <p><strong>{"云平台: "}</strong>{ &resource.cloud_category }</p>
                                        <p><strong>{"区域: "}</strong>{ &resource.cloud_region }</p>
                                        <p><strong>{"客户: "}</strong>{ &resource.customer_name }</p>
                                        <p><strong>{"配置: "}</strong>{ resource.cpu_cores }{" vCPU, "}{ resource.memory_gb }{" GB"}</p>
                                        <hr />
                                        {
                                            if resource.ecs_status == "待审批" {
                                                html! {
                                                    <div class="field">
                                                        <label class="label">{"审批决定"}</label>
                                                        <div class="control">
                                                            <label class="radio">
                                                                <input type="radio" name="approve_decision" checked={true} />
                                                                {"通过"}
                                                            </label>
                                                            <label class="radio">
                                                                <input type="radio" name="approve_decision" />
                                                                {"驳回"}
                                                            </label>
                                                        </div>
                                                    </div>
                                                }
                                            } else {
                                                html! {
                                                    <div class="notification is-info">
                                                        {"该申请已进入审批流程，是否确认通过并进入待交付状态？"}
                                                    </div>
                                                }
                                            }
                                        }
                                    </section>
                                    <footer class="modal-card-foot">
                                        <button class="button is-success"
                                            onclick={
                                                let resource_id = resource.id.unwrap_or(0);
                                                let on_submit_approve = on_submit_approve.clone();
                                                Callback::from(move |_| on_submit_approve.emit((resource_id, "approved".to_string(), String::new())))
                                            }>
                                            {"通过"}
                                        </button>
                                        <button class="button is-danger"
                                            onclick={
                                                let resource_id = resource.id.unwrap_or(0);
                                                let on_submit_approve = on_submit_approve.clone();
                                                Callback::from(move |_| on_submit_approve.emit((resource_id, "rejected".to_string(), String::new())))
                                            }>
                                            {"驳回"}
                                        </button>
                                        <button class="button" onclick={
                                            let show_approve_modal = show_approve_modal.clone();
                                            Callback::from(move |_| show_approve_modal.set(false))
                                        }>{"取消"}</button>
                                    </footer>
                                </div>
                            </div>
                        }
                    } else { html! {} }
                } else { html! {} }
            }

            // Supplement Info Modal
            {
                if *show_supplement_modal {
                    if let Some(resource) = &*selected_resource {
                        html! {
                            <div class="modal is-active">
                                <div class="modal-background" onclick={
                                    let show_supplement_modal = show_supplement_modal.clone();
                                    Callback::from(move |_| show_supplement_modal.set(false))
                                }></div>
                                <div class="modal-card" style="width: 800px;">
                                    <header class="modal-card-head">
                                        <p class="modal-card-title">
                                            { if resource.ecs_status == "待交付" { "录入云资源" } else { "补充信息" } }
                                        </p>
                                        <button class="delete" onclick={
                                            let show_supplement_modal = show_supplement_modal.clone();
                                            Callback::from(move |_| show_supplement_modal.set(false))
                                        }></button>
                                    </header>
                                    <section class="modal-card-body">
                                        <div class="columns">
                                            <div class="column is-6">
                                                <div class="field">
                                                    <label class="label">{"IP地址"}</label>
                                                    <div class="control">
                                                        <input class="input" type="text"
                                                            value={(*supplement_form).ip_address.clone().unwrap_or_default()}
                                                            placeholder="192.168.1.100"
                                                            onchange={
                                                                let on_form_input = on_form_input.clone();
                                                                Callback::from(move |e: Event| {
                                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                                    on_form_input.emit(("ip_address".to_string(), input.value()));
                                                                })
                                                            }
                                                        />
                                                    </div>
                                                </div>
                                            </div>
                                            <div class="column is-6">
                                                <div class="field">
                                                    <label class="label">{"登录方式"}</label>
                                                    <div class="control">
                                                        <div class="select is-fullwidth">
                                                            <select
                                                                onchange={
                                                                    let on_form_input = on_form_input.clone();
                                                                    Callback::from(move |e: Event| {
                                                                        let select: HtmlSelectElement = e.target_unchecked_into();
                                                                        on_form_input.emit(("ecs_login_method".to_string(), select.value()));
                                                                    })
                                                                }
                                                            >
                                                                <option value="" selected={(*supplement_form).ecs_login_method.is_none()}>{"请选择"}</option>
                                                                <option value="SSH" selected={(*supplement_form).ecs_login_method.as_ref() == Some(&"SSH".to_string())}>{"SSH"}</option>
                                                                <option value="RDP" selected={(*supplement_form).ecs_login_method.as_ref() == Some(&"RDP".to_string())}>{"RDP"}</option>
                                                                <option value="堡垒机" selected={(*supplement_form).ecs_login_method.as_ref() == Some(&"堡垒机".to_string())}>{"堡垒机"}</option>
                                                            </select>
                                                        </div>
                                                    </div>
                                                </div>
                                            </div>
                                        </div>

                                        <div class="columns">
                                            <div class="column is-6">
                                                <div class="field">
                                                    <label class="label">{"登录用户名"}</label>
                                                    <div class="control">
                                                        <input class="input" type="text"
                                                            value={(*supplement_form).ecs_login_username.clone().unwrap_or_default()}
                                                            placeholder="root / administrator"
                                                            onchange={
                                                                let on_form_input = on_form_input.clone();
                                                                Callback::from(move |e: Event| {
                                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                                    on_form_input.emit(("ecs_login_username".to_string(), input.value()));
                                                                })
                                                            }
                                                        />
                                                    </div>
                                                </div>
                                            </div>
                                            <div class="column is-6">
                                                <div class="field">
                                                    <label class="label">{"初始密码"}</label>
                                                    <div class="control">
                                                        <input class="input" type="password"
                                                            value={(*supplement_form).ecs_initial_password.clone().unwrap_or_default()}
                                                            placeholder="••••••••"
                                                            onchange={
                                                                let on_form_input = on_form_input.clone();
                                                                Callback::from(move |e: Event| {
                                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                                    on_form_input.emit(("ecs_initial_password".to_string(), input.value()));
                                                                })
                                                            }
                                                        />
                                                    </div>
                                                </div>
                                            </div>
                                        </div>

                                        // 堡垒机信息
                                        <div class="columns">
                                            <div class="column is-6">
                                                <div class="field">
                                                    <label class="label">{"堡垒机地址"}</label>
                                                    <div class="control">
                                                        <input class="input" type="text"
                                                            value={(*supplement_form).bastion_address.clone().unwrap_or_default()}
                                                            placeholder="bastion.example.com"
                                                            onchange={
                                                                let on_form_input = on_form_input.clone();
                                                                Callback::from(move |e: Event| {
                                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                                    on_form_input.emit(("bastion_address".to_string(), input.value()));
                                                                })
                                                            }
                                                        />
                                                    </div>
                                                </div>
                                            </div>
                                            <div class="column is-6">
                                                <div class="field">
                                                    <label class="label">{"堡垒机账号"}</label>
                                                    <div class="control">
                                                        <input class="input" type="text"
                                                            value={(*supplement_form).bastion_admin_account.clone().unwrap_or_default()}
                                                            placeholder="admin"
                                                            onchange={
                                                                let on_form_input = on_form_input.clone();
                                                                Callback::from(move |e: Event| {
                                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                                    on_form_input.emit(("bastion_admin_account".to_string(), input.value()));
                                                                })
                                                            }
                                                        />
                                                    </div>
                                                </div>
                                            </div>
                                        </div>

                                        <div class="field">
                                            <label class="label">{"备注"}</label>
                                            <div class="control">
                                                <textarea class="textarea"
                                                    placeholder="添加备注信息..."
                                                    rows="3"
                                                    onchange={
                                                        let on_form_input = on_form_input.clone();
                                                        Callback::from(move |e: Event| {
                                                            let textarea: HtmlTextAreaElement = e.target_unchecked_into();
                                                            on_form_input.emit(("remarks".to_string(), textarea.value()));
                                                        })
                                                    }
                                                >
                                                    { (*supplement_form).remarks.clone().unwrap_or_default() }
                                                </textarea>
                                            </div>
                                        </div>

                                        {
                                            if resource.ecs_status == "待交付" {
                                                html! {
                                                    <div class="notification is-info">
                                                        <p class="heading">{"提示"}</p>
                                                        <p>{"提交后将创建云资产记录，请确保信息填写完整。"}</p>
                                                    </div>
                                                }
                                            } else {
                                                html! {}
                                            }
                                        }
                                    </section>
                                    <footer class="modal-card-foot">
                                        <button class="button is-primary" onclick={on_submit_supplement.clone()}>
                                            {"保存"}
                                        </button>
                                        <button class="button" onclick={
                                            let show_supplement_modal = show_supplement_modal.clone();
                                            Callback::from(move |_| show_supplement_modal.set(false))
                                        }>{"取消"}</button>
                                    </footer>
                                </div>
                            </div>
                        }
                    } else { html! {} }
                } else { html! {} }
            }
        </div>
    }
}

// Helper function for active tab
fn active_tab_filter<F>(f: F, resources: &UseStateHandle<Vec<BusinessResource>>) -> &'static str
where
    F: Fn(&BusinessResource) -> bool,
{
    if (*resources).iter().filter(|r| f(r)).count() > 0 {
        "is-active"
    } else {
        ""
    }
}

// ============== Cloud Service Asset Management Component ==============
// 统一纳管物理机和云虚拟机

#[function_component]
fn CloudServiceAssetManagement() -> Html {
    let lang = use_state(|| Language::Zh);
    let assets = use_state(|| Vec::<CloudServiceAsset>::new());
    let stats = use_state(|| None::<CloudServiceAssetStats>);
    let loading = use_state(|| true);

    // 过滤条件
    let filter_asset_type = use_state(|| None::<String>);
    let filter_status = use_state(|| None::<String>);
    let filter_provider = use_state(|| None::<String>);
    let search_keyword = use_state(|| String::new());

    let token = get_auth_token();

    // 获取统计数据
    {
        let stats = stats.clone();
        let token = token.clone();
        use_effect_with((), move |_: &()| {
            spawn_local(async move {
                if let Ok(resp) = Request::get(&api_url("cloud-service-assets/stats"))
                    .header("Authorization", &token)
                    .send()
                    .await
                {
                    if let Ok(data) = resp.json::<CloudServiceAssetStats>().await {
                        stats.set(Some(data));
                    }
                }
            });
            || ()
        });
    }

    // 获取资产列表的函数
    let fetch_assets_fn = {
        let assets = assets.clone();
        let loading = loading.clone();
        let token = token.clone();
        let filter_asset_type = filter_asset_type.clone();
        let filter_status = filter_status.clone();
        let filter_provider = filter_provider.clone();
        let search_keyword = search_keyword.clone();

        move || {
            let assets = assets.clone();
            let loading = loading.clone();
            let token = token.clone();
            let filter_asset_type = filter_asset_type.clone();
            let filter_status = filter_status.clone();
            let filter_provider = filter_provider.clone();
            let search_keyword = search_keyword.clone();

            spawn_local(async move {
                loading.set(true);
                let mut url = api_url("cloud-service-assets");
                let mut has_params = false;

                if let Some(t) = &*filter_asset_type {
                    url.push_str(&format!("?asset_type={}", t));
                    has_params = true;
                }
                if let Some(s) = &*filter_status {
                    url.push_str(&format!("{}status={}", if has_params { "&" } else { "?" }, s));
                    has_params = true;
                }
                if let Some(p) = &*filter_provider {
                    url.push_str(&format!("{}cloud_provider={}", if has_params { "&" } else { "?" }, p));
                    has_params = true;
                }
                if !search_keyword.is_empty() {
                    let sk = (*search_keyword).clone();
                    url.push_str(&format!("{}search_keyword={}", if has_params { "&" } else { "?" }, sk));
                }

                if let Ok(resp) = Request::get(&url).header("Authorization", &token).send().await {
                    if let Ok(data) = resp.json::<Vec<CloudServiceAsset>>().await {
                        assets.set(data);
                    }
                }
                loading.set(false);
            });
        }
    };

    // 初始加载
    {
        let fetch_assets_fn = fetch_assets_fn.clone();
        use_effect_with((), move |_: &()| {
            fetch_assets_fn();
            || ()
        });
    }

    // 状态标签样式
    fn status_class(status: &str) -> &'static str {
        match status {
            "运行中" => "is-success",
            "stopped" | "已停止" => "is-dark",
            "released" | "已释放" => "is-danger",
            s if s.contains("待") => "is-warning",
            _ => "is-info",
        }
    }

    // 资产类型标签样式
    fn asset_type_class(asset_type: &str) -> &'static str {
        match asset_type {
            "physical" => "is-primary",
            "virtual" => "is-link",
            _ => "is-light",
        }
    }

    // 克隆 fetch_assets_fn 供各个事件使用
    let fetch_assets_onclick = {
        let fetch_assets_fn = fetch_assets_fn.clone();
        Callback::from(move |_| {
            fetch_assets_fn();
        })
    };

    let on_clear_filters = {
        let filter_asset_type = filter_asset_type.clone();
        let filter_status = filter_status.clone();
        let filter_provider = filter_provider.clone();
        let search_keyword = search_keyword.clone();
        let fetch_assets_fn = fetch_assets_fn.clone();

        Callback::from(move |_| {
            filter_asset_type.set(None);
            filter_status.set(None);
            filter_provider.set(None);
            search_keyword.set(String::new());
            fetch_assets_fn();
        })
    };

    html! {
        <div class="container" style="margin-top: 20px;">
            <h1 class="title">{ "云服务资产管理" }</h1>
            <p class="subtitle">{ "统一纳管物理机和云虚拟机资产" }</p>

            // 统计卡片
            if let Some(s) = (*stats).clone() {
                <div class="columns" style="margin-top: 20px;">
                    <div class="column">
                        <div class="box">
                            <p class="heading">{ "总资产数" }</p>
                            <p class="title">{ s.total_count }</p>
                        </div>
                    </div>
                    <div class="column">
                        <div class="box">
                            <p class="heading">{ "物理机" }</p>
                            <p class="title has-text-primary">{ s.physical_count }</p>
                        </div>
                    </div>
                    <div class="column">
                        <div class="box">
                            <p class="heading">{ "云虚拟机" }</p>
                            <p class="title has-text-link">{ s.virtual_count }</p>
                        </div>
                    </div>
                    <div class="column">
                        <div class="box">
                            <p class="heading">{ "运行中" }</p>
                            <p class="title has-text-success">{ s.running_count }</p>
                        </div>
                    </div>
                    <div class="column">
                        <div class="box">
                            <p class="heading">{ "总CPU核数" }</p>
                            <p class="title">{ s.total_cpu_cores }</p>
                        </div>
                    </div>
                    <div class="column">
                        <div class="box">
                            <p class="heading">{ "总内存(GB)" }</p>
                            <p class="title">{ s.total_memory_gb }</p>
                        </div>
                    </div>
                </div>
            }

            // 过滤器
            <div class="box" style="margin-top: 20px;">
                <div class="columns">
                    <div class="column is-2">
                        <label class="label">{ "资产类型" }</label>
                        <div class="select is-fullwidth">
                            <select
                                onchange={
                                    let filter_asset_type = filter_asset_type.clone();
                                    let fetch_assets_fn = fetch_assets_fn.clone();
                                    Callback::from(move |e: Event| {
                                        let select: HtmlSelectElement = e.target_unchecked_into();
                                        let value = select.value();
                                        filter_asset_type.set(if value.is_empty() { None } else { Some(value) });
                                        fetch_assets_fn();
                                    })
                                }
                            >
                                <option value="" selected={(*filter_asset_type).is_none()}>{ "全部" }</option>
                                <option value="physical" selected={(*filter_asset_type).as_ref() == Some(&"physical".to_string())}>{ "物理机" }</option>
                                <option value="virtual" selected={(*filter_asset_type).as_ref() == Some(&"virtual".to_string())}>{ "云虚拟机" }</option>
                            </select>
                        </div>
                    </div>
                    <div class="column is-2">
                        <label class="label">{ "状态" }</label>
                        <div class="select is-fullwidth">
                            <select
                                onchange={
                                    let filter_status = filter_status.clone();
                                    let fetch_assets_fn = fetch_assets_fn.clone();
                                    Callback::from(move |e: Event| {
                                        let select: HtmlSelectElement = e.target_unchecked_into();
                                        let value = select.value();
                                        filter_status.set(if value.is_empty() { None } else { Some(value) });
                                        fetch_assets_fn();
                                    })
                                }
                            >
                                <option value="" selected={(*filter_status).is_none()}>{ "全部" }</option>
                                <option value="运行中" selected={(*filter_status).as_ref() == Some(&"运行中".to_string())}>{ "运行中" }</option>
                                <option value="已停止" selected={(*filter_status).as_ref() == Some(&"已停止".to_string())}>{ "已停止" }</option>
                            </select>
                        </div>
                    </div>
                    <div class="column is-2">
                        <label class="label">{ "云厂商" }</label>
                        <div class="select is-fullwidth">
                            <select
                                onchange={
                                    let filter_provider = filter_provider.clone();
                                    let fetch_assets_fn = fetch_assets_fn.clone();
                                    Callback::from(move |e: Event| {
                                        let select: HtmlSelectElement = e.target_unchecked_into();
                                        let value = select.value();
                                        filter_provider.set(if value.is_empty() { None } else { Some(value) });
                                        fetch_assets_fn();
                                    })
                                }
                            >
                                <option value="" selected={(*filter_provider).is_none()}>{ "全部" }</option>
                                <option value="阿里云" selected={(*filter_provider).as_ref() == Some(&"阿里云".to_string())}>{ "阿里云" }</option>
                                <option value="腾讯云" selected={(*filter_provider).as_ref() == Some(&"腾讯云".to_string())}>{ "腾讯云" }</option>
                                <option value="华为云" selected={(*filter_provider).as_ref() == Some(&"华为云".to_string())}>{ "华为云" }</option>
                                <option value="AWS" selected={(*filter_provider).as_ref() == Some(&"AWS".to_string())}>{ "AWS" }</option>
                            </select>
                        </div>
                    </div>
                    <div class="column is-4">
                        <label class="label">{ "搜索" }</label>
                        <input class="input"
                            type="text"
                            placeholder="资产名称/实例ID/IP地址"
                            value={(*search_keyword).clone()}
                            onchange={
                                let search_keyword = search_keyword.clone();
                                Callback::from(move |e: Event| {
                                    let input: HtmlInputElement = e.target_unchecked_into();
                                    search_keyword.set(input.value());
                                })
                            }
                        />
                    </div>
                    <div class="column is-2">
                        <label class="label">{ " " }</label>
                        <div class="buttons">
                            <button class="button is-primary" onclick={fetch_assets_onclick.clone()}>
                                { "搜索" }
                            </button>
                            <button class="button" onclick={on_clear_filters.clone()}>
                                { "清除" }
                            </button>
                        </div>
                    </div>
                </div>
            </div>

            // 资产列表
            if *loading {
                <div class="section">
                    <progress class="progress is-small is-primary" max="100">{ "30%" }</progress>
                </div>
            } else {
                <div class="table-container" style="margin-top: 20px;">
                    <table class="table is-fullwidth is-hoverable">
                        <thead>
                            <tr>
                                <th>{ "ID" }</th>
                                <th>{ "ECS名称" }</th>
                                <th>{ "云平台" }</th>
                                <th>{ "区域" }</th>
                                <th>{ "客户" }</th>
                                <th>{ "状态" }</th>
                                <th>{ "来源" }</th>
                                <th>{ "操作" }</th>
                            </tr>
                        </thead>
                        <tbody>
                            { for (*assets).iter().map(|asset| {
                                let source_label = if asset.source_type == "business_resource" { "业务受理" } else { "混合云" };

                                html! {
                                    <tr>
                                        <td>{ &asset.id }</td>
                                        <td>{ &asset.name }</td>
                                        <td>{ &asset.cloud_provider }</td>
                                        <td>{ &asset.region }</td>
                                        <td>{ &asset.customer_name }</td>
                                        <td>
                                            <span class={classes!("tag", status_class(&asset.status))}>
                                                { &asset.status }
                                            </span>
                                        </td>
                                        <td>
                                            <span class="tag is-light">{ source_label }</span>
                                        </td>
                                        <td>
                                            <button class="button is-small is-info is-light">
                                                { "查看" }
                                            </button>
                                        </td>
                                    </tr>
                                }
                            }).collect::<Vec<_>>() }
                        </tbody>
                    </table>
                    { if (*assets).is_empty() {
                        html! { <div class="has-text-centered" style="padding: 40px;">{ "暂无数据" }</div> }
                    } else { html! {} }}
                </div>
            }
        </div>
    }
}

// ============== User Management Component ==============

#[function_component]
fn UserManagement() -> Html {
    let lang = use_state(|| Language::Zh);
    let users = use_state(|| Vec::new());
    let loading = use_state(|| true);

    // Permission editing state
    let show_permissions_modal = use_state(|| false);
    let editing_user = use_state(|| None as Option<(String, String, Permissions)>); // (id, username, permissions)
    let temp_permissions = use_state(|| None as Option<Permissions>);
    let success_message = use_state(|| None as Option<String>);

    // Create user state
    let show_create_modal = use_state(|| false);
    let new_username = use_state(|| String::new());
    let new_password = use_state(|| String::new());
    let new_role = use_state(|| String::from("SysAdmin"));
    let create_message = use_state(|| None as Option<String>);

    // Get token from localStorage
    let token = get_auth_token();

    // Get current user role
    let user_role = get_user_role();

    // Use effect to fetch data only once on mount
    use_effect_with((), {
        let users = users.clone();
        let loading = loading.clone();
        let token = token.clone();

        move |_| {
            spawn_local(async move {
                if let Ok(resp) = Request::get(&api_url("users")).header("Authorization", &token).send().await {
                    if let Ok(data) = resp.json::<Vec<User>>().await {
                        users.set(data);
                    }
                }
                loading.set(false);
            });
            || ()
        }
    });

    let role_name = |role: &Role| -> String {
        match role {
            Role::SysAdmin => "SysAdmin".to_string(),
            Role::SecAdmin => "SecAdmin".to_string(),
            Role::Auditor => "Auditor".to_string(),
            Role::Custom(name) => name.clone(),
        }
    };

    // Open create user modal
    let on_open_create_modal = {
        let show_create_modal = show_create_modal.clone();
        let new_username = new_username.clone();
        let new_password = new_password.clone();
        let new_role = new_role.clone();
        let create_message = create_message.clone();

        Callback::from(move |_| {
            new_username.set(String::new());
            new_password.set(String::new());
            new_role.set(String::from("SysAdmin"));
            create_message.set(None);
            show_create_modal.set(true);
        })
    };

    // Close create modal
    let on_close_create_modal = {
        let show_create_modal = show_create_modal.clone();

        Callback::from(move |_| {
            show_create_modal.set(false);
        })
    };

    // Create user
    let on_create_user = {
        let token = token.clone();
        let users = users.clone();
        let new_username = new_username.clone();
        let new_password = new_password.clone();
        let new_role = new_role.clone();
        let show_create_modal = show_create_modal.clone();
        let create_message = create_message.clone();

        Callback::from(move |_| {
            let username = (*new_username).clone();
            let password = (*new_password).clone();
            let role_str = (*new_role).clone();
            let token = token.clone();
            let users = users.clone();
            let show_create_modal = show_create_modal.clone();
            let create_message = create_message.clone();

            if username.is_empty() || password.is_empty() {
                create_message.set(Some("用户名和密码不能为空".to_string()));
                return;
            }

            // Parse role
            let role = match role_str.as_str() {
                "SecAdmin" => Role::SecAdmin,
                "Auditor" => Role::Auditor,
                _ => Role::SysAdmin,
            };

            spawn_local(async move {
                let req = CreateUserRequest {
                    username: username.clone(),
                    password: password.clone(),
                    role,
                };

                if let Ok(resp) = Request::post(&api_url("users"))
                    .header("Authorization", &token)
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_string(&req).unwrap_or_default())
                    .unwrap()
                    .send()
                    .await
                {
                    if resp.ok() {
                        // Refresh users
                        if let Ok(resp) = Request::get(&api_url("users"))
                            .header("Authorization", &token)
                            .send()
                            .await
                        {
                            if let Ok(data) = resp.json::<Vec<User>>().await {
                                users.set(data);
                            }
                        }
                        show_create_modal.set(false);
                    } else {
                        create_message.set(Some("创建用户失败".to_string()));
                    }
                }
            });
        })
    };

    // Delete user
    let on_delete_user = {
        let token = token.clone();
        let users = users.clone();

        Callback::from(move |user_id: String| {
            let token = token.clone();
            let users = users.clone();

            spawn_local(async move {
                if let Ok(resp) = Request::delete(&format!("{}/{}", api_url("users"), user_id))
                    .header("Authorization", &token)
                    .send()
                    .await
                {
                    if resp.ok() {
                        // Refresh users
                        if let Ok(resp) = Request::get(&api_url("users"))
                            .header("Authorization", &token)
                            .send()
                            .await
                        {
                            if let Ok(data) = resp.json::<Vec<User>>().await {
                                users.set(data);
                            }
                        }
                    }
                }
            });
        })
    };

    // Open permission edit modal
    let on_edit_permissions = {
        let users = users.clone();
        let show_permissions_modal = show_permissions_modal.clone();
        let editing_user = editing_user.clone();
        let temp_permissions = temp_permissions.clone();

        Callback::from(move |user_id: String| {
            if let Some(user) = users.iter().find(|u| u.id == user_id) {
                if let Some(perms) = &user.permissions {
                    editing_user.set(Some((user.id.clone(), user.username.clone(), perms.clone())));
                    temp_permissions.set(Some(perms.clone()));
                    show_permissions_modal.set(true);
                }
            }
        })
    };

    // Close modal
    let on_close_modal = {
        let show_permissions_modal = show_permissions_modal.clone();
        let editing_user = editing_user.clone();
        let temp_permissions = temp_permissions.clone();

        Callback::from(move |_| {
            show_permissions_modal.set(false);
            editing_user.set(None);
            temp_permissions.set(None);
        })
    };

    // Save permissions
    let on_save_permissions = {
        let token = token.clone();
        let users = users.clone();
        let editing_user = editing_user.clone();
        let temp_permissions = temp_permissions.clone();
        let show_permissions_modal = show_permissions_modal.clone();
        let success_message = success_message.clone();

        Callback::from(move |_| {
            if let Some((user_id, username, _)) = &*editing_user {
                if let Some(perms) = &*temp_permissions {
                    let user_id = user_id.clone();
                    let perms = perms.clone();
                    let token = token.clone();
                    let users = users.clone();
                    let show_permissions_modal = show_permissions_modal.clone();
                    let username = username.clone();
                    let success_message = success_message.clone();

                    spawn_local(async move {
                        let json_body = serde_json::to_string(&perms).unwrap_or_default();
                        let url = format!("{}/{}/permissions", api_url("users"), user_id);

                        if let Ok(resp) = Request::put(&url)
                            .header("Authorization", &token)
                            .header("Content-Type", "application/json")
                            .body(json_body)
                            .unwrap()
                            .send()
                            .await
                        {
                            if resp.ok() {
                                // Refresh users
                                if let Ok(resp) = Request::get(&api_url("users"))
                                    .header("Authorization", &token)
                                    .send()
                                    .await
                                {
                                    if let Ok(data) = resp.json::<Vec<User>>().await {
                                        users.set(data);
                                    }
                                }
                                show_permissions_modal.set(false);
                                // 显示成功提示
                                success_message.set(Some(format!(
                                    "✅ 用户 {} 的权限已成功更新！\n\n⚠️ 注意：该用户需要退出重新登录才能使新权限生效。",
                                    username
                                )));
                                // 5秒后自动隐藏提示
                                let success_message = success_message.clone();
                                Timeout::new(5000, move || {
                                    success_message.set(None);
                                }).forget();
                            }
                        }
                    });
                }
            }
        })
    };

    // Toggle permission - create individual callbacks for each permission
    // 三层级权限关联逻辑：顶级 -> 子级 -> 孙级
    let temp_perms_for_callbacks = temp_permissions.clone();

    let make_toggle_callback = |field: String| {
        let temp_permissions = temp_perms_for_callbacks.clone();
        Callback::from(move |e: Event| {
            let target = e.target_unchecked_into::<HtmlInputElement>();
            let value = target.checked();
            let field = field.clone();

            if let Some(mut perms) = (*temp_permissions).clone() {
                match field.as_str() {
                    // ========== 顶级权限：控制整个模块 ==========
                    "can_access_general" => {
                        perms.can_access_general = value;
                        if !value {
                            // 取消顶级时，自动取消所有子级和孙级
                            perms.can_view_dashboard = false;
                            perms.can_view_tasks = false;
                            perms.can_create_task = false;
                            perms.can_delete_task = false;
                            perms.can_update_task = false;
                            perms.can_view_advanced_scan = false;
                            perms.can_create_scan = false;
                            perms.can_delete_scan = false;
                            perms.can_export_scan = false;
                        }
                    }

                    "can_access_assets_risks" => {
                        perms.can_access_assets_risks = value;
                        if !value {
                            perms.can_view_cloud_assets = false;
                            perms.can_create_cloud_asset = false;
                            perms.can_update_cloud_asset = false;
                            perms.can_delete_cloud_asset = false;
                            perms.can_view_risks = false;
                            perms.can_resolve_risk = false;
                            perms.can_delete_risk = false;
                            perms.can_view_business_process = false;
                        }
                    }

                    "can_access_cloud" => {
                        perms.can_access_cloud = value;
                        if !value {
                            perms.can_view_cloud_providers = false;
                            perms.can_manage_cloud_providers = false;
                            perms.can_view_cloud_management = false;
                            perms.can_manage_cloud = false;
                            perms.can_delete_cloud = false;
                            perms.can_sync_cloud = false;
                        }
                    }

                    "can_access_user_management" => {
                        perms.can_access_user_management = value;
                        if !value {
                            perms.can_view_users = false;
                            perms.can_create_user = false;
                            perms.can_update_user = false;
                            perms.can_delete_user = false;
                            perms.can_manage_permissions = false;
                            perms.can_view_password_policy = false;
                            perms.can_manage_password_policy = false;
                        }
                    }

                    "can_access_audit" => {
                        perms.can_access_audit = value;
                        if !value {
                            perms.can_view_audit_logs = false;
                        }
                    }

                    // ========== 子级权限：控制页面访问 ==========
                    "can_view_dashboard" => perms.can_view_dashboard = value,

                    "can_view_tasks" => {
                        perms.can_view_tasks = value;
                        if !value {
                            perms.can_create_task = false;
                            perms.can_delete_task = false;
                            perms.can_update_task = false;
                        }
                    }

                    "can_view_advanced_scan" => {
                        perms.can_view_advanced_scan = value;
                        if !value {
                            perms.can_create_scan = false;
                            perms.can_delete_scan = false;
                            perms.can_export_scan = false;
                        }
                    }

                    "can_view_cloud_assets" => {
                        perms.can_view_cloud_assets = value;
                        if !value {
                            perms.can_create_cloud_asset = false;
                            perms.can_update_cloud_asset = false;
                            perms.can_delete_cloud_asset = false;
                        }
                    }

                    "can_view_risks" => {
                        perms.can_view_risks = value;
                        if !value {
                            perms.can_resolve_risk = false;
                            perms.can_delete_risk = false;
                        }
                    }

                    "can_view_business_process" => perms.can_view_business_process = value,

                    "can_view_cloud_providers" => {
                        perms.can_view_cloud_providers = value;
                        if !value {
                            perms.can_manage_cloud_providers = false;
                        }
                    }

                    "can_view_cloud_management" => {
                        perms.can_view_cloud_management = value;
                        if !value {
                            perms.can_manage_cloud = false;
                            perms.can_delete_cloud = false;
                            perms.can_sync_cloud = false;
                        }
                    }

                    "can_view_users" => {
                        perms.can_view_users = value;
                        if !value {
                            perms.can_create_user = false;
                            perms.can_update_user = false;
                            perms.can_delete_user = false;
                            perms.can_manage_permissions = false;
                        }
                    }

                    "can_view_password_policy" => {
                        perms.can_view_password_policy = value;
                        if !value {
                            perms.can_manage_password_policy = false;
                        }
                    }

                    "can_view_audit_logs" => perms.can_view_audit_logs = value,

                    // ========== 孙级权限：控制具体操作 ==========
                    "can_create_task" => perms.can_create_task = value,
                    "can_delete_task" => perms.can_delete_task = value,
                    "can_update_task" => perms.can_update_task = value,
                    "can_create_scan" => perms.can_create_scan = value,
                    "can_delete_scan" => perms.can_delete_scan = value,
                    "can_export_scan" => perms.can_export_scan = value,
                    "can_create_cloud_asset" => perms.can_create_cloud_asset = value,
                    "can_update_cloud_asset" => perms.can_update_cloud_asset = value,
                    "can_delete_cloud_asset" => perms.can_delete_cloud_asset = value,
                    "can_resolve_risk" => perms.can_resolve_risk = value,
                    "can_delete_risk" => perms.can_delete_risk = value,
                    "can_manage_cloud_providers" => perms.can_manage_cloud_providers = value,
                    "can_manage_cloud" => perms.can_manage_cloud = value,
                    "can_delete_cloud" => perms.can_delete_cloud = value,
                    "can_sync_cloud" => perms.can_sync_cloud = value,
                    "can_create_user" => perms.can_create_user = value,
                    "can_update_user" => perms.can_update_user = value,
                    "can_delete_user" => perms.can_delete_user = value,
                    "can_manage_permissions" => perms.can_manage_permissions = value,
                    "can_manage_password_policy" => perms.can_manage_password_policy = value,

                    _ => {}
                }
                temp_permissions.set(Some(perms));
            }
        })
    };

    html! {
        <div class="container p-4">
            <div class="is-flex is-justify-content-space-between is-align-items-center mb-4">
                <h1 class="title">{ lang.t("user_management") }</h1>
                if user_role == Some(Role::SysAdmin) {
                    <button class="button is-primary" onclick={on_open_create_modal.clone()}>
                        <span class="icon"><i class="fas fa-plus"></i></span>
                        <span>{ "创建用户" }</span>
                    </button>
                }
            </div>

            // 成功提示消息
            if let Some(msg) = &*success_message {
                <div class="notification is-success is-light" style="margin-bottom: 1rem; position: relative;">
                    <button class="delete" onclick={ {
                        let success_message = success_message.clone();
                        Callback::from(move |_| success_message.set(None))
                    } }></button>
                    <p style="white-space: pre-line;">{ msg.clone() }</p>
                </div>
            }

            <div class="box">
                if (*users).is_empty() && *loading {
                    <p class="has-text-centered has-text-grey">{ "Loading..." }</p>
                } else if (*users).is_empty() {
                    <p class="has-text-centered has-text-grey">{ "No users found" }</p>
                } else {
                    <table class="table is-fullwidth is-hoverable">
                        <thead>
                            <tr>
                                <th>{ lang.t("username") }</th>
                                <th>{ lang.t("role") }</th>
                                <th>{ lang.t("timestamp") }</th>
                                <th>{ "操作" }</th>
                            </tr>
                        </thead>
                        <tbody>
                            { for users.iter().map(|user| {
                                let user_id_edit = user.id.clone();
                                let user_id_delete = user.id.clone();
                                let on_edit = on_edit_permissions.clone();
                                let current_user_role = user_role.clone();
                                let on_delete = on_delete_user.clone();
                                let username = user.username.clone();

                                html! {
                                    <tr>
                                        <td>{ &user.username }</td>
                                        <td>
                                            <span class="tag">{ role_name(&user.role) }</span>
                                        </td>
                                        <td>{ &user.created_at.format("%Y-%m-%d %H:%M:%S").to_string() }</td>
                                        <td>
                                            <div class="buttons are-small">
                                                // Only admin can edit permissions
                                                if current_user_role == Some(Role::SysAdmin) && user.permissions.is_some() {
                                                    <button
                                                        class="button is-small is-info"
                                                        onclick={move |_| on_edit.emit(user_id_edit.clone())}
                                                    >
                                                        { "编辑权限" }
                                                    </button>
                                                }
                                                // Only admin can delete users (but not themselves)
                                                if current_user_role == Some(Role::SysAdmin) && username != "admin" {
                                                    <button
                                                        class="button is-small is-danger"
                                                        onclick={move |_| on_delete.emit(user_id_delete.clone())}
                                                    >
                                                        { "删除" }
                                                    </button>
                                                }
                                            </div>
                                        </td>
                                    </tr>
                                }
                            })}
                        </tbody>
                    </table>
                }
            </div>

            // Create User Modal
            if *show_create_modal {
                <div class="modal is-active">
                    <div class="modal-background" onclick={on_close_create_modal.clone()}></div>
                    <div class="modal-card" style="width: 500px;">
                        <header class="modal-card-head">
                            <p class="modal-card-title">{ "创建用户" }</p>
                            <button class="delete" onclick={on_close_create_modal.clone()}></button>
                        </header>
                        <section class="modal-card-body">
                            if let Some(msg) = &*create_message {
                                <div class="notification is-danger">{ msg }</div>
                            }
                            <div class="field">
                                <label class="label">{ "用户名" }</label>
                                <div class="control">
                                    <input
                                        class="input"
                                        type="text"
                                        placeholder="输入用户名"
                                        value={(*new_username).clone()}
                                        oninput={
                                            let new_username = new_username.clone();
                                            Callback::from(move |e: InputEvent| {
                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                new_username.set(input.value());
                                            })
                                        }
                                    />
                                </div>
                            </div>
                            <div class="field">
                                <label class="label">{ "密码" }</label>
                                <div class="control">
                                    <input
                                        class="input"
                                        type="password"
                                        placeholder="输入密码"
                                        value={(*new_password).clone()}
                                        oninput={
                                            let new_password = new_password.clone();
                                            Callback::from(move |e: InputEvent| {
                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                new_password.set(input.value());
                                            })
                                        }
                                    />
                                </div>
                            </div>
                            <div class="field">
                                <label class="label">{ "角色" }</label>
                                <div class="control">
                                    <div class="select is-fullwidth">
                                        <select
                                            value={(*new_role).clone()}
                                            onchange={
                                                let new_role = new_role.clone();
                                                Callback::from(move |e: Event| {
                                                    let select = e.target_unchecked_into::<HtmlSelectElement>();
                                                    new_role.set(select.value());
                                                })
                                            }
                                        >
                                            <option value="SysAdmin">{ "系统管理员 (SysAdmin) - 全部权限" }</option>
                                            <option value="SecAdmin">{ "安全管理员 (SecAdmin) - 资产、扫描、风险管理" }</option>
                                            <option value="Auditor">{ "审计员 (Auditor) - 日志查看" }</option>
                                        </select>
                                    </div>
                                </div>
                            </div>
                        </section>
                        <footer class="modal-card-foot" style="justify-content: flex-end;">
                            <button class="button" onclick={on_close_create_modal.clone()}>{ "取消" }</button>
                            <button class="button is-primary" onclick={on_create_user}>{ "创建" }</button>
                        </footer>
                    </div>
                </div>
            }

            // Permission Editing Modal
            if *show_permissions_modal {
                <div class="modal is-active">
                    <div class="modal-background" onclick={on_close_modal.clone()}></div>
                    <div class="modal-card" style="width: 900px;">
                        <header class="modal-card-head">
                            <p class="modal-card-title">
                                { "编辑权限 - " }
                                { if let Some((_, username, _)) = &*editing_user {
                                    username.clone()
                                } else {
                                    String::new()
                                }}
                            </p>
                            <button class="delete" onclick={on_close_modal.clone()}></button>
                        </header>
                        <section class="modal-card-body" style="max-height: 70vh; overflow-y: auto;">
                            if let Some(perms) = &*temp_permissions {
                                <div style="margin-bottom: 1rem;">
                                    <h4 class="title is-6">{ "📋 基本信息" }</h4>
                                    <div class="box" style="background-color: #f5f5f5;">
                                        <p><strong>{ "用户名: " }</strong>{
                                            if let Some((_, username, _)) = &*editing_user {
                                                username.clone()
                                            } else {
                                                String::new()
                                            }
                                        }</p>
                                    </div>
                                </div>

                                // ========== 通用模块 ==========
                                <div style="margin-bottom: 1rem;">
                                    <h4 class="title is-6">{ "📊 通用模块" }</h4>
                                    <div class="box">
                                        // 顶级权限 - 访问通用模块
                                        <label class="checkbox" style="display: block; margin: 0.5rem 0; font-weight: bold; font-size: 1.1em;">
                                            <input
                                                type="checkbox"
                                                checked={perms.can_access_general}
                                                onchange={make_toggle_callback(String::from("can_access_general"))}/
                                            >
                                            { " 访问通用模块" }
                                        </label>
                                        // 子级权限 - 缩进显示
                                        <div style="margin-left: 1.5rem; border-left: 3px solid #3273dc; padding-left: 1rem; margin-top: 0.5rem;">
                                            // 仪表盘
                                            <label class="checkbox" style="display: block; margin: 0.5rem 0;">
                                                <input
                                                    type="checkbox"
                                                    checked={perms.can_view_dashboard}
                                                    onchange={make_toggle_callback(String::from("can_view_dashboard"))}
                                                    disabled={!perms.can_access_general}
                                                    style={if !perms.can_access_general { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                >
                                                { " 查看仪表盘" }
                                            </label>
                                            // 任务中心
                                            <label class="checkbox" style="display: block; margin: 0.5rem 0;">
                                                <input
                                                    type="checkbox"
                                                    checked={perms.can_view_tasks}
                                                    onchange={make_toggle_callback(String::from("can_view_tasks"))}
                                                    disabled={!perms.can_access_general}
                                                    style={if !perms.can_access_general { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                >
                                                { " 查看任务中心" }
                                            </label>
                                            // 孙级权限 - 任务操作
                                            <div style="margin-left: 1.5rem; border-left: 2px solid #ddd; padding-left: 1rem; margin-top: 0.5rem;">
                                                <label class="checkbox" style="display: block; margin: 0.4rem 0;">
                                                    <input
                                                        type="checkbox"
                                                        checked={perms.can_create_task}
                                                        onchange={make_toggle_callback(String::from("can_create_task"))}
                                                        disabled={!perms.can_view_tasks}
                                                        style={if !perms.can_view_tasks { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                    >
                                                    { " 创建任务" }
                                                </label>
                                                <label class="checkbox" style="display: block; margin: 0.4rem 0;">
                                                    <input
                                                        type="checkbox"
                                                        checked={perms.can_update_task}
                                                        onchange={make_toggle_callback(String::from("can_update_task"))}
                                                        disabled={!perms.can_view_tasks}
                                                        style={if !perms.can_view_tasks { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                    >
                                                    { " 更新任务" }
                                                </label>
                                                <label class="checkbox" style="display: block; margin: 0.4rem 0;">
                                                    <input
                                                        type="checkbox"
                                                        checked={perms.can_delete_task}
                                                        onchange={make_toggle_callback(String::from("can_delete_task"))}
                                                        disabled={!perms.can_view_tasks}
                                                        style={if !perms.can_view_tasks { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                    >
                                                    { " 删除任务" }
                                                </label>
                                            </div>
                                            // 高级扫描
                                            <label class="checkbox" style="display: block; margin: 0.5rem 0;">
                                                <input
                                                    type="checkbox"
                                                    checked={perms.can_view_advanced_scan}
                                                    onchange={make_toggle_callback(String::from("can_view_advanced_scan"))}
                                                    disabled={!perms.can_access_general}
                                                    style={if !perms.can_access_general { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                >
                                                { " 查看高级扫描" }
                                            </label>
                                            // 孙级权限 - 扫描操作
                                            <div style="margin-left: 1.5rem; border-left: 2px solid #ddd; padding-left: 1rem; margin-top: 0.5rem;">
                                                <label class="checkbox" style="display: block; margin: 0.4rem 0;">
                                                    <input
                                                        type="checkbox"
                                                        checked={perms.can_create_scan}
                                                        onchange={make_toggle_callback(String::from("can_create_scan"))}
                                                        disabled={!perms.can_view_advanced_scan}
                                                        style={if !perms.can_view_advanced_scan { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                    >
                                                    { " 创建扫描" }
                                                </label>
                                                <label class="checkbox" style="display: block; margin: 0.4rem 0;">
                                                    <input
                                                        type="checkbox"
                                                        checked={perms.can_delete_scan}
                                                        onchange={make_toggle_callback(String::from("can_delete_scan"))}
                                                        disabled={!perms.can_view_advanced_scan}
                                                        style={if !perms.can_view_advanced_scan { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                    >
                                                    { " 删除扫描" }
                                                </label>
                                                <label class="checkbox" style="display: block; margin: 0.4rem 0;">
                                                    <input
                                                        type="checkbox"
                                                        checked={perms.can_export_scan}
                                                        onchange={make_toggle_callback(String::from("can_export_scan"))}
                                                        disabled={!perms.can_view_advanced_scan}
                                                        style={if !perms.can_view_advanced_scan { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                    >
                                                    { " 导出结果" }
                                                </label>
                                            </div>
                                            // 风险监控
                                            <label class="checkbox" style="display: block; margin: 0.5rem 0;">
                                                <input
                                                    type="checkbox"
                                                    checked={perms.can_view_risks}
                                                    onchange={make_toggle_callback(String::from("can_view_risks"))}
                                                    disabled={!perms.can_access_general}
                                                    style={if !perms.can_access_general { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                >
                                                { " 查看风险监控" }
                                            </label>
                                            // 孙级权限 - 风险操作
                                            <div style="margin-left: 1.5rem; border-left: 2px solid #ddd; padding-left: 1rem; margin-top: 0.5rem;">
                                                <label class="checkbox" style="display: block; margin: 0.4rem 0;">
                                                    <input
                                                        type="checkbox"
                                                        checked={perms.can_resolve_risk}
                                                        onchange={make_toggle_callback(String::from("can_resolve_risk"))}
                                                        disabled={!perms.can_view_risks}
                                                        style={if !perms.can_view_risks { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                    >
                                                    { " 处置风险" }
                                                </label>
                                                <label class="checkbox" style="display: block; margin: 0.4rem 0;">
                                                    <input
                                                        type="checkbox"
                                                        checked={perms.can_delete_risk}
                                                        onchange={make_toggle_callback(String::from("can_delete_risk"))}
                                                        disabled={!perms.can_view_risks}
                                                        style={if !perms.can_view_risks { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                    >
                                                    { " 删除风险" }
                                                </label>
                                            </div>
                                        </div>
                                    </div>
                                </div>

                                // ========== 业务流程 ==========
                                <div style="margin-bottom: 1rem;">
                                    <h4 class="title is-6">{ "🔄 业务流程" }</h4>
                                    <div class="box">
                                        // 顶级权限
                                        <label class="checkbox" style="display: block; margin: 0.5rem 0; font-weight: bold; font-size: 1.1em;">
                                            <input
                                                type="checkbox"
                                                checked={perms.can_view_business_process}
                                                onchange={make_toggle_callback(String::from("can_view_business_process"))}/
                                            >
                                            { " 访问业务流程" }
                                        </label>
                                        // 子级权限
                                        <div style="margin-left: 1.5rem; border-left: 3px solid #3273dc; padding-left: 1rem; margin-top: 0.5rem;">
                                            // 业务申请
                                            <label class="checkbox" style="display: block; margin: 0.5rem 0;">
                                                <input
                                                    type="checkbox"
                                                    checked={perms.can_view_business_applications}
                                                    onchange={make_toggle_callback(String::from("can_view_business_applications"))}
                                                    disabled={!perms.can_view_business_process}
                                                    style={if !perms.can_view_business_process { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                >
                                                { " 📝 查看业务申请" }
                                            </label>
                                            // 孙级权限 - 业务申请操作
                                            <div style="margin-left: 1.5rem; border-left: 2px solid #ddd; padding-left: 1rem; margin-top: 0.5rem;">
                                                <label class="checkbox" style="display: block; margin: 0.4rem 0;">
                                                    <input
                                                        type="checkbox"
                                                        checked={perms.can_create_business_application}
                                                        onchange={make_toggle_callback(String::from("can_create_business_application"))}
                                                        disabled={!perms.can_view_business_applications}
                                                        style={if !perms.can_view_business_applications { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                    >
                                                    { " 创建业务申请" }
                                                </label>
                                                <label class="checkbox" style="display: block; margin: 0.4rem 0;">
                                                    <input
                                                        type="checkbox"
                                                        checked={perms.can_approve_business_application}
                                                        onchange={make_toggle_callback(String::from("can_approve_business_application"))}
                                                        disabled={!perms.can_view_business_applications}
                                                        style={if !perms.can_view_business_applications { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                    >
                                                    { " 审批业务申请" }
                                                </label>
                                                <label class="checkbox" style="display: block; margin: 0.4rem 0;">
                                                    <input
                                                        type="checkbox"
                                                        checked={perms.can_supplement_business_application}
                                                        onchange={make_toggle_callback(String::from("can_supplement_business_application"))}
                                                        disabled={!perms.can_view_business_applications}
                                                        style={if !perms.can_view_business_applications { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                    >
                                                    { " 补充业务申请信息" }
                                                </label>
                                                <label class="checkbox" style="display: block; margin: 0.4rem 0;">
                                                    <input
                                                        type="checkbox"
                                                        checked={perms.can_delete_business_application}
                                                        onchange={make_toggle_callback(String::from("can_delete_business_application"))}
                                                        disabled={!perms.can_view_business_applications}
                                                        style={if !perms.can_view_business_applications { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                    >
                                                    { " 删除业务申请" }
                                                </label>
                                            </div>
                                            // 运维管理
                                            <label class="checkbox" style="display: block; margin: 0.5rem 0;">
                                                <input
                                                    type="checkbox"
                                                    checked={perms.can_view_operations_management}
                                                    onchange={make_toggle_callback(String::from("can_view_operations_management"))}
                                                    disabled={!perms.can_view_business_process}
                                                    style={if !perms.can_view_business_process { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                >
                                                { " 🔧 查看运维管理" }
                                            </label>
                                            // 孙级权限 - 运维操作
                                            <div style="margin-left: 1.5rem; border-left: 2px solid #ddd; padding-left: 1rem; margin-top: 0.5rem;">
                                                <label class="checkbox" style="display: block; margin: 0.4rem 0;">
                                                    <input
                                                        type="checkbox"
                                                        checked={perms.can_manage_operations}
                                                        onchange={make_toggle_callback(String::from("can_manage_operations"))}
                                                        disabled={!perms.can_view_operations_management}
                                                        style={if !perms.can_view_operations_management { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                    >
                                                    { " 运维操作权限" }
                                                </label>
                                            </div>
                                            // 自动化资源编排
                                            <label class="checkbox" style="display: block; margin: 0.5rem 0;">
                                                <input
                                                    type="checkbox"
                                                    checked={perms.can_view_automation_orchestration}
                                                    onchange={make_toggle_callback(String::from("can_view_automation_orchestration"))}
                                                    disabled={!perms.can_view_business_process}
                                                    style={if !perms.can_view_business_process { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                >
                                                { " ⚙️ 查看自动化资源编排" }
                                            </label>
                                            // 孙级权限 - 编排操作
                                            <div style="margin-left: 1.5rem; border-left: 2px solid #ddd; padding-left: 1rem; margin-top: 0.5rem;">
                                                <label class="checkbox" style="display: block; margin: 0.4rem 0;">
                                                    <input
                                                        type="checkbox"
                                                        checked={perms.can_execute_orchestration}
                                                        onchange={make_toggle_callback(String::from("can_execute_orchestration"))}
                                                        disabled={!perms.can_view_automation_orchestration}
                                                        style={if !perms.can_view_automation_orchestration { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                    >
                                                    { " 执行编排任务" }
                                                </label>
                                                <label class="checkbox" style="display: block; margin: 0.4rem 0;">
                                                    <input
                                                        type="checkbox"
                                                        checked={perms.can_manage_orchestration}
                                                        onchange={make_toggle_callback(String::from("can_manage_orchestration"))}
                                                        disabled={!perms.can_view_automation_orchestration}
                                                        style={if !perms.can_view_automation_orchestration { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                    >
                                                    { " 管理编排任务" }
                                                </label>
                                            </div>
                                        </div>
                                    </div>
                                </div>

                                // ========== 云管理 ==========
                                <div style="margin-bottom: 1rem;">
                                    <h4 class="title is-6">{ "☁️ 云管理" }</h4>
                                    <div class="box">
                                        // 顶级权限
                                        <label class="checkbox" style="display: block; margin: 0.5rem 0; font-weight: bold; font-size: 1.1em;">
                                            <input
                                                type="checkbox"
                                                checked={perms.can_access_cloud}
                                                onchange={make_toggle_callback(String::from("can_access_cloud"))}/
                                            >
                                            { " 访问云管理模块" }
                                        </label>
                                        // 子级权限
                                        <div style="margin-left: 1.5rem; border-left: 3px solid #3273dc; padding-left: 1rem; margin-top: 0.5rem;">
                                            // 云区对接管理
                                            <label class="checkbox" style="display: block; margin: 0.5rem 0;">
                                                <input
                                                    type="checkbox"
                                                    checked={perms.can_view_cloud_providers}
                                                    onchange={make_toggle_callback(String::from("can_view_cloud_providers"))}
                                                    disabled={!perms.can_access_cloud}
                                                    style={if !perms.can_access_cloud { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                >
                                                { " 查看云区对接" }
                                            </label>
                                            // 孙级权限 - 云区操作
                                            <div style="margin-left: 1.5rem; border-left: 2px solid #ddd; padding-left: 1rem; margin-top: 0.5rem;">
                                                <label class="checkbox" style="display: block; margin: 0.4rem 0;">
                                                    <input
                                                        type="checkbox"
                                                        checked={perms.can_manage_cloud_providers}
                                                        onchange={make_toggle_callback(String::from("can_manage_cloud_providers"))}
                                                        disabled={!perms.can_view_cloud_providers}
                                                        style={if !perms.can_view_cloud_providers { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                    >
                                                    { " 管理云区对接" }
                                                </label>
                                            </div>
                                            // 混合云管理
                                            <label class="checkbox" style="display: block; margin: 0.5rem 0;">
                                                <input
                                                    type="checkbox"
                                                    checked={perms.can_view_cloud_management}
                                                    onchange={make_toggle_callback(String::from("can_view_cloud_management"))}
                                                    disabled={!perms.can_access_cloud}
                                                    style={if !perms.can_access_cloud { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                >
                                                { " 查看混合云管理" }
                                            </label>
                                            // 孙级权限 - 混合云操作
                                            <div style="margin-left: 1.5rem; border-left: 2px solid #ddd; padding-left: 1rem; margin-top: 0.5rem;">
                                                <label class="checkbox" style="display: block; margin: 0.4rem 0;">
                                                    <input
                                                        type="checkbox"
                                                        checked={perms.can_manage_cloud}
                                                        onchange={make_toggle_callback(String::from("can_manage_cloud"))}
                                                        disabled={!perms.can_view_cloud_management}
                                                        style={if !perms.can_view_cloud_management { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                    >
                                                    { " 管理云资产" }
                                                </label>
                                                <label class="checkbox" style="display: block; margin: 0.4rem 0;">
                                                    <input
                                                        type="checkbox"
                                                        checked={perms.can_delete_cloud}
                                                        onchange={make_toggle_callback(String::from("can_delete_cloud"))}
                                                        disabled={!perms.can_view_cloud_management}
                                                        style={if !perms.can_view_cloud_management { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                    >
                                                    { " 删除云资产" }
                                                </label>
                                                <label class="checkbox" style="display: block; margin: 0.4rem 0;">
                                                    <input
                                                        type="checkbox"
                                                        checked={perms.can_sync_cloud}
                                                        onchange={make_toggle_callback(String::from("can_sync_cloud"))}
                                                        disabled={!perms.can_view_cloud_management}
                                                        style={if !perms.can_view_cloud_management { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                    >
                                                    { " 同步云资产" }
                                                </label>
                                            </div>
                                            // 云服务资产
                                            <label class="checkbox" style="display: block; margin: 0.5rem 0;">
                                                <input
                                                    type="checkbox"
                                                    checked={perms.can_view_cloud_assets}
                                                    onchange={make_toggle_callback(String::from("can_view_cloud_assets"))}
                                                    disabled={!perms.can_access_cloud}
                                                    style={if !perms.can_access_cloud { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                >
                                                { " 查看云服务资产" }
                                            </label>
                                            // 孙级权限 - 云资产操作
                                            <div style="margin-left: 1.5rem; border-left: 2px solid #ddd; padding-left: 1rem; margin-top: 0.5rem;">
                                                <label class="checkbox" style="display: block; margin: 0.4rem 0;">
                                                    <input
                                                        type="checkbox"
                                                        checked={perms.can_create_cloud_asset}
                                                        onchange={make_toggle_callback(String::from("can_create_cloud_asset"))}
                                                        disabled={!perms.can_view_cloud_assets}
                                                        style={if !perms.can_view_cloud_assets { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                    >
                                                    { " 创建云资产" }
                                                </label>
                                                <label class="checkbox" style="display: block; margin: 0.4rem 0;">
                                                    <input
                                                        type="checkbox"
                                                        checked={perms.can_update_cloud_asset}
                                                        onchange={make_toggle_callback(String::from("can_update_cloud_asset"))}
                                                        disabled={!perms.can_view_cloud_assets}
                                                        style={if !perms.can_view_cloud_assets { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                    >
                                                    { " 更新云资产" }
                                                </label>
                                                <label class="checkbox" style="display: block; margin: 0.4rem 0;">
                                                    <input
                                                        type="checkbox"
                                                        checked={perms.can_delete_cloud_asset}
                                                        onchange={make_toggle_callback(String::from("can_delete_cloud_asset"))}
                                                        disabled={!perms.can_view_cloud_assets}
                                                        style={if !perms.can_view_cloud_assets { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                    >
                                                    { " 删除云资产" }
                                                </label>
                                            </div>
                                        </div>
                                    </div>
                                </div>

                                // ========== 用户管理模块 ==========
                                <div style="margin-bottom: 1rem;">
                                    <h4 class="title is-6">{ "👥 用户管理模块" }</h4>
                                    <div class="box">
                                        // 顶级权限
                                        <label class="checkbox" style="display: block; margin: 0.5rem 0; font-weight: bold; font-size: 1.1em;">
                                            <input
                                                type="checkbox"
                                                checked={perms.can_access_user_management}
                                                onchange={make_toggle_callback(String::from("can_access_user_management"))}/
                                            >
                                            { " 访问用户管理模块" }
                                        </label>
                                        // 子级权限
                                        <div style="margin-left: 1.5rem; border-left: 3px solid #3273dc; padding-left: 1rem; margin-top: 0.5rem;">
                                            // 用户管理
                                            <label class="checkbox" style="display: block; margin: 0.5rem 0;">
                                                <input
                                                    type="checkbox"
                                                    checked={perms.can_view_users}
                                                    onchange={make_toggle_callback(String::from("can_view_users"))}
                                                    disabled={!perms.can_access_user_management}
                                                    style={if !perms.can_access_user_management { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                >
                                                { " 查看用户管理" }
                                            </label>
                                            // 孙级权限 - 用户操作
                                            <div style="margin-left: 1.5rem; border-left: 2px solid #ddd; padding-left: 1rem; margin-top: 0.5rem;">
                                                <label class="checkbox" style="display: block; margin: 0.4rem 0;">
                                                    <input
                                                        type="checkbox"
                                                        checked={perms.can_create_user}
                                                        onchange={make_toggle_callback(String::from("can_create_user"))}
                                                        disabled={!perms.can_view_users}
                                                        style={if !perms.can_view_users { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                    >
                                                    { " 创建用户" }
                                                </label>
                                                <label class="checkbox" style="display: block; margin: 0.4rem 0;">
                                                    <input
                                                        type="checkbox"
                                                        checked={perms.can_update_user}
                                                        onchange={make_toggle_callback(String::from("can_update_user"))}
                                                        disabled={!perms.can_view_users}
                                                        style={if !perms.can_view_users { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                    >
                                                    { " 更新用户" }
                                                </label>
                                                <label class="checkbox" style="display: block; margin: 0.4rem 0;">
                                                    <input
                                                        type="checkbox"
                                                        checked={perms.can_delete_user}
                                                        onchange={make_toggle_callback(String::from("can_delete_user"))}
                                                        disabled={!perms.can_view_users}
                                                        style={if !perms.can_view_users { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                    >
                                                    { " 删除用户" }
                                                </label>
                                                <label class="checkbox" style="display: block; margin: 0.4rem 0;">
                                                    <input
                                                        type="checkbox"
                                                        checked={perms.can_manage_permissions}
                                                        onchange={make_toggle_callback(String::from("can_manage_permissions"))}
                                                        disabled={!perms.can_view_users}
                                                        style={if !perms.can_view_users { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                    >
                                                    { " 管理权限" }
                                                </label>
                                            </div>
                                            // 密码策略管理
                                            <label class="checkbox" style="display: block; margin: 0.5rem 0;">
                                                <input
                                                    type="checkbox"
                                                    checked={perms.can_view_password_policy}
                                                    onchange={make_toggle_callback(String::from("can_view_password_policy"))}
                                                    disabled={!perms.can_access_user_management}
                                                    style={if !perms.can_access_user_management { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                >
                                                { " 查看密码策略" }
                                            </label>
                                            // 孙级权限 - 密码策略操作
                                            <div style="margin-left: 1.5rem; border-left: 2px solid #ddd; padding-left: 1rem; margin-top: 0.5rem;">
                                                <label class="checkbox" style="display: block; margin: 0.4rem 0;">
                                                    <input
                                                        type="checkbox"
                                                        checked={perms.can_manage_password_policy}
                                                        onchange={make_toggle_callback(String::from("can_manage_password_policy"))}
                                                        disabled={!perms.can_view_password_policy}
                                                        style={if !perms.can_view_password_policy { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                    >
                                                    { " 管理密码策略" }
                                                </label>
                                            </div>
                                        </div>
                                    </div>
                                </div>

                                // ========== 审计模块 ==========
                                <div style="margin-bottom: 1rem;">
                                    <h4 class="title is-6">{ "📋 审计模块" }</h4>
                                    <div class="box">
                                        // 顶级权限
                                        <label class="checkbox" style="display: block; margin: 0.5rem 0; font-weight: bold; font-size: 1.1em;">
                                            <input
                                                type="checkbox"
                                                checked={perms.can_access_audit}
                                                onchange={make_toggle_callback(String::from("can_access_audit"))}/
                                            >
                                            { " 访问审计模块" }
                                        </label>
                                        // 子级权限
                                        <div style="margin-left: 1.5rem; border-left: 3px solid #3273dc; padding-left: 1rem; margin-top: 0.5rem;">
                                            <label class="checkbox" style="display: block; margin: 0.5rem 0;">
                                                <input
                                                    type="checkbox"
                                                    checked={perms.can_view_audit_logs}
                                                    onchange={make_toggle_callback(String::from("can_view_audit_logs"))}
                                                    disabled={!perms.can_access_audit}
                                                    style={if !perms.can_access_audit { "opacity: 0.5; cursor: not-allowed;" } else { "" }}/
                                                >
                                                { " 查看审计日志" }
                                            </label>
                                        </div>
                                    </div>
                                </div>
                            }
                        </section>
                        <footer class="modal-card-foot" style="justify-content: flex-end;">
                            <button class="button" onclick={on_close_modal.clone()}>{ "取消" }</button>
                            <button class="button is-primary" onclick={on_save_permissions}>{ "保存" }</button>
                        </footer>
                    </div>
                </div>
            }
        </div>
    }
}

// ============== Audit Logs Component ==============

#[function_component]
fn AuditLogs() -> Html {
    let lang = use_state(|| Language::Zh);
    let logs = use_state(|| Vec::new());
    let loading = use_state(|| true);

    // Get token from localStorage
    let token = get_auth_token();

    // Use effect to fetch data only once on mount
    use_effect_with((), {
        let logs = logs.clone();
        let loading = loading.clone();
        let token = token.clone();

        move |_| {
            spawn_local(async move {
                if let Ok(resp) = Request::get(&api_url("logs")).header("Authorization", &token).send().await {
                    if let Ok(data) = resp.json::<Vec<AuditLog>>().await {
                        logs.set(data);
                    }
                }
                loading.set(false);
            });
            || ()
        }
    });

    html! {
        <div class="container p-4">
            <h1 class="title">{ lang.t("audit_logs") }</h1>
            <div class="box">
                if (*logs).is_empty() && *loading {
                    <p class="has-text-centered has-text-grey">{ "Loading..." }</p>
                } else if (*logs).is_empty() {
                    <p class="has-text-centered has-text-grey">{ "No audit logs yet" }</p>
                } else {
                    <table class="table is-fullwidth is-hoverable">
                        <thead>
                            <tr>
                                <th>{ lang.t("timestamp") }</th>
                                <th>{ lang.t("username") }</th>
                                <th>{ lang.t("action") }</th>
                                <th>{ lang.t("details") }</th>
                            </tr>
                        </thead>
                        <tbody>
                            { for logs.iter().map(|log| {
                                html! {
                                    <tr>
                                        <td>{ log.timestamp.format("%Y-%m-%d %H:%M:%S").to_string() }</td>
                                        <td>{ &log.username }</td>
                                        <td>{ &log.action }</td>
                                        <td>{ &log.details }</td>
                                    </tr>
                                }
                            })}
                        </tbody>
                    </table>
                }
            </div>
        </div>
    }
}

// ============== Not Found Component ==============

#[function_component]
fn NotFound() -> Html {
    html! {
        <h1 class="title">{ "404 Not Found" }</h1>
    }
}

// ============== Automation Orchestration Component ==============
// 自动化资源编排 - 云资源自动化编排（仅处理云资源，物理机不进入此流程）
// 注意：本系统不直接调用云厂商API创建资源。运维人员需在云厂商控制台手动创建资源后，在此模块录入系统。

#[derive(Clone, Debug)]
struct OrchestrationTask {
    id: i32,
    business_resource_id: i32,
    business_resource_name: String,
    status: String, // 待编排, 编排中, 已完成, 失败
    cloud_provider: String,
    region: String,
    instance_type: String,
    created_at: String,
    completed_at: Option<String>,
    error_message: Option<String>,
    terraform_state: Option<String>, // Terraform 状态文件引用
    cloud_asset_id: Option<i32>,      // 关联的云资产ID
}

#[function_component]
fn AutomationOrchestration() -> Html {
    let lang = use_state(|| Language::Zh);
    let tasks = use_state(|| Vec::new());
    let loading = use_state(|| true);
    let show_logs_modal = use_state(|| false);
    let selected_task_logs = use_state(|| String::new());

    let token = get_auth_token();

    // 加载编排任务列表
    let load_tasks = {
        let token = token.clone();
        let tasks = tasks.clone();
        let loading = loading.clone();

        Callback::from(move |_| {
            let token = token.clone();
            let tasks = tasks.clone();
            let loading = loading.clone();

            spawn_local(async move {
                loading.set(true);

                // 获取已审批通过的业务资源作为编排任务源
                let business_resources_req = Request::get(&api_url("business-resources"))
                    .header("Authorization", &token)
                    .send()
                    .await;

                let mut orchestration_tasks = vec![];

                if let Ok(resp) = business_resources_req {
                    if let Ok(resources) = resp.json::<Vec<BusinessResource>>().await {
                        // 只处理已审批通过的云资源业务申请（物理机不进入自动化编排流程）
                        let mut idx = 0;
                        for resource in resources.iter() {
                            // 云资源且已审批通过状态（待交付）
                            let is_approved = resource.ecs_status == "已通过" || resource.ecs_status == "待交付";
                            if is_approved && resource.resource_type == "cloud" {
                                let is_completed = !resource.instance_id.is_empty();
                                idx += 1;
                                orchestration_tasks.push(OrchestrationTask {
                                    id: idx,
                                    business_resource_id: resource.id.unwrap_or(0),
                                    business_resource_name: resource.ecs_name.clone(),
                                    status: if is_completed {
                                        "已完成".to_string()
                                    } else {
                                        "待编排".to_string()
                                    },
                                    cloud_provider: resource.cloud_category.clone(),
                                    region: resource.cloud_region.clone(),
                                    instance_type: resource.ecs_type.clone(),
                                    created_at: resource.created_at.map(|d| d.format("%Y-%m-%d %H:%M").to_string()).unwrap_or_else(|| "-".to_string()),
                                    completed_at: None,
                                    error_message: None,
                                    terraform_state: None,
                                    cloud_asset_id: None,
                                });
                            }
                        }
                    }
                }

                tasks.set(orchestration_tasks);
                loading.set(false);
            });
        })
    };

    // Initial load
    use_effect_with((), {
        let load_tasks = load_tasks.clone();
        move |_| {
            load_tasks.emit(());
            || ()
        }
    });

    // 执行编排 (模拟调用 Terraform/云 SDK)
    let execute_orchestration = {
        let token = token.clone();
        let tasks = tasks.clone();
        let load_tasks = load_tasks.clone();

        Callback::from(move |task_id: i32| {
            let token = token.clone();
            let tasks_clone = tasks.clone();
            let load_tasks = load_tasks.clone();

            spawn_local(async move {
                // 更新状态为编排中
                let updated_tasks = tasks_clone.iter().map(|t| {
                    if t.id == task_id {
                        OrchestrationTask {
                            status: "编排中".to_string(),
                            ..t.clone()
                        }
                    } else {
                        t.clone()
                    }
                }).collect::<Vec<_>>();
                tasks_clone.set(updated_tasks);

                // 模拟异步编排过程 (实际应调用后端编排 API)
                // 这里会调用 approve_business_resource API 来创建云资产
                let current_task = tasks_clone.iter().find(|t| t.id == task_id);

                if let Some(task) = current_task {
                    let url = format!("{}/{}/approve", api_url("business-resources"), task.business_resource_id);

                    if let Ok(resp) = Request::post(&url)
                        .header("Authorization", &token)
                        .send()
                        .await
                    {
                        if resp.status() == 200 {
                            // 编排成功，重新加载任务状态
                            load_tasks.emit(());
                        } else {
                            // 编排失败
                            let failed_tasks = tasks_clone.iter().map(|t| {
                                if t.id == task_id {
                                    OrchestrationTask {
                                        status: "失败".to_string(),
                                        error_message: Some("编排失败，请检查云服务配置".to_string()),
                                        ..t.clone()
                                    }
                                } else {
                                    t.clone()
                                }
                            }).collect::<Vec<_>>();
                            tasks_clone.set(failed_tasks);
                        }
                    }
                }
            });
        })
    };

    let on_view_logs = {
        let selected_task_logs = selected_task_logs.clone();
        let show_logs_modal = show_logs_modal.clone();

        Callback::from(move |task: OrchestrationTask| {
            // 生成云资源编排日志（此模块仅处理云资源）
            let logs = format!(
                "编排任务日志 - {}\n\
                 状态: {}\n\
                 资源类型: 云服务器\n\
                 云服务商: {}\n\
                 区域: {}\n\
                 实例类型: {}\n\
                 \n\
                 [云资源交付流程]\n\
                 1. 业务申请审批通过，状态变为「待交付」\n\
                 2. 运维人员登录云厂商控制台（{}）创建资源\n\
                 3. 记录云厂商返回的实例ID、IP地址等信息\n\
                 4. 在系统中录入云资源信息\n\
                 5. 混合云管理模块可API同步资源状态\n\
                 \n\
                 [Terraform 执行日志]\n\
                 2024-01-15 10:30:00 [INFO] 初始化 Terraform 配置\n\
                 2024-01-15 10:30:05 [INFO] 连接 {} API\n\
                 2024-01-15 10:30:10 [INFO] 创建 VPC 网络配置\n\
                 2024-01-15 10:30:15 [INFO] 创建安全组规则\n\
                 2024-01-15 10:30:20 [INFO] 启动云主机实例 {}\n\
                 2024-01-15 10:32:00 [INFO] 实例创建成功\n\
                 2024-01-15 10:32:05 [INFO] 配置网络规则完成\n\
                 \n\
                 [请注意] 请手动在云厂商控制台创建资源后，将实例ID等信息录入本系统。",
                task.business_resource_name,
                task.status,
                task.cloud_provider,
                task.region,
                task.instance_type,
                task.cloud_provider,
                task.cloud_provider,
                task.business_resource_name
            );

            selected_task_logs.set(logs);
            show_logs_modal.set(true);
        })
    };

    let get_status_class = |status: &str| -> &str {
        match status {
            "待编排" => "is-warning",
            "编排中" => "is-info",
            "已完成" => "is-success",
            "失败" => "is-danger",
            _ => "",
        }
    };

    let lang_enum = (*lang).clone();

    html! {
        <div class="container">
            <div class="columns">
                <div class="column is-12">
                    <h1 class="title">{ lang_enum.t("automation_orchestration") }</h1>
                    <p class="subtitle">{ "云资源自动化编排" }</p>

                    // 工作流程说明
                    <div class="content">
                        <div class="notification is-info is-light">
                            <p class="heading">{ "云资源交付流程" }</p>
                            <p class="mb-3">
                                <strong>{ "注意：本系统不直接调用云厂商API创建资源。运维人员需在云厂商控制台手动创建资源后，在此模块录入系统。" }</strong>
                            </p>
                            <p class="mb-3">
                                <strong>{ "正确流程说明:" }</strong>
                            </p>
                            <ol class="mb-3">
                                <li>{ "业务申请审批通过后，状态变为「待交付」" }</li>
                                <li>{ "运维人员登录云厂商控制台（阿里云/腾讯云等）创建资源" }</li>
                                <li>{ "记录云厂商返回的实例ID、IP地址等信息" }</li>
                                <li>{ "在系统中录入云资源信息" }</li>
                                <li>{ "混合云管理模块可API同步资源状态" }</li>
                            </ol>
                            <p class="has-text-grey">
                                { "注：物理机不进入此流程，物理机交付请直接在「云服务资产管理」模块录入。" }
                            </p>
                        </div>
                    </div>

                    // 统计卡片
                    <div class="columns">
                        <div class="column">
                            <div class="card">
                                <div class="card-content">
                                    <div class="content">
                                        <p class="title is-5">{ "待编排" }</p>
                                        <p class="subtitle is-3">
                                            { tasks.iter().filter(|t| t.status == "待编排").count() }
                                        </p>
                                    </div>
                                </div>
                            </div>
                        </div>
                        <div class="column">
                            <div class="card">
                                <div class="card-content">
                                    <div class="content">
                                        <p class="title is-5">{ "编排中" }</p>
                                        <p class="subtitle is-3">
                                            { tasks.iter().filter(|t| t.status == "编排中").count() }
                                        </p>
                                    </div>
                                </div>
                            </div>
                        </div>
                        <div class="column">
                            <div class="card">
                                <div class="card-content">
                                    <div class="content">
                                        <p class="title is-5">{ "已完成" }</p>
                                        <p class="subtitle is-3">
                                            { tasks.iter().filter(|t| t.status == "已完成").count() }
                                        </p>
                                    </div>
                                </div>
                            </div>
                        </div>
                        <div class="column">
                            <div class="card">
                                <div class="card-content">
                                    <div class="content">
                                        <p class="title is-5">{ "失败" }</p>
                                        <p class="subtitle is-3 has-text-danger">
                                            { tasks.iter().filter(|t| t.status == "失败").count() }
                                        </p>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>

                    // 任务列表
                    <div class="card">
                        <div class="card-content">
                            if *loading {
                                <progress class="progress is-primary" />
                            } else if tasks.is_empty() {
                                <div class="content has-text-centered">
                                    <p class="has-text-grey">{ "暂无编排任务，已审批通过的云资源业务申请（待交付状态）将在此显示" }</p>
                                    <p class="has-text-grey is-size-7">{ "注：物理机不进入此流程" }</p>
                                </div>
                            } else {
                                <table class="table is-fullwidth is-hoverable">
                                    <thead>
                                        <tr>
                                            <th>{ "业务名称" }</th>
                                            <th>{ "云服务商" }</th>
                                            <th>{ "区域" }</th>
                                            <th>{ "实例类型" }</th>
                                            <th>{ "状态" }</th>
                                            <th>{ "创建时间" }</th>
                                            <th>{ "操作" }</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        { for tasks.iter().map(|task| {
                                            let task_id = task.id;
                                            let task_clone = task.clone();
                                            let on_execute = execute_orchestration.clone();
                                            let on_view = on_view_logs.clone();

                                            html! {
                                                <tr>
                                                    <td>{ &task.business_resource_name }</td>
                                                    <td>{ &task.cloud_provider }</td>
                                                    <td>{ &task.region }</td>
                                                    <td>{ &task.instance_type }</td>
                                                    <td>
                                                        <span class={format!("tag {}", get_status_class(&task.status))}>
                                                            { &task.status }
                                                        </span>
                                                    </td>
                                                    <td>{ &task.created_at }</td>
                                                    <td>
                                                        <div class="buttons are-small">
                                                            if task.status == "待编排" {
                                                                <button class="button is-primary" onclick={Callback::from(move |_| on_execute.emit(task_id))}>
                                                                    { "创建云资源" }
                                                                </button>
                                                            }
                                                            <button class="button is-light" onclick={Callback::from(move |_| on_view.emit(task_clone.clone()))}>
                                                                { "查看日志" }
                                                            </button>
                                                        </div>
                                                    </td>
                                                </tr>
                                            }
                                        })}
                                    </tbody>
                                </table>
                            }
                        </div>
                    </div>

                    // 日志模态框
                    if *show_logs_modal {
                        <div class="modal is-active">
                            <div class="modal-background" onclick={
                                let show_logs_modal = show_logs_modal.clone();
                                Callback::from(move |_| show_logs_modal.set(false))
                            } />
                            <div class="modal-card">
                                <header class="modal-card-head">
                                    <p class="modal-card-title">{ "编排日志" }</p>
                                    <button class="delete" aria-label="close" onclick={
                                        let show_logs_modal = show_logs_modal.clone();
                                        Callback::from(move |_| show_logs_modal.set(false))
                                    } />
                                </header>
                                <section class="modal-card-body">
                                    <pre class="is-family-monogram" style="white-space: pre-wrap; background: #f5f5f5; padding: 1rem;">
                                        { (*selected_task_logs).clone() }
                                    </pre>
                                </section>
                                <footer class="modal-card-foot">
                                    <button class="button" onclick={
                                        let show_logs_modal = show_logs_modal.clone();
                                        Callback::from(move |_| show_logs_modal.set(false))
                                    }>{ "关闭" }</button>
                                </footer>
                            </div>
                        </div>
                    }
                </div>
            </div>
        </div>
    }
}

// ============== Cloud Management Component ==============

#[function_component]
fn CloudManagement() -> Html {
    let lang = use_state(|| Language::Zh);
    let assets = use_state(|| Vec::new());
    let stats = use_state(|| None as Option<CloudAssetStats>);
    let loading = use_state(|| true);

    // Edit modal state
    let editing_asset = use_state(|| None as Option<CloudAsset>);
    let show_edit_modal = use_state(|| false);
    let edit_message = use_state(|| None as Option<String>);

    // Filters
    let filter_provider = use_state(|| String::new());
    let filter_status = use_state(|| String::new());
    let filter_search = use_state(|| String::new());

    // Get user role
    let user_role = get_user_role();

    // Get token from localStorage
    let token = get_auth_token();

    // Use effect to fetch data only once on mount
    use_effect_with((), {
        let assets = assets.clone();
        let stats = stats.clone();
        let loading = loading.clone();
        let token = token.clone();

        move |_| {
            spawn_local(async move {
                // Fetch cloud assets
                if let Ok(resp) = Request::get(&api_url("cloud-assets"))
                    .header("Authorization", &token)
                    .send()
                    .await
                {
                    if let Ok(data) = resp.json::<Vec<CloudAsset>>().await {
                        assets.set(data);
                    }
                }

                // Fetch stats
                if let Ok(resp) = Request::get(&api_url("cloud-assets/stats"))
                    .header("Authorization", &token)
                    .send()
                    .await
                {
                    if let Ok(data) = resp.json::<CloudAssetStats>().await {
                        stats.set(Some(data));
                    }
                }

                loading.set(false);
            });
            || ()
        }
    });

    // Helper functions for display
    let provider_name = |provider: &CloudProvider| -> String {
        match provider {
            CloudProvider::Aliyun => "阿里云".to_string(),
            CloudProvider::Tencent => "腾讯云".to_string(),
            CloudProvider::Huawei => "华为云".to_string(),
            CloudProvider::Aws => "AWS".to_string(),
            CloudProvider::Azure => "Azure".to_string(),
            CloudProvider::Gcp => "GCP".to_string(),
            CloudProvider::Baidu => "百度云".to_string(),
            CloudProvider::Custom(s) => s.clone(),
        }
    };

    let status_text = |status: &VMStatus| -> &'static str {
        match status {
            VMStatus::Running => "运行中",
            VMStatus::Stopped => "已停止",
            VMStatus::Starting => "启动中",
            VMStatus::Stopping => "停止中",
            VMStatus::Rebooting => "重启中",
            VMStatus::Deleted => "已释放",
            VMStatus::Error => "异常",
        }
    };

    let billing_text = |mode: &BillingMode| -> &'static str {
        match mode {
            BillingMode::PayAsYouGo => "按量付费",
            BillingMode::Subscription => "包年包月",
            BillingMode::Spot => "抢占式",
        }
    };

    // Helper to check provider match with string filter
    let provider_matches = |provider: &CloudProvider, filter: &str| -> bool {
        match (provider, filter) {
            (CloudProvider::Aliyun, "aliyun") => true,
            (CloudProvider::Tencent, "tencent") => true,
            (CloudProvider::Huawei, "huawei") => true,
            (CloudProvider::Aws, "aws") => true,
            (CloudProvider::Azure, "azure") => true,
            (CloudProvider::Gcp, "gcp") => true,
            (CloudProvider::Baidu, "baidu") => true,
            _ => false,
        }
    };

    // Helper to check status match with string filter
    let status_matches = |status: &VMStatus, filter: &str| -> bool {
        match (status, filter) {
            (VMStatus::Running, "running") => true,
            (VMStatus::Stopped, "stopped") => true,
            _ => false,
        }
    };

    let format_datetime = |dt: &chrono::DateTime<chrono::Utc>| -> String {
        dt.format("%Y-%m-%d %H:%M").to_string()
    };

    let format_expire = |dt: &Option<chrono::DateTime<chrono::Utc>>| -> String {
        match dt {
            Some(d) => {
                let now = chrono::Utc::now();
                let duration = *d - now;
                let days = duration.num_days();
                if days <= 7 {
                    format!("{} ({}天)", d.format("%Y-%m-%d"), days)
                } else {
                    d.format("%Y-%m-%d").to_string()
                }
            }
            None => "永久".to_string(),
        }
    };

    // Filter assets
    let filtered_assets = (*assets).iter().filter(|asset| {
        let provider_match = filter_provider.is_empty()
            || provider_matches(&asset.cloud_region.provider, &filter_provider);
        let status_match = filter_status.is_empty()
            || status_matches(&asset.status, &filter_status);
        let search_match = filter_search.is_empty()
            || asset.asset_name.contains(&*filter_search)
            || asset.instance_id.contains(&*filter_search);
        provider_match && status_match && search_match
    }).cloned().collect::<Vec<_>>();

    let on_provider_change = {
        let filter_provider = filter_provider.clone();
        Callback::from(move |e: Event| {
            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
            filter_provider.set(select.value());
        })
    };

    let on_status_change = {
        let filter_status = filter_status.clone();
        Callback::from(move |e: Event| {
            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
            filter_status.set(select.value());
        })
    };

    let on_search_input = {
        let filter_search = filter_search.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            filter_search.set(input.value());
        })
    };

    let on_refresh = {
        let assets = assets.clone();
        let stats = stats.clone();
        let loading = loading.clone();
        let token = token.clone();

        Callback::from(move |_| {
            let assets = assets.clone();
            let stats = stats.clone();
            let loading = loading.clone();
            let token = token.clone();

            spawn_local(async move {
                loading.set(true);

                if let Ok(resp) = Request::get(&api_url("cloud-assets"))
                    .header("Authorization", &token)
                    .send()
                    .await
                {
                    if let Ok(data) = resp.json::<Vec<CloudAsset>>().await {
                        assets.set(data);
                    }
                }

                if let Ok(resp) = Request::get(&api_url("cloud-assets/stats"))
                    .header("Authorization", &token)
                    .send()
                    .await
                {
                    if let Ok(data) = resp.json::<CloudAssetStats>().await {
                        stats.set(Some(data));
                    }
                }

                loading.set(false);
            });
        })
    };

    // Edit handlers
    let on_edit_click = {
        let editing_asset = editing_asset.clone();
        let show_edit_modal = show_edit_modal.clone();

        Callback::from(move |asset: CloudAsset| {
            editing_asset.set(Some(asset));
            show_edit_modal.set(true);
        })
    };

    let on_close_modal = {
        let show_edit_modal = show_edit_modal.clone();
        let editing_asset = editing_asset.clone();
        let edit_message = edit_message.clone();

        Callback::from(move |_| {
            show_edit_modal.set(false);
            editing_asset.set(None);
            edit_message.set(None);
        })
    };

    let on_save_asset = {
        let assets = assets.clone();
        let editing_asset = editing_asset.clone();
        let show_edit_modal = show_edit_modal.clone();
        let edit_message = edit_message.clone();
        let token = token.clone();

        Callback::from(move |_: Event| {
            if let Some(asset) = (*editing_asset).clone() {
                let assets = assets.clone();
                let token = token.clone();
                let edit_message = edit_message.clone();
                let show_edit_modal = show_edit_modal.clone();
                let editing_asset = editing_asset.clone();
                let asset_id = asset.id.unwrap_or(0);

                spawn_local(async move {
                    let asset_json = match serde_json::to_string(&asset) {
                        Ok(json) => json,
                        Err(e) => {
                            edit_message.set(Some(format!("序列化失败: {}", e)));
                            return;
                        }
                    };

                    if let Ok(resp) = Request::put(&format!("{}/{}", api_url("cloud-assets"), asset_id))
                        .header("Authorization", &token)
                        .header("Content-Type", "application/json")
                        .body(asset_json)
                        .unwrap()
                        .send()
                        .await
                    {
                        if resp.ok() {
                            edit_message.set(Some("保存成功".to_string()));
                            // Refresh assets
                            if let Ok(resp) = Request::get(&api_url("cloud-assets"))
                                .header("Authorization", &token)
                                .send()
                                .await
                            {
                                if let Ok(data) = resp.json::<Vec<CloudAsset>>().await {
                                    assets.set(data);
                                }
                            }
                            // Close modal after a short delay
                            spawn_local(async move {
                                Timeout::new(1000, move || {
                                    show_edit_modal.set(false);
                                    editing_asset.set(None);
                                }).forget();
                            });
                        } else {
                            edit_message.set(Some("保存失败".to_string()));
                        }
                    }
                });
            }
        })
    };

    html! {
        <div class="container p-4">
            <h1 class="title">{ lang.t("cloud_management") }</h1>

            // 混合云管理说明
            <div class="notification is-info is-light mb-4">
                <div class="content">
                    <p class="heading">{ "☁️ 混合云管理说明" }</p>
                    <p>
                        { "本模块统一纳管各云厂商（阿里云/腾讯云/华为云/AWS等）的资源，支持手动录入和API自动同步。" }
                    </p>
                    <p class="is-size-7 has-text-grey mt-2">
                        { "运维流程: 业务申请 → 审批通过 → 云厂商平台创建资源 → 手动录入系统 → API同步状态" }
                    </p>
                </div>
            </div>

            // Statistics Cards
            if let Some(ref s) = *stats {
                <div class="columns is-multiline mb-4">
                    <div class="column is-2">
                        <div class="box has-background-info-light is-paddingless">
                            <div class="p-3">
                                <div class="is-size-7 has-text-grey">{"资产总数"}</div>
                                <div class="title is-5">{ s.total_count }</div>
                            </div>
                        </div>
                    </div>
                    <div class="column is-2">
                        <div class="box has-background-success-light is-paddingless">
                            <div class="p-3">
                                <div class="is-size-7 has-text-grey">{"运行中"}</div>
                                <div class="title is-5">{ s.running_count }</div>
                            </div>
                        </div>
                    </div>
                    <div class="column is-2">
                        <div class="box has-background-warning-light is-paddingless">
                            <div class="p-3">
                                <div class="is-size-7 has-text-grey">{"CPU核心"}</div>
                                <div class="title is-5">{ s.total_cpu_cores }</div>
                            </div>
                        </div>
                    </div>
                    <div class="column is-2">
                        <div class="box has-background-primary-light is-paddingless">
                            <div class="p-3">
                                <div class="is-size-7 has-text-grey">{"内存(GB)"}</div>
                                <div class="title is-5">{ s.total_memory_gb }</div>
                            </div>
                        </div>
                    </div>
                    <div class="column is-2">
                        <div class="box has-background-danger-light is-paddingless">
                            <div class="p-3">
                                <div class="is-size-7 has-text-grey">{"即将到期"}</div>
                                <div class="title is-5 has-text-danger">{ s.expiring_soon_count }</div>
                            </div>
                        </div>
                    </div>
                </div>
            }

            // Filters
            <div class="box mb-4">
                <div class="columns">
                    <div class="column is-2">
                        <label class="label is-small">{"云厂商"}</label>
                        <div class="select is-small is-fullwidth">
                            <select onchange={on_provider_change}>
                                <option value="">{"全部"}</option>
                                <option value="aliyun">{"阿里云"}</option>
                                <option value="tencent">{"腾讯云"}</option>
                                <option value="huawei">{"华为云"}</option>
                                <option value="aws">{"AWS"}</option>
                                <option value="azure">{"Azure"}</option>
                                <option value="gcp">{"GCP"}</option>
                                <option value="baidu">{"百度云"}</option>
                            </select>
                        </div>
                    </div>
                    <div class="column is-2">
                        <label class="label is-small">{"状态"}</label>
                        <div class="select is-small is-fullwidth">
                            <select onchange={on_status_change}>
                                <option value="">{"全部"}</option>
                                <option value="running">{"运行中"}</option>
                                <option value="stopped">{"已停止"}</option>
                            </select>
                        </div>
                    </div>
                    <div class="column is-4">
                        <label class="label is-small">{"搜索"}</label>
                        <input
                            class="input is-small"
                            type="text"
                            placeholder="资产名称或实例ID"
                            value={(*filter_search).clone()}
                            oninput={on_search_input}
                        />
                    </div>
                    <div class="column is-2">
                        <label class="label is-small">{"操作"}</label>
                        <button class="button is-small is-primary is-fullwidth" onclick={on_refresh}>
                            <span class="icon"><i class="fas fa-sync"></i></span>
                            <span>{"刷新"}</span>
                        </button>
                    </div>
                </div>
            </div>

            // Loading state
            if *loading {
                <div class="has-text-centered py-6">
                    <progress class="progress is-primary is-small" value="100">{ "Loading..." }</progress>
                </div>
            } else if filtered_assets.is_empty() {
                <div class="box has-text-centered py-6">
                    <p class="has-text-grey">{"暂无云资产数据"}</p>
                </div>
            } else {
                // Cloud Assets Table - Excel-like with resizable columns
                <div class="box is-paddingless" style="overflow: hidden;">
                    <div class="excel-table-wrapper" style="overflow-x: auto; position: relative; border: 1px solid #ddd; border-radius: 4px;">
                        <table class="excel-table" style="border-collapse: collapse; width: 100%; min-width: 3200px; font-size: 14px;">
                            <thead>
                                <tr style="background: #f5f5f5;">
                                    <th style="border: 1px solid #ddd; padding: 16px 10px; min-width: 180px; position: relative; text-align: left; font-weight: 600; color: #333;">
                                        {"\u{1f4c1} 资产名称"}
                                        <span class="resizer" style="position: absolute; right: 0; top: 0; height: 100%; width: 5px; cursor: col-resize; user-select: none;"></span>
                                    </th>
                                    <th style="border: 1px solid #ddd; padding: 16px 10px; min-width: 140px; position: relative; text-align: left; font-weight: 600; color: #333;">
                                        {"\u{1f4ca} 规格"}
                                        <span class="resizer" style="position: absolute; right: 0; top: 0; height: 100%; width: 5px; cursor: col-resize; user-select: none;"></span>
                                    </th>
                                    <th style="border: 1px solid #ddd; padding: 16px 10px; min-width: 80px; position: relative; text-align: left; font-weight: 600; color: #333;">
                                        {"CPU"}
                                        <span class="resizer" style="position: absolute; right: 0; top: 0; height: 100%; width: 5px; cursor: col-resize; user-select: none;"></span>
                                    </th>
                                    <th style="border: 1px solid #ddd; padding: 16px 10px; min-width: 80px; position: relative; text-align: left; font-weight: 600; color: #333;">
                                        {"内存"}
                                        <span class="resizer" style="position: absolute; right: 0; top: 0; height: 100%; width: 5px; cursor: col-resize; user-select: none;"></span>
                                    </th>
                                    <th style="border: 1px solid #ddd; padding: 16px 10px; min-width: 140px; position: relative; text-align: left; font-weight: 600; color: #333;">
                                        {"系统盘"}
                                        <span class="resizer" style="position: absolute; right: 0; top: 0; height: 100%; width: 5px; cursor: col-resize; user-select: none;"></span>
                                    </th>
                                    <th style="border: 1px solid #ddd; padding: 16px 10px; min-width: 100px; position: relative; text-align: left; font-weight: 600; color: #333;">
                                        {"\u{2601}️ 云厂商"}
                                        <span class="resizer" style="position: absolute; right: 0; top: 0; height: 100%; width: 5px; cursor: col-resize; user-select: none;"></span>
                                    </th>
                                    <th style="border: 1px solid #ddd; padding: 16px 10px; min-width: 140px; position: relative; text-align: left; font-weight: 600; color: #333;">
                                        {"\u{1f30e} 云区"}
                                        <span class="resizer" style="position: absolute; right: 0; top: 0; height: 100%; width: 5px; cursor: col-resize; user-select: none;"></span>
                                    </th>
                                    <th style="border: 1px solid #ddd; padding: 16px 10px; min-width: 140px; position: relative; text-align: left; font-weight: 600; color: #333;">
                                        {"公网IP"}
                                        <span class="resizer" style="position: absolute; right: 0; top: 0; height: 100%; width: 5px; cursor: col-resize; user-select: none;"></span>
                                    </th>
                                    <th style="border: 1px solid #ddd; padding: 16px 10px; min-width: 140px; position: relative; text-align: left; font-weight: 600; color: #333;">
                                        {"内网IP"}
                                        <span class="resizer" style="position: absolute; right: 0; top: 0; height: 100%; width: 5px; cursor: col-resize; user-select: none;"></span>
                                    </th>
                                    <th style="border: 1px solid #ddd; padding: 16px 10px; min-width: 100px; position: relative; text-align: left; font-weight: 600; color: #333;">
                                        {"计费模式"}
                                        <span class="resizer" style="position: absolute; right: 0; top: 0; height: 100%; width: 5px; cursor: col-resize; user-select: none;"></span>
                                    </th>
                                    <th style="border: 1px solid #ddd; padding: 16px 10px; min-width: 140px; position: relative; text-align: left; font-weight: 600; color: #333;">
                                        {"到期时间"}
                                        <span class="resizer" style="position: absolute; right: 0; top: 0; height: 100%; width: 5px; cursor: col-resize; user-select: none;"></span>
                                    </th>
                                    <th style="border: 1px solid #ddd; padding: 16px 10px; min-width: 90px; position: relative; text-align: left; font-weight: 600; color: #333;">
                                        {"状态"}
                                        <span class="resizer" style="position: absolute; right: 0; top: 0; height: 100%; width: 5px; cursor: col-resize; user-select: none;"></span>
                                    </th>
                                    <th style="border: 1px solid #ddd; padding: 16px 10px; min-width: 160px; position: relative; text-align: left; font-weight: 600; color: #333;">
                                        {"操作系统"}
                                        <span class="resizer" style="position: absolute; right: 0; top: 0; height: 100%; width: 5px; cursor: col-resize; user-select: none;"></span>
                                    </th>
                                    <th style="border: 1px solid #ddd; padding: 16px 10px; min-width: 160px; position: relative; text-align: left; font-weight: 600; color: #333;">
                                        {"镜像ID"}
                                        <span class="resizer" style="position: absolute; right: 0; top: 0; height: 100%; width: 5px; cursor: col-resize; user-select: none;"></span>
                                    </th>
                                    <th style="border: 1px solid #ddd; padding: 16px 10px; min-width: 120px; position: relative; text-align: left; font-weight: 600; color: #333;">
                                        {"部门"}
                                        <span class="resizer" style="position: absolute; right: 0; top: 0; height: 100%; width: 5px; cursor: col-resize; user-select: none;"></span>
                                    </th>
                                    <th style="border: 1px solid #ddd; padding: 16px 10px; min-width: 120px; position: relative; text-align: left; font-weight: 600; color: #333;">
                                        {"项目"}
                                        <span class="resizer" style="position: absolute; right: 0; top: 0; height: 100%; width: 5px; cursor: col-resize; user-select: none;"></span>
                                    </th>
                                    <th style="border: 1px solid #ddd; padding: 16px 10px; min-width: 100px; position: relative; text-align: left; font-weight: 600; color: #333;">
                                        {"负责人"}
                                        <span class="resizer" style="position: absolute; right: 0; top: 0; height: 100%; width: 5px; cursor: col-resize; user-select: none;"></span>
                                    </th>
                                    <th style="border: 1px solid #ddd; padding: 16px 10px; min-width: 140px; position: relative; text-align: left; font-weight: 600; color: #333;">
                                        {"创建时间"}
                                        <span class="resizer" style="position: absolute; right: 0; top: 0; height: 100%; width: 5px; cursor: col-resize; user-select: none;"></span>
                                    </th>
                                    <th style="border: 1px solid #ddd; padding: 16px 10px; min-width: 80px; position: relative; text-align: left; font-weight: 600; color: #333;">
                                        {"云盘数"}
                                        <span class="resizer" style="position: absolute; right: 0; top: 0; height: 100%; width: 5px; cursor: col-resize; user-select: none;"></span>
                                    </th>
                                    <th style="border: 1px solid #ddd; padding: 16px 10px; min-width: 100px; position: relative; text-align: left; font-weight: 600; color: #333;">
                                        {"云盘总量"}
                                        <span class="resizer" style="position: absolute; right: 0; top: 0; height: 100%; width: 5px; cursor: col-resize; user-select: none;"></span>
                                    </th>
                                    <th style="border: 1px solid #ddd; padding: 16px 10px; min-width: 80px; position: relative; text-align: left; font-weight: 600; color: #333;">
                                        {"快照"}
                                        <span class="resizer" style="position: absolute; right: 0; top: 0; height: 100%; width: 5px; cursor: col-resize; user-select: none;"></span>
                                    </th>
                                    <th style="border: 1px solid #ddd; padding: 16px 10px; min-width: 140px; position: sticky; right: 0; background: #f5f5f5; border-left: 2px solid #ccc; z-index: 10; text-align: left; font-weight: 600; color: #333;">
                                        {"\u{2699}️ 操作"}
                                    </th>
                                </tr>
                            </thead>
                            <tbody>
                                { for filtered_assets.iter().enumerate().map(|(idx, asset)| {
                                    let provider = provider_name(&asset.cloud_region.provider);
                                    let region = &asset.cloud_region.region_name;
                                    let status_display = status_text(&asset.status);
                                    let is_running = asset.status == VMStatus::Running;
                                    let is_stopped = asset.status == VMStatus::Stopped;
                                    let asset_for_edit = asset.clone();
                                    let asset_for_ssh = asset.clone();
                                    let asset_for_console = asset.clone();
                                    let row_bg = if idx % 2 == 0 { "#ffffff" } else { "#fafafa" };

                                    html! {
                                        <tr class="excel-row" style={format!("background: {}; transition: background-color 0.15s ease;", row_bg)}>
                                            <td style="border: 1px solid #ddd; padding: 16px 10px;">
                                                <div style="font-weight: 500; color: #2c3e50; margin-bottom: 2px;">{ &asset.asset_name }</div>
                                                <div style="font-size: 11px; color: #95a5a6;">{ &asset.instance_id }</div>
                                            </td>
                                            <td style="border: 1px solid #ddd; padding: 16px 10px; font-family: monospace; font-size: 12px;">{ &asset.spec.instance_type }</td>
                                            <td style="border: 1px solid #ddd; padding: 16px 10px;">{ format!("{}核", asset.spec.cpu_cores) }</td>
                                            <td style="border: 1px solid #ddd; padding: 16px 10px;">{ format!("{}GB", asset.spec.memory_gb) }</td>
                                            <td style="border: 1px solid #ddd; padding: 16px 10px; font-size: 12px;">{ format!("{}GB {}", asset.system_disk.size_gb, asset.system_disk.disk_type) }</td>
                                            <td style="border: 1px solid #ddd; padding: 16px 10px;">{ provider }</td>
                                            <td style="border: 1px solid #ddd; padding: 16px 10px; font-size: 12px;">{ region }</td>
                                            <td style="border: 1px solid #ddd; padding: 16px 10px;">
                                                { if let Some(ref pub_ip) = asset.public_ip {
                                                    html! { <span style="color: #3273dc; font-family: monospace; font-size: 12px;">{ pub_ip }</span> }
                                                } else {
                                                    html! { <span style="color: #bdc3c7;">{"-"}</span> }
                                                }}
                                            </td>
                                            <td style="border: 1px solid #ddd; padding: 16px 10px; font-family: monospace; font-size: 12px;">{ &asset.private_ip }</td>
                                            <td style="border: 1px solid #ddd; padding: 16px 10px;">{ billing_text(&asset.billing_mode) }</td>
                                            <td style="border: 1px solid #ddd; padding: 16px 10px; font-size: 12px;">{ format_expire(&asset.expire_time) }</td>
                                            <td style="border: 1px solid #ddd; padding: 16px 10px;">
                                                <span style={format!("background: {}; color: white; padding: 3px 8px; border-radius: 3px; font-size: 11px; font-weight: 500;",
                                                    if is_running { "#27ae60" } else if is_stopped { "#e74c3c" } else { "#f39c12" })}>
                                                    { status_display }
                                                </span>
                                            </td>
                                            <td style="border: 1px solid #ddd; padding: 16px 10px; font-size: 12px; max-width: 160px; overflow: hidden; text-overflow: ellipsis;">{ &asset.os_name }</td>
                                            <td style="border: 1px solid #ddd; padding: 16px 10px; font-family: monospace; font-size: 11px; max-width: 160px; overflow: hidden; text-overflow: ellipsis;">{ &asset.image_id }</td>
                                            <td style="border: 1px solid #ddd; padding: 16px 10px;">{ &asset.department.name }</td>
                                            <td style="border: 1px solid #ddd; padding: 16px 10px;">{ &asset.project.name }</td>
                                            <td style="border: 1px solid #ddd; padding: 16px 10px;">{ &asset.owner.name }</td>
                                            <td style="border: 1px solid #ddd; padding: 16px 10px; font-size: 12px;">{ format_datetime(&asset.created_at) }</td>
                                            <td style="border: 1px solid #ddd; padding: 16px 10px; text-align: center;">{ asset.cloud_disk_count }</td>
                                            <td style="border: 1px solid #ddd; padding: 16px 10px;">{ format!("{}GB", asset.cloud_disk_total_size_gb) }</td>
                                            <td style="border: 1px solid #ddd; padding: 16px 10px; text-align: center;">
                                                { if asset.snapshot_info.has_snapshot {
                                                    html! { <span style="color: #27ae60; font-weight: bold;">{"✓"}</span> }
                                                } else {
                                                    html! { <span style="color: #bdc3c7;">{"-"}</span> }
                                                }}
                                                { if asset.snapshot_info.has_snapshot {
                                                    html! { <span style="font-size: 11px; color: #7f8c8d; margin-left: 4px;">{ format!("({})", asset.snapshot_info.snapshot_count) }</span> }
                                                } else {
                                                    html! {}
                                                }}
                                            </td>
                                            <td style="border: 1px solid #ddd; padding: 6px; position: sticky; right: 0; background: {row_bg}; border-left: 2px solid #ccc; z-index: 10;">
                                                <div class="select is-small is-fullwidth" style="margin-bottom: 0;">
                                                    <select style="border: 1px solid #ddd; border-radius: 4px; padding: 4px 8px; font-size: 12px; cursor: pointer;" onchange={ {
                                                        let on_edit_click = on_edit_click.clone();
                                                        let asset_for_console = asset_for_console.clone();
                                                        let asset_for_ssh = asset_for_ssh.clone();
                                                        let user_role = user_role.clone();
                                                        Callback::from(move |e: Event| {
                                                            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
                                                            match select.value().as_str() {
                                                                "view" => {
                                                                    let on_edit_click = on_edit_click.clone();
                                                                    on_edit_click.emit(asset_for_edit.clone());
                                                                }
                                                                "console" => {
                                                                    if let Some(window) = web_sys::window() {
                                                                        let _ = window.alert_with_message("控制台链接功能：点击后跳转到对应云厂商控制台");
                                                                    }
                                                                }
                                                                "ssh" => {
                                                                    if let Some(window) = web_sys::window() {
                                                                        let _ = window.alert_with_message(&format!("SSH命令: ssh root@{}", &asset_for_ssh.private_ip));
                                                                    }
                                                                }
                                                                "reboot" => {
                                                                    if let Some(window) = web_sys::window() {
                                                                        let _ = window.alert_with_message(&format!("重启实例: {}", &asset_for_console.asset_name));
                                                                    }
                                                                }
                                                                "start" => {
                                                                    if let Some(window) = web_sys::window() {
                                                                        let _ = window.alert_with_message(&format!("启动实例: {}", &asset_for_console.asset_name));
                                                                    }
                                                                }
                                                                "stop" => {
                                                                    if let Some(window) = web_sys::window() {
                                                                        let _ = window.alert_with_message(&format!("停止实例: {}", &asset_for_console.asset_name));
                                                                    }
                                                                }
                                                                "delete" => {
                                                                    if let Some(window) = web_sys::window() {
                                                                        let _ = window.alert_with_message(&format!("释放实例: {}", &asset_for_console.asset_name));
                                                                    }
                                                                }
                                                                _ => {}
                                                            }
                                                            // Reset select to default
                                                            select.set_value("");
                                                        })
                                                    } }>
                                                        <option value="">{"\u{1f4cb} 操作..."}</option>
                                                        <option value="view">{"\u{1f440} 查看详情"}</option>
                                                        {
                                                            // SecAdmin 和 SysAdmin 可以看到更多操作
                                                            if user_role == Some(Role::SecAdmin) || user_role == Some(Role::SysAdmin) {
                                                                html! {
                                                                    <>
                                                                        <option value="console">{"\u{1f5a5} 云控制台"}</option>
                                                                        <option value="ssh">{"\u{1f4bb} SSH连接"}</option>
                                                                        <option value="reboot">{"\u{1f504} 重启实例"}</option>
                                                                        <option value="start">{"\u{25b6} 启动实例"}</option>
                                                                        <option value="stop">{"\u{23f9} 停止实例"}</option>
                                                                    </>
                                                                }
                                                            } else {
                                                                html! {}
                                                            }
                                                        }
                                                        {
                                                            // 只有 SecAdmin 可以删除实例
                                                            if user_role == Some(Role::SecAdmin) {
                                                                html! {
                                                                    <option value="delete" style="color: #e74c3c;">{"\u{1f5d1} 释放实例"}</option>
                                                                }
                                                            } else {
                                                                html! {}
                                                            }
                                                        }
                                                    </select>
                                                </div>
                                            </td>
                                        </tr>
                                    }
                                }) }
                            </tbody>
                        </table>
                    </div>
                </div>

                // Summary info
                <div class="box">
                    <p class="is-size-7">
                        { format!("共 {} 个云资产", filtered_assets.len()) }
                    </p>
                </div>
            }

            // Edit Modal
            if *show_edit_modal {
                if let Some(ref asset) = *editing_asset {
                    <div class="modal is-active">
                        <div class="modal-background" onclick={on_close_modal.clone()}></div>
                        <div class="modal-card" style="width: 800px;">
                            <header class="modal-card-head">
                                <p class="modal-card-title">{ format!("编辑: {}", asset.asset_name) }</p>
                                <button class="delete" aria-label="close" onclick={on_close_modal.clone()}></button>
                            </header>
                            <section class="modal-card-body" style="max-height: 60vh; overflow-y: auto;">
                                // Display message if any
                                if let Some(ref msg) = *edit_message {
                                    <div class={classes!("notification", if msg.contains("成功") { "is-success" } else { "is-danger" })}>
                                        { msg }
                                    </div>
                                }

                                <div class="columns is-multiline">
                                    // Asset Name (readonly - from cloud)
                                    <div class="column is-6">
                                        <label class="label is-small">{"资产名称"}</label>
                                        <input class="input is-small" type="text" value={asset.asset_name.clone()} readonly=true />
                                    </div>

                                    // Instance ID (readonly)
                                    <div class="column is-6">
                                        <label class="label is-small">{"实例ID"}</label>
                                        <input class="input is-small" type="text" value={asset.instance_id.clone()} readonly=true />
                                    </div>

                                    // Department (editable - for internal management)
                                    <div class="column is-6">
                                        <label class="label is-small">{"部门"}</label>
                                        <input class="input is-small" type="text" value={asset.department.name.clone()} readonly=true />
                                        <p class="help is-size-7">{"部门信息同步自组织架构"}</p>
                                    </div>

                                    // Project (editable)
                                    <div class="column is-6">
                                        <label class="label is-small">{"项目"}</label>
                                        <input class="input is-small" type="text" value={asset.project.name.clone()} readonly=true />
                                    </div>

                                    // Owner Name
                                    <div class="column is-6">
                                        <label class="label is-small">{"负责人"}</label>
                                        <input class="input is-small" type="text" value={asset.owner.name.clone()} readonly=true />
                                    </div>

                                    // Owner Email
                                    <div class="column is-6">
                                        <label class="label is-small">{"负责人邮箱"}</label>
                                        <input class="input is-small" type="text" value={asset.owner.email.clone()} readonly=true />
                                    </div>

                                    // Cloud Provider (readonly)
                                    <div class="column is-6">
                                        <label class="label is-small">{"云厂商"}</label>
                                        <input class="input is-small" type="text" value={provider_name(&asset.cloud_region.provider)} readonly=true />
                                    </div>

                                    // Region (readonly)
                                    <div class="column is-6">
                                        <label class="label is-small">{"云区"}</label>
                                        <input class="input is-small" type="text" value={asset.cloud_region.region_name.clone()} readonly=true />
                                    </div>

                                    // Instance Type
                                    <div class="column is-6">
                                        <label class="label is-small">{"实例规格"}</label>
                                        <input class="input is-small" type="text" value={asset.spec.instance_type.clone()} readonly=true />
                                    </div>

                                    // Status (readonly - from cloud)
                                    <div class="column is-6">
                                        <label class="label is-small">{"状态"}</label>
                                        <input class="input is-small" type="text" value={status_text(&asset.status)} readonly=true />
                                    </div>

                                    // CPU
                                    <div class="column is-4">
                                        <label class="label is-small">{"CPU核心"}</label>
                                        <input class="input is-small" type="text" value={format!("{}", asset.spec.cpu_cores)} readonly=true />
                                    </div>

                                    // Memory
                                    <div class="column is-4">
                                        <label class="label is-small">{"内存(GB)"}</label>
                                        <input class="input is-small" type="text" value={format!("{}", asset.spec.memory_gb)} readonly=true />
                                    </div>

                                    // System Disk
                                    <div class="column is-4">
                                        <label class="label is-small">{"系统盘(GB)"}</label>
                                        <input class="input is-small" type="text" value={format!("{}", asset.system_disk.size_gb)} readonly=true />
                                    </div>

                                    // IPs (readonly)
                                    <div class="column is-6">
                                        <label class="label is-small">{"公网IP"}</label>
                                        <input class="input is-small" type="text" value={asset.public_ip.clone().unwrap_or_else(|| "无".to_string())} readonly=true />
                                    </div>

                                    <div class="column is-6">
                                        <label class="label is-small">{"内网IP"}</label>
                                        <input class="input is-small" type="text" value={asset.private_ip.clone()} readonly=true />
                                    </div>

                                    // Billing Mode
                                    <div class="column is-6">
                                        <label class="label is-small">{"计费模式"}</label>
                                        <input class="input is-small" type="text" value={billing_text(&asset.billing_mode)} readonly=true />
                                    </div>

                                    // Expire Time
                                    <div class="column is-6">
                                        <label class="label is-small">{"到期时间"}</label>
                                        <input class="input is-small" type="text" value={format_expire(&asset.expire_time)} readonly=true />
                                    </div>

                                    // OS Info
                                    <div class="column is-6">
                                        <label class="label is-small">{"操作系统"}</label>
                                        <input class="input is-small" type="text" value={asset.os_name.clone()} readonly=true />
                                    </div>

                                    // Image ID
                                    <div class="column is-6">
                                        <label class="label is-small">{"镜像ID"}</label>
                                        <input class="input is-small" type="text" value={asset.image_id.clone()} readonly=true />
                                    </div>

                                    // Created Time
                                    <div class="column is-6">
                                        <label class="label is-small">{"创建时间"}</label>
                                        <input class="input is-small" type="text" value={format_datetime(&asset.created_at)} readonly=true />
                                    </div>

                                    // Cloud Disks
                                    <div class="column is-6">
                                        <label class="label is-small">{"云盘"}</label>
                                        <input class="input is-small" type="text" value={format!("{}个 / {}GB", asset.cloud_disk_count, asset.cloud_disk_total_size_gb)} readonly=true />
                                    </div>

                                    // Snapshots
                                    <div class="column is-12">
                                        <label class="label is-small">{"快照信息"}</label>
                                        <div class="box is-small">
                                            { if asset.snapshot_info.has_snapshot {
                                                html! {
                                                    <>
                                                        <p>{ format!("快照数量: {}", asset.snapshot_info.snapshot_count) }</p>
                                                        <p>{ format!("快照总量: {}GB", asset.snapshot_info.total_snapshot_size_gb) }</p>
                                                        {
                                                            if let Some(ref snap_time) = asset.snapshot_info.latest_snapshot_time {
                                                                html! { <p>{ format!("最新快照: {}", format_datetime(snap_time)) }</p> }
                                                            } else {
                                                                html! {}
                                                            }
                                                        }
                                                    </>
                                                }
                                            } else {
                                                html! { <p class="has-text-grey">{"无快照"}</p> }
                                            }}
                                        </div>
                                    </div>

                                    // Tags
                                    <div class="column is-12">
                                        <label class="label is-small">{"标签"}</label>
                                        <div class="tags">
                                            { for asset.tags.iter().map(|tag| {
                                                html! { <span class="tag is-light">{ tag }</span> }
                                            }) }
                                        </div>
                                    </div>
                                </div>

                                <div class="notification is-info is-light">
                                    <p class="is-size-7">{"注意：云资产的基础信息（如规格、IP、状态等）由云平台同步，只读。组织信息（部门、项目、负责人）可通过云平台控制台修改。"}</p>
                                </div>
                            </section>
                            <footer class="modal-card-foot" style="justify-content: flex-end;">
                                <button class="button" onclick={on_close_modal.clone()}>{"关闭"}</button>
                            </footer>
                        </div>
                    </div>
                }
            }
        </div>
    }
}

// ============== Cloud Provider Management Component ==============
// 云区对接管理组件

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CloudProviderConfigDisplay {
    pub id: Option<i32>,
    pub provider: CloudProvider,
    pub region_id: String,
    pub region_name: String,
    pub available_zones: Vec<String>,
    pub account_name: String,
    pub access_key_id: String,
    pub status: CloudProviderConfigStatus,
    pub remarks: Option<String>,
    pub last_test_time: Option<chrono::DateTime<chrono::Utc>>,
    pub last_test_result: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[function_component]
fn CloudProviderManagement() -> Html {
    let configs = use_state(|| Vec::new());
    let loading = use_state(|| true);
    let error_message = use_state(|| None as Option<String>);
    let success_message = use_state(|| None as Option<String>);

    // Modal states
    let show_create_modal = use_state(|| false);

    // Form states
    let form_provider = use_state(|| CloudProvider::Aliyun);
    let form_region_id = use_state(|| String::new());
    let form_region_name = use_state(|| String::new());
    let form_account_name = use_state(|| String::new());
    let form_access_key_id = use_state(|| String::new());
    let form_access_key_secret = use_state(|| String::new());
    let form_remarks = use_state(|| None as Option<String>);

    // Filter states
    let filter_provider = use_state(|| String::new());
    let filter_status = use_state(|| String::new());

    let token = get_auth_token();

    // Fetch configs
    let fetch_configs = {
        let configs = configs.clone();
        let loading = loading.clone();
        let error_message = error_message.clone();
        let token = token.clone();

        Callback::from(move |_| {
            let configs = configs.clone();
            let loading = loading.clone();
            let error_message = error_message.clone();
            let token = token.clone();

            spawn_local(async move {
                loading.set(true);
                error_message.set(None);

                match Request::get(&api_url("cloud-provider-configs"))
                    .header("Authorization", &token)
                    .send()
                    .await
                {
                    Ok(resp) if resp.ok() => {
                        match resp.json::<Vec<serde_json::Value>>().await {
                            Ok(data) => {
                                let parsed_configs: Vec<CloudProviderConfigDisplay> = data
                                    .into_iter()
                                    .filter_map(|v| serde_json::from_value(v).ok())
                                    .collect();
                                configs.set(parsed_configs);
                            }
                            Err(e) => {
                                error_message.set(Some(format!("解析响应失败: {}", e)));
                            }
                        }
                    }
                    Ok(resp) => {
                        let status = resp.status();
                        error_message.set(Some(format!("获取配置失败: HTTP {}", status)));
                    }
                    Err(e) => {
                        error_message.set(Some(format!("网络错误: {}", e)));
                    }
                }

                loading.set(false);
            });
        })
    };

    // Initial fetch
    use_effect_with((), {
        let fetch_configs = fetch_configs.clone();
        move |_| {
            fetch_configs.emit(());
            || ()
        }
    });

    // Helper functions
    let provider_name = |provider: &CloudProvider| -> String {
        match provider {
            CloudProvider::Aliyun => "阿里云".to_string(),
            CloudProvider::Tencent => "腾讯云".to_string(),
            CloudProvider::Huawei => "华为云".to_string(),
            CloudProvider::Aws => "AWS".to_string(),
            CloudProvider::Azure => "Azure".to_string(),
            CloudProvider::Gcp => "GCP".to_string(),
            CloudProvider::Baidu => "百度云".to_string(),
            CloudProvider::Custom(s) => s.clone(),
        }
    };

    let status_badge = |status: &CloudProviderConfigStatus| -> Html {
        match status {
            CloudProviderConfigStatus::Active => {
                html! { <span class="tag is-success">{ "已启用" }</span> }
            }
            CloudProviderConfigStatus::Inactive => {
                html! { <span class="tag is-light">{ "已停用" }</span> }
            }
            CloudProviderConfigStatus::Testing => {
                html! { <span class="tag is-warning">{ "测试中" }</span> }
            }
            CloudProviderConfigStatus::Error => {
                html! { <span class="tag is-danger">{ "连接错误" }</span> }
            }
        }
    };

    // Filter configs
    let filtered_configs = (*configs).iter().filter(|config| {
        let provider_match = filter_provider.is_empty()
            || config.provider.as_str() == &*filter_provider;
        let status_match = filter_status.is_empty()
            || config.status.as_str() == &*filter_status;
        provider_match && status_match
    }).cloned().collect::<Vec<_>>();

    let on_provider_filter_change = {
        let filter_provider = filter_provider.clone();
        Callback::from(move |e: Event| {
            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
            filter_provider.set(select.value());
        })
    };

    let on_status_filter_change = {
        let filter_status = filter_status.clone();
        Callback::from(move |e: Event| {
            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
            filter_status.set(select.value());
        })
    };

    let on_open_create_modal = Callback::from({
        let show_create_modal = show_create_modal.clone();
        move |_| show_create_modal.set(true)
    });

    let on_close_create_modal = {
        let show_create_modal = show_create_modal.clone();
        let form_provider = form_provider.clone();
        let form_region_id = form_region_id.clone();
        let form_region_name = form_region_name.clone();
        let form_account_name = form_account_name.clone();
        let form_access_key_id = form_access_key_id.clone();
        let form_access_key_secret = form_access_key_secret.clone();
        let form_remarks = form_remarks.clone();

        Callback::from(move |_: web_sys::MouseEvent| {
            show_create_modal.set(false);
            form_provider.set(CloudProvider::Aliyun);
            form_region_id.set(String::new());
            form_region_name.set(String::new());
            form_account_name.set(String::new());
            form_access_key_id.set(String::new());
            form_access_key_secret.set(String::new());
            form_remarks.set(None);
        })
    };

    let on_create_config = {
        let fetch_configs = fetch_configs.clone();
        let show_create_modal = show_create_modal.clone();
        let success_message = success_message.clone();
        let error_message = error_message.clone();
        let token = token.clone();
        let form_provider = form_provider.clone();
        let form_region_id = form_region_id.clone();
        let form_region_name = form_region_name.clone();
        let form_account_name = form_account_name.clone();
        let form_access_key_id = form_access_key_id.clone();
        let form_access_key_secret = form_access_key_secret.clone();
        let form_remarks = form_remarks.clone();
        let on_close_create_modal = on_close_create_modal.clone();

        Callback::from(move |_: web_sys::MouseEvent| {
            let provider = (*form_provider).clone();
            let region_id = (*form_region_id).clone();
            let region_name = (*form_region_name).clone();
            let account_name = (*form_account_name).clone();
            let access_key_id = (*form_access_key_id).clone();
            let access_key_secret = (*form_access_key_secret).clone();
            let remarks = (*form_remarks).clone();

            let fetch_configs = fetch_configs.clone();
            let show_create_modal = show_create_modal.clone();
            let success_message = success_message.clone();
            let error_message = error_message.clone();
            let token = token.clone();
            let on_close_create_modal = on_close_create_modal.clone();

            spawn_local(async move {
                let body = serde_json::json!({
                    "provider": provider,
                    "region_id": region_id,
                    "region_name": region_name,
                    "available_zones": [],
                    "account_name": account_name,
                    "access_key_id": access_key_id,
                    "access_key_secret": access_key_secret,
                    "remarks": remarks,
                });

                match Request::post(&api_url("cloud-provider-configs"))
                    .header("Authorization", &token)
                    .header("Content-Type", "application/json")
                    .body(body.to_string())
                    .unwrap()
                    .send()
                    .await
                {
                    Ok(resp) if resp.ok() => {
                        success_message.set(Some("云区对接配置创建成功".to_string()));
                        show_create_modal.set(false);
                        fetch_configs.emit(());
                        spawn_local(async move {
                            Timeout::new(3000, move || {
                                success_message.set(None);
                            }).forget();
                        });
                    }
                    Ok(resp) => {
                        let status = resp.status();
                        error_message.set(Some(format!("创建失败: HTTP {}", status)));
                    }
                    Err(e) => {
                        error_message.set(Some(format!("网络错误: {}", e)));
                    }
                }
                on_close_create_modal.emit(web_sys::MouseEvent::new("click").unwrap());
            });
        })
    };

    let on_test_connection = {
        let fetch_configs = fetch_configs.clone();
        let success_message = success_message.clone();
        let error_message = error_message.clone();
        let token = token.clone();

        Callback::from(move |config_id: i32| {
            let fetch_configs = fetch_configs.clone();
            let success_message = success_message.clone();
            let error_message = error_message.clone();
            let token = token.clone();

            spawn_local(async move {
                match Request::post(&format!("{}{}/test", api_url("cloud-provider-configs"), config_id))
                    .header("Authorization", &token)
                    .send()
                    .await
                {
                    Ok(resp) if resp.ok() => {
                        match resp.json::<serde_json::Value>().await {
                            Ok(result) => {
                                if result.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                                    success_message.set(Some(format!(
                                        "连接测试成功: {}",
                                        result.get("message").and_then(|v| v.as_str()).unwrap_or("成功")
                                    )));
                                } else {
                                    error_message.set(Some(format!(
                                        "连接测试失败: {}",
                                        result.get("message").and_then(|v| v.as_str()).unwrap_or("未知错误")
                                    )));
                                }
                                fetch_configs.emit(());
                            }
                            Err(e) => {
                                error_message.set(Some(format!("解析响应失败: {}", e)));
                            }
                        }
                    }
                    Ok(resp) => {
                        let status = resp.status();
                        error_message.set(Some(format!("测试失败: HTTP {}", status)));
                    }
                    Err(e) => {
                        error_message.set(Some(format!("网络错误: {}", e)));
                    }
                }
            });
        })
    };

    let on_delete_config = {
        let fetch_configs = fetch_configs.clone();
        let success_message = success_message.clone();
        let error_message = error_message.clone();
        let token = token.clone();

        Callback::from(move |config_id: i32| {
            if !web_sys::window().unwrap().confirm_with_message("确定要删除这个云区对接配置吗？").unwrap_or(true) {
                return;
            }

            let fetch_configs = fetch_configs.clone();
            let success_message = success_message.clone();
            let error_message = error_message.clone();
            let token = token.clone();

            spawn_local(async move {
                match Request::delete(&format!("{}{}", api_url("cloud-provider-configs"), config_id))
                    .header("Authorization", &token)
                    .send()
                    .await
                {
                    Ok(resp) if resp.ok() => {
                        success_message.set(Some("云区对接配置已删除".to_string()));
                        fetch_configs.emit(());
                        spawn_local(async move {
                            Timeout::new(3000, move || {
                                success_message.set(None);
                            }).forget();
                        });
                    }
                    Ok(resp) => {
                        let status = resp.status();
                        error_message.set(Some(format!("删除失败: HTTP {}", status)));
                    }
                    Err(e) => {
                        error_message.set(Some(format!("网络错误: {}", e)));
                    }
                }
            });
        })
    };

    let on_toggle_status = {
        let fetch_configs = fetch_configs.clone();
        let success_message = success_message.clone();
        let error_message = error_message.clone();
        let token = token.clone();

        Callback::from(move |(config_id, new_status): (i32, CloudProviderConfigStatus)| {
            let fetch_configs = fetch_configs.clone();
            let success_message = success_message.clone();
            let error_message = error_message.clone();
            let token = token.clone();

            spawn_local(async move {
                let body = serde_json::json!({
                    "status": new_status,
                });

                match Request::put(&format!("{}{}", api_url("cloud-provider-configs"), config_id))
                    .header("Authorization", &token)
                    .header("Content-Type", "application/json")
                    .body(body.to_string())
                    .unwrap()
                    .send()
                    .await
                {
                    Ok(resp) if resp.ok() => {
                        let msg = match new_status {
                            CloudProviderConfigStatus::Active => "已启用",
                            CloudProviderConfigStatus::Inactive => "已停用",
                            _ => "状态已更新",
                        };
                        success_message.set(Some(format!("配置{}", msg)));
                        fetch_configs.emit(());
                        spawn_local(async move {
                            Timeout::new(3000, move || {
                                success_message.set(None);
                            }).forget();
                        });
                    }
                    Ok(resp) => {
                        let status = resp.status();
                        error_message.set(Some(format!("更新失败: HTTP {}", status)));
                    }
                    Err(e) => {
                        error_message.set(Some(format!("网络错误: {}", e)));
                    }
                }
            });
        })
    };

    html! {
        <div class="cloud-provider-management">
            <div class="level mb-4">
                <div class="level-left">
                    <div class="level-item">
                        <h1 class="title is-4">{ "🌐 云区对接管理" }</h1>
                    </div>
                </div>
                <div class="level-right">
                    <div class="level-item">
                        <button class="button is-primary" onclick={on_open_create_modal}>
                            <span class="icon"><i class="fas fa-plus"></i></span>
                            <span>{ "添加云区配置" }</span>
                        </button>
                    </div>
                </div>
            </div>

            { if let Some(msg) = (*error_message).clone() {
                html! {
                    <div class="notification is-danger is-light">
                        <button class="delete" onclick={
                            let error_message = error_message.clone();
                            Callback::from(move |_| error_message.set(None))
                        }></button>
                        { msg }
                    </div>
                }
            } else { html! {} } }

            { if let Some(msg) = (*success_message).clone() {
                html! {
                    <div class="notification is-success is-light">
                        <button class="delete" onclick={
                            let success_message = success_message.clone();
                            Callback::from(move |_| success_message.set(None))
                        }></button>
                        { msg }
                    </div>
                }
            } else { html! {} } }

            <div class="box">
                <div class="field is-horizontal">
                    <div class="field-label is-normal">
                        <label class="label">{ "筛选条件" }</label>
                    </div>
                    <div class="field-body">
                        <div class="field">
                            <label class="label">{ "云厂商" }</label>
                            <div class="control">
                                <div class="select is-fullwidth">
                                    <select onchange={on_provider_filter_change.clone()}>
                                        <option value="">{ "全部" }</option>
                                        <option value="aliyun">{ "阿里云" }</option>
                                        <option value="tencent">{ "腾讯云" }</option>
                                        <option value="huawei">{ "华为云" }</option>
                                        <option value="aws">{ "AWS" }</option>
                                    </select>
                                </div>
                            </div>
                        </div>
                        <div class="field">
                            <label class="label">{ "状态" }</label>
                            <div class="control">
                                <div class="select is-fullwidth">
                                    <select onchange={on_status_filter_change.clone()}>
                                        <option value="">{ "全部" }</option>
                                        <option value="active">{ "已启用" }</option>
                                        <option value="inactive">{ "已停用" }</option>
                                        <option value="testing">{ "测试中" }</option>
                                        <option value="error">{ "连接错误" }</option>
                                    </select>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            {
                if *loading {
                    html! {
                        <div class="has-text-centered py-6">
                            <span class="button is-loading is-large is-white"></span>
                            <p class="mt-4">{ "加载中..." }</p>
                        </div>
                    }
                } else if filtered_configs.is_empty() {
                    html! {
                        <div class="box has-text-centered py-6">
                            <p class="has-text-grey">{ "暂无云区对接配置" }</p>
                            <p class="has-text-grey is-size-7 mt-2">
                                { "点击上方「添加云区配置」按钮添加新的云平台对接" }
                            </p>
                        </div>
                    }
                } else {
                    html! {
                        <div class="columns is-multiline">
                        { for filtered_configs.iter().map(|config| {
                            let config_id = config.id.unwrap_or(0);
                            let on_test_click = on_test_connection.clone();
                            let on_delete_click = on_delete_config.clone();
                            let on_toggle_click = on_toggle_status.clone();
                            let provider = config.provider.clone();
                            let status = config.status.clone();
                            let config_name = format!("{} - {} ({})",
                                provider_name(&provider),
                                config.region_name,
                                config.account_name
                            );

                            let can_enable = status != CloudProviderConfigStatus::Active;
                            let can_disable = status != CloudProviderConfigStatus::Inactive;

                            html! {
                                <div key={format!("{:?}", config_id)} class="column is-6">
                                    <div class="box">
                                        <div class="level">
                                            <div class="level-left">
                                                <div class="level-item">
                                                    <span class="icon is-medium has-text-info">
                                                        <i class="fas fa-cloud fa-lg"></i>
                                                    </span>
                                                    <div class="ml-3">
                                                        <p class="title is-5 mb-1">{ config_name }</p>
                                                        <p class="subtitle is-7 mb-0">
                                                            { format!("Region: {} | Zone: {}",
                                                                config.region_id,
                                                                if config.available_zones.is_empty() {
                                                                    "默认".to_string()
                                                                } else {
                                                                    config.available_zones.join(", ")
                                                                }
                                                            )}
                                                        </p>
                                                    </div>
                                                </div>
                                            </div>
                                            <div class="level-right">
                                                <div class="level-item">
                                                    { status_badge(&status) }
                                                </div>
                                            </div>
                                        </div>

                                        if let Some(result) = &config.last_test_result {
                                            <div class="content is-small mt-3">
                                                <p class={classes![
                                                    "has-text-weight-normal",
                                                    if config.status == CloudProviderConfigStatus::Active {
                                                        "has-text-success"
                                                    } else {
                                                        "has-text-danger"
                                                    }
                                                ]}>
                                                    <span class="icon is-small">
                                                        <i class="fas fa-info-circle"></i>
                                                    </span>
                                                    { result.clone() }
                                                </p>
                                                if let Some(test_time) = config.last_test_time {
                                                    <p class="has-text-grey is-size-7 mt-1">
                                                        { format!("最后测试: {}", test_time.format("%Y-%m-%d %H:%M")) }
                                                    </p>
                                                }
                                            </div>
                                        }

                                        <div class="buttons is-right mt-4">
                                            if can_enable {
                                                <button class="button is-small is-success"
                                                    onclick={on_toggle_click.reform(move |_| (config_id, CloudProviderConfigStatus::Active))}>
                                                    <span class="icon"><i class="fas fa-check"></i></span>
                                                    <span>{ "启用" }</span>
                                                </button>
                                            }
                                            if can_disable {
                                                <button class="button is-small is-light"
                                                    onclick={on_toggle_click.reform(move |_| (config_id, CloudProviderConfigStatus::Inactive))}>
                                                    <span class="icon"><i class="fas fa-pause"></i></span>
                                                    <span>{ "停用" }</span>
                                                </button>
                                            }
                                            <button class="button is-small is-info"
                                                onclick={on_test_click.reform(move |_| config_id)}>
                                                <span class="icon"><i class="fas fa-plug"></i></span>
                                                <span>{ "测试连接" }</span>
                                            </button>
                                            <button class="button is-small is-danger is-outlined"
                                                onclick={on_delete_click.reform(move |_| config_id)}>
                                                <span class="icon"><i class="fas fa-trash"></i></span>
                                                <span>{ "删除" }</span>
                                            </button>
                                        </div>

                                        if let Some(remarks) = &config.remarks {
                                            <div class="content is-small mt-3 pt-3" style="border-top: 1px solid #eee;">
                                                <p class="has-text-grey">{ format!("备注: {}", remarks) }</p>
                                            </div>
                                        }
                                    </div>
                                </div>
                            }
                        }) }
                    </div>
                    }
                }
            }

            {
                if *show_create_modal {
                    html! {
                    <div class="modal is-active">
                        <div class="modal-background" onclick={on_close_create_modal.clone()}></div>
                        <div class="modal-card">
                        <header class="modal-card-head">
                            <p class="modal-card-title">{ "添加云区对接配置" }</p>
                            <button class="delete" onclick={on_close_create_modal.clone()}></button>
                        </header>
                        <section class="modal-card-body">
                            <div class="field">
                                <label class="label">{ "云厂商" }</label>
                                <div class="control">
                                    <div class="select is-fullwidth">
                                        <select onchange={
                                            let form_provider = form_provider.clone();
                                            Callback::from(move |e: Event| {
                                                let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
                                                match select.value().as_str() {
                                                    "aliyun" => form_provider.set(CloudProvider::Aliyun),
                                                    "tencent" => form_provider.set(CloudProvider::Tencent),
                                                    "huawei" => form_provider.set(CloudProvider::Huawei),
                                                    "aws" => form_provider.set(CloudProvider::Aws),
                                                    _ => form_provider.set(CloudProvider::Aliyun),
                                                }
                                            })
                                        }>
                                            <option value="aliyun">{ "阿里云" }</option>
                                            <option value="tencent">{ "腾讯云" }</option>
                                            <option value="huawei">{ "华为云" }</option>
                                            <option value="aws">{ "AWS" }</option>
                                        </select>
                                    </div>
                                </div>
                            </div>

                            <div class="field">
                                <label class="label">{ "区域ID" }</label>
                                <div class="control">
                                    <input type="text" class="input"
                                        placeholder="例如: cn-hangzhou (阿里云) / ap-guangzhou (腾讯云)"
                                        value={(*form_region_id).clone()}
                                        oninput={
                                            let form_region_id = form_region_id.clone();
                                            Callback::from(move |e: InputEvent| {
                                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                form_region_id.set(input.value());
                                            })
                                        }
                                    />
                                </div>
                            </div>

                            <div class="field">
                                <label class="label">{ "区域名称" }</label>
                                <div class="control">
                                    <input type="text" class="input"
                                        placeholder="例如: 华东1(杭州)"
                                        value={(*form_region_name).clone()}
                                        oninput={
                                            let form_region_name = form_region_name.clone();
                                            Callback::from(move |e: InputEvent| {
                                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                form_region_name.set(input.value());
                                            })
                                        }
                                    />
                                </div>
                            </div>

                            <div class="field">
                                <label class="label">{ "账户名称" }</label>
                                <div class="control">
                                    <input type="text" class="input"
                                        placeholder="用于标识不同的云账户"
                                        value={(*form_account_name).clone()}
                                        oninput={
                                            let form_account_name = form_account_name.clone();
                                            Callback::from(move |e: InputEvent| {
                                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                form_account_name.set(input.value());
                                            })
                                        }
                                    />
                                </div>
                            </div>

                            <div class="field">
                                <label class="label">{ "Access Key ID" }</label>
                                <div class="control">
                                    <input type="text" class="input"
                                        placeholder="云平台 Access Key ID"
                                        value={(*form_access_key_id).clone()}
                                        oninput={
                                            let form_access_key_id = form_access_key_id.clone();
                                            Callback::from(move |e: InputEvent| {
                                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                form_access_key_id.set(input.value());
                                            })
                                        }
                                    />
                                </div>
                            </div>

                            <div class="field">
                                <label class="label">{ "Access Key Secret" }</label>
                                <div class="control">
                                    <input type="password" class="input"
                                        placeholder="云平台 Access Key Secret"
                                        value={(*form_access_key_secret).clone()}
                                        oninput={
                                            let form_access_key_secret = form_access_key_secret.clone();
                                            Callback::from(move |e: InputEvent| {
                                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                form_access_key_secret.set(input.value());
                                            })
                                        }
                                    />
                                </div>
                            </div>

                            <div class="field">
                                <label class="label">{ "备注" }</label>
                                <div class="control">
                                    <input type="text" class="input"
                                        placeholder="选填"
                                        value={(*form_remarks).clone().unwrap_or_default()}
                                        oninput={
                                            let form_remarks = form_remarks.clone();
                                            Callback::from(move |e: InputEvent| {
                                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                form_remarks.set(Some(input.value()));
                                            })
                                        }
                                    />
                                </div>
                            </div>

                            <article class="message is-small is-info">
                                <div class="message-body">
                                    <p class="is-size-7">
                                        { "注意：创建后请先进行「测试连接」，确认配置正确后再启用。" }
                                    </p>
                                </div>
                            </article>
                        </section>
                        <footer class="modal-card-foot" style="justify-content: flex-end;">
                            <button class="button" onclick={on_close_create_modal.clone()}>{ "取消" }</button>
                            <button class="button is-primary" onclick={on_create_config}>{ "创建" }</button>
                        </footer>
                    </div>
                    </div>
                    }
                } else {
                    html! {}
                }
            }

            <div class="box mt-5">
                <p class="heading">{ "🌐 云区对接管理说明" }</p>
                <ul>
                    <li>{ "云区对接配置用于管理已对接的云平台账户和区域信息" }</li>
                    <li>{ "业务申请时只能选择已配置且启用的云区" }</li>
                    <li>{ "创建配置后请先进行「测试连接」，确认凭证正确后再启用" }</li>
                    <li>{ "停用配置不会删除数据，业务申请时将不会显示该配置" }</li>
                </ul>
            </div>
        </div>
    }
}

// ============== Main App Component ==============

#[function_component]
pub fn App() -> Html {
    let current_page = use_state(|| {
        // Check if user is logged in
        if !get_auth_token().is_empty() {
            Page::Dashboard
        } else {
            Page::Login
        }
    });

    let page = *current_page;

    html! {
        <div>
            { match page {
                Page::Login => html! {
                    <Login current_page={current_page.clone()} />
                },
                _ => html! {
                    <div class="columns is-gapless">
                        <div class="column is-2">
                            <Sidebar current_page={current_page.clone()} />
                        </div>
                        <div class="column">
                            { match page {
                                Page::Dashboard => html! { <Dashboard /> },
                                Page::TaskCenter => html! { <TaskCenter /> },
                                Page::AdvancedScanning => html! { <AdvancedScanning /> },
                                Page::BusinessAcceptance => html! { <BusinessApplication /> }, // 兼容旧路由，跳转到业务申请
                                Page::BusinessApplication => html! { <BusinessApplication /> },
                                Page::OperationsManagement => html! { <OperationsManagement /> },
                                Page::AutomationOrchestration => html! { <AutomationOrchestration /> },
                                Page::RiskCenter => html! { <RiskCenter /> },
                                Page::UserManagement => html! { <UserManagement /> },
                                Page::PermissionManagement => html! { <PermissionManagement /> },
                                Page::AuditLogs => html! { <AuditLogs /> },
                                Page::CloudManagement => html! { <CloudManagement /> },
                                Page::CloudProviderManagement => html! { <CloudProviderManagement /> },
                                Page::CloudServiceAssetManagement => html! { <CloudServiceAssetManagement /> },
                                Page::UserProfile => html! { <UserProfile current_page={current_page.clone()} /> },
                                Page::PasswordPolicyManagement => html! { <PasswordPolicyManagement /> },
                                Page::Login => html! { <Login current_page={current_page.clone()} /> },
                            }}
                        </div>
                    </div>
                }
            }}
        </div>
    }
}

// ============== Advanced Scanning Component ==============

#[function_component]
fn AdvancedScanning() -> Html {
    let lang = use_state(|| Language::Zh);
    let tasks = use_state(|| Vec::new());
    let loading = use_state(|| true);
    let show_create_modal = use_state(|| false);

    // 扫描配置状态
    let scan_name = use_state(|| "".to_string());
    let scan_targets = use_state(|| "".to_string());
    let scan_strategy = use_state(|| ScanStrategy::Standard);
    let scan_engine = use_state(|| ScanEngine::Hybrid);

    let token = get_auth_token();

    // 获取用户权限
    let user_permissions = get_user_permissions();

    // 加载任务列表
    let load_tasks = {
        let tasks = tasks.clone();
        let token = token.clone();

        Callback::from(move |_| {
            let tasks = tasks.clone();
            let token = token.clone();

            spawn_local(async move {
                if let Ok(resp) = Request::get(&api_url("scan/advanced/tasks"))
                    .header("Authorization", &token)
                    .send()
                    .await
                {
                    if let Ok(data) = resp.json::<Vec<AdvancedScanTask>>().await {
                        tasks.set(data);
                    }
                }
            });
        })
    };

    // 初始加载任务列表
    {
        let loading = loading.clone();
        let load_tasks = load_tasks.clone();

        use_effect_with((), move |_| {
            load_tasks.emit(());
            loading.set(false);
            || ()
        });
    }

    // 自动刷新运行中的任务
    {
        let tasks = tasks.clone();
        let load_tasks = load_tasks.clone();

        use_effect_with((), move |_| {
            let has_running = tasks.iter().any(|t| matches!(t.status, TaskStatus::Running));

            if has_running {
                let load_tasks = load_tasks.clone();
                Timeout::new(2000, move || {
                    load_tasks.emit(());
                }).forget();
            }

            || ()
        });
    }

    // 创建扫描任务
    let on_create_scan = {
        let scan_name = scan_name.clone();
        let scan_targets = scan_targets.clone();
        let scan_strategy = scan_strategy.clone();
        let scan_engine = scan_engine.clone();
        let show_create_modal = show_create_modal.clone();
        let token = token.clone();
        let load_tasks = load_tasks.clone();

        Callback::from(move |_| {
            let name = (*scan_name).clone();
            let targets: Vec<String> = (*scan_targets).clone()
                .split('\n')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            let strategy = (*scan_strategy).clone();
            let engine = (*scan_engine).clone();
            let token = token.clone();
            let load_tasks = load_tasks.clone();

            spawn_local(async move {
                let request = CreateAdvancedScanRequest {
                    name: name.clone(),
                    targets: targets.clone(),
                    strategy,
                    engine,
                    concurrency: Some(1000),
                    timeout_ms: Some(5000),
                    service_detection: Some(true),
                    os_detection: Some(false),
                    web_fingerprint: Some(true),
                    cloud_tag_sync: Some(true),
                };

                let json = serde_json::to_string(&request).unwrap();
                let http_req = Request::post(&api_url("scan/advanced"))
                    .header("Authorization", &token)
                    .header("Content-Type", "application/json")
                    .body(json)
                    .unwrap();
                let resp = http_req.send().await;

                match resp {
                    Ok(response) if response.ok() => {
                        // 刷新任务列表
                        load_tasks.emit(());
                    }
                    Ok(response) => {
                        let status = response.status();
                        gloo_console::error!("Create scan failed with status:", status as i32);
                    }
                    Err(e) => {
                        gloo_console::error!("Network Error:", e.to_string());
                    }
                }
            });

            show_create_modal.set(false);
        })
    };

    // Modal open/close callbacks
    let on_open_modal = {
        let show_create_modal = show_create_modal.clone();
        Callback::from(move |_| show_create_modal.set(true))
    };

    let on_close_modal = {
        let show_create_modal = show_create_modal.clone();
        Callback::from(move |_| show_create_modal.set(false))
    };

    // Input callbacks
    let on_scan_name_input = {
        let scan_name = scan_name.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(input) = e.target_dyn_into::<HtmlInputElement>() {
                scan_name.set(input.value());
            }
        })
    };

    let on_scan_targets_input = {
        let scan_targets = scan_targets.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(input) = e.target_dyn_into::<HtmlTextAreaElement>() {
                scan_targets.set(input.value());
            }
        })
    };

    let status_class = |status: &TaskStatus| -> &'static str {
        match status {
            TaskStatus::Pending => "is-warning",
            TaskStatus::Running => "is-info",
            TaskStatus::Completed => "is-success",
            TaskStatus::Failed => "is-danger",
        }
    };

    let on_strategy_change = {
        let scan_strategy = scan_strategy.clone();
        Callback::from(move |e: Event| {
            if let Some(select) = e.target_dyn_into::<HtmlSelectElement>() {
                let value = select.value();
                let strategy = match value.as_str() {
                    "Quick" => ScanStrategy::Quick,
                    "Standard" => ScanStrategy::Standard,
                    "Full" => ScanStrategy::Full,
                    "Cloud" => ScanStrategy::Cloud,
                    _ => ScanStrategy::Standard,
                };
                scan_strategy.set(strategy);
            }
        })
    };

    let on_engine_change = {
        let scan_engine = scan_engine.clone();
        Callback::from(move |e: Event| {
            if let Some(select) = e.target_dyn_into::<HtmlSelectElement>() {
                let value = select.value();
                let engine = match value.as_str() {
                    "BasicTcp" => ScanEngine::BasicTcp,
                    "RustScan" => ScanEngine::RustScan,
                    "Nmap" => ScanEngine::Nmap,
                    "Hybrid" => ScanEngine::Hybrid,
                    _ => ScanEngine::Hybrid,
                };
                scan_engine.set(engine);
            }
        })
    };

    // 导出扫描结果
    let on_export_results = {
        let token = token.clone();

        Callback::from(move |task_id: String| {
            let token = token.clone();
            let task_id_clone = task_id.clone();

            spawn_local(async move {
                if let Ok(resp) = Request::get(&format!(
                    "{}/{}/export",
                    api_url("scan/advanced/tasks"),
                    task_id_clone
                ))
                .header("Authorization", &token)
                .send()
                .await
                {
                    if let Ok(results) = resp.json::<Vec<shared::QuickScanResult>>().await {
                        // 转换为 CSV 格式
                        let mut csv = String::from("IP,Status,Open Ports,Services\n");
                        for result in &results {
                            let status = if result.is_alive { "Alive" } else { "Down" };
                            let ports: Vec<String> = result.open_ports.iter()
                                .map(|p| format!("{}", p.port))
                                .collect();
                            let services: Vec<String> = result.open_ports.iter()
                                .filter_map(|p| p.service.as_ref())
                                .cloned()
                                .collect();

                            csv.push_str(&format!(
                                "{},{},{},{}\n",
                                result.ip,
                                status,
                                ports.join(";"),
                                services.join(";")
                            ));
                        }

                        // 下载 CSV 文件
                        if let Some(window) = web_sys::window() {
                            let blob_array = Array::new();
                            let bytes = js_sys::Uint8Array::from(csv.as_bytes());
                            blob_array.push(&bytes);

                            let js_blob = web_sys::Blob::new_with_u8_array_sequence_and_options(
                                &blob_array.into(),
                                web_sys::BlobPropertyBag::new().type_("text/csv")
                            ).unwrap();

                            let url = Url::create_object_url_with_blob(&js_blob).unwrap();

                            if let Some(document) = window.document() {
                                let a = document.create_element("a").unwrap();
                                a.set_attribute("href", &url).unwrap();
                                a.set_attribute("download", "scan_results.csv").unwrap();
                                let event = web_sys::Event::new("click").unwrap();
                                a.dispatch_event(&event).unwrap();
                            }
                        }
                    }
                }
            });
        })
    };

    html! {
        <div class="container p-4">
            <div class="level">
                <div class="level-left">
                    <h1 class="title">{ lang.t("advanced_scanning") }</h1>
                </div>
                <div class="level-right">
                    // 根据权限显示创建扫描按钮
                    if user_permissions.as_ref().map(|p| p.can_create_scan).unwrap_or(false) {
                        <button class="button is-primary" onclick={on_open_modal}>
                            { lang.t("create_scan") }
                        </button>
                    }
                </div>
            </div>

            // 创建扫描任务对话框
            if *show_create_modal {
                <div class="modal is-active">
                    <div class="modal-background" onclick={on_close_modal.clone()}></div>
                    <div class="modal-card">
                        <header class="modal-card-head">
                            <p class="modal-card-title">{ lang.t("create_scan") }</p>
                            <button class="delete" aria-label="close" onclick={on_close_modal.clone()}></button>
                        </header>
                        <section class="modal-card-body">
                            <div class="field">
                                <label class="label">{ lang.t("scan_name") }</label>
                                <div class="control">
                                    <input
                                        class="input"
                                        type="text"
                                        value={(*scan_name).clone()}
                                        oninput={on_scan_name_input}
                                    />
                                </div>
                            </div>

                            <div class="field">
                                <label class="label">{ lang.t("targets") }</label>
                                <div class="control">
                                    <textarea
                                        class="textarea"
                                        rows="5"
                                        placeholder="192.168.1.1&#10;192.168.1.100-192.168.1.200&#10;10.0.0.0/24"
                                        value={(*scan_targets).clone()}
                                        oninput={on_scan_targets_input}
                                    ></textarea>
                                    <p class="help">{"支持单个 IP、IP 范围（192.168.1.1-192.168.1.100）或 CIDR（192.168.1.0/24），每行一个"}</p>
                                </div>
                            </div>

                            <div class="field">
                                <label class="label">{ lang.t("strategy") }</label>
                                <div class="control">
                                    <div class="select is-fullwidth">
                                        <select
                                            onchange={on_strategy_change}
                                        >
                                            <option value="Quick" selected={*scan_strategy == ScanStrategy::Quick}>{"快速扫描 (TOP 100)"}</option>
                                            <option value="Standard" selected={*scan_strategy == ScanStrategy::Standard}>{"标准扫描 (TOP 1000)"}</option>
                                            <option value="Full" selected={*scan_strategy == ScanStrategy::Full}>{"全端口扫描 (1-65535)"}</option>
                                            <option value="Cloud" selected={*scan_strategy == ScanStrategy::Cloud}>{"云平台优化扫描"}</option>
                                        </select>
                                    </div>
                                </div>
                            </div>

                            <div class="field">
                                <label class="label">{ lang.t("engine") }</label>
                                <div class="control">
                                    <div class="select is-fullwidth">
                                        <select
                                            onchange={on_engine_change}
                                        >
                                            <option value="Hybrid" selected={*scan_engine == ScanEngine::Hybrid}>{"混合模式 (推荐)"}</option>
                                            <option value="RustScan" selected={*scan_engine == ScanEngine::RustScan}>{"RustScan (极速)"}</option>
                                            <option value="Nmap" selected={*scan_engine == ScanEngine::Nmap}>{"Nmap (深度)"}</option>
                                            <option value="BasicTcp" selected={*scan_engine == ScanEngine::BasicTcp}>{"基础 TCP (兼容)"}</option>
                                        </select>
                                    </div>
                                </div>
                            </div>

                            <div class="field">
                                <label class="checkbox">
                                    <input type="checkbox" checked={true} />
                                    {"服务指纹识别"}
                                </label>
                                <label class="checkbox">
                                    <input type="checkbox" />
                                    {"操作系统识别"}
                                </label>
                                <label class="checkbox">
                                    <input type="checkbox" checked={true} />
                                    {"云平台标签同步"}
                                </label>
                            </div>
                        </section>
                        <footer class="modal-card-foot">
                            <button class="button is-success" onclick={on_create_scan}>
                                { lang.t("start_scan") }
                            </button>
                            <button class="button" onclick={on_close_modal.clone()}>{"取消"}</button>
                        </footer>
                    </div>
                </div>
            }

            // 扫描任务列表
            <div class="box mt-4">
                if (*tasks).is_empty() && *loading {
                    <p>{"Loading..." }</p>
                } else if (*tasks).is_empty() {
                    <div class="has-text-centered">
                        <p class="has-text-grey">{"暂无扫描任务"}</p>
                    </div>
                } else {
                    <table class="table is-fullwidth is-hoverable">
                        <thead>
                            <tr>
                                <th>{ lang.t("scan_name") }</th>
                                <th>{"目标数量"}</th>
                                <th>{"策略"}</th>
                                <th>{"引擎"}</th>
                                <th>{ lang.t("scan_progress") }</th>
                                <th>{"状态"}</th>
                                <th>{"操作"}</th>
                            </tr>
                        </thead>
                        <tbody>
                            { for tasks.iter().map(|task| {
                                let progress_percent = (task.progress * 100.0) as i32;
                                let status_str = format!("{:?}", task.status);
                                let task_id = task.id.clone();
                                html! {
                                    <tr>
                                        <td>{ &task.name }</td>
                                        <td>{ task.total_count }</td>
                                        <td>{ format!("{:?}", task.config.strategy) }</td>
                                        <td>{ format!("{:?}", task.config.engine) }</td>
                                        <td>
                                            <progress
                                                value={format!("{}", task.progress)}
                                                max="1"
                                                class={if task.progress >= 1.0 { "progress is-success" } else { "progress is-primary" }}
                                            >
                                                { format!("{}%", progress_percent) }
                                            </progress>
                                            <span class="ml-2">{ format!("{}%", progress_percent) }</span>
                                        </td>
                                        <td>
                                            <span class={format!("tag {}", status_class(&task.status))}>
                                                { status_str }
                                            </span>
                                        </td>
                                        <td>
                                            if task.status == TaskStatus::Running {
                                                <button class="button is-small is-warning is-light">{"取消"}</button>
                                            } else if task.status == TaskStatus::Completed {
                                                // 根据权限显示导出按钮
                                                if user_permissions.as_ref().map(|p| p.can_export_scan).unwrap_or(false) {
                                                    <button
                                                        class="button is-small is-info is-light"
                                                        onclick={on_export_results.reform(move |_| task_id.clone())}
                                                    >{"导出结果"}</button>
                                                }
                                            }
                                            // 根据权限显示删除按钮
                                            if user_permissions.as_ref().map(|p| p.can_delete_scan).unwrap_or(false) {
                                                <button class="button is-small is-danger is-light">{"删除"}</button>
                                            }
                                        </td>
                                    </tr>
                                }
                            })}
                        </tbody>
                    </table>
                }
            </div>
        </div>
    }
}



// ============== User Profile Component ==============
// ============== User Profile Component ==============

#[derive(Properties, PartialEq)]
struct UserProfileProps {
    current_page: UseStateHandle<Page>,
}

#[function_component]
fn UserProfile(UserProfileProps { current_page }: &UserProfileProps) -> Html {
    let lang = use_state(|| Language::Zh);
    let current_user = use_state(|| None);
    let token = get_auth_token();

    // 修改密码状态
    let show_change_password = use_state(|| false);
    let current_password = use_state(|| "".to_string());
    let new_password = use_state(|| "".to_string());
    let confirm_password = use_state(|| "".to_string());
    let message = use_state(|| None as Option<(String, String)>); // (type, message)

    // 加载当前用户信息
    {
        let current_user = current_user.clone();

        use_effect_with((), move |_| {
            spawn_local(async move {
                if let Some(auth_user) = get_auth_user() {
                    current_user.set(Some(auth_user));
                }
            });
            || ()
        });
    }

    let on_logout = {
        let current_page = current_page.clone();
        Callback::from(move |_| {
            if let Some(window) = web_sys::window() {
                let storage = window.local_storage().unwrap().unwrap();
                storage.delete("auth_token").unwrap();
                storage.delete("auth_user").unwrap();
            }
            current_page.set(Page::Login);
        })
    };

    let on_change_password = {
        let show_change_password = show_change_password.clone();
        let message = message.clone();
        Callback::from(move |_| {
            show_change_password.set(true);
            message.set(None);
        })
    };

    let on_close_modal = {
        let show_change_password = show_change_password.clone();
        let message = message.clone();
        Callback::from(move |_| {
            show_change_password.set(false);
            message.set(None);
        })
    };

    let on_submit_password = {
        let token = token.clone();
        let current_pwd = current_password.clone();
        let new_pwd = new_password.clone();
        let confirm_pwd = confirm_password.clone();
        let show_change_password = show_change_password.clone();
        let msg = message.clone();
        let lang = lang.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            // 验证密码
            if *new_pwd != *confirm_pwd {
                msg.set(Some(("is-danger".to_string(), lang.t("password_mismatch"))));
                return;
            }

            if new_pwd.len() < 6 {
                msg.set(Some(("is-danger".to_string(), "密码长度至少6位".to_string())));
                return;
            }

            let token = token.clone();
            let current_pwd = current_pwd.clone();
            let new_pwd = new_pwd.clone();
            let show_change_password = show_change_password.clone();
            let msg = msg.clone();
            let lang = lang.clone();

            spawn_local(async move {
                match Request::post(&api_url("users/change-password"))
                    .header("Authorization", &token)
                    .header("Content-Type", "application/json")
                    .body(
                        serde_json::to_string(&(serde_json::json!({
                            "current_password": current_pwd.as_str(),
                            "new_password": new_pwd.as_str(),
                        }))).unwrap_or_default()
                    )
                    .unwrap()
                    .send()
                    .await
                {
                    Ok(resp) if resp.ok() => {
                        msg.set(Some(("is-success".to_string(), lang.t("password_updated"))));
                        gloo_timers::callback::Timeout::new(1500, move || {
                            show_change_password.set(false);
                        }).forget();
                    }
                    Ok(resp) => {
                        if let Ok(text) = resp.text().await {
                            msg.set(Some(("is-danger".to_string(), text)));
                        }
                    }
                    Err(e) => {
                        msg.set(Some(("is-danger".to_string(), format!("请求失败: {:?}", e))));
                    }
                }
            });
        })
    };

    match current_user.as_ref() {
        None => html! {
            <div class="section">
                <div class="container">
                    <div class="has-text-centered">
                        <progress class="progress is-small is-info" max="100">{"30%"}</progress>
                        <p class="mt-4 has-text-grey">{"加载中..."}</p>
                    </div>
                </div>
            </div>
        },
        Some(user) => {
            let (role_display, role_color) = match &user.role {
                Role::SysAdmin => ("系统管理员", "is-primary"),
                Role::SecAdmin => ("安全管理员", "is-success"),
                Role::Auditor => ("审计员", "is-warning"),
                Role::Custom(name) => (name.as_str(), "is-info"),
            };

            let total_permissions = user.permissions.as_ref().map(|p| {
                let mut count = 0;
                if p.can_create_scan { count += 1; }
                if p.can_delete_scan { count += 1; }
                if p.can_export_scan { count += 1; }
                if p.can_view_cloud_assets { count += 1; }
                if p.can_create_cloud_asset { count += 1; }
                if p.can_update_cloud_asset { count += 1; }
                if p.can_delete_cloud_asset { count += 1; }
                if p.can_view_cloud_providers { count += 1; }
                if p.can_manage_cloud_providers { count += 1; }
                if p.can_view_cloud_management { count += 1; }
                if p.can_manage_cloud { count += 1; }
                if p.can_delete_cloud { count += 1; }
                if p.can_sync_cloud { count += 1; }
                if p.can_view_risks { count += 1; }
                if p.can_resolve_risk { count += 1; }
                if p.can_delete_risk { count += 1; }
                if p.can_view_users { count += 1; }
                if p.can_create_user { count += 1; }
                if p.can_update_user { count += 1; }
                if p.can_delete_user { count += 1; }
                if p.can_manage_permissions { count += 1; }
                if p.can_view_audit_logs { count += 1; }
                count
            }).unwrap_or(0);

            html! {
                <div class="section" style="background-color: #f5f7fa; min-height: 100vh;">
                    <div class="container">
                        // 页面标题
                        <div class="level mb-5">
                            <div class="level-left">
                                <h1 class="title is-3">
                                    <span class="icon mr-2"><i class="fas fa-user-circle"></i></span>
                                    { lang.t("user_profile") }
                                </h1>
                            </div>
                            <div class="level-right">
                                <button class="button is-danger is-outlined" onclick={on_logout}>
                                    <span class="icon"><i class="fas fa-sign-out-alt"></i></span>
                                    <span>{ lang.t("logout") }</span>
                                </button>
                            </div>
                        </div>

                        <div class="columns">
                            // 左侧：个人信息卡片
                            <div class="column is-4">
                                // 用户信息卡片
                                <div class="card mb-4">
                                    <div class="card-content">
                                        <div class="has-text-centered">
                                            <div class="image is-128x128 is-inline-block mb-3">
                                                <img class="is-rounded" 
                                                     src="https://bulma.io/images/placeholders/128x128.png" 
                                                     alt="User Avatar"
                                                     style="border: 4px solid #3273dc; box-shadow: 0 4px 6px rgba(0,0,0,0.1);" />
                                            </div>
                                            <h2 class="title is-4 mb-1">{ &user.username }</h2>
                                            <span class={classes!("tag", "is-medium", role_color)}>
                                                { role_display }
                                            </span>
                                        </div>

                                        <hr class="my-4" />

                                        <div class="content">
                                            <table class="table is-fullwidth is-borderless">
                                                <tbody>
                                                    <tr>
                                                        <td class="has-text-grey-light">{ "用户 ID" }</td>
                                                        <td class="has-text-right">
                                                            <code class="is-size-7">{ &user.id[..8] }{"..."}</code>
                                                        </td>
                                                    </tr>
                                                    <tr>
                                                        <td class="has-text-grey">{ "📅 " }{ lang.t("created_at") }</td>
                                                        <td class="has-text-right">
                                                            { format!("{}", user.created_at.format("%Y-%m-%d")) }
                                                        </td>
                                                    </tr>
                                                    <tr>
                                                        <td class="has-text-grey">{ "🔑 拥有权限" }</td>
                                                        <td class="has-text-right">
                                                            <span class="tag is-info">{ format!("{} 项", total_permissions) }</span>
                                                        </td>
                                                    </tr>
                                                </tbody>
                                            </table>
                                        </div>

                                        <div class="buttons is-centered">
                                            <button class="button is-info is-fullwidth" onclick={on_change_password}>
                                                <span class="icon"><i class="fas fa-key"></i></span>
                                                <span>{ lang.t("change_password") }</span>
                                            </button>
                                        </div>
                                    </div>
                                </div>

                                // 账户安全提示
                                <div class="card">
                                    <div class="card-content">
                                        <p class="title is-6 mb-2">
                                            <span class="icon has-text-info"><i class="fas fa-shield-alt"></i></span>
                                            { "安全提示" }
                                        </p>
                                        <div class="content is-small">
                                            <ul>
                                                <li>{"定期修改密码以保护账户安全"}</li>
                                                <li>{"不要与他人分享账户信息"}</li>
                                                <li>{"退出时记得点击退出登录"}</li>
                                            </ul>
                                        </div>
                                    </div>
                                </div>
                            </div>

                            // 右侧：权限信息
                            <div class="column is-8">
                                if let Some(permissions) = &user.permissions {
                                    <div class="columns is-multiline">
                                        // 扫描权限
                                        <div class="column is-6">
                                            <div class="card">
                                                <div class="card-header">
                                                    <p class="card-header-title">
                                                        <span class="icon mr-2"><i class="fas fa-search has-text-primary"></i></span>
                                                        {"扫描权限"}
                                                    </p>
                                                    <span class="card-header-icon">
                                                        <span class="tag is-light">{ 
                                                            format!("{}/3", 
                                                                [permissions.can_create_scan, permissions.can_delete_scan, permissions.can_export_scan]
                                                                .iter().filter(|&&x| x).count())
                                                            }</span>
                                                    </span>
                                                </div>
                                                <div class="card-content">
                                                    <div class="tags are-small">
                                                        { for [permissions.can_create_scan, permissions.can_delete_scan, permissions.can_export_scan]
                                                            .iter()
                                                            .zip(["创建扫描", "删除扫描", "导出结果"].iter())
                                                            .map(|(&enabled, name)| {
                                                                if enabled {
                                                                    html! { <span class="tag is-success is-light">{"✓ "}{ name }</span> }
                                                                } else {
                                                                    html! { <span class="tag is-danger is-light is-light">{"✗ "}{ name }</span> }
                                                                }
                                                            })
                                                        }
                                                    </div>
                                                </div>
                                            </div>
                                        </div>

                                        // 资产权限
                                        <div class="column is-6">
                                            <div class="card">
                                                <div class="card-header">
                                                    <p class="card-header-title">
                                                        <span class="icon mr-2"><i class="fas fa-box has-text-info"></i></span>
                                                        {"资产权限"}
                                                    </p>
                                                    <span class="card-header-icon">
                                                        <span class="tag is-light">{
                                                            format!("{}/4",
                                                                [permissions.can_view_cloud_assets, permissions.can_create_cloud_asset, permissions.can_update_cloud_asset, permissions.can_delete_cloud_asset]
                                                                .iter().filter(|&&x| x).count())
                                                            }</span>
                                                    </span>
                                                </div>
                                                <div class="card-content">
                                                    <div class="tags are-small">
                                                        { for [permissions.can_view_cloud_assets, permissions.can_create_cloud_asset, permissions.can_update_cloud_asset, permissions.can_delete_cloud_asset]
                                                            .iter()
                                                            .zip(["查看资产", "创建资产", "更新资产", "删除资产"].iter())
                                                            .map(|(&enabled, name)| {
                                                                if enabled {
                                                                    html! { <span class="tag is-success is-light">{"✓ "}{ name }</span> }
                                                                } else {
                                                                    html! { <span class="tag is-danger is-light">{"✗ "}{ name }</span> }
                                                                }
                                                            })
                                                        }
                                                    </div>
                                                </div>
                                            </div>
                                        </div>

                                        // 云资产权限
                                        <div class="column is-6">
                                            <div class="card">
                                                <div class="card-header">
                                                    <p class="card-header-title">
                                                        <span class="icon mr-2"><i class="fas fa-cloud has-text-link"></i></span>
                                                        {"云区对接权限"}
                                                    </p>
                                                    <span class="card-header-icon">
                                                        <span class="tag is-light">{
                                                            format!("{}/2",
                                                                [permissions.can_view_cloud_providers, permissions.can_manage_cloud_providers]
                                                                .iter().filter(|&&x| x).count())
                                                            }</span>
                                                    </span>
                                                </div>
                                                <div class="card-content">
                                                    <div class="tags are-small">
                                                        { for [permissions.can_view_cloud_providers, permissions.can_manage_cloud_providers]
                                                            .iter()
                                                            .zip(["查看云区对接", "管理云区对接"].iter())
                                                            .map(|(&enabled, name)| {
                                                                if enabled {
                                                                    html! { <span class="tag is-success is-light">{"✓ "}{ name }</span> }
                                                                } else {
                                                                    html! { <span class="tag is-danger is-light">{"✗ "}{ name }</span> }
                                                                }
                                                            })
                                                        }
                                                    </div>
                                                </div>
                                            </div>
                                        </div>

                                        // 混合云管理权限
                                        <div class="column is-6">
                                            <div class="card">
                                                <div class="card-header">
                                                    <p class="card-header-title">
                                                        <span class="icon mr-2"><i class="fas fa-cloud-upload-alt has-text-primary"></i></span>
                                                        {"混合云管理权限"}
                                                    </p>
                                                    <span class="card-header-icon">
                                                        <span class="tag is-light">{
                                                            format!("{}/4",
                                                                [permissions.can_view_cloud_management, permissions.can_manage_cloud, permissions.can_delete_cloud, permissions.can_sync_cloud]
                                                                .iter().filter(|&&x| x).count())
                                                            }</span>
                                                    </span>
                                                </div>
                                                <div class="card-content">
                                                    <div class="tags are-small">
                                                        { for [permissions.can_view_cloud_management, permissions.can_manage_cloud, permissions.can_delete_cloud, permissions.can_sync_cloud]
                                                            .iter()
                                                            .zip(["查看混合云", "管理云资产", "删除云资产", "同步云资产"].iter())
                                                            .map(|(&enabled, name)| {
                                                                if enabled {
                                                                    html! { <span class="tag is-success is-light">{"✓ "}{ name }</span> }
                                                                } else {
                                                                    html! { <span class="tag is-danger is-light">{"✗ "}{ name }</span> }
                                                                }
                                                            })
                                                        }
                                                    </div>
                                                </div>
                                            </div>
                                        </div>

                                        // 风险权限
                                        <div class="column is-6">
                                            <div class="card">
                                                <div class="card-header">
                                                    <p class="card-header-title">
                                                        <span class="icon mr-2"><i class="fas fa-exclamation-triangle has-text-warning"></i></span>
                                                        {"风险权限"}
                                                    </p>
                                                    <span class="card-header-icon">
                                                        <span class="tag is-light">{ 
                                                            format!("{}/3", 
                                                                [permissions.can_view_risks, permissions.can_resolve_risk, permissions.can_delete_risk]
                                                                .iter().filter(|&&x| x).count())
                                                            }</span>
                                                    </span>
                                                </div>
                                                <div class="card-content">
                                                    <div class="tags are-small">
                                                        { for [permissions.can_view_risks, permissions.can_resolve_risk, permissions.can_delete_risk]
                                                            .iter()
                                                            .zip(["查看风险", "处置风险", "删除风险"].iter())
                                                            .map(|(&enabled, name)| {
                                                                if enabled {
                                                                    html! { <span class="tag is-success is-light">{"✓ "}{ name }</span> }
                                                                } else {
                                                                    html! { <span class="tag is-danger is-light">{"✗ "}{ name }</span> }
                                                                }
                                                            })
                                                        }
                                                    </div>
                                                </div>
                                            </div>
                                        </div>

                                        // 用户管理权限
                                        <div class="column is-6">
                                            <div class="card">
                                                <div class="card-header">
                                                    <p class="card-header-title">
                                                        <span class="icon mr-2"><i class="fas fa-users-cog has-text-primary"></i></span>
                                                        {"用户管理权限"}
                                                    </p>
                                                    <span class="card-header-icon">
                                                        <span class="tag is-light">{ 
                                                            format!("{}/5", 
                                                                [permissions.can_view_users, permissions.can_create_user, permissions.can_update_user, permissions.can_delete_user, permissions.can_manage_permissions]
                                                                .iter().filter(|&&x| x).count())
                                                            }</span>
                                                    </span>
                                                </div>
                                                <div class="card-content">
                                                    <div class="tags are-small">
                                                        { for [permissions.can_view_users, permissions.can_create_user, permissions.can_update_user, permissions.can_delete_user, permissions.can_manage_permissions]
                                                            .iter()
                                                            .zip(["查看用户", "创建用户", "更新用户", "删除用户", "管理权限"].iter())
                                                            .map(|(&enabled, name)| {
                                                                if enabled {
                                                                    html! { <span class="tag is-success is-light">{"✓ "}{ name }</span> }
                                                                } else {
                                                                    html! { <span class="tag is-danger is-light">{"✗ "}{ name }</span> }
                                                                }
                                                            })
                                                        }
                                                    </div>
                                                </div>
                                            </div>
                                        </div>

                                        // 审计权限
                                        <div class="column is-6">
                                            <div class="card">
                                                <div class="card-header">
                                                    <p class="card-header-title">
                                                        <span class="icon mr-2"><i class="fas fa-clipboard-list has-text-grey"></i></span>
                                                        {"审计权限"}
                                                    </p>
                                                    <span class="card-header-icon">
                                                        <span class="tag is-light">{
                                                            format!("{}/1",
                                                                [permissions.can_view_audit_logs]
                                                                .iter().filter(|&&x| x).count())
                                                        }</span>
                                                    </span>
                                                </div>
                                                <div class="card-content">
                                                    <div class="tags are-small">
                                                        { for [permissions.can_view_audit_logs]
                                                            .iter()
                                                            .zip(["查看审计日志"].iter())
                                                            .map(|(&enabled, name)| {
                                                                if enabled {
                                                                    html! { <span class="tag is-success is-light">{"✓ "}{ name }</span> }
                                                                } else {
                                                                    html! { <span class="tag is-danger is-light">{"✗ "}{ name }</span> }
                                                                }
                                                            })
                                                        }
                                                    </div>
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                } else {
                                    <div class="notification is-warning is-light">
                                        <p class="has-text-centered">
                                            <span class="icon is-medium"><i class="fas fa-exclamation-circle"></i></span>
                                            <span class="ml-2">{"暂无权限信息配置"}</span>
                                        </p>
                                    </div>
                                }
                            </div>
                        </div>

                        // 修改密码对话框
                        if *show_change_password {
                            <div class="modal is-active">
                                <div class="modal-background" onclick={on_close_modal.clone()}></div>
                                <div class="modal-card" style="max-width: 450px;">
                                    <header class="modal-card-head">
                                        <p class="modal-card-title">
                                            <span class="icon mr-2"><i class="fas fa-key"></i></span>
                                            { lang.t("change_password") }
                                        </p>
                                        <button class="delete" aria-label="close" onclick={on_close_modal.clone()}></button>
                                    </header>
                                    <section class="modal-card-body">
                                        if let Some((msg_type, msg_text)) = message.as_ref() {
                                            <div class={classes!("notification", msg_type)}>
                                                <button class="delete" onclick={let message = message.clone(); move |_| message.set(None)}></button>
                                                { msg_text }
                                            </div>
                                        }

                                        <form onsubmit={on_submit_password}>
                                            <div class="field">
                                                <label class="label">{ lang.t("current_password") }</label>
                                                <div class="control has-icons-left">
                                                    <input
                                                        type="password"
                                                        class="input"
                                                        placeholder="请输入当前密码"
                                                        value={(*current_password).clone()}
                                                        onchange={let current_password = current_password.clone(); move |e: Event| {
                                                            let input = e.target_unchecked_into::<HtmlInputElement>();
                                                            current_password.set(input.value());
                                                        }}
                                                        required=true
                                                    />
                                                    <span class="icon is-small is-left">
                                                        <i class="fas fa-lock"></i>
                                                    </span>
                                                </div>
                                            </div>

                                            <div class="field">
                                                <label class="label">{ lang.t("new_password") }</label>
                                                <div class="control has-icons-left">
                                                    <input
                                                        type="password"
                                                        class="input"
                                                        placeholder="请输入新密码（至少6位）"
                                                        value={(*new_password).clone()}
                                                        onchange={let new_password = new_password.clone(); move |e: Event| {
                                                            let input = e.target_unchecked_into::<HtmlInputElement>();
                                                            new_password.set(input.value());
                                                        }}
                                                        required=true
                                                        minlength="6"
                                                    />
                                                    <span class="icon is-small is-left">
                                                        <i class="fas fa-key"></i>
                                                    </span>
                                                </div>
                                                <p class="help">{"密码长度至少 6 位字符"}</p>
                                            </div>

                                            <div class="field">
                                                <label class="label">{ lang.t("confirm_password") }</label>
                                                <div class="control has-icons-left">
                                                    <input
                                                        type="password"
                                                        class="input"
                                                        placeholder="请再次输入新密码"
                                                        value={(*confirm_password).clone()}
                                                        onchange={let confirm_password = confirm_password.clone(); move |e: Event| {
                                                            let input = e.target_unchecked_into::<HtmlInputElement>();
                                                            confirm_password.set(input.value());
                                                        }}
                                                        required=true
                                                        minlength="6"
                                                    />
                                                    <span class="icon is-small is-left">
                                                        <i class="fas fa-check-circle"></i>
                                                    </span>
                                                </div>
                                            </div>

                                            <div class="field mt-5">
                                                <div class="control">
                                                    <button type="submit" class="button is-primary is-fullwidth">
                                                        <span class="icon"><i class="fas fa-save"></i></span>
                                                        <span>{ lang.t("save") }</span>
                                                    </button>
                                                </div>
                                            </div>
                                        </form>
                                    </section>
                                    <footer class="modal-card-foot">
                                        <button class="button" onclick={on_close_modal.clone()}>{ lang.t("cancel") }</button>
                                    </footer>
                                </div>
                            </div>
                        }
                    </div>
                </div>
            }
        }
    }
}

// ============== Permission Management Component ==============

#[function_component]
fn PermissionManagement() -> Html {
    let lang = use_state(|| Language::Zh);
    let users = use_state(|| Vec::new());
    let loading = use_state(|| true);
    let show_edit_modal = use_state(|| false);
    let editing_user = use_state(|| None as Option<(String, String, Permissions)>);
    let temp_permissions = use_state(|| None as Option<Permissions>);
    let success_message = use_state(|| None as Option<String>);

    let token = get_auth_token();
    let user_role = get_user_role();

    // Fetch users
    use_effect_with((), {
        let users = users.clone();
        let loading = loading.clone();
        let token = token.clone();

        move |_| {
            spawn_local(async move {
                if let Ok(resp) = Request::get(&api_url("users"))
                    .header("Authorization", &token)
                    .send()
                    .await
                {
                    if let Ok(data) = resp.json::<Vec<User>>().await {
                        users.set(data);
                    }
                }
                loading.set(false);
            });
            || ()
        }
    });

    // Open edit modal
    let on_edit = {
        let users = users.clone();
        let show_edit_modal = show_edit_modal.clone();
        let editing_user = editing_user.clone();
        let temp_permissions = temp_permissions.clone();

        Callback::from(move |user_id: String| {
            if let Some(user) = users.iter().find(|u| u.id == user_id) {
                if let Some(perms) = &user.permissions {
                    editing_user.set(Some((user.id.clone(), user.username.clone(), perms.clone())));
                    temp_permissions.set(Some(perms.clone()));
                    show_edit_modal.set(true);
                }
            }
        })
    };

    // Close modal
    let on_close_modal = {
        let show_edit_modal = show_edit_modal.clone();
        Callback::from(move |_| {
            show_edit_modal.set(false);
        })
    };

    // Save permissions
    let on_save = {
        let token = token.clone();
        let editing_user = editing_user.clone();
        let temp_permissions = temp_permissions.clone();
        let show_edit_modal = show_edit_modal.clone();
        let success_message = success_message.clone();

        Callback::from(move |_| {
            if let Some((user_id, _, _)) = &*editing_user {
                if let Some(perms) = &*temp_permissions {
                    let token = token.clone();
                    let user_id = user_id.clone();
                    let perms = perms.clone();
                    let show_edit_modal = show_edit_modal.clone();
                    let success_message = success_message.clone();

                    spawn_local(async move {
                        if let Ok(resp) = Request::put(&format!("{}/users/{}/permissions", api_url(""), user_id))
                            .header("Authorization", &token)
                            .header("Content-Type", "application/json")
                            .body(serde_json::to_string(&perms).unwrap_or_default())
                            .unwrap()
                            .send()
                            .await
                        {
                            if resp.ok() {
                                success_message.set(Some("权限更新成功".to_string()));
                                show_edit_modal.set(false);
                            } else {
                                success_message.set(Some("权限更新失败".to_string()));
                            }
                        }
                    });
                }
            }
        })
    };

    // Toggle permission
    let on_toggle_permission = {
        let temp_permissions = temp_permissions.clone();
        Callback::from(move |(perm_key, value): (String, bool)| {
            if let Some(mut perms) = (*temp_permissions).clone() {
                match perm_key.as_str() {
                    // 顶级权限
                    "can_access_general" => perms.can_access_general = value,
                    "can_view_business_process" => perms.can_view_business_process = value,
                    "can_access_cloud" => perms.can_access_cloud = value,
                    "can_access_user_management" => perms.can_access_user_management = value,
                    "can_access_audit" => perms.can_access_audit = value,
                    // 子级权限 - 通用模块
                    "can_view_dashboard" => perms.can_view_dashboard = value,
                    "can_view_tasks" => perms.can_view_tasks = value,
                    "can_view_advanced_scan" => perms.can_view_advanced_scan = value,
                    "can_view_risks" => perms.can_view_risks = value,
                    // 子级权限 - 业务流程
                    "can_view_business_applications" => perms.can_view_business_applications = value,
                    "can_create_business_application" => perms.can_create_business_application = value,
                    "can_approve_business_application" => perms.can_approve_business_application = value,
                    "can_supplement_business_application" => perms.can_supplement_business_application = value,
                    "can_delete_business_application" => perms.can_delete_business_application = value,
                    "can_view_operations_management" => perms.can_view_operations_management = value,
                    "can_manage_operations" => perms.can_manage_operations = value,
                    "can_view_automation_orchestration" => perms.can_view_automation_orchestration = value,
                    "can_execute_orchestration" => perms.can_execute_orchestration = value,
                    "can_manage_orchestration" => perms.can_manage_orchestration = value,
                    // 子级权限 - 云管理
                    "can_view_cloud_providers" => perms.can_view_cloud_providers = value,
                    "can_view_cloud_management" => perms.can_view_cloud_management = value,
                    "can_view_cloud_assets" => perms.can_view_cloud_assets = value,
                    // 子级权限 - 用户管理
                    "can_view_users" => perms.can_view_users = value,
                    "can_view_password_policy" => perms.can_view_password_policy = value,
                    "can_manage_permissions" => perms.can_manage_permissions = value,
                    // 子级权限 - 审计日志
                    "can_view_audit_logs" => perms.can_view_audit_logs = value,
                    _ => {}
                }
                temp_permissions.set(Some(perms));
            }
        })
    };

    html! {
        <div class="container p-4">
            <div class="is-flex is-justify-content-space-between is-align-items-center mb-4">
                <h1 class="title">{ lang.t("permission_management") }</h1>
            </div>

            if let Some(msg) = &*success_message {
                <div class="notification is-success is-light" style="margin-bottom: 1rem; position: relative;">
                    <button class="delete" onclick={
                        let success_message = success_message.clone();
                        Callback::from(move |_| success_message.set(None))
                    }></button>
                    <p>{ msg.clone() }</p>
                </div>
            }

            <div class="box">
                if *loading {
                    <div class="has-text-centered">
                        <span class="icon is-large">
                            <i class="fas fa-spinner fa-spin fa-2x"></i>
                        </span>
                        <p>{"加载中..."}</p>
                    </div>
                } else {
                    <table class="table is-fullwidth is-hoverable">
                        <thead>
                            <tr>
                                <th>{"用户名"}</th>
                                <th>{"角色"}</th>
                                <th>{"通用"}</th>
                                <th>{"业务流程"}</th>
                                <th>{"云管理"}</th>
                                <th>{"用户管理"}</th>
                                <th>{"审计日志"}</th>
                                <th>{"操作"}</th>
                            </tr>
                        </thead>
                        <tbody>
                            { for users.iter().map(|user| {
                                let role_display = match &user.role {
                                    Role::SysAdmin => "SysAdmin",
                                    Role::SecAdmin => "SecAdmin",
                                    Role::Auditor => "Auditor",
                                    Role::Custom(name) => name,
                                };
                                let perms = user.permissions.as_ref();
                                let on_edit = on_edit.clone();
                                let user_id = user.id.clone();

                                html! {
                                    <tr key={user_id.clone()}>
                                        <td>{ &user.username }</td>
                                        <td><span class="tag is-info">{ role_display }</span></td>
                                        <td>
                                            { perms.map(|p| {
                                                if p.can_access_general || p.can_view_tasks || p.can_view_advanced_scan || p.can_view_risks {
                                                    html! { <span class="icon has-text-success"><i class="fas fa-check"></i></span> }
                                                } else {
                                                    html! { <span class="icon has-text-danger"><i class="fas fa-times"></i></span> }
                                                }
                                            }).unwrap_or_else(|| html! { <span class="icon"><i class="fas fa-minus"></i></span> }) }
                                        </td>
                                        <td>
                                            { perms.map(|p| {
                                                if p.can_view_business_process {
                                                    html! { <span class="icon has-text-success"><i class="fas fa-check"></i></span> }
                                                } else {
                                                    html! { <span class="icon has-text-danger"><i class="fas fa-times"></i></span> }
                                                }
                                            }).unwrap_or_else(|| html! { <span class="icon"><i class="fas fa-minus"></i></span> }) }
                                        </td>
                                        <td>
                                            { perms.map(|p| {
                                                if p.can_access_cloud {
                                                    html! { <span class="icon has-text-success"><i class="fas fa-check"></i></span> }
                                                } else {
                                                    html! { <span class="icon has-text-danger"><i class="fas fa-times"></i></span> }
                                                }
                                            }).unwrap_or_else(|| html! { <span class="icon"><i class="fas fa-minus"></i></span> }) }
                                        </td>
                                        <td>
                                            { perms.map(|p| {
                                                if p.can_access_user_management {
                                                    html! { <span class="icon has-text-success"><i class="fas fa-check"></i></span> }
                                                } else {
                                                    html! { <span class="icon has-text-danger"><i class="fas fa-times"></i></span> }
                                                }
                                            }).unwrap_or_else(|| html! { <span class="icon"><i class="fas fa-minus"></i></span> }) }
                                        </td>
                                        <td>
                                            { perms.map(|p| {
                                                if p.can_access_audit {
                                                    html! { <span class="icon has-text-success"><i class="fas fa-check"></i></span> }
                                                } else {
                                                    html! { <span class="icon has-text-danger"><i class="fas fa-times"></i></span> }
                                                }
                                            }).unwrap_or_else(|| html! { <span class="icon"><i class="fas fa-minus"></i></span> }) }
                                        </td>
                                        <td>
                                            if user.role != Role::SysAdmin {
                                                <button class="button is-small is-info" onclick={
                                                    let on_edit = on_edit.clone();
                                                    move |_| on_edit.emit(user_id.clone())
                                                }>
                                                    <span class="icon"><i class="fas fa-edit"></i></span>
                                                    <span>{"编辑"}</span>
                                                </button>
                                            }
                                        </td>
                                    </tr>
                                }
                            })}
                        </tbody>
                    </table>
                }
            </div>

            // Edit Modal
            if *show_edit_modal {
                if let Some((user_id, username, perms)) = &*editing_user {
                    <div class="modal is-active">
                        <div class="modal-background" onclick={on_close_modal.clone()}></div>
                        <div class="modal-card" style="width: 900px;">
                            <header class="modal-card-head">
                                <p class="modal-card-title">{ format!("编辑用户权限: {}", username) }</p>
                                <button class="delete" onclick={on_close_modal.clone()}></button>
                            </header>
                            <section class="modal-card-body" style="max-height: 70vh; overflow-y: auto;">
                                if let Some(current_perms) = &*temp_permissions {
                                    <div class="content">
                                        // ========== 通用模块 ==========
                                        <div class="box has-background-light">
                                            <h4 class="title is-6 mb-3">{"📋 通用模块"}</h4>
                                            <div class="field" style="margin-bottom: 0.5rem;">
                                                <label class="checkbox">
                                                    <input type="checkbox"
                                                        checked={current_perms.can_access_general}
                                                        onchange={let on_toggle = on_toggle_permission.clone(); move |e: Event| {
                                                            let input = e.target_unchecked_into::<HtmlInputElement>();
                                                            on_toggle.emit(("can_access_general".to_string(), input.checked()))
                                                        }}
                                                    />
                                                    <strong>{ " 访问通用模块" }</strong>
                                                </label>
                                                <p class="help is-size-7 ml-5">{"控制是否可以访问通用模块下的所有功能"}</p>
                                            </div>
                                            <div class="ml-5">
                                                <div class="field" style="margin-bottom: 0.5rem;">
                                                    <label class="checkbox">
                                                        <input type="checkbox"
                                                            checked={current_perms.can_view_dashboard}
                                                            onchange={let on_toggle = on_toggle_permission.clone(); move |e: Event| {
                                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                                on_toggle.emit(("can_view_dashboard".to_string(), input.checked()))
                                                            }}
                                                        />
                                                        { " 仪表盘查看" }
                                                    </label>
                                                </div>
                                                <div class="field" style="margin-bottom: 0.5rem;">
                                                    <label class="checkbox">
                                                        <input type="checkbox"
                                                            checked={current_perms.can_view_tasks}
                                                            onchange={let on_toggle = on_toggle_permission.clone(); move |e: Event| {
                                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                                on_toggle.emit(("can_view_tasks".to_string(), input.checked()))
                                                            }}
                                                        />
                                                        { " 任务中心查看" }
                                                    </label>
                                                </div>
                                                <div class="field" style="margin-bottom: 0.5rem;">
                                                    <label class="checkbox">
                                                        <input type="checkbox"
                                                            checked={current_perms.can_view_advanced_scan}
                                                            onchange={let on_toggle = on_toggle_permission.clone(); move |e: Event| {
                                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                                on_toggle.emit(("can_view_advanced_scan".to_string(), input.checked()))
                                                            }}
                                                        />
                                                        { " 高级扫描查看" }
                                                    </label>
                                                </div>
                                                <div class="field" style="margin-bottom: 0.5rem;">
                                                    <label class="checkbox">
                                                        <input type="checkbox"
                                                            checked={current_perms.can_view_risks}
                                                            onchange={let on_toggle = on_toggle_permission.clone(); move |e: Event| {
                                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                                on_toggle.emit(("can_view_risks".to_string(), input.checked()))
                                                            }}
                                                        />
                                                        { " 风险监控查看" }
                                                    </label>
                                                </div>
                                            </div>
                                        </div>

                                        // ========== 业务流程 ==========
                                        <div class="box has-background-light">
                                            <h4 class="title is-6 mb-3">{"🔄 业务流程"}</h4>
                                            <div class="field" style="margin-bottom: 0.5rem;">
                                                <label class="checkbox">
                                                    <input type="checkbox"
                                                        checked={current_perms.can_view_business_process}
                                                        onchange={let on_toggle = on_toggle_permission.clone(); move |e: Event| {
                                                            let input = e.target_unchecked_into::<HtmlInputElement>();
                                                            on_toggle.emit(("can_view_business_process".to_string(), input.checked()))
                                                        }}
                                                    />
                                                    <strong>{ " 访问业务流程" }</strong>
                                                </label>
                                                <p class="help is-size-7 ml-5">{"控制是否可以访问业务流程管理功能"}</p>
                                            </div>
                                            <div class="ml-5">
                                                <div class="field" style="margin-bottom: 0.5rem;">
                                                    <label class="checkbox">
                                                        <input type="checkbox"
                                                            checked={current_perms.can_view_business_applications}
                                                            onchange={let on_toggle = on_toggle_permission.clone(); move |e: Event| {
                                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                                on_toggle.emit(("can_view_business_applications".to_string(), input.checked()))
                                                            }}
                                                        />
                                                        { " 📝 业务申请查看" }
                                                    </label>
                                                </div>
                                                <div class="field" style="margin-bottom: 0.5rem;">
                                                    <label class="checkbox">
                                                        <input type="checkbox"
                                                            checked={current_perms.can_create_business_application}
                                                            onchange={let on_toggle = on_toggle_permission.clone(); move |e: Event| {
                                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                                on_toggle.emit(("can_create_business_application".to_string(), input.checked()))
                                                            }}
                                                        />
                                                        { " 创建业务申请" }
                                                    </label>
                                                </div>
                                                <div class="field" style="margin-bottom: 0.5rem;">
                                                    <label class="checkbox">
                                                        <input type="checkbox"
                                                            checked={current_perms.can_approve_business_application}
                                                            onchange={let on_toggle = on_toggle_permission.clone(); move |e: Event| {
                                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                                on_toggle.emit(("can_approve_business_application".to_string(), input.checked()))
                                                            }}
                                                        />
                                                        { " 审批业务申请" }
                                                    </label>
                                                </div>
                                                <div class="field" style="margin-bottom: 0.5rem;">
                                                    <label class="checkbox">
                                                        <input type="checkbox"
                                                            checked={current_perms.can_supplement_business_application}
                                                            onchange={let on_toggle = on_toggle_permission.clone(); move |e: Event| {
                                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                                on_toggle.emit(("can_supplement_business_application".to_string(), input.checked()))
                                                            }}
                                                        />
                                                        { " 补充业务申请信息" }
                                                    </label>
                                                </div>
                                                <div class="field" style="margin-bottom: 0.5rem;">
                                                    <label class="checkbox">
                                                        <input type="checkbox"
                                                            checked={current_perms.can_delete_business_application}
                                                            onchange={let on_toggle = on_toggle_permission.clone(); move |e: Event| {
                                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                                on_toggle.emit(("can_delete_business_application".to_string(), input.checked()))
                                                            }}
                                                        />
                                                        { " 删除业务申请" }
                                                    </label>
                                                </div>
                                                <div class="field" style="margin-bottom: 0.5rem;">
                                                    <label class="checkbox">
                                                        <input type="checkbox"
                                                            checked={current_perms.can_view_operations_management}
                                                            onchange={let on_toggle = on_toggle_permission.clone(); move |e: Event| {
                                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                                on_toggle.emit(("can_view_operations_management".to_string(), input.checked()))
                                                            }}
                                                        />
                                                        { " 🔧 运维管理查看" }
                                                    </label>
                                                </div>
                                                <div class="field" style="margin-bottom: 0.5rem;">
                                                    <label class="checkbox">
                                                        <input type="checkbox"
                                                            checked={current_perms.can_manage_operations}
                                                            onchange={let on_toggle = on_toggle_permission.clone(); move |e: Event| {
                                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                                on_toggle.emit(("can_manage_operations".to_string(), input.checked()))
                                                            }}
                                                        />
                                                        { " 运维操作权限" }
                                                    </label>
                                                </div>
                                                <div class="field" style="margin-bottom: 0.5rem;">
                                                    <label class="checkbox">
                                                        <input type="checkbox"
                                                            checked={current_perms.can_view_automation_orchestration}
                                                            onchange={let on_toggle = on_toggle_permission.clone(); move |e: Event| {
                                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                                on_toggle.emit(("can_view_automation_orchestration".to_string(), input.checked()))
                                                            }}
                                                        />
                                                        { " ⚙️ 自动化资源编排查看" }
                                                    </label>
                                                </div>
                                                <div class="field" style="margin-bottom: 0.5rem;">
                                                    <label class="checkbox">
                                                        <input type="checkbox"
                                                            checked={current_perms.can_execute_orchestration}
                                                            onchange={let on_toggle = on_toggle_permission.clone(); move |e: Event| {
                                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                                on_toggle.emit(("can_execute_orchestration".to_string(), input.checked()))
                                                            }}
                                                        />
                                                        { " 执行编排任务" }
                                                    </label>
                                                </div>
                                                <div class="field" style="margin-bottom: 0.5rem;">
                                                    <label class="checkbox">
                                                        <input type="checkbox"
                                                            checked={current_perms.can_manage_orchestration}
                                                            onchange={let on_toggle = on_toggle_permission.clone(); move |e: Event| {
                                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                                on_toggle.emit(("can_manage_orchestration".to_string(), input.checked()))
                                                            }}
                                                        />
                                                        { " 管理编排任务" }
                                                    </label>
                                                </div>
                                            </div>
                                        </div>

                                        // ========== 云管理 ==========
                                        <div class="box has-background-light">
                                            <h4 class="title is-6 mb-3">{"☁️ 云管理"}</h4>
                                            <div class="field" style="margin-bottom: 0.5rem;">
                                                <label class="checkbox">
                                                    <input type="checkbox"
                                                        checked={current_perms.can_access_cloud}
                                                        onchange={let on_toggle = on_toggle_permission.clone(); move |e: Event| {
                                                            let input = e.target_unchecked_into::<HtmlInputElement>();
                                                            on_toggle.emit(("can_access_cloud".to_string(), input.checked()))
                                                        }}
                                                    />
                                                    <strong>{ " 访问云管理模块" }</strong>
                                                </label>
                                                <p class="help is-size-7 ml-5">{"控制是否可以访问云管理模块下的所有功能"}</p>
                                            </div>
                                            <div class="ml-5">
                                                <div class="field" style="margin-bottom: 0.5rem;">
                                                    <label class="checkbox">
                                                        <input type="checkbox"
                                                            checked={current_perms.can_view_cloud_providers}
                                                            onchange={let on_toggle = on_toggle_permission.clone(); move |e: Event| {
                                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                                on_toggle.emit(("can_view_cloud_providers".to_string(), input.checked()))
                                                            }}
                                                        />
                                                        { " 云服务商管理查看" }
                                                    </label>
                                                </div>
                                                <div class="field" style="margin-bottom: 0.5rem;">
                                                    <label class="checkbox">
                                                        <input type="checkbox"
                                                            checked={current_perms.can_view_cloud_management}
                                                            onchange={let on_toggle = on_toggle_permission.clone(); move |e: Event| {
                                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                                on_toggle.emit(("can_view_cloud_management".to_string(), input.checked()))
                                                            }}
                                                        />
                                                        { " 云管理查看" }
                                                    </label>
                                                </div>
                                                <div class="field" style="margin-bottom: 0.5rem;">
                                                    <label class="checkbox">
                                                        <input type="checkbox"
                                                            checked={current_perms.can_view_cloud_assets}
                                                            onchange={let on_toggle = on_toggle_permission.clone(); move |e: Event| {
                                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                                on_toggle.emit(("can_view_cloud_assets".to_string(), input.checked()))
                                                            }}
                                                        />
                                                        { " 云服务资产查看" }
                                                    </label>
                                                </div>
                                            </div>
                                        </div>

                                        // ========== 用户管理 ==========
                                        <div class="box has-background-light">
                                            <h4 class="title is-6 mb-3">{"👥 用户管理"}</h4>
                                            <div class="field" style="margin-bottom: 0.5rem;">
                                                <label class="checkbox">
                                                    <input type="checkbox"
                                                        checked={current_perms.can_access_user_management}
                                                        onchange={let on_toggle = on_toggle_permission.clone(); move |e: Event| {
                                                            let input = e.target_unchecked_into::<HtmlInputElement>();
                                                            on_toggle.emit(("can_access_user_management".to_string(), input.checked()))
                                                        }}
                                                    />
                                                    <strong>{ " 访问用户管理模块" }</strong>
                                                </label>
                                                <p class="help is-size-7 ml-5">{"控制是否可以访问用户管理模块下的所有功能"}</p>
                                            </div>
                                            <div class="ml-5">
                                                <div class="field" style="margin-bottom: 0.5rem;">
                                                    <label class="checkbox">
                                                        <input type="checkbox"
                                                            checked={current_perms.can_view_users}
                                                            onchange={let on_toggle = on_toggle_permission.clone(); move |e: Event| {
                                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                                on_toggle.emit(("can_view_users".to_string(), input.checked()))
                                                            }}
                                                        />
                                                        { " 用户管理查看" }
                                                    </label>
                                                </div>
                                                <div class="field" style="margin-bottom: 0.5rem;">
                                                    <label class="checkbox">
                                                        <input type="checkbox"
                                                            checked={current_perms.can_view_password_policy}
                                                            onchange={let on_toggle = on_toggle_permission.clone(); move |e: Event| {
                                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                                on_toggle.emit(("can_view_password_policy".to_string(), input.checked()))
                                                            }}
                                                        />
                                                        { " 密码策略查看" }
                                                    </label>
                                                </div>
                                                <div class="field" style="margin-bottom: 0.5rem;">
                                                    <label class="checkbox">
                                                        <input type="checkbox"
                                                            checked={current_perms.can_manage_permissions}
                                                            onchange={let on_toggle = on_toggle_permission.clone(); move |e: Event| {
                                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                                on_toggle.emit(("can_manage_permissions".to_string(), input.checked()))
                                                            }}
                                                        />
                                                        { " 权限管理（可编辑其他用户权限）" }
                                                    </label>
                                                </div>
                                            </div>
                                        </div>

                                        // ========== 审计日志 ==========
                                        <div class="box has-background-light">
                                            <h4 class="title is-6 mb-3">{"📜 审计日志"}</h4>
                                            <div class="field" style="margin-bottom: 0.5rem;">
                                                <label class="checkbox">
                                                    <input type="checkbox"
                                                        checked={current_perms.can_access_audit}
                                                        onchange={let on_toggle = on_toggle_permission.clone(); move |e: Event| {
                                                            let input = e.target_unchecked_into::<HtmlInputElement>();
                                                            on_toggle.emit(("can_access_audit".to_string(), input.checked()))
                                                        }}
                                                    />
                                                    <strong>{ " 访问审计日志" }</strong>
                                                </label>
                                                <p class="help is-size-7 ml-5">{"控制是否可以访问审计日志功能"}</p>
                                            </div>
                                            <div class="ml-5">
                                                <div class="field" style="margin-bottom: 0.5rem;">
                                                    <label class="checkbox">
                                                        <input type="checkbox"
                                                            checked={current_perms.can_view_audit_logs}
                                                            onchange={let on_toggle = on_toggle_permission.clone(); move |e: Event| {
                                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                                on_toggle.emit(("can_view_audit_logs".to_string(), input.checked()))
                                                            }}
                                                        />
                                                        { " 审计日志查看" }
                                                    </label>
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                }
                            </section>
                            <footer class="modal-card-foot">
                                <button class="button is-success" onclick={on_save}>{ "保存" }</button>
                                <button class="button" onclick={on_close_modal}>{ "取消" }</button>
                            </footer>
                        </div>
                    </div>
                }
            }
        </div>
    }
}

// ============== Password Policy Management Component ==============

#[function_component]
fn PasswordPolicyManagement() -> Html {
    let lang = use_state(|| Language::Zh);
    let policy = use_state(|| None as Option<PasswordPolicy>);
    let loading = use_state(|| true);
    let message = use_state(|| None as Option<(String, String)>); // (type, message)
    let token = get_auth_token();

    // Form state
    let min_length = use_state(|| 6u32);
    let require_uppercase = use_state(|| false);
    let require_lowercase = use_state(|| false);
    let require_number = use_state(|| false);
    let require_special = use_state(|| false);
    let max_age_days = use_state(|| Some(90u32));  // 默认90天
    let prevent_reuse = use_state(|| 3u32);
    let min_strength = use_state(|| "weak".to_string());
    let max_login_attempts = use_state(|| Some(5u32));
    let lockout_duration_minutes = use_state(|| 30u32);

    // Load current policy
    {
        let policy = policy.clone();
        let loading = loading.clone();
        let token = token.clone();
        let min_length = min_length.clone();
        let require_uppercase = require_uppercase.clone();
        let require_lowercase = require_lowercase.clone();
        let require_number = require_number.clone();
        let require_special = require_special.clone();
        let max_age_days = max_age_days.clone();
        let prevent_reuse = prevent_reuse.clone();
        let min_strength = min_strength.clone();
        let max_login_attempts = max_login_attempts.clone();
        let lockout_duration_minutes = lockout_duration_minutes.clone();
        let msg = message.clone();
        let lang_clone = lang.clone();

        use_effect_with((), move |_| {
            spawn_local(async move {
                match Request::get(&api_url("password-policy"))
                    .header("Authorization", &token)
                    .send()
                    .await
                {
                    Ok(response) => {
                        if response.ok() {
                            if let Ok(p) = response.json::<PasswordPolicy>().await {
                                min_length.set(p.min_length);
                                require_uppercase.set(p.require_uppercase);
                                require_lowercase.set(p.require_lowercase);
                                require_number.set(p.require_number);
                                require_special.set(p.require_special);
                                max_age_days.set(p.max_age_days);
                                prevent_reuse.set(p.prevent_reuse);
                                min_strength.set(p.min_strength.clone());
                                max_login_attempts.set(p.max_login_attempts);
                                lockout_duration_minutes.set(p.lockout_duration_minutes);
                                policy.set(Some(p));
                            }
                        } else {
                            msg.set(Some(("is-danger".to_string(), lang_clone.t("policy_error"))));
                        }
                    }
                    Err(_) => {
                        msg.set(Some(("is-danger".to_string(), lang_clone.t("policy_error"))));
                    }
                }
                loading.set(false);
            });
            || ()
        });
    }

    let on_save = {
        let token = token.clone();
        let policy_state = policy.clone();
        let min_length = min_length.clone();
        let require_uppercase = require_uppercase.clone();
        let require_lowercase = require_lowercase.clone();
        let require_number = require_number.clone();
        let require_special = require_special.clone();
        let max_age_days = max_age_days.clone();
        let prevent_reuse = prevent_reuse.clone();
        let min_strength = min_strength.clone();
        let max_login_attempts = max_login_attempts.clone();
        let lockout_duration_minutes = lockout_duration_minutes.clone();
        let msg = message.clone();
        let lang = lang.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            let new_policy = PasswordPolicy {
                min_length: *min_length,
                require_uppercase: *require_uppercase,
                require_lowercase: *require_lowercase,
                require_number: *require_number,
                require_special: *require_special,
                max_age_days: *max_age_days,
                prevent_reuse: *prevent_reuse,
                min_strength: (*min_strength).clone(),
                max_login_attempts: *max_login_attempts,
                lockout_duration_minutes: *lockout_duration_minutes,
            };

            let token = token.clone();
            let policy_state = policy_state.clone();
            let msg = msg.clone();
            let lang = lang.clone();

            spawn_local(async move {
                match Request::put(&api_url("password-policy"))
                    .header("Authorization", &token)
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_string(&new_policy).unwrap_or_default())
                    .unwrap()
                    .send()
                    .await
                {
                    Ok(response) => {
                        if response.ok() {
                            if let Ok(p) = response.json::<PasswordPolicy>().await {
                                policy_state.set(Some(p));
                                msg.set(Some(("is-success".to_string(), lang.t("policy_saved"))));
                            }
                        } else {
                            msg.set(Some(("is-danger".to_string(), lang.t("policy_error"))));
                        }
                    }
                    Err(_) => {
                        msg.set(Some(("is-danger".to_string(), lang.t("policy_error"))));
                    }
                }
            });
        })
    };

    let on_reset = {
        let policy = policy.clone();
        let min_length = min_length.clone();
        let require_uppercase = require_uppercase.clone();
        let require_lowercase = require_lowercase.clone();
        let require_number = require_number.clone();
        let require_special = require_special.clone();
        let max_age_days = max_age_days.clone();
        let prevent_reuse = prevent_reuse.clone();
        let min_strength = min_strength.clone();
        let max_login_attempts = max_login_attempts.clone();
        let lockout_duration_minutes = lockout_duration_minutes.clone();

        Callback::from(move |_| {
            if let Some(p) = (*policy).clone() {
                min_length.set(p.min_length);
                require_uppercase.set(p.require_uppercase);
                require_lowercase.set(p.require_lowercase);
                require_number.set(p.require_number);
                require_special.set(p.require_special);
                max_age_days.set(p.max_age_days);
                prevent_reuse.set(p.prevent_reuse);
                min_strength.set(p.min_strength);
                max_login_attempts.set(p.max_login_attempts);
                lockout_duration_minutes.set(p.lockout_duration_minutes);
            }
        })
    };

    html! {
        <div class="container p-4">
            <h1 class="title">{ lang.t("password_policy_management") }</h1>

            if *loading {
                <div class="notification is-info">
                    { "Loading..." }
                </div>
            } else {
                if let Some((msg_type, msg_text)) = message.as_ref() {
                    <div class={classes!("notification", msg_type)}>
                        <button class="delete" onclick={let message = message.clone(); move |_| message.set(None)}></button>
                        { msg_text }
                    </div>
                }

                <div class="box">
                    <form onsubmit={on_save}>
                        <div class="columns">
                            // Left column - basic settings
                            <div class="column is-6">
                                <div class="field">
                                    <label class="label">{ lang.t("min_length") }</label>
                                    <div class="control">
                                        <input
                                            type="number"
                                            class="input"
                                            min="1"
                                            max="128"
                                            value={(*min_length).to_string()}
                                            onchange={let min_length = min_length.clone(); move |e: Event| {
                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                if let Ok(val) = input.value().parse::<u32>() {
                                                    min_length.set(val);
                                                }
                                            }}
                                        />
                                    </div>
                                    <p class="help">{"密码的最小长度（1-128位）"}</p>
                                </div>

                                <div class="field">
                                    <label class="label">{ lang.t("max_age_days") }</label>
                                    <div class="control">
                                        <input
                                            type="number"
                                            class="input"
                                            min="0"
                                            placeholder="0表示不限制"
                                            value={max_age_days.map(|d| d.to_string()).unwrap_or_else(|| "0".to_string())}
                                            onchange={let max_age_days = max_age_days.clone(); move |e: Event| {
                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                let val = input.value();
                                                if let Ok(d) = val.parse::<u32>() {
                                                    // 0 表示不限制，设置为 None
                                                    if d == 0 {
                                                        max_age_days.set(None);
                                                    } else {
                                                        max_age_days.set(Some(d));
                                                    }
                                                }
                                            }}
                                        />
                                    </div>
                                    <p class="help">{"密码最大有效期（天），输入 0 表示不限制"}</p>
                                </div>

                                <div class="field">
                                    <label class="label">{ lang.t("prevent_reuse") }</label>
                                    <div class="control">
                                        <input
                                            type="number"
                                            class="input"
                                            min="0"
                                            max="50"
                                            value={(*prevent_reuse).to_string()}
                                            onchange={let prevent_reuse = prevent_reuse.clone(); move |e: Event| {
                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                if let Ok(val) = input.value().parse::<u32>() {
                                                    prevent_reuse.set(val);
                                                }
                                            }}
                                        />
                                    </div>
                                    <p class="help">{"防止用户重用最近 N 次的旧密码"}</p>
                                </div>

                                <div class="field">
                                    <label class="label">{ lang.t("min_strength") }</label>
                                    <div class="control has-icons-left">
                                        <div class="select is-fullwidth">
                                            <select
                                                value={(*min_strength).clone()}
                                                onchange={let min_strength = min_strength.clone(); move |e: Event| {
                                                    let select = e.target_unchecked_into::<HtmlSelectElement>();
                                                    min_strength.set(select.value());
                                                }}
                                            >
                                                <option value="weak">{ lang.t("weak") }</option>
                                                <option value="medium">{ lang.t("medium") }</option>
                                                <option value="strong">{ lang.t("strong") }</option>
                                            </select>
                                        </div>
                                        <span class="icon is-small is-left">
                                            <i class="fas fa-shield-alt"></i>
                                        </span>
                                    </div>
                                    <p class="help">{"密码最低强度要求"}</p>
                                    <div class="box has-background-info-light has-text-info mt-2 p-3">
                                        <p class="is-size-7 mb-1"><strong>{"密码强度说明："}</strong></p>
                                        <ul class="is-size-7 ml-4 mt-1">
                                            <li>{"🔴 弱：长度 < 8 位，或字符类型单一"}</li>
                                            <li>{"🟡 中：长度 ≥ 8 位，包含 2-3 种字符类型（大小写、数字、特殊字符）"}</li>
                                            <li>{"🟢 强：长度 ≥ 12 位，包含所有字符类型（大小写、数字、特殊字符）"}</li>
                                        </ul>
                                    </div>
                                </div>

                                <hr class="has-background-grey-light mt-4 mb-4" />

                                <h4 class="title is-6 has-text-primary">{ lang.t("account_lockout") }</h4>

                                <div class="field">
                                    <label class="label">{ lang.t("max_login_attempts") }</label>
                                    <div class="control has-icons-left">
                                        <input
                                            type="number"
                                            class="input"
                                            min="0"
                                            placeholder="留空表示不限制"
                                            value={max_login_attempts.map(|n| n.to_string()).unwrap_or_default()}
                                            onchange={let max_login_attempts = max_login_attempts.clone(); move |e: Event| {
                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                let val = input.value();
                                                if val.is_empty() {
                                                    max_login_attempts.set(None);
                                                } else if let Ok(n) = val.parse::<u32>() {
                                                    max_login_attempts.set(Some(n));
                                                }
                                            }}
                                        />
                                        <span class="icon is-small is-left">
                                            <i class="fas fa-user-lock"></i>
                                        </span>
                                    </div>
                                    <p class="help">{format!("登录失败多少次后锁定账户，0 或留空表示{}", lang.t("no_limit"))}</p>
                                </div>

                                <div class="field">
                                    <label class="label">{ lang.t("lockout_duration_minutes") }</label>
                                    <div class="control has-icons-left">
                                        <input
                                            type="number"
                                            class="input"
                                            min="1"
                                            max="1440"
                                            value={(*lockout_duration_minutes).to_string()}
                                            onchange={let lockout_duration_minutes = lockout_duration_minutes.clone(); move |e: Event| {
                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                if let Ok(val) = input.value().parse::<u32>() {
                                                    lockout_duration_minutes.set(val.max(1));
                                                }
                                            }}
                                        />
                                        <span class="icon is-small is-left">
                                            <i class="fas fa-clock"></i>
                                        </span>
                                    </div>
                                    <p class="help">{"账户锁定后多久自动解锁（分钟）"}</p>
                                </div>
                            </div>

                            // Right column - character requirements
                            <div class="column is-6">
                                <div class="field">
                                    <label class="checkbox">
                                        <input
                                            type="checkbox"
                                            checked={*require_uppercase}
                                            onchange={let require_uppercase = require_uppercase.clone(); move |e: Event| {
                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                require_uppercase.set(input.checked());
                                            }}
                                        />
                                        <span class="ml-2">{ lang.t("require_uppercase") }</span>
                                    </label>
                                    <p class="help ml-6">{"密码必须包含至少一个大写字母（A-Z）"}</p>
                                </div>

                                <div class="field">
                                    <label class="checkbox">
                                        <input
                                            type="checkbox"
                                            checked={*require_lowercase}
                                            onchange={let require_lowercase = require_lowercase.clone(); move |e: Event| {
                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                require_lowercase.set(input.checked());
                                            }}
                                        />
                                        <span class="ml-2">{ lang.t("require_lowercase") }</span>
                                    </label>
                                    <p class="help ml-6">{"密码必须包含至少一个小写字母（a-z）"}</p>
                                </div>

                                <div class="field">
                                    <label class="checkbox">
                                        <input
                                            type="checkbox"
                                            checked={*require_number}
                                            onchange={let require_number = require_number.clone(); move |e: Event| {
                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                require_number.set(input.checked());
                                            }}
                                        />
                                        <span class="ml-2">{ lang.t("require_number") }</span>
                                    </label>
                                    <p class="help ml-6">{"密码必须包含至少一个数字（0-9）"}</p>
                                </div>

                                <div class="field">
                                    <label class="checkbox">
                                        <input
                                            type="checkbox"
                                            checked={*require_special}
                                            onchange={let require_special = require_special.clone(); move |e: Event| {
                                                let input = e.target_unchecked_into::<HtmlInputElement>();
                                                require_special.set(input.checked());
                                            }}
                                        />
                                        <span class="ml-2">{ lang.t("require_special") }</span>
                                    </label>
                                    <p class="help ml-6">{"密码必须包含至少一个特殊字符（如 !@#$%^&*）"}</p>
                                </div>

                                // Policy preview card
                                <div class="box has-background-light mt-5">
                                    <h4 class="title is-6">{"密码要求预览"}</h4>
                                    <ul class="is-size-7">
                                        <li>{format!("• 最小长度：{} 位", *min_length)}</li>
                                        <li>{format!("• 强度要求：{}", match min_strength.as_str() {
                                            "weak" => "弱",
                                            "medium" => "中",
                                            "strong" => "强",
                                            _ => "未知"
                                        })}</li>
                                        <li>{format!("• 大写字母：{}", if *require_uppercase { "✓ 必需" } else { "✗ 不必需" })}</li>
                                        <li>{format!("• 小写字母：{}", if *require_lowercase { "✓ 必需" } else { "✗ 不必需" })}</li>
                                        <li>{format!("• 数字：{}", if *require_number { "✓ 必需" } else { "✗ 不必需" })}</li>
                                        <li>{format!("• 特殊字符：{}", if *require_special { "✓ 必需" } else { "✗ 不必需" })}</li>
                                        <li>{format!("• 有效期：{}", max_age_days.map(|d| format!("{} 天", d)).unwrap_or("不限制".to_string()))}</li>
                                        <li>{format!("• 防重用：最近 {} 次", *prevent_reuse)}</li>
                                        <li>{format!("• 账户锁定：{}", max_login_attempts.map(|n| format!("{} 次失败后锁定", n)).unwrap_or("不限制".to_string()))}</li>
                                        <li>{format!("• 锁定时长：{} 分钟", *lockout_duration_minutes)}</li>
                                    </ul>
                                </div>
                            </div>
                        </div>

                        <hr />

                        <div class="field is-grouped">
                            <div class="control">
                                <button type="submit" class="button is-primary">
                                    <span class="icon"><i class="fas fa-save"></i></span>
                                    <span>{ lang.t("save_policy") }</span>
                                </button>
                            </div>
                            <div class="control">
                                <button type="button" class="button" onclick={on_reset}>
                                    <span class="icon"><i class="fas fa-undo"></i></span>
                                    <span>{ lang.t("cancel") }</span>
                                </button>
                            </div>
                        </div>
                    </form>
                </div>
                }
            </div>
        }
    }
