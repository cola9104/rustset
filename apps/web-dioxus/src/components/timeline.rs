use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub struct TimelineItem {
    pub title: String,
    pub time: Option<String>,
    pub description: Option<String>,
    pub color: Option<String>,
}

#[component]
pub fn Timeline(items: Vec<TimelineItem>) -> Element {
    rsx! {
        div { class: "relative pl-8",
            for (i, item) in items.iter().enumerate() {
                div { class: format!("relative pb-6 {}", if i == items.len() - 1 { "" } else { "border-l-2 border-gray-200" }),
                    // Dot
                    div {
                        class: format!("absolute -left-[21px] top-0 w-3 h-3 rounded-full border-2 border-white {}",
                            item.color.as_deref().unwrap_or("bg-primary-500")),
                    }
                    div { class: "ml-6",
                        div { class: "text-sm font-medium text-gray-800", "{item.title}" }
                        if let Some(ref t) = item.time {
                            div { class: "text-xs text-gray-400 mt-0.5", "{t}" }
                        }
                        if let Some(ref d) = item.description {
                            div { class: "text-sm text-gray-500 mt-1", "{d}" }
                        }
                    }
                }
            }
        }
    }
}
