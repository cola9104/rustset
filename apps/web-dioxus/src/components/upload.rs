use dioxus::prelude::*;

#[component]
pub fn FileUpload(
    accept: Option<String>,
    multiple: Option<bool>,
    on_files: EventHandler<Vec<String>>,
) -> Element {
    let mut hover = use_signal(|| false);
    let mut file_names = use_signal(Vec::<String>::new);
    let acc = accept.unwrap_or_default();
    let multi = multiple.unwrap_or(false);

    rsx! {
        div {
            div {
                class: format!("border-2 border-dashed rounded-lg p-8 text-center transition-colors cursor-pointer {}",
                    if hover() { "border-primary-400 bg-primary-50" } else { "border-gray-300 hover:border-primary-300" }),
                ondragover: move |e| { e.prevent_default(); hover.set(true); },
                ondragleave: move |_| hover.set(false),
                ondrop: move |e| { e.prevent_default(); hover.set(false); },
                div { class: "text-3xl mb-3 text-gray-400", "📁" }
                p { class: "text-sm text-gray-500 mb-1", "拖拽文件到此处或点击上传" }
                p { class: "text-xs text-gray-400", "支持格式: {acc}" }
                input {
                    class: "mt-4 block w-full text-sm text-gray-500 file:mr-4 file:py-2 file:px-4 file:rounded-full file:border-0 file:text-sm file:font-semibold file:bg-primary-50 file:text-primary-700 hover:file:bg-primary-100",
                    r#type: "file",
                    accept: "{acc}",
                    multiple: multi,
                    onchange: move |evt| {
                        let files = evt.files();
                        let names: Vec<String> = files.iter().map(|f| f.name()).collect();
                        file_names.set(names.clone());
                        on_files.call(names);
                    }
                }
                if !file_names().is_empty() {
                    div { class: "mt-4 pt-4 border-t text-left",
                        for n in file_names() {
                            div { class: "flex items-center gap-2 text-sm text-gray-600 py-1",
                                span { class: "text-green-500", "✓" }
                                span { "{n}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn ImageUpload(
    on_images: EventHandler<Vec<String>>,
) -> Element {
    rsx! {
        div { class: "grid grid-cols-4 gap-3",
            label {
                class: "border-2 border-dashed border-gray-300 rounded-lg aspect-square flex flex-col items-center justify-center cursor-pointer hover:border-primary-400 hover:bg-primary-50 transition-colors",
                input {
                    class: "hidden",
                    r#type: "file",
                    accept: "image/*",
                    multiple: true,
                }
                span { class: "text-2xl text-gray-400", "+" }
                span { class: "text-xs text-gray-400 mt-1", "上传图片" }
            }
        }
    }
}
