use dioxus::prelude::*;
use serde_json::{Value, json};
use crate::components::*;
use crate::services::api;
use crate::services::stream;

#[component]
pub fn AiChat() -> Element {
    let mut convs = use_signal(Vec::<Value>::new);
    let mut msgs = use_signal(Vec::<Value>::new);
    let mut active = use_signal(String::new);
    let mut input_val = use_signal(String::new);
    let mut loading = use_signal(|| true);
    let mut msg_loading = use_signal(|| false);
    let mut sending = use_signal(|| false);
    let mut streaming = use_signal(String::new);
    let mut is_streaming = use_signal(|| false);

    let load = move || { loading.set(true); spawn(async move {
        if let Ok(v) = api::action("/ai/chat/conversation/my-list", json!({"pageNo":1,"pageSize":50})).await {
            if let Some(arr) = v["data"]["list"].as_array() { convs.set(arr.clone()); }
        }
        loading.set(false);
    }); };
    use_effect(load);

    let load_m = move || { let id = active(); if id.is_empty() { return; } msg_loading.set(true); let id = id.clone(); spawn(async move {
        if let Ok(v) = api::action("/ai/chat/message/list", json!({"conversationId":id,"pageNo":1,"pageSize":100})).await {
            if let Some(arr) = v["data"]["list"].as_array() { msgs.set(arr.clone()); }
        }
        msg_loading.set(false);
    }); };
    use_effect(load_m);

    let cl = convs(); let ml = msgs(); let aid = active(); let lv = loading();
    let lmv = msg_loading(); let sv = sending(); let iv = input_val(); let is_str = is_streaming(); let str_content = streaming();

    let conv_items: Vec<Element> = cl.iter().map(|c| {
        let id = c["id"].as_str().unwrap_or("").to_string();
        let title = c["title"].as_str().unwrap_or("未命名").to_string();
        let is_act = aid == id;
        rsx!{ div { class: format!("px-4 py-3 cursor-pointer text-sm border-b hover:bg-gray-50 {}", if is_act { "bg-primary-50 border-l-2 border-l-primary-500" } else { "" }),
            onclick: move |_| active.set(id.clone()), div { class: "font-medium text-gray-800 truncate", "{title}" }
        }}
    }).collect();

    let msg_items: Vec<Element> = ml.iter().map(|m| {
        let mtype = m["type"].as_str().unwrap_or("user");
        let content = m["content"].as_str().unwrap_or("").to_string();
        let is_user = mtype == "user";
        rsx!{ div { class: format!("flex mb-4 {}", if is_user { "justify-end" } else { "justify-start" }),
            div { class: format!("max-w-[70%] rounded-lg px-4 py-3 text-sm {}", if is_user { "bg-primary-500 text-white rounded-br-none" } else { "bg-gray-100 text-gray-800 rounded-bl-none" }),
                div { class: "whitespace-pre-wrap break-words", "{content}" }
            }
        }}
    }).collect();

    rsx!{ div { class: "bg-white rounded-lg shadow-sm border flex h-[calc(100vh-140px)]",
        div { class: "w-[280px] border-r flex flex-col flex-shrink-0",
            div { class: "p-4 border-b flex items-center justify-between",
                h3 { class: "text-base font-semibold", "对话列表" }
                button { class: "ant-btn ant-btn-primary text-xs py-1 px-3",
                    onclick: move |_| { spawn(async move {
                        if let Ok(v) = api::action("/ai/chat/conversation/create-my", json!({"title":"新对话","modelId":"deepseek-chat"})).await {
                            if let Some(id) = v["data"].as_str() { active.set(id.to_string()); }
                        }
                        loading.set(true);
                        if let Ok(v) = api::action("/ai/chat/conversation/my-list", json!({"pageNo":1,"pageSize":50})).await {
                            if let Some(arr) = v["data"]["list"].as_array() { convs.set(arr.clone()); }
                        }
                        loading.set(false);
                    }); },
                    "+ 新对话" }
            }
            div { class: "flex-1 overflow-y-auto",
                if lv && conv_items.is_empty() { Spinner { text: Some("加载中...".into()) } }
                else if conv_items.is_empty() { div { class: "px-4 py-8 text-center text-sm text-gray-400", "暂无对话" } }
                else { for item in &conv_items { {item.clone()} } }
            }
        }
        div { class: "flex-1 flex flex-col",
            div { class: "flex-1 overflow-y-auto p-6",
                if aid.is_empty() {
                    div { class: "text-center text-gray-400 mt-32",
                        span { class: "text-6xl block mb-4", "💬" }
                        p { class: "text-lg mb-2", "AI 智能对话" }
                        p { class: "text-sm", "选择或创建对话开始" }
                    }
                } else if lmv && !is_str { Spinner { text: Some("加载消息...".into()) } }
                else if msg_items.is_empty() && !is_str { div { class: "text-center text-gray-400 mt-20", p { "发送第一条消息" } } }
                else {
                    for item in &msg_items { {item.clone()} }
                }
                if is_str {
                    div { class: "flex mb-4 justify-start",
                        div { class: "max-w-[70%] rounded-lg px-4 py-3 text-sm bg-gray-100 text-gray-800 rounded-bl-none",
                            div { class: "whitespace-pre-wrap break-words", "{str_content}" }
                            span { class: "inline-block w-1.5 h-4 bg-gray-400 animate-pulse align-middle", " " }
                        }
                    }
                }
            }
            if !aid.is_empty() {
                div { class: "p-4 border-t bg-white", div { class: "flex gap-2",
                    input { class: "ant-input flex-1", placeholder: "输入消息...", value: "{iv}", disabled: sv,
                        oninput: move |e| input_val.set(e.value()),
                        onkeydown: move |e| { if e.key() == Key::Enter {
                            let t = input_val(); if t.is_empty() { return; }
                            let cid = active(); if cid.is_empty() { return; }
                            sending.set(true); is_streaming.set(true); streaming.set(String::new()); input_val.set(String::new());
                            let cid2 = cid.clone();
                            spawn(async move {
                                let token = api::get_token().unwrap_or_default();
                                stream::sse_post("/ai/chat/message/send-stream", &json!({"conversationId":cid2,"content":t,"modelId":"deepseek-chat","attachmentUrls":[]}), &token, |data| {
                                    if let Ok(v) = serde_json::from_str::<Value>(&data) {
                                        if let Some(c) = v["data"]["receive"]["content"].as_str() {
                                            streaming.set(format!("{}{}", streaming(), c));
                                        }
                                    }
                                }).await;
                                is_streaming.set(false); sending.set(false); msg_loading.set(true);
                                if let Ok(v) = api::action("/ai/chat/message/list", json!({"conversationId":cid2,"pageNo":1,"pageSize":100})).await {
                                    if let Some(arr) = v["data"]["list"].as_array() { msgs.set(arr.clone()); }
                                }
                                msg_loading.set(false);
                            });
                        }}
                    }
                    button { class: "ant-btn ant-btn-primary", disabled: sv || iv.is_empty(),
                        onclick: move |_| {
                            let t = input_val(); if t.is_empty() { return; }
                            let cid = active(); if cid.is_empty() { return; }
                            sending.set(true); is_streaming.set(true); streaming.set(String::new()); input_val.set(String::new());
                            let cid2 = cid.clone();
                            spawn(async move {
                                let token = api::get_token().unwrap_or_default();
                                stream::sse_post("/ai/chat/message/send-stream", &json!({"conversationId":cid2,"content":t,"modelId":"deepseek-chat","attachmentUrls":[]}), &token, |data| {
                                    if let Ok(v) = serde_json::from_str::<Value>(&data) {
                                        if let Some(c) = v["data"]["receive"]["content"].as_str() {
                                            streaming.set(format!("{}{}", streaming(), c));
                                        }
                                    }
                                }).await;
                                is_streaming.set(false); sending.set(false); msg_loading.set(true);
                                if let Ok(v) = api::action("/ai/chat/message/list", json!({"conversationId":cid2,"pageNo":1,"pageSize":100})).await {
                                    if let Some(arr) = v["data"]["list"].as_array() { msgs.set(arr.clone()); }
                                }
                                msg_loading.set(false);
                            });
                        },
                        if sv { "发送中..." } else { "发送" } }
                }}
            }
        }
    }}
}