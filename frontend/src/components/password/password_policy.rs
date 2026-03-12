use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaShield, FaKey, FaClock, FaLock, FaUserShield, FaCheck
};

/// 全局安全策略状态
pub static SECURITY_POLICY_STATE: GlobalSignal<SecurityPolicy> = Signal::global(SecurityPolicy::default);

/// 安全策略配置（全局唯一）
#[derive(Clone, Debug, PartialEq)]
pub struct SecurityPolicy {
    // 密码策略
    pub min_length: i32,
    pub max_length: i32,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_numbers: bool,
    pub require_special_chars: bool,
    pub expire_days: i32,
    pub prevent_reuse: i32,
    pub expire_warning_days: i32,

    // 账户锁定策略
    pub lockout_attempts: i32,
    pub lockout_duration: i32,
    pub lockout_reset_minutes: i32,

    // 会话策略
    pub session_timeout_minutes: i32,
    pub idle_timeout_minutes: i32,
    pub concurrent_login: bool,
    pub max_concurrent_sessions: i32,

    // 登录策略
    pub enable_captcha: bool,
    pub captcha_threshold: i32,
    pub ip_whitelist_enabled: bool,
    pub ip_whitelist: String,

    // MFA 策略
    pub mfa_enabled: bool,
    pub mfa_for_admin: bool,
    pub mfa_for_remote_login: bool,

    // 其他安全设置
    pub password_change_on_first_login: bool,
    pub password_change_frequency: i32,
    pub audit_log_retention_days: i32,
}

impl Default for SecurityPolicy {
    fn default() -> Self {
        Self {
            min_length: 8,
            max_length: 32,
            require_uppercase: true,
            require_lowercase: true,
            require_numbers: true,
            require_special_chars: true,
            expire_days: 90,
            prevent_reuse: 5,
            expire_warning_days: 7,
            lockout_attempts: 5,
            lockout_duration: 30,
            lockout_reset_minutes: 15,
            session_timeout_minutes: 60,
            idle_timeout_minutes: 30,
            concurrent_login: false,
            max_concurrent_sessions: 1,
            enable_captcha: false,
            captcha_threshold: 3,
            ip_whitelist_enabled: false,
            ip_whitelist: String::new(),
            mfa_enabled: false,
            mfa_for_admin: true,
            mfa_for_remote_login: true,
            password_change_on_first_login: true,
            password_change_frequency: 0,
            audit_log_retention_days: 180,
        }
    }
}

/// 安全策略页面
#[allow(non_snake_case)]
pub fn PasswordPolicy() -> Element {
    // 使用全局状态
    let policy = use_signal(|| SECURITY_POLICY_STATE.read().clone());
    let mut show_success = use_signal(|| false);
    let mut active_section = use_signal(|| "password".to_string());

    rsx! {
        div { class: "space-y-6",
            // 页面标题
            div { class: "flex justify-between items-center",
                div {
                    h1 { class: "text-2xl font-bold text-gray-800", "安全策略配置" }
                    p { class: "text-sm text-gray-500 mt-1", "配置系统的全局安全策略，适用于所有用户" }
                }
            }

            // 成功提示
            if *show_success.read() {
                div { class: "bg-green-50 border border-green-200 text-green-700 px-4 py-3 rounded-lg flex items-center",
                    Icon { icon: FaCheck, width: 20, height: 20, class: "mr-2" }
                    "安全策略配置已保存"
                }
            }

            // 导航标签
            div { class: "bg-white rounded-lg shadow-sm border border-gray-200",
                div { class: "border-b border-gray-200",
                    div { class: "flex",
                        button {
                            class: format!("px-6 py-4 text-sm font-medium border-b-2 transition-colors {}",
                                if *active_section.read() == "password" {
                                    "border-blue-500 text-blue-600"
                                } else {
                                    "border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300"
                                }
                            ),
                            onclick: move |_| active_section.set("password".to_string()),
                            Icon { icon: FaKey, width: 16, height: 16, class: "mr-2" }
                            "密码策略"
                        }
                        button {
                            class: format!("px-6 py-4 text-sm font-medium border-b-2 transition-colors {}",
                                if *active_section.read() == "account" {
                                    "border-blue-500 text-blue-600"
                                } else {
                                    "border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300"
                                }
                            ),
                            onclick: move |_| active_section.set("account".to_string()),
                            Icon { icon: FaLock, width: 16, height: 16, class: "mr-2" }
                            "账户锁定"
                        }
                        button {
                            class: format!("px-6 py-4 text-sm font-medium border-b-2 transition-colors {}",
                                if *active_section.read() == "session" {
                                    "border-blue-500 text-blue-600"
                                } else {
                                    "border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300"
                                }
                            ),
                            onclick: move |_| active_section.set("session".to_string()),
                            Icon { icon: FaClock, width: 16, height: 16, class: "mr-2" }
                            "会话管理"
                        }
                        button {
                            class: format!("px-6 py-4 text-sm font-medium border-b-2 transition-colors {}",
                                if *active_section.read() == "login" {
                                    "border-blue-500 text-blue-600"
                                } else {
                                    "border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300"
                                }
                            ),
                            onclick: move |_| active_section.set("login".to_string()),
                            Icon { icon: FaUserShield, width: 16, height: 16, class: "mr-2" }
                            "登录安全"
                        }
                    }
                }

                div { class: "p-6",
                    // 密码策略部分
                    if *active_section.read() == "password" {
                        PasswordPolicySection { policy: policy.clone() }
                    }

                    // 账户锁定部分
                    if *active_section.read() == "account" {
                        AccountLockoutSection { policy: policy.clone() }
                    }

                    // 会话管理部分
                    if *active_section.read() == "session" {
                        SessionManagementSection { policy: policy.clone() }
                    }

                    // 登录安全部分
                    if *active_section.read() == "login" {
                        LoginSecuritySection { policy: policy.clone() }
                    }
                }
            }

            // 保存按钮
            div { class: "flex justify-end",
                button {
                    class: "flex items-center px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors",
                    onclick: move |_| {
                        // 保存到全局状态
                        *SECURITY_POLICY_STATE.write() = policy.read().clone();
                        show_success.set(true);
                    },
                    Icon { icon: FaCheck, width: 16, height: 16, class: "mr-2" }
                    "保存配置"
                }
                // 添加关闭提示按钮
                if *show_success.read() {
                    button {
                        class: "ml-4 flex items-center px-4 py-2 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 transition-colors",
                        onclick: move |_| show_success.set(false),
                        "关闭提示"
                    }
                }
            }
        }
    }
}

/// 密码策略配置部分
#[component]
fn PasswordPolicySection(policy: Signal<SecurityPolicy>) -> Element {
    rsx! {
        div { class: "space-y-6",
            // 密码长度
            div { class: "grid grid-cols-1 md:grid-cols-2 gap-6",
                div {
                    label { class: "block text-sm font-medium text-gray-700 mb-2",
                        "最小密码长度"
                    }
                    input {
                        r#type: "number",
                        class: "w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500",
                        min: 6,
                        max: 20,
                        value: "{policy.read().min_length}",
                        oninput: move |e| {
                            if let Ok(v) = e.value().parse::<i32>() {
                                policy.with_mut(|p| p.min_length = v);
                            }
                        }
                    }
                    p { class: "mt-1 text-xs text-gray-500", "推荐：8-12位" }
                }
                div {
                    label { class: "block text-sm font-medium text-gray-700 mb-2",
                        "最大密码长度"
                    }
                    input {
                        r#type: "number",
                        class: "w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500",
                        min: 16,
                        max: 128,
                        value: "{policy.read().max_length}",
                        oninput: move |e| {
                            if let Ok(v) = e.value().parse::<i32>() {
                                policy.with_mut(|p| p.max_length = v);
                            }
                        }
                    }
                }
            }

            // 密码复杂度
            div { class: "border-t border-gray-200 pt-6" },
            h3 { class: "text-sm font-semibold text-gray-700 mb-4", "密码复杂度要求" }
            div { class: "space-y-3",
                div { class: "flex items-center justify-between p-3 bg-gray-50 rounded-lg",
                    div { class: "flex items-center",
                        input {
                            r#type: "checkbox",
                            class: "h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded",
                            checked: policy.read().require_uppercase,
                            oninput: move |e| policy.with_mut(|p| p.require_uppercase = e.checked()),
                        }
                        label { class: "ml-3 text-sm text-gray-700", "必须包含大写字母 (A-Z)" }
                    }
                }
                div { class: "flex items-center justify-between p-3 bg-gray-50 rounded-lg",
                    div { class: "flex items-center",
                        input {
                            r#type: "checkbox",
                            class: "h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded",
                            checked: policy.read().require_lowercase,
                            oninput: move |e| policy.with_mut(|p| p.require_lowercase = e.checked()),
                        }
                        label { class: "ml-3 text-sm text-gray-700", "必须包含小写字母 (a-z)" }
                    }
                }
                div { class: "flex items-center justify-between p-3 bg-gray-50 rounded-lg",
                    div { class: "flex items-center",
                        input {
                            r#type: "checkbox",
                            class: "h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded",
                            checked: policy.read().require_numbers,
                            oninput: move |e| policy.with_mut(|p| p.require_numbers = e.checked()),
                        }
                        label { class: "ml-3 text-sm text-gray-700", "必须包含数字 (0-9)" }
                    }
                }
                div { class: "flex items-center justify-between p-3 bg-gray-50 rounded-lg",
                    div { class: "flex items-center",
                        input {
                            r#type: "checkbox",
                            class: "h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded",
                            checked: policy.read().require_special_chars,
                            oninput: move |e| policy.with_mut(|p| p.require_special_chars = e.checked()),
                        }
                        label { class: "ml-3 text-sm text-gray-700", "必须包含特殊字符 (!@#$%^&*)" }
                    }
                }
            }

            // 密码有效期
            div { class: "border-t border-gray-200 pt-6" },
            h3 { class: "text-sm font-semibold text-gray-700 mb-4", "密码有效期" }
            div { class: "grid grid-cols-1 md:grid-cols-3 gap-6",
                div {
                    label { class: "block text-sm font-medium text-gray-700 mb-2",
                        "密码过期天数"
                    }
                    input {
                        r#type: "number",
                        class: "w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500",
                        min: 0,
                        max: 365,
                        value: "{policy.read().expire_days}",
                        oninput: move |e| {
                            if let Ok(v) = e.value().parse::<i32>() {
                                policy.with_mut(|p| p.expire_days = v);
                            }
                        }
                    }
                    p { class: "mt-1 text-xs text-gray-500", "设为0表示永不过期" }
                }
                div {
                    label { class: "block text-sm font-medium text-gray-700 mb-2",
                        "过期前提醒天数"
                    }
                    input {
                        r#type: "number",
                        class: "w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500",
                        min: 1,
                        max: 30,
                        value: "{policy.read().expire_warning_days}",
                        oninput: move |e| {
                            if let Ok(v) = e.value().parse::<i32>() {
                                policy.with_mut(|p| p.expire_warning_days = v);
                            }
                        }
                    }
                }
                div {
                    label { class: "block text-sm font-medium text-gray-700 mb-2",
                        "禁止复用历史次数"
                    }
                    input {
                        r#type: "number",
                        class: "w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500",
                        min: 0,
                        max: 24,
                        value: "{policy.read().prevent_reuse}",
                        oninput: move |e| {
                            if let Ok(v) = e.value().parse::<i32>() {
                                policy.with_mut(|p| p.prevent_reuse = v);
                            }
                        }
                    }
                    p { class: "mt-1 text-xs text-gray-500", "不能使用最近N次的旧密码" }
                }
            }
        }
    }
}

/// 账户锁定策略部分
#[component]
fn AccountLockoutSection(policy: Signal<SecurityPolicy>) -> Element {
    rsx! {
        div { class: "space-y-6",
            div { class: "flex items-center p-4 bg-blue-50 border border-blue-200 rounded-lg",
                Icon { icon: FaShield, width: 24, height: 24, class: "text-blue-600 mr-3" }
                div { class: "text-sm text-blue-800",
                    p { class: "font-medium", "账户锁定策略" }
                    p { class: "text-blue-600 mt-1", "防止暴力破解攻击，保护账户安全" }
                }
            }

            div { class: "grid grid-cols-1 md:grid-cols-3 gap-6",
                div {
                    label { class: "block text-sm font-medium text-gray-700 mb-2",
                        "锁定尝试次数"
                    }
                    input {
                        r#type: "number",
                        class: "w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500",
                        min: 3,
                        max: 10,
                        value: "{policy.read().lockout_attempts}",
                        oninput: move |e| {
                            if let Ok(v) = e.value().parse::<i32>() {
                                policy.with_mut(|p| p.lockout_attempts = v);
                            }
                        }
                    }
                    p { class: "mt-1 text-xs text-gray-500", "连续失败N次后锁定账户" }
                }
                div {
                    label { class: "block text-sm font-medium text-gray-700 mb-2",
                        "锁定时长（分钟）"
                    }
                    input {
                        r#type: "number",
                        class: "w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500",
                        min: 5,
                        max: 1440,
                        value: "{policy.read().lockout_duration}",
                        oninput: move |e| {
                            if let Ok(v) = e.value().parse::<i32>() {
                                policy.with_mut(|p| p.lockout_duration = v);
                            }
                        }
                    }
                }
                div {
                    label { class: "block text-sm font-medium text-gray-700 mb-2",
                        "锁定重置时间（分钟）"
                    }
                    input {
                        r#type: "number",
                        class: "w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500",
                        min: 5,
                        max: 60,
                        value: "{policy.read().lockout_reset_minutes}",
                        oninput: move |e| {
                            if let Ok(v) = e.value().parse::<i32>() {
                                policy.with_mut(|p| p.lockout_reset_minutes = v);
                            }
                        }
                    }
                    p { class: "mt-1 text-xs text-gray-500", "N分钟无失败尝试后重置计数" }
                }
            }
        }
    }
}

/// 会话管理策略部分
#[component]
fn SessionManagementSection(policy: Signal<SecurityPolicy>) -> Element {
    rsx! {
        div { class: "space-y-6",
            div { class: "grid grid-cols-1 md:grid-cols-2 gap-6",
                div {
                    label { class: "block text-sm font-medium text-gray-700 mb-2",
                        "会话超时时间（分钟）"
                    }
                    input {
                        r#type: "number",
                        class: "w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500",
                        min: 15,
                        max: 480,
                        value: "{policy.read().session_timeout_minutes}",
                        oninput: move |e| {
                            if let Ok(v) = e.value().parse::<i32>() {
                                policy.with_mut(|p| p.session_timeout_minutes = v);
                            }
                        }
                    }
                    p { class: "mt-1 text-xs text-gray-500", "用户登录后的最长活动时间" }
                }
                div {
                    label { class: "block text-sm font-medium text-gray-700 mb-2",
                        "空闲超时时间（分钟）"
                    }
                    input {
                        r#type: "number",
                        class: "w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500",
                        min: 5,
                        max: 120,
                        value: "{policy.read().idle_timeout_minutes}",
                        oninput: move |e| {
                            if let Ok(v) = e.value().parse::<i32>() {
                                policy.with_mut(|p| p.idle_timeout_minutes = v);
                            }
                        }
                    }
                    p { class: "mt-1 text-xs text-gray-500", "无操作后自动退出" }
                }
            }

            div { class: "border-t border-gray-200 pt-6" },
            h3 { class: "text-sm font-semibold text-gray-700 mb-4", "并发登录控制" }
            div { class: "space-y-3",
                div { class: "flex items-center justify-between p-3 bg-gray-50 rounded-lg",
                    div { class: "flex items-center",
                        input {
                            r#type: "checkbox",
                            class: "h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded",
                            checked: policy.read().concurrent_login,
                            oninput: move |e| policy.with_mut(|p| p.concurrent_login = e.checked()),
                        }
                        label { class: "ml-3 text-sm text-gray-700", "允许同一账户并发登录" }
                    }
                }
                div { class: "grid grid-cols-1 md:grid-cols-2 gap-6",
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-2",
                            "最大并发会话数"
                        }
                        input {
                            r#type: "number",
                            class: "w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500",
                            min: 1,
                            max: 10,
                            value: "{policy.read().max_concurrent_sessions}",
                            oninput: move |e| {
                                if let Ok(v) = e.value().parse::<i32>() {
                                    policy.with_mut(|p| p.max_concurrent_sessions = v);
                                }
                            }
                        }
                        p { class: "mt-1 text-xs text-gray-500", "单用户最多同时登录的设备数" }
                    }
                }
            }
        }
    }
}

/// 登录安全策略部分
#[component]
fn LoginSecuritySection(policy: Signal<SecurityPolicy>) -> Element {
    rsx! {
        div { class: "space-y-6",
            // MFA 设置
            div { class: "border-t border-gray-200 pt-6" },
            h3 { class: "text-sm font-semibold text-gray-700 mb-4", "多因素认证 (MFA)" }
            div { class: "space-y-3",
                div { class: "flex items-center justify-between p-3 bg-gray-50 rounded-lg",
                    div { class: "flex items-center",
                        input {
                            r#type: "checkbox",
                            class: "h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded",
                            checked: policy.read().mfa_enabled,
                            oninput: move |e| policy.with_mut(|p| p.mfa_enabled = e.checked()),
                        }
                        label { class: "ml-3 text-sm text-gray-700", "启用多因素认证" }
                    }
                }
                div { class: "flex items-center justify-between p-3 bg-gray-50 rounded-lg",
                    div { class: "flex items-center",
                        input {
                            r#type: "checkbox",
                            class: "h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded",
                            checked: policy.read().mfa_for_admin,
                            oninput: move |e| policy.with_mut(|p| p.mfa_for_admin = e.checked()),
                        }
                        label { class: "ml-3 text-sm text-gray-700", "管理员强制使用MFA" }
                    }
                }
                div { class: "flex items-center justify-between p-3 bg-gray-50 rounded-lg",
                    div { class: "flex items-center",
                        input {
                            r#type: "checkbox",
                            class: "h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded",
                            checked: policy.read().mfa_for_remote_login,
                            oninput: move |e| policy.with_mut(|p| p.mfa_for_remote_login = e.checked()),
                        }
                        label { class: "ml-3 text-sm text-gray-700", "远程登录强制使用MFA" }
                    }
                }
            }

            // 验证码设置
            div { class: "border-t border-gray-200 pt-6" },
            h3 { class: "text-sm font-semibold text-gray-700 mb-4", "验证码防护" }
            div { class: "space-y-3",
                div { class: "flex items-center justify-between p-3 bg-gray-50 rounded-lg",
                    div { class: "flex items-center",
                        input {
                            r#type: "checkbox",
                            class: "h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded",
                            checked: policy.read().enable_captcha,
                            oninput: move |e| policy.with_mut(|p| p.enable_captcha = e.checked()),
                        }
                        label { class: "ml-3 text-sm text-gray-700", "启用登录验证码" }
                    }
                }
                div {
                    label { class: "block text-sm font-medium text-gray-700 mb-2",
                        "触发验证码的失败次数"
                    }
                    input {
                        r#type: "number",
                        class: "w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500",
                        min: 1,
                        max: 10,
                        value: "{policy.read().captcha_threshold}",
                        oninput: move |e| {
                            if let Ok(v) = e.value().parse::<i32>() {
                                policy.with_mut(|p| p.captcha_threshold = v);
                            }
                        }
                    }
                    p { class: "mt-1 text-xs text-gray-500", "连续失败N次后显示验证码" }
                }
            }

            // IP 白名单
            div { class: "border-t border-gray-200 pt-6" },
            h3 { class: "text-sm font-semibold text-gray-700 mb-4", "IP 访问控制" }
            div { class: "space-y-3",
                div { class: "flex items-center justify-between p-3 bg-gray-50 rounded-lg",
                    div { class: "flex items-center",
                        input {
                            r#type: "checkbox",
                            class: "h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded",
                            checked: policy.read().ip_whitelist_enabled,
                            oninput: move |e| policy.with_mut(|p| p.ip_whitelist_enabled = e.checked()),
                        }
                        label { class: "ml-3 text-sm text-gray-700", "启用IP白名单限制" }
                    }
                }
                div {
                    label { class: "block text-sm font-medium text-gray-700 mb-2",
                        "允许的IP地址（每行一个）"
                    }
                    textarea {
                        class: "w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 font-mono text-sm",
                        rows: 4,
                        placeholder: "192.168.1.100\n10.0.0.0/24",
                        value: "{policy.read().ip_whitelist}",
                        oninput: move |e| policy.with_mut(|p| p.ip_whitelist = e.value()),
                    }
                    p { class: "mt-1 text-xs text-gray-500", "支持单个IP或CIDR格式的网段" }
                }
            }

            // 其他安全设置
            div { class: "border-t border-gray-200 pt-6" },
            h3 { class: "text-sm font-semibold text-gray-700 mb-4", "其他安全设置" }
            div { class: "space-y-3",
                div { class: "flex items-center justify-between p-3 bg-gray-50 rounded-lg",
                    div { class: "flex items-center",
                        input {
                            r#type: "checkbox",
                            class: "h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded",
                            checked: policy.read().password_change_on_first_login,
                            oninput: move |e| policy.with_mut(|p| p.password_change_on_first_login = e.checked()),
                        }
                        label { class: "ml-3 text-sm text-gray-700", "首次登录强制修改密码" }
                    }
                }
                div { class: "grid grid-cols-1 md:grid-cols-2 gap-6",
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-2",
                            "密码定期更换周期（天）"
                        }
                        input {
                            r#type: "number",
                            class: "w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500",
                            min: 0,
                            max: 365,
                            value: "{policy.read().password_change_frequency}",
                            oninput: move |e| {
                                if let Ok(v) = e.value().parse::<i32>() {
                                    policy.with_mut(|p| p.password_change_frequency = v);
                                }
                            }
                        }
                        p { class: "mt-1 text-xs text-gray-500", "设为0表示不强制定期更换" }
                    }
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-2",
                            "审计日志保留天数"
                        }
                        input {
                            r#type: "number",
                            class: "w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500",
                            min: 30,
                            max: 730,
                            value: "{policy.read().audit_log_retention_days}",
                            oninput: move |e| {
                                if let Ok(v) = e.value().parse::<i32>() {
                                    policy.with_mut(|p| p.audit_log_retention_days = v);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
