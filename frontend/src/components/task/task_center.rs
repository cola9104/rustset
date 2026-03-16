use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaPlus, FaPlay, FaPause, FaRotate, FaTrash, FaList,
    FaCircleCheck, FaClock, FaSpinner
};
use crate::components::common::VirtualScroller;

/// 任务状态
#[derive(Clone, Debug, PartialEq)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

impl TaskStatus {
    fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Pending => "等待中",
            TaskStatus::Running => "运行中",
            TaskStatus::Completed => "已完成",
            TaskStatus::Failed => "失败",
        }
    }

    fn color_class(&self) -> &'static str {
        match self {
            TaskStatus::Pending => "bg-yellow-100 text-yellow-800",
            TaskStatus::Running => "bg-blue-100 text-blue-800",
            TaskStatus::Completed => "bg-green-100 text-green-800",
            TaskStatus::Failed => "bg-red-100 text-red-800",
        }
    }
}

/// 任务数据模型
#[derive(Clone, Debug, PartialEq)]
pub struct Task {
    pub id: i32,
    pub name: String,
    pub task_type: String,
    pub status: TaskStatus,
    pub progress: u8,
    pub created_at: String,
    pub target: String,
}

/// 任务中心页面
#[allow(non_snake_case)]
pub fn TaskCenter() -> Element {
    let mut tasks = use_signal(|| vec![
        Task {
            id: 1,
            name: "服务器端口扫描".to_string(),
            task_type: "端口扫描".to_string(),
            status: TaskStatus::Completed,
            progress: 100,
            created_at: "2024-01-15 10:30".to_string(),
            target: "192.168.1.0/24".to_string(),
        },
        Task {
            id: 2,
            name: "漏洞扫描任务".to_string(),
            task_type: "漏洞扫描".to_string(),
            status: TaskStatus::Running,
            progress: 65,
            created_at: "2024-01-15 14:00".to_string(),
            target: "192.168.1.10-20".to_string(),
        },
        Task {
            id: 3,
            name: "资产发现任务".to_string(),
            task_type: "资产发现".to_string(),
            status: TaskStatus::Pending,
            progress: 0,
            created_at: "2024-01-15 15:30".to_string(),
            target: "10.0.0.0/16".to_string(),
        },
        Task {
            id: 4,
            name: "Web应用扫描".to_string(),
            task_type: "Web扫描".to_string(),
            status: TaskStatus::Failed,
            progress: 30,
            created_at: "2024-01-14 09:00".to_string(),
            target: "https://example.com".to_string(),
        },
    ]);

    let mut show_add_modal = use_signal(|| false);

    // 统计数据
    let total_count = tasks.read().len() as i32;
    let running_count = tasks.read().iter().filter(|t| t.status == TaskStatus::Running).count() as i32;
    let completed_count = tasks.read().iter().filter(|t| t.status == TaskStatus::Completed).count() as i32;
    let pending_count = tasks.read().iter().filter(|t| t.status == TaskStatus::Pending).count() as i32;

    rsx! {
        div { class: "space-y-6",
            // 页面标题和操作栏
            div { class: "flex justify-between items-center",
                h1 { class: "text-2xl font-bold text-gray-800", "任务中心" }
                button {
                    class: "flex items-center px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors",
                    onclick: move |_| show_add_modal.set(true),
                    Icon { icon: FaPlus, width: 16, height: 16 }
                    span { class: "ml-2", "新建任务" }
                }
            }

            // 统计卡片
            div { class: "grid grid-cols-1 md:grid-cols-4 gap-4",
                // 总任务
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-blue-500",
                            Icon { icon: FaList, width: 20, height: 20 }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "总任务" }
                            p { class: "text-xl font-bold text-gray-800", {total_count.to_string()} }
                        }
                    }
                }
                // 运行中
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-blue-500 animate-pulse",
                            Icon { icon: FaSpinner, width: 20, height: 20 }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "运行中" }
                            p { class: "text-xl font-bold text-gray-800", {running_count.to_string()} }
                        }
                    }
                }
                // 已完成
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-green-500",
                            Icon { icon: FaCircleCheck, width: 20, height: 20 }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "已完成" }
                            p { class: "text-xl font-bold text-gray-800", {completed_count.to_string()} }
                        }
                    }
                }
                // 等待中
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-yellow-500",
                            Icon { icon: FaClock, width: 20, height: 20 }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "等待中" }
                            p { class: "text-xl font-bold text-gray-800", {pending_count.to_string()} }
                        }
                    }
                }
            }

            // 任务列表 - 使用虚拟滚动（分页版本）
            VirtualScroller {
                items: tasks.read().to_vec(),
                page_size: 20,
                render_item: move |task: Task| rsx! {
                    div { class: "flex items-center px-6 py-4",
                        // 任务名称
                        div { class: "flex-1 text-sm font-medium text-gray-900 whitespace-nowrap",
                            {task.name.clone()}
                        }
                        // 类型
                        div { class: "flex-1 whitespace-nowrap",
                            span { class: "px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-gray-100 text-gray-800",
                                {task.task_type.clone()}
                            }
                        }
                        // 目标
                        div { class: "flex-1 text-sm text-gray-500 whitespace-nowrap",
                            {task.target.clone()}
                        }
                        // 状态
                        div { class: "flex-1 whitespace-nowrap",
                            span {
                                class: "px-2 inline-flex text-xs leading-5 font-semibold rounded-full {task.status.color_class()}",
                                {task.status.as_str()}
                            }
                        }
                        // 进度
                        div { class: "flex-1 flex items-center whitespace-nowrap",
                            div { class: "w-full bg-gray-200 rounded-full h-2",
                                div {
                                    class: "bg-blue-600 h-2 rounded-full",
                                    style: "width: {task.progress}%",
                                }
                            }
                            span { class: "ml-2 text-sm text-gray-500", "{task.progress}%" }
                        }
                        // 创建时间
                        div { class: "flex-1 text-sm text-gray-500 whitespace-nowrap",
                            {task.created_at.clone()}
                        }
                        // 操作
                        div { class: "flex-1 text-sm font-medium whitespace-nowrap",
                            // 根据状态显示不同操作按钮
                            if task.status == TaskStatus::Pending {
                                button {
                                    class: "text-green-600 hover:text-green-900 mr-2",
                                    title: "启动",
                                    Icon { icon: FaPlay, width: 16, height: 16 }
                                }
                            }
                            if task.status == TaskStatus::Running {
                                button {
                                    class: "text-yellow-600 hover:text-yellow-900 mr-2",
                                    title: "暂停",
                                    Icon { icon: FaPause, width: 16, height: 16 }
                                }
                            }
                            if task.status == TaskStatus::Completed || task.status == TaskStatus::Failed {
                                button {
                                    class: "text-blue-600 hover:text-blue-900 mr-2",
                                    title: "重新运行",
                                    Icon { icon: FaRotate, width: 16, height: 16 }
                                }
                            }
                            button {
                                class: "text-red-600 hover:text-red-900",
                                title: "删除",
                                Icon { icon: FaTrash, width: 16, height: 16 }
                            }
                        }
                    }
                }
            }
        }

        // 添加任务模态框
        if *show_add_modal.read() {
            AddTaskModal {
                on_close: move |_| show_add_modal.set(false),
                on_save: move |new_task: Task| {
                    tasks.write().push(new_task);
                    show_add_modal.set(false);
                }
            }
        }
    }
}

/// 添加任务模态框
#[component]
fn AddTaskModal(on_close: EventHandler<()>, on_save: EventHandler<Task>) -> Element {
    let mut name = use_signal(String::new);
    let mut task_type = use_signal(|| "端口扫描".to_string());
    let mut target = use_signal(String::new);

    rsx! {
        div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            div { class: "bg-white rounded-lg shadow-xl w-full max-w-md mx-4",
                div { class: "flex justify-between items-center p-4 border-b",
                    h3 { class: "text-lg font-semibold", "新建任务" }
                    button {
                        class: "text-gray-400 hover:text-gray-600",
                        onclick: move |_| on_close.call(()),
                        "×"
                    }
                }

                div { class: "p-4 space-y-4",
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "任务名称" }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            placeholder: "输入任务名称",
                            value: name,
                            oninput: move |e| name.set(e.value()),
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "任务类型" }
                        select {
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: task_type,
                            onchange: move |e| task_type.set(e.value()),
                            option { value: "端口扫描", "端口扫描" }
                            option { value: "漏洞扫描", "漏洞扫描" }
                            option { value: "资产发现", "资产发现" }
                            option { value: "Web扫描", "Web扫描" }
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "目标" }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
                            placeholder: "例如: 192.168.1.0/24 或 https://example.com",
                            value: target,
                            oninput: move |e| target.set(e.value()),
                        }
                    }
                }

                div { class: "flex justify-end space-x-3 p-4 border-t",
                    button {
                        class: "px-4 py-2 text-gray-600 border border-gray-300 rounded-md hover:bg-gray-50",
                        onclick: move |_| on_close.call(()),
                        "取消"
                    }
                    button {
                        class: "px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700",
                        onclick: move |_| {
                            let new_task = Task {
                                id: chrono::Utc::now().timestamp() as i32,
                                name: name.read().clone(),
                                task_type: task_type.read().clone(),
                                status: TaskStatus::Pending,
                                progress: 0,
                                created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M").to_string(),
                                target: target.read().clone(),
                            };
                            on_save.call(new_task);
                        },
                        "创建任务"
                    }
                }
            }
        }
    }
}
