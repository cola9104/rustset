use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub struct TransferItem {
    pub key: String,
    pub label: String,
}

#[component]
pub fn Transfer(
    all_items: Vec<TransferItem>,
    selected_keys: Vec<String>,
    on_change: EventHandler<Vec<String>>,
) -> Element {
    let mut left_sel = use_signal(Vec::<String>::new);
    let mut right_sel = use_signal(Vec::<String>::new);

    let left_items: Vec<TransferItem> = all_items.iter()
        .filter(|i| !selected_keys.contains(&i.key))
        .cloned().collect();
    let right_items: Vec<TransferItem> = all_items.iter()
        .filter(|i| selected_keys.contains(&i.key))
        .cloned().collect();

    let move_right = {
        let sk = selected_keys.clone();
        let on_ch = on_change.clone();
        move |_| {
            let mut current = sk.clone();
            for k in left_sel() { if !current.contains(&k) { current.push(k); } }
            on_ch.call(current);
        }
    };
    let move_left = {
        let sk = selected_keys.clone();
        let on_ch = on_change.clone();
        move |_| {
            let current: Vec<String> = sk.iter().filter(|k| !right_sel().contains(k)).cloned().collect();
            on_ch.call(current);
        }
    };

    rsx! {
        div { class: "flex items-start gap-2",
            TransferPanel {
                items: left_items,
                selected: left_sel(),
                placeholder: "待选项".to_string(),
                on_select: move |k: String| { left_sel.write().push(k); },
            }
            div { class: "flex flex-col gap-1 pt-8",
                button { class: "ant-btn text-xs py-1 px-2", onclick: move_right, "→" }
                button { class: "ant-btn text-xs py-1 px-2", onclick: move_left, "←" }
            }
            TransferPanel {
                items: right_items,
                selected: right_sel(),
                placeholder: "已选项".to_string(),
                on_select: move |k: String| { right_sel.write().push(k); },
            }
        }
    }
}

#[component]
fn TransferPanel(
    items: Vec<TransferItem>,
    selected: Vec<String>,
    placeholder: String,
    on_select: EventHandler<String>,
) -> Element {
    let count = items.len();
    rsx! {
        div { class: "flex-1 border rounded-lg overflow-hidden",
            div { class: "px-3 py-2 bg-gray-50 border-b text-xs text-gray-500 font-medium", "{placeholder} ({count})" }
            div { class: "max-h-[240px] overflow-y-auto",
                if items.is_empty() {
                    div { class: "px-3 py-8 text-center text-sm text-gray-400", "无数据" }
                } else {
                    for item in &items {
                        {
                            let k = item.key.clone();
                            let is_sel = selected.contains(&item.key);
                            let label = item.label.clone();
                            rsx! {
                                div {
                                    class: format!("px-3 py-2 text-sm cursor-pointer hover:bg-gray-50 flex items-center gap-2 {}",
                                        if is_sel { "bg-primary-50" } else { "" }),
                                    onclick: move |_| on_select.call(k.clone()),
                                    span { class: if is_sel { "text-primary-500" } else { "text-gray-300" },
                                        if is_sel { "☑" } else { "☐" }
                                    }
                                    "{label}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
