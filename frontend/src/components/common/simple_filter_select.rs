use dioxus::prelude::*;
use std::collections::HashSet;

/// 简化版过滤选择器 - 直接使用 Vec<(i32, String)>
#[component]
pub fn SimpleFilterSelect(
    /// 字段标签
    label: String,
    /// 当前选中的ID
    selected_id: Option<i32>,
    /// 占位符文本
    #[props(default = "请选择".to_string())]
    placeholder: String,
    /// 是否必填
    #[props(default = false)]
    required: bool,
    /// 帮助文本
    #[props(default = None)]
    help_text: Option<String>,
    /// 值变化事件
    onchange: EventHandler<Option<i32>>,
    /// 可选项列表 (id, display_name)
    items: Vec<(i32, String)>,
    /// 已被分配的ID集合
    #[props(default = HashSet::new())]
    assigned_ids: HashSet<i32>,
    /// 当前编辑项的ID
    #[props(default = None)]
    editing_id: Option<i32>,
) -> Element {
    let selected_value = selected_id.map(|id| id.to_string()).unwrap_or_default();

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
                value: "{selected_value}",
                onchange: move |e| {
                    let val = e.value();
                    if val.is_empty() {
                        onchange.call(None);
                    } else if let Ok(id) = val.parse::<i32>() {
                        onchange.call(Some(id));
                    }
                },
                option { value: "", "{placeholder}" }

                for (item_id, display) in items.iter() {
                    // 检查是否应该显示此选项
                    if !assigned_ids.contains(item_id) || editing_id == Some(*item_id) {
                        option {
                            value: "{item_id}",
                            selected: selected_id == Some(*item_id),
                            "{display}"
                        }
                    }
                }
            }
            if let Some(help) = &help_text {
                p { class: "mt-1 text-xs text-gray-500", "{help}" }
            }
        }
    }
}

/// Hook: 计算已分配ID集合（排除当前编辑项）
///
/// # 参数
/// - `all_assigned`: 所有已分配的ID列表
/// - `editing_id`: 当前正在编辑的项ID（这个ID会被排除在结果之外）
///
/// # 返回
/// - 响应式的已分配ID集合
pub fn use_assigned_ids_excluding(
    all_assigned: Vec<i32>,
    editing_id: Option<i32>,
) -> Memo<HashSet<i32>> {
    use_memo(move || {
        all_assigned
            .iter()
            .filter(|id| Some(**id) != editing_id)
            .copied()
            .collect()
    })
}
