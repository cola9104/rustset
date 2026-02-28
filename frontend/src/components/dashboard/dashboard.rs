use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaServer, FaCircleCheck, FaList, FaTriangleExclamation
};

/// 仪表板页面
#[allow(non_snake_case)]
pub fn Dashboard() -> Element {
    rsx! {
        div { class: "space-y-6",
            // 页面标题
            h1 { class: "text-2xl font-bold text-gray-800", "仪表板" }

            // 统计卡片
            div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6",
                // 资产总数
                div { class: "bg-white rounded-lg shadow p-6",
                    div { class: "flex items-center",
                        div { class: "p-3 rounded-full bg-blue-500",
                            Icon { icon: FaServer, width: 24, height: 24 }
                        }
                        div { class: "ml-4",
                            p { class: "text-sm text-gray-500", "资产总数" }
                            p { class: "text-2xl font-bold text-gray-800", "0" }
                        }
                    }
                }
                // 在线资产
                div { class: "bg-white rounded-lg shadow p-6",
                    div { class: "flex items-center",
                        div { class: "p-3 rounded-full bg-green-500",
                            Icon { icon: FaCircleCheck, width: 24, height: 24 }
                        }
                        div { class: "ml-4",
                            p { class: "text-sm text-gray-500", "在线资产" }
                            p { class: "text-2xl font-bold text-gray-800", "0" }
                        }
                    }
                }
                // 任务数量
                div { class: "bg-white rounded-lg shadow p-6",
                    div { class: "flex items-center",
                        div { class: "p-3 rounded-full bg-yellow-500",
                            Icon { icon: FaList, width: 24, height: 24 }
                        }
                        div { class: "ml-4",
                            p { class: "text-sm text-gray-500", "任务数量" }
                            p { class: "text-2xl font-bold text-gray-800", "0" }
                        }
                    }
                }
                // 风险数量
                div { class: "bg-white rounded-lg shadow p-6",
                    div { class: "flex items-center",
                        div { class: "p-3 rounded-full bg-red-500",
                            Icon { icon: FaTriangleExclamation, width: 24, height: 24 }
                        }
                        div { class: "ml-4",
                            p { class: "text-sm text-gray-500", "风险数量" }
                            p { class: "text-2xl font-bold text-gray-800", "0" }
                        }
                    }
                }
            }

            // 系统信息
            div { class: "bg-white rounded-lg shadow p-6",
                h2 { class: "text-lg font-semibold text-gray-800 mb-4", "系统信息" }
                div { class: "space-y-3",
                    div { class: "flex justify-between",
                        span { class: "text-gray-600", "版本" }
                        span { class: "text-gray-800", "v1.0.0" }
                    }
                    div { class: "flex justify-between",
                        span { class: "text-gray-600", "运行状态" }
                        span { class: "text-gray-800", "正常" }
                    }
                    div { class: "flex justify-between",
                        span { class: "text-gray-600", "数据库" }
                        span { class: "text-gray-800", "已连接" }
                    }
                }
            }
        }
    }
}
