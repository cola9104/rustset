//! RustSet Frontend - Yew 0.21 Version
//! Security Asset Management Platform

use gloo_net::http::Request;
use shared::{
    Asset, NetworkZone, Task, TaskStatus,
    Risk, ZoneConfig, User, Role, LoginRequest, LoginResponse, AuditLog,
    // Multi-Cloud types
    CloudAsset, CloudAssetStats, CloudProvider, VMStatus, BillingMode,
    // Advanced Scanning types
    ScanStrategy, ScanEngine, AdvancedScanConfig, AdvancedScanTask,
    CreateAdvancedScanRequest, ScanResult, Permissions, PasswordPolicy,
};
use wasm_bindgen_futures::spawn_local;
use web_sys::{InputEvent, Event, HtmlSelectElement, HtmlTextAreaElement, HtmlInputElement, window, Url};
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
    AssetCenter,
    ZoneManagement,
    RiskCenter,
    UserManagement,
    AuditLogs,
    CloudManagement, // 混合云管理
    UserProfile, // 个人中心
    PasswordPolicyManagement, // 密码策略管理
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
            (Language::Zh, "logout") => "退出登录".to_string(),
            (Language::En, "logout") => "Logout".to_string(),
            (Language::Zh, "user_management") => "👥 用户管理".to_string(),
            (Language::En, "user_management") => "👥 User Management".to_string(),
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
            (Language::Zh, "asset_management") => "📂 资产管理".to_string(),
            (Language::En, "asset_management") => "📂 Asset Management".to_string(),
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
            (Language::Zh, "zone_management") => "🌐 区域管理".to_string(),
            (Language::En, "zone_management") => "🌐 Zone Management".to_string(),
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
                let http_req = Request::post("http://localhost:3003/api/login")
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
        <aside class="menu p-4" style="height: 100vh; background-color: #f5f5f5;">
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
                if user_role == Some(Role::SecAdmin) || user_role == Some(Role::Auditor) {
                    <li><a onclick={navigate(Page::TaskCenter)}>{ lang.t("task_center") }</a></li>
                    <li><a onclick={navigate(Page::AdvancedScanning)}>{ lang.t("advanced_scanning") }</a></li>
                }
            </ul>
            if user_role == Some(Role::SecAdmin) {
                <p class="menu-label">{ lang.t("assets_risks") }</p>
                <ul class="menu-list">
                    <li><a onclick={navigate(Page::AssetCenter)}>{ lang.t("asset_management") }</a></li>
                    <li><a onclick={navigate(Page::ZoneManagement)}>{ lang.t("zone_management") }</a></li>
                    <li><a onclick={navigate(Page::RiskCenter)}>{ lang.t("risk_monitoring") }</a></li>
                </ul>
            }
            // 混合云管理 - 根据用户权限显示
            if user_permissions.as_ref().map(|p| p.can_view_cloud).unwrap_or(false) {
                <p class="menu-label">{ "Cloud" }</p>
                <ul class="menu-list">
                    <li><a onclick={navigate(Page::CloudManagement)}>{ lang.t("cloud_management") }</a></li>
                </ul>
            }
            if user_role == Some(Role::SysAdmin) {
                <p class="menu-label">{ lang.t("user_management") }</p>
                <ul class="menu-list">
                    <li><a onclick={navigate(Page::UserManagement)}>{ lang.t("user_management") }</a></li>
                    <li><a onclick={navigate(Page::PasswordPolicyManagement)}>{ lang.t("password_policy_management") }</a></li>
                </ul>
            }
            if user_role == Some(Role::Auditor) {
                <p class="menu-label">{ lang.t("audit_logs") }</p>
                <ul class="menu-list">
                    <li><a onclick={navigate(Page::AuditLogs)}>{ lang.t("audit_logs") }</a></li>
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

                let assets_req = Request::get("http://localhost:3003/api/assets")
                    .header("Authorization", &token)
                    .send()
                    .await;
                let tasks_req = Request::get("http://localhost:3003/api/tasks")
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
                if let Ok(resp) = Request::get("http://localhost:3003/api/tasks").header("Authorization", &token).send().await {
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
                if let Ok(resp) = Request::get("http://localhost:3003/api/risks").header("Authorization", &token).send().await {
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

// ============== Asset Center Component ==============

#[function_component]
fn AssetCenter() -> Html {
    let lang = use_state(|| Language::Zh);
    let assets = use_state(|| Vec::new());
    let loading = use_state(|| true);

    // Get token from localStorage
    let token = get_auth_token();

    // Use effect to fetch data only once on mount
    use_effect_with((), {
        let assets = assets.clone();
        let loading = loading.clone();
        let token = token.clone();

        move |_| {
            spawn_local(async move {
                if let Ok(resp) = Request::get("http://localhost:3003/api/assets").header("Authorization", &token).send().await {
                    if let Ok(data) = resp.json::<Vec<Asset>>().await {
                        assets.set(data);
                    }
                }
                loading.set(false);
            });
            || ()
        }
    });

    let zone_name = |zone: &NetworkZone| -> String {
        match zone {
            NetworkZone::Internet => "Internet".to_string(),
            NetworkZone::DMZ => "DMZ".to_string(),
            NetworkZone::Intranet => "Intranet".to_string(),
            NetworkZone::Custom(s) => s.clone(),
        }
    };

    let zone_class = |zone: &NetworkZone| -> &'static str {
        match zone {
            NetworkZone::Internet => "is-info",
            NetworkZone::DMZ => "is-warning",
            NetworkZone::Intranet => "is-success",
            NetworkZone::Custom(_) => "is-light",
        }
    };

    html! {
        <div class="container p-4">
            <h1 class="title">{ lang.t("asset_management") }</h1>
            <div class="box">
                if (*assets).is_empty() && *loading {
                    <p class="has-text-centered has-text-grey">{ "Loading..." }</p>
                } else if (*assets).is_empty() {
                    <p class="has-text-centered has-text-grey">{ "No assets yet" }</p>
                } else {
                    <div class="table-container">
                        <table class="table is-fullwidth is-hoverable is-striped">
                            <thead>
                                <tr>
                                    <th>{ lang.t("name") }</th>
                                    <th>{ lang.t("ip") }</th>
                                    <th>{ lang.t("zone") }</th>
                                    <th>{ lang.t("owner") }</th>
                                    <th>{ lang.t("weight") }</th>
                                    <th>{ lang.t("ports") }</th>
                                </tr>
                            </thead>
                            <tbody>
                                { for assets.iter().map(|asset| {
                                    let zone = asset.zone.clone();
                                    html! {
                                        <tr>
                                            <td>{ &asset.name }</td>
                                            <td><code>{ &asset.ip }</code></td>
                                            <td>
                                                <span class={classes!("tag", zone_class(&zone))}>
                                                    { zone_name(&zone) }
                                                </span>
                                            </td>
                                            <td>{ asset.owner.clone().unwrap_or_default() }</td>
                                            <td>{ asset.weight }</td>
                                            <td>
                                                <div class="tags are-small">
                                                    { for asset.ports.iter().take(3).map(|port| {
                                                        html! {
                                                            <span class={if port.is_bound { "tag is-primary" } else { "tag is-light" }}>
                                                                { format!("{} ({})", port.port, port.service.clone().unwrap_or_default()) }
                                                            </span>
                                                        }
                                                    })}
                                                    if asset.ports.len() > 3 {
                                                        <span class="tag">{ format!("+{}", asset.ports.len() - 3) }</span>
                                                    }
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

// ============== Zone Management Component ==============

#[function_component]
fn ZoneManagement() -> Html {
    let lang = use_state(|| Language::Zh);
    let zones = use_state(|| Vec::new());
    let loading = use_state(|| true);

    // Get token from localStorage
    let token = get_auth_token();

    // Use effect to fetch data only once on mount
    use_effect_with((), {
        let zones = zones.clone();
        let loading = loading.clone();
        let token = token.clone();

        move |_| {
            spawn_local(async move {
                if let Ok(resp) = Request::get("http://localhost:3003/api/zones").header("Authorization", &token).send().await {
                    if let Ok(data) = resp.json::<Vec<ZoneConfig>>().await {
                        zones.set(data);
                    }
                }
                loading.set(false);
            });
            || ()
        }
    });

    html! {
        <div class="container p-4">
            <h1 class="title">{ lang.t("zone_management") }</h1>
            <div class="box">
                if (*zones).is_empty() && *loading {
                    <p class="has-text-centered has-text-grey">{ "Loading..." }</p>
                } else if (*zones).is_empty() {
                    <p class="has-text-centered has-text-grey">{ "No zones configured" }</p>
                } else {
                    <table class="table is-fullwidth is-hoverable">
                        <thead>
                            <tr>
                                <th>{ lang.t("zone") }</th>
                                <th>{ lang.t("cidr") }</th>
                                <th>{ lang.t("priority") }</th>
                            </tr>
                        </thead>
                        <tbody>
                            { for zones.iter().map(|zone| {
                                html! {
                                    <tr>
                                        <td>{ &zone.name }</td>
                                        <td><code>{ &zone.cidr }</code></td>
                                        <td>{ zone.priority }</td>
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
                if let Ok(resp) = Request::get("http://localhost:3003/api/users").header("Authorization", &token).send().await {
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

        Callback::from(move |_| {
            if let Some((user_id, _, _)) = &*editing_user {
                if let Some(perms) = &*temp_permissions {
                    let user_id = user_id.clone();
                    let perms = perms.clone();
                    let token = token.clone();
                    let users = users.clone();
                    let show_permissions_modal = show_permissions_modal.clone();

                    spawn_local(async move {
                        let json_body = serde_json::to_string(&perms).unwrap_or_default();
                        let url = format!("http://localhost:3003/api/users/{}/permissions", user_id);

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
                                if let Ok(resp) = Request::get("http://localhost:3003/api/users")
                                    .header("Authorization", &token)
                                    .send()
                                    .await
                                {
                                    if let Ok(data) = resp.json::<Vec<User>>().await {
                                        users.set(data);
                                    }
                                }
                                show_permissions_modal.set(false);
                            }
                        }
                    });
                }
            }
        })
    };

    // Toggle permission - create individual callbacks for each permission
    let temp_perms_for_callbacks = temp_permissions.clone();

    let make_toggle_callback = |field: String| {
        let temp_permissions = temp_perms_for_callbacks.clone();
        Callback::from(move |e: Event| {
            let target = e.target_unchecked_into::<HtmlInputElement>();
            let value = target.checked();
            let field = field.clone();

            if let Some(mut perms) = (*temp_permissions).clone() {
                match field.as_str() {
                    "can_create_scan" => perms.can_create_scan = value,
                    "can_delete_scan" => perms.can_delete_scan = value,
                    "can_export_scan" => perms.can_export_scan = value,
                    "can_view_assets" => perms.can_view_assets = value,
                    "can_create_asset" => perms.can_create_asset = value,
                    "can_update_asset" => perms.can_update_asset = value,
                    "can_delete_asset" => perms.can_delete_asset = value,
                    "can_view_cloud" => perms.can_view_cloud = value,
                    "can_manage_cloud" => perms.can_manage_cloud = value,
                    "can_delete_cloud" => perms.can_delete_cloud = value,
                    "can_sync_cloud" => perms.can_sync_cloud = value,
                    "can_view_risks" => perms.can_view_risks = value,
                    "can_resolve_risk" => perms.can_resolve_risk = value,
                    "can_delete_risk" => perms.can_delete_risk = value,
                    "can_view_users" => perms.can_view_users = value,
                    "can_create_user" => perms.can_create_user = value,
                    "can_update_user" => perms.can_update_user = value,
                    "can_delete_user" => perms.can_delete_user = value,
                    "can_manage_permissions" => perms.can_manage_permissions = value,
                    "can_view_audit_logs" => perms.can_view_audit_logs = value,
                    "can_view_zones" => perms.can_view_zones = value,
                    "can_manage_zones" => perms.can_manage_zones = value,
                    _ => {}
                }
                temp_permissions.set(Some(perms));
            }
        })
    };

    html! {
        <div class="container p-4">
            <h1 class="title">{ lang.t("user_management") }</h1>
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
                                let user_id = user.id.clone();
                                let on_edit = on_edit_permissions.clone();
                                let current_user_role = user_role.clone();

                                html! {
                                    <tr>
                                        <td>{ &user.username }</td>
                                        <td>
                                            <span class="tag">{ role_name(&user.role) }</span>
                                        </td>
                                        <td>{ &user.created_at.format("%Y-%m-%d %H:%M:%S").to_string() }</td>
                                        <td>
                                            // Only admin can edit permissions
                                            if current_user_role == Some(Role::SysAdmin) && user.permissions.is_some() {
                                                <button
                                                    class="button is-small is-info"
                                                    onclick={move |_| on_edit.emit(user_id.clone())}
                                                >
                                                    { "编辑权限" }
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

            // Permission Editing Modal
            if *show_permissions_modal {
                <div class="modal is-active">
                    <div class="modal-background" onclick={on_close_modal.clone()}></div>
                    <div class="modal-card" style="width: 800px;">
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

                                <div style="margin-bottom: 1rem;">
                                    <h4 class="title is-6">{ "🔍 扫描权限" }</h4>
                                    <div class="box">
                                        <label class="checkbox" style="display: block; margin: 0.5rem 0;">
                                            <input
                                                type="checkbox"
                                                checked={perms.can_create_scan}
                                                onchange={make_toggle_callback(String::from("can_create_scan"))}/
                                            >
                                            { " 创建扫描" }
                                        </label>
                                        <label class="checkbox" style="display: block; margin: 0.5rem 0;">
                                            <input
                                                type="checkbox"
                                                checked={perms.can_delete_scan}
                                                onchange={make_toggle_callback(String::from("can_delete_scan"))}/
                                            >
                                            { " 删除扫描" }
                                        </label>
                                        <label class="checkbox" style="display: block; margin: 0.5rem 0;">
                                            <input
                                                type="checkbox"
                                                checked={perms.can_export_scan}
                                                onchange={make_toggle_callback(String::from("can_export_scan"))}/
                                            >
                                            { " 导出结果" }
                                        </label>
                                    </div>
                                </div>

                                <div style="margin-bottom: 1rem;">
                                    <h4 class="title is-6">{ "💰 资产权限" }</h4>
                                    <div class="box">
                                        <label class="checkbox" style="display: block; margin: 0.5rem 0;">
                                            <input
                                                type="checkbox"
                                                checked={perms.can_view_assets}
                                                onchange={make_toggle_callback(String::from("can_view_assets"))}/
                                            >
                                            { " 查看资产" }
                                        </label>
                                        <label class="checkbox" style="display: block; margin: 0.5rem 0;">
                                            <input
                                                type="checkbox"
                                                checked={perms.can_create_asset}
                                                onchange={make_toggle_callback(String::from("can_create_asset"))}/
                                            >
                                            { " 创建资产" }
                                        </label>
                                        <label class="checkbox" style="display: block; margin: 0.5rem 0;">
                                            <input
                                                type="checkbox"
                                                checked={perms.can_update_asset}
                                                onchange={make_toggle_callback(String::from("can_update_asset"))}/
                                            >
                                            { " 更新资产" }
                                        </label>
                                        <label class="checkbox" style="display: block; margin: 0.5rem 0;">
                                            <input
                                                type="checkbox"
                                                checked={perms.can_delete_asset}
                                                onchange={make_toggle_callback(String::from("can_delete_asset"))}/
                                            >
                                            { " 删除资产" }
                                        </label>
                                    </div>
                                </div>

                                <div style="margin-bottom: 1rem;">
                                    <h4 class="title is-6">{ "☁️ 云资产权限" }</h4>
                                    <div class="box">
                                        <label class="checkbox" style="display: block; margin: 0.5rem 0;">
                                            <input
                                                type="checkbox"
                                                checked={perms.can_view_cloud}
                                                onchange={make_toggle_callback(String::from("can_view_cloud"))}/
                                            >
                                            { " 查看云资产" }
                                        </label>
                                        <label class="checkbox" style="display: block; margin: 0.5rem 0;">
                                            <input
                                                type="checkbox"
                                                checked={perms.can_manage_cloud}
                                                onchange={make_toggle_callback(String::from("can_manage_cloud"))}/
                                            >
                                            { " 管理云资产" }
                                        </label>
                                        <label class="checkbox" style="display: block; margin: 0.5rem 0;">
                                            <input
                                                type="checkbox"
                                                checked={perms.can_delete_cloud}
                                                onchange={make_toggle_callback(String::from("can_delete_cloud"))}/
                                            >
                                            { " 删除云资产" }
                                        </label>
                                        <label class="checkbox" style="display: block; margin: 0.5rem 0;">
                                            <input
                                                type="checkbox"
                                                checked={perms.can_sync_cloud}
                                                onchange={make_toggle_callback(String::from("can_sync_cloud"))}/
                                            >
                                            { " 同步云资产" }
                                        </label>
                                    </div>
                                </div>

                                <div style="margin-bottom: 1rem;">
                                    <h4 class="title is-6">{ "⚠️ 风险权限" }</h4>
                                    <div class="box">
                                        <label class="checkbox" style="display: block; margin: 0.5rem 0;">
                                            <input
                                                type="checkbox"
                                                checked={perms.can_view_risks}
                                                onchange={make_toggle_callback(String::from("can_view_risks"))}/
                                            >
                                            { " 查看风险" }
                                        </label>
                                        <label class="checkbox" style="display: block; margin: 0.5rem 0;">
                                            <input
                                                type="checkbox"
                                                checked={perms.can_resolve_risk}
                                                onchange={make_toggle_callback(String::from("can_resolve_risk"))}/
                                            >
                                            { " 处置风险" }
                                        </label>
                                        <label class="checkbox" style="display: block; margin: 0.5rem 0;">
                                            <input
                                                type="checkbox"
                                                checked={perms.can_delete_risk}
                                                onchange={make_toggle_callback(String::from("can_delete_risk"))}/
                                            >
                                            { " 删除风险" }
                                        </label>
                                    </div>
                                </div>

                                <div style="margin-bottom: 1rem;">
                                    <h4 class="title is-6">{ "👥 用户管理权限" }</h4>
                                    <div class="box">
                                        <label class="checkbox" style="display: block; margin: 0.5rem 0;">
                                            <input
                                                type="checkbox"
                                                checked={perms.can_view_users}
                                                onchange={make_toggle_callback(String::from("can_view_users"))}/
                                            >
                                            { " 查看用户" }
                                        </label>
                                        <label class="checkbox" style="display: block; margin: 0.5rem 0;">
                                            <input
                                                type="checkbox"
                                                checked={perms.can_create_user}
                                                onchange={make_toggle_callback(String::from("can_create_user"))}/
                                            >
                                            { " 创建用户" }
                                        </label>
                                        <label class="checkbox" style="display: block; margin: 0.5rem 0;">
                                            <input
                                                type="checkbox"
                                                checked={perms.can_update_user}
                                                onchange={make_toggle_callback(String::from("can_update_user"))}/
                                            >
                                            { " 更新用户" }
                                        </label>
                                        <label class="checkbox" style="display: block; margin: 0.5rem 0;">
                                            <input
                                                type="checkbox"
                                                checked={perms.can_delete_user}
                                                onchange={make_toggle_callback(String::from("can_delete_user"))}/
                                            >
                                            { " 删除用户" }
                                        </label>
                                        <label class="checkbox" style="display: block; margin: 0.5rem 0;">
                                            <input
                                                type="checkbox"
                                                checked={perms.can_manage_permissions}
                                                onchange={make_toggle_callback(String::from("can_manage_permissions"))}/
                                            >
                                            { " 管理权限" }
                                        </label>
                                    </div>
                                </div>

                                <div style="margin-bottom: 1rem;">
                                    <h4 class="title is-6">{ "📋 审计权限" }</h4>
                                    <div class="box">
                                        <label class="checkbox" style="display: block; margin: 0.5rem 0;">
                                            <input
                                                type="checkbox"
                                                checked={perms.can_view_audit_logs}
                                                onchange={make_toggle_callback(String::from("can_view_audit_logs"))}/
                                            >
                                            { " 查看审计日志" }
                                        </label>
                                    </div>
                                </div>

                                <div style="margin-bottom: 1rem;">
                                    <h4 class="title is-6">{ "🌐 区域管理权限" }</h4>
                                    <div class="box">
                                        <label class="checkbox" style="display: block; margin: 0.5rem 0;">
                                            <input
                                                type="checkbox"
                                                checked={perms.can_view_zones}
                                                onchange={make_toggle_callback(String::from("can_view_zones"))}/
                                            >
                                            { " 查看区域" }
                                        </label>
                                        <label class="checkbox" style="display: block; margin: 0.5rem 0;">
                                            <input
                                                type="checkbox"
                                                checked={perms.can_manage_zones}
                                                onchange={make_toggle_callback(String::from("can_manage_zones"))}/
                                            >
                                            { " 管理区域" }
                                        </label>
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
                if let Ok(resp) = Request::get("http://localhost:3003/api/logs").header("Authorization", &token).send().await {
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
                if let Ok(resp) = Request::get("http://localhost:3003/api/cloud-assets")
                    .header("Authorization", &token)
                    .send()
                    .await
                {
                    if let Ok(data) = resp.json::<Vec<CloudAsset>>().await {
                        assets.set(data);
                    }
                }

                // Fetch stats
                if let Ok(resp) = Request::get("http://localhost:3003/api/cloud-assets/stats")
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

                if let Ok(resp) = Request::get("http://localhost:3003/api/cloud-assets")
                    .header("Authorization", &token)
                    .send()
                    .await
                {
                    if let Ok(data) = resp.json::<Vec<CloudAsset>>().await {
                        assets.set(data);
                    }
                }

                if let Ok(resp) = Request::get("http://localhost:3003/api/cloud-assets/stats")
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

                    if let Ok(resp) = Request::put(&format!("http://localhost:3003/api/cloud-assets/{}", asset_id))
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
                            if let Ok(resp) = Request::get("http://localhost:3003/api/cloud-assets")
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
                                Page::AssetCenter => html! { <AssetCenter /> },
                                Page::ZoneManagement => html! { <ZoneManagement /> },
                                Page::RiskCenter => html! { <RiskCenter /> },
                                Page::UserManagement => html! { <UserManagement /> },
                                Page::AuditLogs => html! { <AuditLogs /> },
                                Page::CloudManagement => html! { <CloudManagement /> },
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

    // 加载任务列表
    let load_tasks = {
        let tasks = tasks.clone();
        let token = token.clone();

        Callback::from(move |_| {
            let tasks = tasks.clone();
            let token = token.clone();

            spawn_local(async move {
                if let Ok(resp) = Request::get("http://localhost:3003/api/scan/advanced/tasks")
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
                let http_req = Request::post("http://localhost:3003/api/scan/advanced")
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
                    "http://localhost:3003/api/scan/advanced/tasks/{}/export",
                    task_id_clone
                ))
                .header("Authorization", &token)
                .send()
                .await
                {
                    if let Ok(results) = resp.json::<Vec<shared::ScanResult>>().await {
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
                    <button class="button is-primary" onclick={on_open_modal}>
                        { lang.t("create_scan") }
                    </button>
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
                                                <button
                                                    class="button is-small is-info is-light"
                                                    onclick={on_export_results.reform(move |_| task_id.clone())}
                                                >{"导出结果"}</button>
                                            }
                                            <button class="button is-small is-danger is-light">{"删除"}</button>
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
                match Request::post("http://localhost:3003/api/users/change-password")
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
                if p.can_view_assets { count += 1; }
                if p.can_create_asset { count += 1; }
                if p.can_update_asset { count += 1; }
                if p.can_delete_asset { count += 1; }
                if p.can_view_cloud { count += 1; }
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
                if p.can_view_zones { count += 1; }
                if p.can_manage_zones { count += 1; }
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
                                                                [permissions.can_view_assets, permissions.can_create_asset, permissions.can_update_asset, permissions.can_delete_asset]
                                                                .iter().filter(|&&x| x).count())
                                                            }</span>
                                                    </span>
                                                </div>
                                                <div class="card-content">
                                                    <div class="tags are-small">
                                                        { for [permissions.can_view_assets, permissions.can_create_asset, permissions.can_update_asset, permissions.can_delete_asset]
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
                                                        {"云资产权限"}
                                                    </p>
                                                    <span class="card-header-icon">
                                                        <span class="tag is-light">{ 
                                                            format!("{}/4", 
                                                                [permissions.can_view_cloud, permissions.can_manage_cloud, permissions.can_delete_cloud, permissions.can_sync_cloud]
                                                                .iter().filter(|&&x| x).count())
                                                            }</span>
                                                    </span>
                                                </div>
                                                <div class="card-content">
                                                    <div class="tags are-small">
                                                        { for [permissions.can_view_cloud, permissions.can_manage_cloud, permissions.can_delete_cloud, permissions.can_sync_cloud]
                                                            .iter()
                                                            .zip(["查看云资产", "管理云资产", "删除云资产", "同步云资产"].iter())
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

                                        // 区域管理权限
                                        <div class="column is-6">
                                            <div class="card">
                                                <div class="card-header">
                                                    <p class="card-header-title">
                                                        <span class="icon mr-2"><i class="fas fa-globe has-text-success"></i></span>
                                                        {"区域管理权限"}
                                                    </p>
                                                    <span class="card-header-icon">
                                                        <span class="tag is-light">{
                                                            format!("{}/2",
                                                                [permissions.can_view_zones, permissions.can_manage_zones]
                                                                .iter().filter(|&&x| x).count())
                                                        }</span>
                                                    </span>
                                                </div>
                                                <div class="card-content">
                                                    <div class="tags are-small">
                                                        { for [permissions.can_view_zones, permissions.can_manage_zones]
                                                            .iter()
                                                            .zip(["查看区域", "管理区域"].iter())
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
                match Request::get("http://localhost:3003/api/password-policy")
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
                match Request::put("http://localhost:3003/api/password-policy")
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
