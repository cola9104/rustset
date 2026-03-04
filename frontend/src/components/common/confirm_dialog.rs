use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::FaTriangleExclamation;

/// 确认对话框类型
#[derive(Clone, Copy, PartialEq)]
pub enum ConfirmType {
    /// 危险操作（删除等）
    Danger,
    /// 警告操作
    Warning,
    /// 普通确认
    Info,
    /// 成功确认
    Success,
}

/// 确认对话框组件
#[component]
pub fn ConfirmDialog(
    /// 是否显示
    show: bool,
    /// 标题
    title: String,
    /// 消息内容
    message: String,
    /// 对话框类型
    #[props(default = ConfirmType::Info)]
    confirm_type: ConfirmType,
    /// 确认按钮文本
    #[props(default = "确认".to_string())]
    confirm_text: String,
    /// 取消按钮文本
    #[props(default = "取消".to_string())]
    cancel_text: String,
    /// 确认事件
    on_confirm: EventHandler<()>,
    /// 取消事件
    on_cancel: EventHandler<()>,
) -> Element {
    if !show {
        return rsx! { "" };
    }

    let (icon_color, confirm_btn_class) = match confirm_type {
        ConfirmType::Danger => (
            "text-red-600 bg-red-100",
            "px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors",
        ),
        ConfirmType::Warning => (
            "text-yellow-600 bg-yellow-100",
            "px-4 py-2 bg-yellow-600 text-white rounded-lg hover:bg-yellow-700 transition-colors",
        ),
        ConfirmType::Info => (
            "text-blue-600 bg-blue-100",
            "px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors",
        ),
        ConfirmType::Success => (
            "text-green-600 bg-green-100",
            "px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors",
        ),
    };

    rsx! {
        div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            onclick: move |_| on_cancel.call(()),

            div { class: "bg-white rounded-lg shadow-xl max-w-md w-full mx-4",
                onclick: |e| e.stop_propagation(),

                div { class: "p-6",
                    div { class: "flex items-start",
                        // 图标
                        div { class: "flex-shrink-0 w-10 h-10 rounded-full flex items-center justify-center {icon_color}",
                            Icon { icon: FaTriangleExclamation, width: 24, height: 24 }
                        }

                        // 内容
                        div { class: "ml-4 flex-1",
                            h3 { class: "text-lg font-semibold text-gray-900", "{title}" }
                            p { class: "mt-2 text-sm text-gray-600", "{message}" }
                        }
                    }
                }

                // 按钮区域
                div { class: "flex justify-end gap-3 p-4 bg-gray-50 rounded-b-lg",
                    button {
                        class: "px-4 py-2 text-gray-700 border border-gray-300 rounded-lg hover:bg-gray-50 transition-colors",
                        onclick: move |_| on_cancel.call(()),
                        "{cancel_text}"
                    }
                    button {
                        class: "{confirm_btn_class}",
                        onclick: move |_| on_confirm.call(()),
                        "{confirm_text}"
                    }
                }
            }
        }
    }
}

/// 快速删除确认对话框
#[component]
pub fn DeleteConfirmDialog(
    /// 是否显示
    show: bool,
    /// 要删除的项目名称
    item_name: String,
    /// 确认事件
    on_confirm: EventHandler<()>,
    /// 取消事件
    on_cancel: EventHandler<()>,
) -> Element {
    rsx! {
        ConfirmDialog {
            show: show,
            title: "确认删除".to_string(),
            message: format!("确定要删除\"{}\"吗？此操作无法撤销。", item_name),
            confirm_type: ConfirmType::Danger,
            confirm_text: "删除".to_string(),
            cancel_text: "取消".to_string(),
            on_confirm: move |_| on_confirm.call(()),
            on_cancel: move |_| on_cancel.call(()),
        }
    }
}

/// 快速停用确认对话框
#[component]
pub fn DeactivateConfirmDialog(
    /// 是否显示
    show: bool,
    /// 项目名称
    item_name: String,
    /// 确认事件
    on_confirm: EventHandler<()>,
    /// 取消事件
    on_cancel: EventHandler<()>,
) -> Element {
    rsx! {
        ConfirmDialog {
            show: show,
            title: "确认停用".to_string(),
            message: format!("确定要停用\"{}\"吗？", item_name),
            confirm_type: ConfirmType::Warning,
            confirm_text: "停用".to_string(),
            cancel_text: "取消".to_string(),
            on_confirm: move |_| on_confirm.call(()),
            on_cancel: move |_| on_cancel.call(()),
        }
    }
}
