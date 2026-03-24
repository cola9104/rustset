use dioxus::prelude::*;

/// 虚拟滚动组件 - 分页版本
///
/// 使用分页方式显示大数据列表，提升性能
/// 每次只渲染指定数量的项
#[derive(Props, Clone, PartialEq)]
pub struct VirtualScrollerProps<T: Clone + PartialEq + 'static> {
    /// 所有数据项
    items: Vec<T>,
    /// 每页显示的项数
    #[props(default = 20)]
    page_size: usize,
    /// 渲染单个项目的回调
    render_item: Callback<T, Element>,
}

/// 虚拟滚动组件 - 分页版本
///
/// # 示例
///
/// ```rust
/// VirtualScroller {
///     items: logs,
///     page_size: 20,
///     render_item: move |log| rsx! {
///         div { class: "p-4", "{log.content}" }
///     }
/// }
/// ```
#[allow(non_snake_case)]
pub fn VirtualScroller<T: Clone + PartialEq + 'static>(props: VirtualScrollerProps<T>) -> Element {
    let mut current_page = use_signal(|| 0);

    let total_items = props.items.len();
    let total_pages = if total_items == 0 {
        0
    } else {
        (total_items + props.page_size - 1) / props.page_size
    };

    let start_index = *current_page.read() * props.page_size;
    let end_index = (start_index + props.page_size).min(total_items);

    let visible_items: Vec<T> = props
        .items
        .iter()
        .cloned()
        .skip(start_index)
        .take(props.page_size)
        .collect();

    // 如果没有项目，显示空状态
    if props.items.is_empty() {
        return rsx! {
            div {
                class: "flex items-center justify-center text-gray-500 py-12",
                "暂无数据"
            }
        };
    }

    rsx! {
        div {
            class: "space-y-2",
            // 显示的项目
            for (index, item) in visible_items.iter().enumerate() {
                div {
                    key: "{start_index + index}",
                    class: "border-b border-gray-100 last:border-b-0",
                    {
                        props.render_item.call(item.clone())
                    }
                }
            }

            // 分页控制
            if total_pages > 1 {
                div { class: "flex items-center justify-between py-4",
                    div { class: "text-sm text-gray-500",
                        "显示 {start_index + 1}-{end_index} 条，共 {total_items} 条"
                    }
                    div { class: "flex space-x-2",
                        button {
                            class: if *current_page.read() == 0 {
                                "px-3 py-1 border border-gray-300 rounded-md text-sm text-gray-400 cursor-not-allowed"
                            } else {
                                "px-3 py-1 border border-gray-300 rounded-md text-sm hover:bg-gray-50"
                            },
                            disabled: *current_page.read() == 0,
                            onclick: move |_| {
                                let current = *current_page.read();
                                if current > 0 {
                                    current_page.set(current - 1);
                                }
                            },
                            "上一页"
                        }

                        // 页码按钮
                        for page in 0..total_pages {
                            button {
                                class: if page == *current_page.read() {
                                    "px-3 py-1 bg-blue-600 text-white rounded-md text-sm"
                                } else {
                                    "px-3 py-1 border border-gray-300 rounded-md text-sm hover:bg-gray-50"
                                },
                                onclick: move |_| current_page.set(page),
                                "{page + 1}"
                            }
                        }

                        button {
                            class: if *current_page.read() >= total_pages - 1 {
                                "px-3 py-1 border border-gray-300 rounded-md text-sm text-gray-400 cursor-not-allowed"
                            } else {
                                "px-3 py-1 border border-gray-300 rounded-md text-sm hover:bg-gray-50"
                            },
                            disabled: *current_page.read() >= total_pages - 1,
                            onclick: move |_| {
                                let current = *current_page.read();
                                if current < total_pages - 1 {
                                    current_page.set(current + 1);
                                }
                            },
                            "下一页"
                        }
                    }
                }
            }
        }
    }
}

/// 虚拟列表组件 - 表格版本
///
/// 使用表格布局的虚拟滚动，适合表格数据
#[derive(Props, Clone, PartialEq)]
pub struct VirtualListProps<T: Clone + PartialEq + 'static> {
    /// 所有数据项
    items: Vec<T>,
    /// 每页显示的行数
    #[props(default = 20)]
    page_size: usize,
    /// 渲染表头（可选）
    #[props(default = None)]
    header: Option<Element>,
    /// 渲染单个行的回调
    render_row: Callback<T, Element>,
}

/// 虚拟列表组件 - 表格版本（分页）
#[allow(non_snake_case)]
#[allow(dead_code)]
pub fn VirtualList<T: Clone + PartialEq + 'static>(props: VirtualListProps<T>) -> Element {
    let mut current_page = use_signal(|| 0);

    let total_items = props.items.len();
    let total_pages = if total_items == 0 {
        0
    } else {
        (total_items + props.page_size - 1) / props.page_size
    };

    let start_index = *current_page.read() * props.page_size;
    let end_index = (start_index + props.page_size).min(total_items);

    let visible_items: Vec<T> = props
        .items
        .iter()
        .cloned()
        .skip(start_index)
        .take(props.page_size)
        .collect();

    // 如果没有项目，显示空状态
    if props.items.is_empty() {
        return rsx! {
            div {
                class: "bg-white rounded-lg shadow",
                div {
                    class: "flex items-center justify-center h-96 text-gray-500",
                    "暂无数据"
                }
            }
        };
    }

    rsx! {
        div { class: "bg-white rounded-lg shadow overflow-hidden",
            // 表头
            if let Some(header) = &props.header {
                div {
                    class: "bg-gray-50",
                    {header}
                }
            }

            // 数据行
            div {
                for (index, item) in visible_items.iter().enumerate() {
                    div {
                        key: "{start_index + index}",
                        class: "border-b border-gray-200 hover:bg-gray-50 transition-colors",
                        {
                            props.render_row.call(item.clone())
                        }
                    }
                }
            }

            // 分页控制
            if total_pages > 1 {
                div { class: "px-4 py-3 flex items-center justify-between bg-gray-50 border-t border-gray-200",
                    div { class: "text-sm text-gray-500",
                        "显示 {start_index + 1}-{end_index} 条，共 {total_items} 条"
                    }
                    div { class: "flex space-x-2",
                        button {
                            class: if *current_page.read() == 0 {
                                "px-3 py-1 border border-gray-300 rounded-md text-sm text-gray-400 cursor-not-allowed"
                            } else {
                                "px-3 py-1 border border-gray-300 rounded-md text-sm hover:bg-gray-50"
                            },
                            disabled: *current_page.read() == 0,
                            onclick: move |_| {
                                let current = *current_page.read();
                                if current > 0 {
                                    current_page.set(current - 1);
                                }
                            },
                            "上一页"
                        }

                        // 页码按钮
                        for page in 0..total_pages {
                            button {
                                class: if page == *current_page.read() {
                                    "px-3 py-1 bg-blue-600 text-white rounded-md text-sm"
                                } else {
                                    "px-3 py-1 border border-gray-300 rounded-md text-sm hover:bg-gray-50"
                                },
                                onclick: move |_| current_page.set(page),
                                "{page + 1}"
                            }
                        }

                        button {
                            class: if *current_page.read() >= total_pages - 1 {
                                "px-3 py-1 border border-gray-300 rounded-md text-sm text-gray-400 cursor-not-allowed"
                            } else {
                                "px-3 py-1 border border-gray-300 rounded-md text-sm hover:bg-gray-50"
                            },
                            disabled: *current_page.read() >= total_pages - 1,
                            onclick: move |_| {
                                let current = *current_page.read();
                                if current < total_pages - 1 {
                                    current_page.set(current + 1);
                                }
                            },
                            "下一页"
                        }
                    }
                }
            }
        }
    }
}
