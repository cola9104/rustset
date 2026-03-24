use crate::services::dashboard_api::{fetch_dashboard_summary, DashboardSummary};
use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaCircleCheck, FaDatabase, FaList, FaServer, FaTriangleExclamation, FaUsers,
};
use dioxus_free_icons::Icon;

#[allow(non_snake_case)]
pub fn Dashboard() -> Element {
    let summary = use_signal(|| Option::<DashboardSummary>::None);
    let loading = use_signal(|| true);
    let error = use_signal(String::new);

    {
        let mut summary = summary;
        let mut loading = loading;
        let mut error = error;
        use_effect(move || {
            spawn(async move {
                match fetch_dashboard_summary().await {
                    Ok(data) => {
                        summary.set(Some(data));
                        error.set(String::new());
                    }
                    Err(err) => {
                        error.set(err);
                    }
                }
                loading.set(false);
            });
        });
    }

    let data = summary.read().clone();

    rsx! {
        div { class: "space-y-6",
            h1 { class: "text-2xl font-bold text-gray-800", "仪表板" }

            if *loading.read() {
                div { class: "bg-white rounded-lg shadow p-6 text-gray-500", "正在加载仪表板数据..." }
            } else if !error.read().is_empty() {
                div { class: "bg-red-50 border border-red-200 text-red-700 rounded-lg p-4", "{error}" }
            } else if let Some(summary) = data {
                div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6",
                    div { class: "bg-white rounded-lg shadow p-6",
                        div { class: "flex items-center",
                            div { class: "p-3 rounded-full bg-blue-500",
                                Icon { icon: FaServer, width: 24, height: 24, class: "text-white" }
                            }
                            div { class: "ml-4",
                                p { class: "text-sm text-gray-500", "资产总数" }
                                p { class: "text-2xl font-bold text-gray-800", "{summary.asset_count}" }
                                p { class: "text-xs text-gray-400 mt-1", "已扫描 {summary.scanned_asset_count} 台" }
                            }
                        }
                    }
                    div { class: "bg-white rounded-lg shadow p-6",
                        div { class: "flex items-center",
                            div { class: "p-3 rounded-full bg-emerald-500",
                                Icon { icon: FaUsers, width: 24, height: 24, class: "text-white" }
                            }
                            div { class: "ml-4",
                                p { class: "text-sm text-gray-500", "活跃用户" }
                                p { class: "text-2xl font-bold text-gray-800", "{summary.active_user_count}" }
                                p { class: "text-xs text-gray-400 mt-1", "总用户 {summary.user_count}" }
                            }
                        }
                    }
                    div { class: "bg-white rounded-lg shadow p-6",
                        div { class: "flex items-center",
                            div { class: "p-3 rounded-full bg-amber-500",
                                Icon { icon: FaList, width: 24, height: 24, class: "text-white" }
                            }
                            div { class: "ml-4",
                                p { class: "text-sm text-gray-500", "运行任务" }
                                p { class: "text-2xl font-bold text-gray-800", "{summary.running_task_count}" }
                                p { class: "text-xs text-gray-400 mt-1", "总任务 {summary.task_count}" }
                            }
                        }
                    }
                    div { class: "bg-white rounded-lg shadow p-6",
                        div { class: "flex items-center",
                            div { class: "p-3 rounded-full bg-red-500",
                                Icon { icon: FaTriangleExclamation, width: 24, height: 24, class: "text-white" }
                            }
                            div { class: "ml-4",
                                p { class: "text-sm text-gray-500", "待处理风险" }
                                p { class: "text-2xl font-bold text-gray-800", "{summary.open_risk_count}" }
                                p { class: "text-xs text-gray-400 mt-1", "总风险 {summary.risk_count}" }
                            }
                        }
                    }
                }

                div { class: "grid grid-cols-1 lg:grid-cols-2 gap-6",
                    div { class: "bg-white rounded-lg shadow p-6",
                        h2 { class: "text-lg font-semibold text-gray-800 mb-4", "系统状态" }
                        div { class: "space-y-4",
                            StatusRow {
                                label: "应用版本".to_string(),
                                value: summary.version.clone(),
                                icon_name: "version".to_string(),
                                value_class: "text-gray-900".to_string(),
                            }
                            StatusRow {
                                label: "数据库连接".to_string(),
                                value: if summary.database_connected { "正常".to_string() } else { "异常".to_string() },
                                icon_name: "database".to_string(),
                                value_class: if summary.database_connected { "text-emerald-600".to_string() } else { "text-red-600".to_string() },
                            }
                        }
                    }

                    div { class: "bg-white rounded-lg shadow p-6",
                        h2 { class: "text-lg font-semibold text-gray-800 mb-4", "当前概览" }
                        ul { class: "space-y-3 text-sm text-gray-600",
                            li { "资源、任务、风险、用户均已切换为后端实时汇总数据。" }
                            li { "仪表板不再展示固定 0 和写死的系统状态。" }
                            li { "当前数据来自 `/api/dashboard-summary` 聚合接口。" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn StatusRow(label: String, value: String, icon_name: String, value_class: String) -> Element {
    rsx! {
        div { class: "flex items-center justify-between",
            div { class: "flex items-center gap-3 text-gray-600",
                if icon_name == "database" {
                    Icon { icon: FaDatabase, width: 16, height: 16 }
                } else {
                    Icon { icon: FaCircleCheck, width: 16, height: 16 }
                }
                span { "{label}" }
            }
            span { class: "font-medium {value_class}", "{value}" }
        }
    }
}
