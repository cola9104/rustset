use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub struct Step {
    pub title: String,
    pub description: Option<String>,
}

#[component]
pub fn Steps(current: usize, items: Vec<Step>) -> Element {
    rsx! {
        div { class: "flex",
            for (i, step) in items.iter().enumerate() {
                div { class: format!("flex-1 relative {}",
                    if i < items.len() - 1 { "pb-8" } else { "" }),
                    // Step indicator
                    div { class: "flex items-center gap-3",
                        div {
                            class: format!("w-8 h-8 rounded-full flex items-center justify-center text-sm font-bold {}",
                                if i < current {
                                    "bg-primary-500 text-white"
                                } else if i == current {
                                    "bg-primary-100 text-primary-600 border-2 border-primary-500"
                                } else {
                                    "bg-gray-100 text-gray-400"
                                }),
                            if i < current { "✓" } else { "{i + 1}" }
                        }
                        div {
                            div { class: "text-sm font-medium text-gray-800", "{step.title}" }
                            if let Some(ref d) = step.description {
                                div { class: "text-xs text-gray-400", "{d}" }
                            }
                        }
                    }
                    // Connector line
                    if i < items.len() - 1 {
                        div {
                            class: format!("absolute left-4 top-8 w-[2px] h-6 {}",
                                if i < current { "bg-primary-500" } else { "bg-gray-200" }),
                        }
                    }
                }
            }
        }
    }
}
