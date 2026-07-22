use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub struct CascaderOption {
    pub value: String,
    pub label: String,
    pub children: Vec<CascaderOption>,
}

#[component]
pub fn Cascader(
    options: Vec<CascaderOption>,
    placeholder: Option<String>,
    on_change: EventHandler<Vec<String>>,
) -> Element {
    let mut visible = use_signal(|| false);
    let mut selected_path = use_signal(Vec::<String>::new);
    let mut current_level = use_signal(|| 0usize);
    let ph = placeholder.unwrap_or_else(|| "请选择".to_string());
    let display_text = if selected_path().is_empty() {
        ph.clone()
    } else {
        selected_path().join(" / ")
    };

    rsx! {
        div { class: "relative",
            button {
                class: "ant-input text-left flex items-center justify-between",
                onclick: move |_| visible.toggle(),
                span { class: if selected_path().is_empty() { "text-gray-400" },
                    "{display_text}"
                }
                span { class: "text-gray-400 text-xs", "▾" }
            }
            if visible() {
                div { class: "absolute top-full left-0 mt-1 bg-white border rounded-lg shadow-lg z-50 flex min-w-[200px]",
                    { {
                        let path = selected_path();
                        let lvl = current_level();
                        let opts = options.clone();
                        rsx! {
                            CascaderColumn {
                                items: opts,
                                selected: path,
                                on_select: move |val: String| {
                                    let mut path = selected_path();
                                    path.truncate(lvl);
                                    path.push(val.clone());
                                    selected_path.set(path.clone());
                                    current_level.set(lvl + 1);
                                    on_change.call(path);
                                }
                            }
                        }
                    }}
                }
            }
        }
    }
}

#[component]
fn CascaderColumn(
    items: Vec<CascaderOption>,
    selected: Vec<String>,
    on_select: EventHandler<String>,
) -> Element {
    let first = selected.first().cloned();
    rsx! {
        div { class: "w-[180px] max-h-[240px] overflow-y-auto py-1 border-r last:border-r-0",
            for item in &items {
                {
                    let v = item.value.clone();
                    let is_sel = first.as_ref() == Some(&item.value);
                    let has_kids = !item.children.is_empty();
                    rsx! {
                        div {
                            class: format!("px-3 py-2 text-sm cursor-pointer hover:bg-gray-50 flex items-center justify-between {}",
                                if is_sel { "bg-primary-50 text-primary-600" } else { "text-gray-700" }),
                            onclick: move |_| on_select.call(v.clone()),
                            "{item.label}"
                            if has_kids {
                                span { class: "text-gray-400 text-xs", "›" }
                            }
                        }
                    }
                }
            }
        }
    }
}
