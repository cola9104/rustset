use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::FaXmark;

/// 通用模态框组件属性
#[derive(Props, Clone, PartialEq)]
pub struct ModalProps {
    /// 是否显示
    show: bool,
    /// 标题
    title: String,
    /// 宽度大小：sm, md, lg, xl, 2xl
    #[props(default = "2xl".to_string())]
    size: String,
    /// 关闭事件
    on_close: EventHandler<()>,
    /// 子内容
    children: Element,
    /// 底部按钮区域（可选）
    #[props(default = None)]
    footer: Option<Element>,
}

/// 通用模态框组件
#[component]
pub fn Modal(props: ModalProps) -> Element {
    if !props.show {
        return rsx! { "" };
    }

    let size_class = match props.size.as_str() {
        "sm" => "max-w-md",
        "md" => "max-w-lg",
        "lg" => "max-w-xl",
        "xl" => "max-w-2xl",
        "2xl" => "max-w-3xl",
        "full" => "max-w-5xl",
        _ => "max-w-2xl",
    };

    rsx! {
        // 背景遮罩
        div {
            class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            onclick: move |_| props.on_close.call(()),

            // 模态框主体
            div {
                class: "bg-white rounded-lg shadow-xl {size_class} w-full mx-4 max-h-[90vh] overflow-y-auto",
                onclick: |e| e.stop_propagation(),

                // 头部
                div { class: "flex items-center justify-between p-6 border-b border-gray-200",
                    h2 { class: "text-xl font-bold text-gray-800", "{props.title}" }
                    button {
                        class: "text-gray-400 hover:text-gray-600 transition-colors p-1 hover:bg-gray-100 rounded",
                        onclick: move |_| props.on_close.call(()),
                        Icon { icon: FaXmark, width: 20, height: 20 }
                    }
                }

                // 内容区域
                div { class: "p-6",
                    {props.children}
                }

                // 底部按钮区域
                if let Some(footer) = props.footer {
                    div { class: "flex justify-end gap-3 p-6 border-t border-gray-200 bg-gray-50",
                        {footer}
                    }
                }
            }
        }
    }
}

/// 模态框底部按钮组
#[component]
pub fn ModalFooter(
    /// 保存按钮文本
    #[props(default = "保存".to_string())]
    save_text: String,
    /// 取消按钮文本
    #[props(default = "取消".to_string())]
    cancel_text: String,
    /// 是否显示删除按钮
    #[props(default = false)]
    show_delete: bool,
    /// 保存按钮是否禁用
    #[props(default = false)]
    save_disabled: bool,
    /// 保存事件
    on_save: EventHandler<()>,
    /// 取消事件
    on_cancel: EventHandler<()>,
    /// 删除事件（可选）
    #[props(default = None)]
    on_delete: Option<EventHandler<()>>,
) -> Element {
    rsx! {
        // 删除按钮（如果需要）
        if show_delete {
            if let Some(on_delete) = on_delete {
                button {
                    class: "px-4 py-2 text-red-600 border border-red-300 rounded-lg hover:bg-red-50 transition-colors",
                    onclick: move |_| on_delete.call(()),
                    "删除"
                }
            }
        }

        div { class: "flex-1" }

        // 取消按钮
        button {
            class: "px-4 py-2 text-gray-700 border border-gray-300 rounded-lg hover:bg-gray-50 transition-colors",
            onclick: move |_| on_cancel.call(()),
            "{cancel_text}"
        }

        // 保存按钮
        button {
            class: if save_disabled {
                "px-4 py-2 bg-gray-400 text-white rounded-lg cursor-not-allowed"
            } else {
                "px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
            },
            disabled: save_disabled,
            onclick: move |_| on_save.call(()),
            "{save_text}"
        }
    }
}

/// 错误提示组件
#[component]
pub fn ErrorMessage(message: String) -> Element {
    rsx! {
        div { class: "mb-4 p-3 bg-red-50 border border-red-200 text-red-700 rounded-lg text-sm",
            "{message}"
        }
    }
}

/// 成功提示组件
#[component]
pub fn SuccessMessage(message: String) -> Element {
    rsx! {
        div { class: "mb-4 p-3 bg-green-50 border border-green-200 text-green-700 rounded-lg text-sm",
            "{message}"
        }
    }
}
