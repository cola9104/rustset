use dioxus::prelude::*;
use crate::components::*;
use crate::services::state;

#[component]
pub fn Login() -> Element {
    let mut u = use_signal(String::new);
    let mut p = use_signal(String::new);
    let mut err = use_signal(|| None::<String>);
    let mut loading = use_signal(|| false);
    let mut remember = use_signal(|| false);

    rsx! {
        div { class: "min-h-screen flex",
            // Left brand panel
            div { class: "hidden lg:flex lg:w-[480px] bg-gradient-to-br from-primary-700 via-primary-600 to-primary-900 flex-col items-center justify-center relative overflow-hidden",
                div { class: "absolute inset-0 opacity-10",
                    div { class: "absolute top-20 left-10 w-60 h-60 rounded-full bg-white" }
                    div { class: "absolute bottom-20 right-10 w-80 h-80 rounded-full bg-white" }
                    div { class: "absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-40 h-40 rounded-full bg-white" }
                }
                div { class: "relative z-10 text-center px-12",
                    div { class: "w-20 h-20 rounded-2xl bg-white/20 backdrop-blur flex items-center justify-center mx-auto mb-6",
                        span { class: "text-4xl font-bold text-white", "R" }
                    }
                    h1 { class: "text-4xl font-bold text-white mb-3", "RustSet" }
                    p { class: "text-primary-200 text-lg", "企业级资产管理平台" }
                    div { class: "mt-8 flex gap-3 justify-center",
                        for (icon,text) in [("🔒","安全可靠"),("⚡","高性能"),("☁️","云原生")] {
                            div { class: "bg-white/10 backdrop-blur rounded-xl px-5 py-3 text-white text-sm",
                                span { class: "mr-2", "{icon}" } span { "{text}" }
                            }
                        }
                    }
                }
            }
            // Right login form
            div { class: "flex-1 flex items-center justify-center bg-gray-50 p-8",
                div { class: "w-full max-w-[420px]",
                    div { class: "mb-10",
                        h2 { class: "text-2xl font-bold text-gray-800", "欢迎回来" }
                        p { class: "text-gray-500 mt-2", "登录您的账户以继续" }
                    }
                    if let Some(ref e) = err() {
                        div { class: "bg-red-50 border border-red-200 text-red-600 px-4 py-3 rounded-lg mb-6 text-sm flex items-center gap-2",
                            span { "⚠️" } span { "{e}" }
                        }
                    }
                    div { class: "space-y-5",
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-2", "用户名" }
                            div { class: "relative",
                                span { class: "absolute left-3 top-1/2 -translate-y-1/2 text-gray-400", "👤" }
                                input { class: "ant-input pl-10 py-2.5", placeholder: "请输入用户名", value: "{u}", oninput: move |e| u.set(e.value()) }
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-2", "密码" }
                            div { class: "relative",
                                span { class: "absolute left-3 top-1/2 -translate-y-1/2 text-gray-400", "🔒" }
                                input { class: "ant-input pl-10 py-2.5", r#type: "password", placeholder: "请输入密码", value: "{p}",
                                    oninput: move |e| p.set(e.value()),
                                    onkeydown: move |e| { if e.key() == Key::Enter {
                                        let un = u(); let pw = p(); if un.is_empty() || pw.is_empty() { return; }
                                        loading.set(true); err.set(None);
                                        spawn(async move {
                                            let req = crate::services::api::LoginRequest { username: un, password: pw, tenant_id: None };
                                            match crate::services::api::login(&req).await {
                                                Ok(_) => { state::load_permission_info().await; let _ = web_sys::window().unwrap().location().set_href("/"); }
                                                Err(e) => { err.set(Some(e)); loading.set(false); }
                                            }
                                        });
                                    }}
                                }
                            }
                        }
                        div { class: "flex items-center justify-between",
                            label { class: "flex items-center gap-2 cursor-pointer",
                                input { r#type: "checkbox", class: "w-4 h-4 rounded accent-primary-500", checked: remember(), onchange: move |_| remember.toggle() }
                                span { class: "text-sm text-gray-500", "记住我" }
                            }
                            a { href: "/auth/forgot", class: "text-sm text-primary-600 hover:text-primary-700", "忘记密码？" }
                        }
                        button { class: "ant-btn ant-btn-primary w-full justify-center py-2.5 text-base",
                            disabled: loading(),
                            onclick: move |_| {
                                let un = u(); let pw = p(); if un.is_empty() || pw.is_empty() { return; }
                                loading.set(true); err.set(None);
                                spawn(async move {
                                    let req = crate::services::api::LoginRequest { username: un, password: pw, tenant_id: None };
                                    match crate::services::api::login(&req).await {
                                        Ok(_) => { state::load_permission_info().await; let _ = web_sys::window().unwrap().location().set_href("/"); }
                                        Err(e) => { err.set(Some(e)); loading.set(false); }
                                    }
                                });
                            },
                            if loading() { span { class: "inline-block w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin mr-2" } "登录中..." }
                            else { "登 录" }
                        }
                    }
                    div { class: "mt-8 text-center text-sm text-gray-400",
                        "还没有账户？" a { href: "/auth/register", class: "text-primary-600 hover:text-primary-700 ml-1", "立即注册" }
                    }
                }
            }
        }
    }
}

fn auth_card(title: &str, msg: &str) -> Element {
    let t = title.to_string(); let m = msg.to_string();
    rsx!{ div{ class:"min-h-screen flex items-center justify-center bg-gray-100",
        div{ class:"bg-white rounded-lg shadow-sm border w-[400px]",
            div{ class:"flex items-center justify-between px-6 py-4 border-b", h3{ class:"text-base font-semibold","{t}"}}
            div{ class:"p-6", p{ class:"text-gray-500","{m}"}}
        }
    }}
}

#[component] pub fn Register() -> Element { auth_card("用户注册", "注册页面建设中...") }
#[component] pub fn ForgotPassword() -> Element { auth_card("忘记密码", "密码重置页面建设中...") }

#[component] pub fn SystemProfile() -> Element {
    rsx!{
        div{
            h1{ class:"text-xl font-bold text-gray-800 mb-6","个人中心"}
            div{ class:"grid grid-cols-3 gap-6",
                div{ class:"col-span-1",
                    div{ class:"bg-white rounded-lg shadow-sm border p-6 text-center",
                        div{ class:"w-20 h-20 bg-primary-500 text-white rounded-full flex items-center justify-center text-2xl font-bold mx-auto mb-4","A"}
                        h2{ class:"text-lg font-semibold","管理员"}
                        p{ class:"text-sm text-gray-500","admin@rustset.local"}
                        button{ class:"ant-btn w-full mt-6","编辑资料"}
                    }
                    div{ class:"bg-white rounded-lg shadow-sm border mt-4",
                        div{ class:"px-6 py-4 border-b", h3{ class:"text-base font-semibold","安全设置"}}
                        div{ class:"p-4 flex flex-col gap-2",
                            button{ class:"ant-btn w-full text-left text-sm","🔒 修改密码"}
                            button{ class:"ant-btn w-full text-left text-sm","📱 绑定手机"}
                            button{ class:"ant-btn w-full text-left text-sm","📧 绑定邮箱"}
                        }
                    }
                }
                div{ class:"col-span-2",
                    div{ class:"bg-white rounded-lg shadow-sm border p-6 mb-4",
                        h3{ class:"text-base font-semibold mb-4","基本信息"}
                        div{ class:"grid grid-cols-2 gap-4 text-sm",
                            for (label,value) in PROFILE_FIELDS {
                                div{ div{ class:"text-gray-400 mb-1","{label}"} div{ class:"text-gray-800","{value}"}}
                            }
                        }
                    }
                    div{ class:"bg-white rounded-lg shadow-sm border p-6",
                        h3{ class:"text-base font-semibold mb-4","最近登录"}
                        for (time,ip,device) in LOGIN_HISTORY {
                            div{ class:"flex items-center justify-between py-2 border-b last:border-0 text-sm",
                                span{ class:"text-gray-600","{time}"} span{ class:"text-gray-500","{ip}"} span{ class:"text-gray-400 text-xs","{device}"}
                            }
                        }
                    }
                }
            }
        }
    }
}

const PROFILE_FIELDS: &[(&str, &str)] = &[
    ("用户名", "admin"), ("昵称", "管理员"), ("邮箱", "admin@rustset.local"),
    ("手机", "138****8888"), ("部门", "技术部"), ("角色", "超级管理员"),
    ("创建时间", "2024-01-01"), ("最后登录", "2026-07-22"),
];
const LOGIN_HISTORY: &[(&str, &str, &str)] = &[
    ("2026-07-22 09:30", "192.168.1.100", "Chrome / Windows"),
    ("2026-07-21 18:15", "192.168.1.100", "Chrome / Windows"),
    ("2026-07-20 08:45", "10.0.0.50", "Safari / macOS"),
];
