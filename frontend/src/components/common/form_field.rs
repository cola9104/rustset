use dioxus::prelude::*;

/// 通用表单字段包装器
#[component]
pub fn FormField(
    /// 字段标签
    label: String,
    /// 是否必填
    #[props(default = false)]
    required: bool,
    /// 帮助文本
    #[props(default = None)]
    help_text: Option<String>,
    /// 子内容
    children: Element,
) -> Element {
    rsx! {
        div { class: "mb-4",
            label { class: "block text-sm font-medium text-gray-700 mb-1",
                "{label}"
                if required {
                    span { class: "text-red-500 ml-1", "*" }
                }
            }
            {children}
            if let Some(help) = help_text {
                p { class: "mt-1 text-xs text-gray-500", "{help}" }
            }
        }
    }
}

/// 通用输入框
#[component]
pub fn InputField(
    /// 字段标签
    label: String,
    /// 输入类型
    #[props(default = "text".to_string())]
    input_type: String,
    /// 占位符
    #[props(default = String::new())]
    placeholder: String,
    /// 当前值
    value: String,
    /// 是否必填
    #[props(default = false)]
    required: bool,
    /// 是否禁用
    #[props(default = false)]
    disabled: bool,
    /// 帮助文本
    #[props(default = None)]
    help_text: Option<String>,
    /// 值变化事件
    onchange: EventHandler<String>,
) -> Element {
    rsx! {
        div { class: "mb-4",
            label { class: "block text-sm font-medium text-gray-700 mb-1",
                "{label}"
                if required {
                    span { class: "text-red-500 ml-1", "*" }
                }
            }
            input {
                r#type: "{input_type}",
                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent disabled:bg-gray-100 disabled:cursor-not-allowed",
                placeholder: "{placeholder}",
                value: "{value}",
                disabled: disabled,
                oninput: move |e| onchange.call(e.value()),
            }
            if let Some(help) = help_text {
                p { class: "mt-1 text-xs text-gray-500", "{help}" }
            }
        }
    }
}

/// 通用下拉选择框
#[component]
pub fn SelectField(
    /// 字段标签
    label: String,
    /// 当前值
    value: String,
    /// 占位符选项文本
    #[props(default = "请选择".to_string())]
    placeholder: String,
    /// 是否必填
    #[props(default = false)]
    required: bool,
    /// 帮助文本
    #[props(default = None)]
    help_text: Option<String>,
    /// 值变化事件
    onchange: EventHandler<String>,
    /// 子选项
    children: Element,
) -> Element {
    rsx! {
        div { class: "mb-4",
            label { class: "block text-sm font-medium text-gray-700 mb-1",
                "{label}"
                if required {
                    span { class: "text-red-500 ml-1", "*" }
                }
            }
            select {
                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                value: "{value}",
                onchange: move |e| onchange.call(e.value()),
                option { value: "", "{placeholder}" }
                {children}
            }
            if let Some(help) = help_text {
                p { class: "mt-1 text-xs text-gray-500", "{help}" }
            }
        }
    }
}

/// 通用文本域
#[component]
pub fn TextAreaField(
    /// 字段标签
    label: String,
    /// 当前值
    value: String,
    /// 占位符
    #[props(default = String::new())]
    placeholder: String,
    /// 行数
    #[props(default = 3)]
    rows: i32,
    /// 是否必填
    #[props(default = false)]
    required: bool,
    /// 帮助文本
    #[props(default = None)]
    help_text: Option<String>,
    /// 值变化事件
    onchange: EventHandler<String>,
) -> Element {
    rsx! {
        div { class: "mb-4",
            label { class: "block text-sm font-medium text-gray-700 mb-1",
                "{label}"
                if required {
                    span { class: "text-red-500 ml-1", "*" }
                }
            }
            textarea {
                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                placeholder: "{placeholder}",
                rows: rows,
                oninput: move |e| onchange.call(e.value()),
                "{value}"
            }
            if let Some(help) = help_text {
                p { class: "mt-1 text-xs text-gray-500", "{help}" }
            }
        }
    }
}

/// 通用复选框
#[component]
pub fn CheckboxField(
    /// 标签文本
    label: String,
    /// 是否选中
    checked: bool,
    /// 是否禁用
    #[props(default = false)]
    disabled: bool,
    /// 值变化事件
    onchange: EventHandler<bool>,
) -> Element {
    rsx! {
        label { class: "flex items-center gap-2 cursor-pointer",
            input {
                r#type: "checkbox",
                class: "w-4 h-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded disabled:cursor-not-allowed",
                checked: checked,
                disabled: disabled,
                oninput: move |e| onchange.call(e.checked()),
            }
            span { class: "text-sm text-gray-700", "{label}" }
        }
    }
}

/// 通用单选按钮组
#[component]
pub fn RadioField(
    /// 字段标签
    label: String,
    /// 选项列表 (value, label)
    options: Vec<(String, String)>,
    /// 当前值
    value: String,
    /// 字段名称（用于分组）
    name: String,
    /// 是否必填
    #[props(default = false)]
    required: bool,
    /// 值变化事件
    onchange: EventHandler<String>,
) -> Element {
    rsx! {
        div { class: "mb-4",
            label { class: "block text-sm font-medium text-gray-700 mb-2",
                "{label}"
                if required {
                    span { class: "text-red-500 ml-1", "*" }
                }
            }
            div { class: "flex flex-wrap gap-4",
                for (opt_value, opt_label) in options.iter() {
                    label { class: "flex items-center gap-2 cursor-pointer",
                        input {
                            r#type: "radio",
                            class: "w-4 h-4 text-blue-600 focus:ring-blue-500",
                            name: "{name}",
                            value: "{opt_value}",
                            checked: *value == *opt_value,
                            oninput: {
                                let v = opt_value.clone();
                                move |_| onchange.call(v.clone())
                            },
                        }
                        span { class: "text-sm text-gray-700", "{opt_label}" }
                    }
                }
            }
        }
    }
}

/// 通用数字输入框
#[component]
pub fn NumberField(
    /// 字段标签
    label: String,
    /// 当前值
    value: i32,
    /// 最小值
    #[props(default = None)]
    min: Option<i32>,
    /// 最大值
    #[props(default = None)]
    max: Option<i32>,
    /// 是否必填
    #[props(default = false)]
    required: bool,
    /// 帮助文本
    #[props(default = None)]
    help_text: Option<String>,
    /// 值变化事件
    onchange: EventHandler<i32>,
) -> Element {
    let min_attr = min.map(|v| v.to_string()).unwrap_or_default();
    let max_attr = max.map(|v| v.to_string()).unwrap_or_default();

    rsx! {
        div { class: "mb-4",
            label { class: "block text-sm font-medium text-gray-700 mb-1",
                "{label}"
                if required {
                    span { class: "text-red-500 ml-1", "*" }
                }
            }
            input {
                r#type: "number",
                class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                min: "{min_attr}",
                max: "{max_attr}",
                value: "{value}",
                oninput: move |e| {
                    if let Ok(v) = e.value().parse::<i32>() {
                        onchange.call(v);
                    }
                },
            }
            if let Some(help) = help_text {
                p { class: "mt-1 text-xs text-gray-500", "{help}" }
            }
        }
    }
}
