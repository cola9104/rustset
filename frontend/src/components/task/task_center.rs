use crate::components::common::VirtualScroller;
use crate::services::task_api::{
    create_task, delete_task, fetch_tasks, CreateTaskPayload, TaskRecord,
};
use crate::state::user_role::use_auth;
use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaArrowRotateRight, FaCircleCheck, FaClock, FaList, FaPlus, FaSpinner, FaTrash,
};
use dioxus_free_icons::Icon;

#[allow(non_snake_case)]
pub fn TaskCenter() -> Element {
    let auth = use_auth();
    let current_auth = auth.read().clone();
    let tasks = use_signal(Vec::<TaskRecord>::new);
    let mut show_add_modal = use_signal(|| false);
    let loading = use_signal(|| true);
    let error = use_signal(String::new);
    let can_create_task = current_auth.can_create_task();
    let can_delete_task = current_auth.can_delete_task();

    {
        let mut tasks = tasks;
        let mut loading = loading;
        let mut error = error;
        use_effect(move || {
            spawn(async move {
                match fetch_tasks().await {
                    Ok(data) => {
                        tasks.set(data);
                        error.set(String::new());
                    }
                    Err(err) => error.set(err),
                }
                loading.set(false);
            });
        });
    }

    let refresh = move |_| {
        let mut tasks = tasks;
        let mut loading = loading;
        let mut error = error;
        spawn(async move {
            loading.set(true);
            match fetch_tasks().await {
                Ok(data) => {
                    tasks.set(data);
                    error.set(String::new());
                }
                Err(err) => error.set(err),
            }
            loading.set(false);
        });
    };

    let total_count = tasks.read().len() as i32;
    let running_count = tasks
        .read()
        .iter()
        .filter(|t| t.status == "Running")
        .count() as i32;
    let completed_count = tasks
        .read()
        .iter()
        .filter(|t| t.status == "Completed")
        .count() as i32;
    let pending_count = tasks
        .read()
        .iter()
        .filter(|t| t.status == "Pending")
        .count() as i32;

    rsx! {
        div { class: "space-y-6",
            div { class: "flex justify-between items-center",
                h1 { class: "text-2xl font-bold text-gray-800", "任务中心" }
                div { class: "flex items-center gap-3",
                    button {
                        class: "flex items-center px-4 py-2 border border-gray-300 text-gray-700 rounded-md hover:bg-gray-50 transition-colors",
                        onclick: refresh,
                        Icon { icon: FaArrowRotateRight, width: 16, height: 16 }
                        span { class: "ml-2", "刷新" }
                    }
                    if can_create_task {
                        button {
                            class: "flex items-center px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors",
                            onclick: move |_| show_add_modal.set(true),
                            Icon { icon: FaPlus, width: 16, height: 16 }
                            span { class: "ml-2", "新建任务" }
                        }
                    }
                }
            }

            if !can_create_task && !can_delete_task {
                div { class: "rounded-lg border border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-800",
                    "当前账号只有任务查看权限，新建和删除操作已禁用。"
                }
            }

            div { class: "grid grid-cols-1 md:grid-cols-4 gap-4",
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-blue-500",
                            Icon { icon: FaList, width: 20, height: 20, class: "text-white" }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "总任务" }
                            p { class: "text-xl font-bold text-gray-800", {total_count.to_string()} }
                        }
                    }
                }
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-indigo-500",
                            Icon { icon: FaSpinner, width: 20, height: 20, class: "text-white" }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "运行中" }
                            p { class: "text-xl font-bold text-gray-800", {running_count.to_string()} }
                        }
                    }
                }
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-green-500",
                            Icon { icon: FaCircleCheck, width: 20, height: 20, class: "text-white" }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "已完成" }
                            p { class: "text-xl font-bold text-gray-800", {completed_count.to_string()} }
                        }
                    }
                }
                div { class: "bg-white rounded-lg shadow p-4",
                    div { class: "flex items-center",
                        div { class: "p-2 rounded-full bg-yellow-500",
                            Icon { icon: FaClock, width: 20, height: 20, class: "text-white" }
                        }
                        div { class: "ml-3",
                            p { class: "text-sm text-gray-500", "等待中" }
                            p { class: "text-xl font-bold text-gray-800", {pending_count.to_string()} }
                        }
                    }
                }
            }

            if *loading.read() {
                div { class: "bg-white rounded-lg shadow p-6 text-gray-500", "正在加载任务数据..." }
            } else if !error.read().is_empty() {
                div { class: "bg-red-50 border border-red-200 text-red-700 rounded-lg p-4", "{error}" }
            } else {
                VirtualScroller {
                    items: tasks.read().to_vec(),
                    page_size: 20,
                    render_item: move |task: TaskRecord| rsx! {
                        div { class: "grid grid-cols-1 lg:grid-cols-7 gap-4 px-6 py-4 border-b border-gray-100 items-center",
                            div {
                                p { class: "text-sm font-medium text-gray-900", "{task.name}" }
                                p { class: "text-xs text-gray-500", "创建人: {task.created_by.clone().unwrap_or_else(|| \"-\".to_string())}" }
                            }
                            div { class: "text-sm text-gray-600", "{task.target}" }
                            div { class: "text-sm text-gray-600", "{task.port_policy}" }
                            div {
                                span {
                                    class: "px-2 inline-flex text-xs leading-5 font-semibold rounded-full {status_color(&task.status)}",
                                    "{status_label(&task.status)}"
                                }
                            }
                            div { class: "text-sm text-gray-600", "{task.found_assets} / {task.found_risks}" }
                            div { class: "text-sm text-gray-500",
                                "{task.start_time.clone().or(task.end_time.clone()).unwrap_or_else(|| \"尚未开始\".to_string())}"
                            }
                            div {
                                if can_delete_task {
                                    button {
                                        class: "text-red-600 hover:text-red-900",
                                        onclick: {
                                            let task_id = task.id.clone();
                                            let mut tasks = tasks;
                                            let mut error = error;
                                            move |_| {
                                                let task_id = task_id.clone();
                                                spawn(async move {
                                                    match delete_task(&task_id).await {
                                                        Ok(()) => tasks.write().retain(|item| item.id != task_id),
                                                        Err(err) => error.set(err),
                                                    }
                                                });
                                            }
                                        },
                                        Icon { icon: FaTrash, width: 16, height: 16 }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if can_create_task && *show_add_modal.read() {
            AddTaskModal {
                on_close: move |_| show_add_modal.set(false),
                on_save: {
                    let mut tasks = tasks;
                    let mut error = error;
                    let mut show_add_modal = show_add_modal;
                    move |payload: CreateTaskPayload| {
                        spawn(async move {
                            match create_task(&payload).await {
                                Ok(task) => {
                                    tasks.write().insert(0, task);
                                    show_add_modal.set(false);
                                    error.set(String::new());
                                }
                                Err(err) => error.set(err),
                            }
                        });
                    }
                }
            }
        }
    }
}

#[component]
fn AddTaskModal(on_close: EventHandler<()>, on_save: EventHandler<CreateTaskPayload>) -> Element {
    let mut name = use_signal(String::new);
    let mut target = use_signal(String::new);
    let mut task_type = use_signal(|| "端口扫描".to_string());

    rsx! {
        div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            div { class: "bg-white rounded-lg shadow-xl w-full max-w-md mx-4",
                div { class: "flex justify-between items-center p-4 border-b",
                    h3 { class: "text-lg font-semibold", "新建任务" }
                    button { class: "text-gray-400 hover:text-gray-600", onclick: move |_| on_close.call(()), "×" }
                }

                div { class: "p-4 space-y-4",
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "任务名称" }
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500",
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
                            let (port_policy, domain_brute, service_detection, os_detection, site_identify) =
                                task_template(task_type.read().as_str());

                            on_save.call(CreateTaskPayload {
                                name: name.read().clone(),
                                target: target.read().clone(),
                                port_policy: port_policy.to_string(),
                                domain_brute,
                                service_detection,
                                os_detection,
                                site_identify,
                            });
                        },
                        "保存"
                    }
                }
            }
        }
    }
}

fn task_template(task_type: &str) -> (&'static str, bool, bool, bool, bool) {
    match task_type {
        "漏洞扫描" => ("TOP1000", false, true, true, true),
        "资产发现" => ("TOP100", false, true, false, false),
        "Web扫描" => ("TOP1000", false, true, false, true),
        _ => ("TOP1000", false, true, false, false),
    }
}

fn status_label(status: &str) -> &str {
    match status {
        "Running" => "运行中",
        "Completed" => "已完成",
        "Failed" => "失败",
        _ => "等待中",
    }
}

fn status_color(status: &str) -> &'static str {
    match status {
        "Running" => "bg-blue-100 text-blue-800",
        "Completed" => "bg-green-100 text-green-800",
        "Failed" => "bg-red-100 text-red-800",
        _ => "bg-yellow-100 text-yellow-800",
    }
}
