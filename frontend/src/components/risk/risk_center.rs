use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaTriangleExclamation, FaCircleCheck, FaClock, FaFilter,
    FaEye, FaCheck, FaXmark
};

/// 风险等级
#[derive(Clone, Debug, PartialEq)]
pub enum RiskLevel {
    Critical,
    High,
    Medium,
    Low,
}

impl RiskLevel {
    fn as_str(&self) -> &'static str {
        match self {
            RiskLevel::Critical => "严重",
            RiskLevel::High => "高危",
            RiskLevel::Medium => "中危",
            RiskLevel::Low => "低危",
        }
    }

    fn color_class(&self) -> &'static str {
        match self {
            RiskLevel::Critical => "bg-red-100 text-red-800 border-red-300",
            RiskLevel::High => "bg-orange-100 text-orange-800 border-orange-300",
            RiskLevel::Medium => "bg-yellow-100 text-yellow-800 border-yellow-300",
            RiskLevel::Low => "bg-blue-100 text-blue-800 border-blue-300",
        }
    }
}

/// 风险状态
#[derive(Clone, Debug, PartialEq)]
pub enum RiskStatus {
    Open,
    InProgress,
    Resolved,
    Ignored,
}

impl RiskStatus {
    fn as_str(&self) -> &'static str {
        match self {
            RiskStatus::Open => "待处理",
            RiskStatus::InProgress => "处理中",
            RiskStatus::Resolved => "已解决",
            RiskStatus::Ignored => "已忽略",
        }
    }

    fn color_class(&self) -> &'static str {
        match self {
            RiskStatus::Open => "bg-red-50 text-red-700",
            RiskStatus::InProgress => "bg-blue-50 text-blue-700",
            RiskStatus::Resolved => "bg-green-50 text-green-700",
            RiskStatus::Ignored => "bg-gray-50 text-gray-700",
        }
    }
}

/// 风险数据模型
#[derive(Clone, Debug, PartialEq)]
pub struct Risk {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub risk_type: String,
    pub level: RiskLevel,
    pub status: RiskStatus,
    pub asset_name: String,
    pub discovered_at: String,
    pub assignee: Option<String>,
}

/// 风险中心页面
#[allow(non_snake_case)]
pub fn RiskCenter() -> Element {
    let risks = use_signal(|| vec![
        Risk {
            id: 1,
            title: "SSH 弱密码".to_string(),
            description: "检测到服务器使用弱密码".to_string(),
            risk_type: "弱口令".to_string(),
            level: RiskLevel::Critical,
            status: RiskStatus::Open,
            asset_name: "服务器-01".to_string(),
            discovered_at: "2024-01-15 10:30".to_string(),
            assignee: None,
        },
        Risk {
            id: 2,
            title: "开放高危端口 445".to_string(),
            description: "SMB端口对外开放，存在勒索病毒风险".to_string(),
            risk_type: "端口风险".to_string(),
            level: RiskLevel::High,
            status: RiskStatus::InProgress,
            asset_name: "服务器-02".to_string(),
            discovered_at: "2024-01-15 11:00".to_string(),
            assignee: Some("张三".to_string()),
        },
        Risk {
            id: 3,
            title: "未安装安全补丁 KB5034441".to_string(),
            description: "Windows系统缺少重要安全更新".to_string(),
            risk_type: "补丁缺失".to_string(),
            level: RiskLevel::Medium,
            status: RiskStatus::Open,
            asset_name: "工作站-01".to_string(),
            discovered_at: "2024-01-14 15:20".to_string(),
            assignee: None,
        },
        Risk {
            id: 4,
            title: "FTP服务未加密".to_string(),
            description: "FTP服务未配置TLS加密，数据传输不安全".to_string(),
            risk_type: "配置风险".to_string(),
            level: RiskLevel::Low,
            status: RiskStatus::Resolved,
            asset_name: "服务器-03".to_string(),
            discovered_at: "2024-01-13 09:00".to_string(),
            assignee: Some("李四".to_string()),
        },
    ]);

    let mut level_filter = use_signal(|| None::<RiskLevel>);
    let mut status_filter = use_signal(|| None::<RiskStatus>);
    let mut selected_risk = use_signal(|| None::<Risk>);

    // 统计数据
    let critical_count = risks.read().iter().filter(|r| r.level == RiskLevel::Critical).count() as i32;
    let high_count = risks.read().iter().filter(|r| r.level == RiskLevel::High).count() as i32;
    let open_count = risks.read().iter().filter(|r| r.status == RiskStatus::Open).count() as i32;
    let resolved_count = risks.read().iter().filter(|r| r.status == RiskStatus::Resolved).count() as i32;

    // 过滤风险
    let filtered_risks: Vec<Risk> = risks.read()
        .iter()
        .filter(|risk| {
            let level_match = level_filter.read().as_ref().map_or(true, |l| risk.level == *l);
            let status_match = status_filter.read().as_ref().map_or(true, |s| risk.status == *s);
            level_match && status_match
        })
        .cloned()
        .collect();

    let is_empty = filtered_risks.is_empty();

    rsx! {
        div { class: "space-y-6",
            // 页面标题
            div { class: "flex justify-between items-center",
                h1 { class: "text-2xl font-bold text-gray-800", "风险中心" }
            }

            // 统计卡片
            div { class: "grid grid-cols-1 md:grid-cols-4 gap-4",
                // 严重
                div { class: "bg-white rounded-lg shadow p-4 border-l-4 border-red-500",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-red-100",
                            Icon { icon: FaTriangleExclamation, width: 20, height: 20 }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "严重风险" }
                            p { class: "text-xl font-bold text-red-600", {critical_count.to_string()} }
                        }
                    }
                }
                // 高危
                div { class: "bg-white rounded-lg shadow p-4 border-l-4 border-orange-500",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-orange-100",
                            Icon { icon: FaTriangleExclamation, width: 20, height: 20 }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "高危风险" }
                            p { class: "text-xl font-bold text-orange-600", {high_count.to_string()} }
                        }
                    }
                }
                // 待处理
                div { class: "bg-white rounded-lg shadow p-4 border-l-4 border-blue-500",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-blue-100",
                            Icon { icon: FaClock, width: 20, height: 20 }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "待处理" }
                            p { class: "text-xl font-bold text-blue-600", {open_count.to_string()} }
                        }
                    }
                }
                // 已解决
                div { class: "bg-white rounded-lg shadow p-4 border-l-4 border-green-500",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-green-100",
                            Icon { icon: FaCircleCheck, width: 20, height: 20 }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "已解决" }
                            p { class: "text-xl font-bold text-green-600", {resolved_count.to_string()} }
                        }
                    }
                }
            }

            // 筛选器
            div { class: "bg-white rounded-lg shadow p-4",
                div { class: "flex items-center space-x-4",
                    Icon { icon: FaFilter, width: 20, height: 20 }
                    span { class: "text-gray-600", "筛选:" }

                    // 风险等级筛选
                    select {
                        class: "px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                        onchange: move |e| {
                            let val = e.value();
                            *level_filter.write() = match val.as_str() {
                                "critical" => Some(RiskLevel::Critical),
                                "high" => Some(RiskLevel::High),
                                "medium" => Some(RiskLevel::Medium),
                                "low" => Some(RiskLevel::Low),
                                _ => None,
                            };
                        },
                        option { value: "", "全部等级" }
                        option { value: "critical", "严重" }
                        option { value: "high", "高危" }
                        option { value: "medium", "中危" }
                        option { value: "low", "低危" }
                    }

                    // 状态筛选
                    select {
                        class: "px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                        onchange: move |e| {
                            let val = e.value();
                            *status_filter.write() = match val.as_str() {
                                "open" => Some(RiskStatus::Open),
                                "progress" => Some(RiskStatus::InProgress),
                                "resolved" => Some(RiskStatus::Resolved),
                                "ignored" => Some(RiskStatus::Ignored),
                                _ => None,
                            };
                        },
                        option { value: "", "全部状态" }
                        option { value: "open", "待处理" }
                        option { value: "progress", "处理中" }
                        option { value: "resolved", "已解决" }
                        option { value: "ignored", "已忽略" }
                    }
                }
            }

            // 风险列表
            div { class: "bg-white rounded-lg shadow overflow-hidden",
                table { class: "min-w-full divide-y divide-gray-200",
                    thead { class: "bg-gray-50",
                        tr {
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "风险标题" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "类型" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "等级" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "状态" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "资产" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "发现时间" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "操作" }
                        }
                    }
                    tbody { class: "bg-white divide-y divide-gray-200",
                        for risk in filtered_risks.iter() {
                            tr { class: "hover:bg-gray-50",
                                td { class: "px-6 py-4",
                                    div { class: "text-sm font-medium text-gray-900", {risk.title.clone()} }
                                    div { class: "text-sm text-gray-500", {risk.description.clone()} }
                                }
                                td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-500",
                                    {risk.risk_type.clone()}
                                }
                                td { class: "px-6 py-4 whitespace-nowrap",
                                    span {
                                        class: "px-2 py-1 text-xs font-semibold rounded-full border {risk.level.color_class()}",
                                        {risk.level.as_str()}
                                    }
                                }
                                td { class: "px-6 py-4 whitespace-nowrap",
                                    span {
                                        class: "px-2 py-1 text-xs font-semibold rounded-full {risk.status.color_class()}",
                                        {risk.status.as_str()}
                                    }
                                }
                                td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-500",
                                    {risk.asset_name.clone()}
                                }
                                td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-500",
                                    {risk.discovered_at.clone()}
                                }
                                td { class: "px-6 py-4 whitespace-nowrap text-sm font-medium",
                                    button {
                                        class: "text-blue-600 hover:text-blue-900 mr-2",
                                        title: "查看详情",
                                        onclick: {
                                            let risk = risk.clone();
                                            move |_| selected_risk.set(Some(risk.clone()))
                                        },
                                        Icon { icon: FaEye, width: 16, height: 16 }
                                    }
                                    if risk.status == RiskStatus::Open {
                                        button {
                                            class: "text-green-600 hover:text-green-900 mr-2",
                                            title: "标记解决",
                                            Icon { icon: FaCheck, width: 16, height: 16 }
                                        }
                                    }
                                    if risk.status != RiskStatus::Ignored {
                                        button {
                                            class: "text-gray-600 hover:text-gray-900",
                                            title: "忽略",
                                            Icon { icon: FaXmark, width: 16, height: 16 }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // 空状态
                if is_empty {
                    div { class: "text-center py-12 text-gray-500",
                        "没有找到匹配的风险"
                    }
                }
            }
        }

        // 风险详情模态框
        if let Some(risk) = selected_risk.read().clone() {
            RiskDetailModal {
                risk: risk.clone(),
                on_close: move |_| selected_risk.set(None),
            }
        }
    }
}

/// 风险详情模态框
#[component]
fn RiskDetailModal(risk: Risk, on_close: EventHandler<()>) -> Element {
    rsx! {
        div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            div { class: "bg-white rounded-lg shadow-xl w-full max-w-lg mx-4",
                div { class: "flex justify-between items-center p-4 border-b",
                    h3 { class: "text-lg font-semibold", "风险详情" }
                    button {
                        class: "text-gray-400 hover:text-gray-600",
                        onclick: move |_| on_close.call(()),
                        "×"
                    }
                }

                div { class: "p-4 space-y-4",
                    div {
                        label { class: "block text-sm font-medium text-gray-500 mb-1", "标题" }
                        p { class: "text-gray-900", {risk.title.clone()} }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-500 mb-1", "描述" }
                        p { class: "text-gray-900", {risk.description.clone()} }
                    }

                    div { class: "grid grid-cols-2 gap-4",
                        div {
                            label { class: "block text-sm font-medium text-gray-500 mb-1", "风险类型" }
                            p { class: "text-gray-900", {risk.risk_type.clone()} }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-500 mb-1", "风险等级" }
                            span {
                                class: "px-2 py-1 text-xs font-semibold rounded-full border {risk.level.color_class()}",
                                {risk.level.as_str()}
                            }
                        }
                    }

                    div { class: "grid grid-cols-2 gap-4",
                        div {
                            label { class: "block text-sm font-medium text-gray-500 mb-1", "状态" }
                            span {
                                class: "px-2 py-1 text-xs font-semibold rounded-full {risk.status.color_class()}",
                                {risk.status.as_str()}
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-500 mb-1", "关联资产" }
                            p { class: "text-gray-900", {risk.asset_name.clone()} }
                        }
                    }

                    div { class: "grid grid-cols-2 gap-4",
                        div {
                            label { class: "block text-sm font-medium text-gray-500 mb-1", "发现时间" }
                            p { class: "text-gray-900", {risk.discovered_at.clone()} }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-500 mb-1", "处理人" }
                            p { class: "text-gray-900",
                                {risk.assignee.clone().unwrap_or_else(|| "未分配".to_string())}
                            }
                        }
                    }
                }

                div { class: "flex justify-end p-4 border-t",
                    button {
                        class: "px-4 py-2 bg-gray-600 text-white rounded-md hover:bg-gray-700",
                        onclick: move |_| on_close.call(()),
                        "关闭"
                    }
                }
            }
        }
    }
}
