use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaFileLines, FaMagnifyingGlass, FaFilter, FaDownload,
    FaUser, FaShield
};

/// 审计日志数据模型
#[derive(Clone, Debug, PartialEq)]
pub struct AuditLog {
    pub id: i32,
    pub timestamp: String,
    pub user: String,
    pub action: String,
    pub module: String,
    pub target: String,
    pub ip_address: String,
    pub status: String,
    pub details: String,
}

/// 审计日志页面
#[allow(non_snake_case)]
pub fn AuditLogs() -> Element {
    let logs = use_signal(|| vec![
        AuditLog {
            id: 1,
            timestamp: "2024-01-15 14:30:25".to_string(),
            user: "admin".to_string(),
            action: "登录".to_string(),
            module: "认证".to_string(),
            target: "系统".to_string(),
            ip_address: "192.168.1.100".to_string(),
            status: "成功".to_string(),
            details: "用户登录系统".to_string(),
        },
        AuditLog {
            id: 2,
            timestamp: "2024-01-15 14:32:10".to_string(),
            user: "admin".to_string(),
            action: "创建".to_string(),
            module: "资产管理".to_string(),
            target: "服务器-03".to_string(),
            ip_address: "192.168.1.100".to_string(),
            status: "成功".to_string(),
            details: "新增资产: 服务器-03 (192.168.1.13)".to_string(),
        },
        AuditLog {
            id: 3,
            timestamp: "2024-01-15 14:35:00".to_string(),
            user: "admin".to_string(),
            action: "扫描".to_string(),
            module: "任务中心".to_string(),
            target: "端口扫描任务".to_string(),
            ip_address: "192.168.1.100".to_string(),
            status: "成功".to_string(),
            details: "启动端口扫描任务，目标: 192.168.1.0/24".to_string(),
        },
        AuditLog {
            id: 4,
            timestamp: "2024-01-15 14:40:15".to_string(),
            user: "张三".to_string(),
            action: "更新".to_string(),
            module: "风险中心".to_string(),
            target: "风险#1024".to_string(),
            ip_address: "192.168.1.101".to_string(),
            status: "成功".to_string(),
            details: "更新风险状态: 待处理 -> 处理中".to_string(),
        },
        AuditLog {
            id: 5,
            timestamp: "2024-01-15 14:45:30".to_string(),
            user: "张三".to_string(),
            action: "登录".to_string(),
            module: "认证".to_string(),
            target: "系统".to_string(),
            ip_address: "192.168.1.101".to_string(),
            status: "成功".to_string(),
            details: "用户登录系统".to_string(),
        },
        AuditLog {
            id: 6,
            timestamp: "2024-01-15 13:20:00".to_string(),
            user: "unknown".to_string(),
            action: "登录".to_string(),
            module: "认证".to_string(),
            target: "系统".to_string(),
            ip_address: "10.0.0.50".to_string(),
            status: "失败".to_string(),
            details: "登录失败: 密码错误".to_string(),
        },
    ]);

    let mut search_query = use_signal(String::new);
    let mut module_filter = use_signal(|| "".to_string());

    // 统计数据
    let total_count = logs.read().len() as i32;
    let login_count = logs.read().iter().filter(|l| l.action == "登录").count() as i32;
    let failed_count = logs.read().iter().filter(|l| l.status == "失败").count() as i32;

    // 过滤日志
    let filtered_logs: Vec<AuditLog> = logs.read()
        .iter()
        .filter(|log| {
            let query = search_query.read().to_lowercase();
            let module = module_filter.read();
            let query_match = query.is_empty() ||
                log.user.to_lowercase().contains(&query) ||
                log.target.to_lowercase().contains(&query) ||
                log.details.to_lowercase().contains(&query);
            let module_match = module.is_empty() || log.module == *module;
            query_match && module_match
        })
        .cloned()
        .collect();

    let is_empty = filtered_logs.is_empty();

    rsx! {
        div { class: "space-y-6",
            // 页面标题和操作栏
            div { class: "flex justify-between items-center",
                h1 { class: "text-2xl font-bold text-gray-800", "审计日志" }
                button {
                    class: "flex items-center px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700 transition-colors",
                    Icon { icon: FaDownload, width: 16, height: 16 }
                    span { class: "ml-2", "导出日志" }
                }
            }

            // 统计卡片
            div { class: "grid grid-cols-1 md:grid-cols-3 gap-4",
                // 总日志
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-blue-500",
                            Icon { icon: FaFileLines, width: 20, height: 20 }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "总日志数" }
                            p { class: "text-xl font-bold text-gray-800", {total_count.to_string()} }
                        }
                    }
                }
                // 登录次数
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-green-500",
                            Icon { icon: FaUser, width: 20, height: 20 }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "登录次数" }
                            p { class: "text-xl font-bold text-gray-800", {login_count.to_string()} }
                        }
                    }
                }
                // 失败操作
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-red-500",
                            Icon { icon: FaShield, width: 20, height: 20 }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "失败操作" }
                            p { class: "text-xl font-bold text-red-600", {failed_count.to_string()} }
                        }
                    }
                }
            }

            // 搜索和筛选
            div { class: "bg-white rounded-lg shadow p-4",
                div { class: "flex items-center space-x-4",
                    // 搜索框
                    div { class: "flex-1 flex items-center",
                        Icon { icon: FaMagnifyingGlass, width: 20, height: 20 }
                        input {
                            r#type: "text",
                            class: "ml-2 flex-1 border-0 focus:outline-none",
                            placeholder: "搜索用户、目标或详情...",
                            value: search_query,
                            oninput: move |e| search_query.set(e.value()),
                        }
                    }

                    // 模块筛选
                    div { class: "flex items-center",
                        Icon { icon: FaFilter, width: 20, height: 20 }
                        select {
                            class: "ml-2 px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: module_filter,
                            onchange: move |e| module_filter.set(e.value()),
                            option { value: "", "全部模块" }
                            option { value: "认证", "认证" }
                            option { value: "资产管理", "资产管理" }
                            option { value: "任务中心", "任务中心" }
                            option { value: "风险中心", "风险中心" }
                            option { value: "系统配置", "系统配置" }
                        }
                    }
                }
            }

            // 日志列表
            div { class: "bg-white rounded-lg shadow overflow-hidden",
                table { class: "min-w-full divide-y divide-gray-200",
                    thead { class: "bg-gray-50",
                        tr {
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "时间" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "用户" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "操作" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "模块" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "目标" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "IP地址" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "状态" }
                        }
                    }
                    tbody { class: "bg-white divide-y divide-gray-200",
                        for log in filtered_logs.iter() {
                            tr { class: "hover:bg-gray-50",
                                td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-500",
                                    {log.timestamp.clone()}
                                }
                                td { class: "px-6 py-4 whitespace-nowrap",
                                    div { class: "flex items-center",
                                        div { class: "flex-shrink-0 h-8 w-8 bg-gray-100 rounded-full flex items-center justify-center",
                                            Icon { icon: FaUser, width: 16, height: 16 }
                                        }
                                        div { class: "ml-2 text-sm font-medium text-gray-900",
                                            {log.user.clone()}
                                        }
                                    }
                                }
                                td { class: "px-6 py-4 whitespace-nowrap",
                                    span { class: "px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-blue-100 text-blue-800",
                                        {log.action.clone()}
                                    }
                                }
                                td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-500",
                                    {log.module.clone()}
                                }
                                td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-500",
                                    {log.target.clone()}
                                }
                                td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-500",
                                    {log.ip_address.clone()}
                                }
                                td { class: "px-6 py-4 whitespace-nowrap",
                                    span {
                                        class: if log.status == "成功" {
                                            "px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-green-100 text-green-800"
                                        } else {
                                            "px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-red-100 text-red-800"
                                        },
                                        {log.status.clone()}
                                    }
                                }
                            }
                        }
                    }
                }

                // 空状态
                if is_empty {
                    div { class: "text-center py-12 text-gray-500",
                        "没有找到匹配的日志记录"
                    }
                }
            }

            // 分页（简化版）
            div { class: "bg-white rounded-lg shadow px-4 py-3 flex items-center justify-between",
                div { class: "text-sm text-gray-500",
                    "显示 {filtered_logs.len()} 条记录"
                }
                div { class: "flex space-x-2",
                    button {
                        class: "px-3 py-1 border border-gray-300 rounded-md text-sm hover:bg-gray-50",
                        "上一页"
                    }
                    button {
                        class: "px-3 py-1 bg-blue-600 text-white rounded-md text-sm",
                        "1"
                    }
                    button {
                        class: "px-3 py-1 border border-gray-300 rounded-md text-sm hover:bg-gray-50",
                        "下一页"
                    }
                }
            }
        }
    }
}
