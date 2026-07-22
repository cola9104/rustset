use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub struct DescItem {
    pub label: String,
    pub value: String,
    pub span: Option<usize>,
}

#[component]
pub fn DescriptionList(items: Vec<DescItem>, column: Option<usize>) -> Element {
    let cols = column.unwrap_or(2);
    rsx! {
        div { class: format!("grid grid-cols-{} gap-4", cols),
            for item in &items {
                div { class: format!("{}",
                    if let Some(s) = item.span { format!("col-span-{}", s) } else { String::new() }),
                    div { class: "text-xs text-gray-400 mb-1", "{item.label}" }
                    div { class: "text-sm text-gray-800", "{item.value}" }
                }
            }
        }
    }
}
