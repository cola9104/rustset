use crate::{Route, services::state};
use crate::components::ToastContainer;
use crate::services::api::{self, MenuItem};
use dioxus::prelude::*;
use dioxus::router::*;
use dioxus::router::hooks::{use_route, use_navigator};

#[component]
pub fn AuthLayout() -> Element {
    rsx! { Outlet::<Route> {} }
}

#[component]
pub fn MainLayout() -> Element {
    let mut collapsed = use_signal(|| false);
    let mut show_search = use_signal(|| false);
    let mut show_user_menu = use_signal(|| false);
    let route: Route = use_route();

    // Fetch menus on mount
    let menu_data = use_resource(move || async move {
        let token = crate::services::api::get_token();
        web_sys::console::log_1(&format!("use_resource: token={}", token.is_some()).into());
        if token.is_none() { return (vec![], String::new(), String::new()); }
        match crate::services::api::get_permission_info().await {
            Ok(info) => {
                web_sys::console::log_1(&format!("menus loaded: {} items, user={}", info.menus.len(), info.user.nickname).into());
                (info.menus, info.user.nickname, info.user.avatar)
            }
            Err(e) => {
                web_sys::console::log_1(&format!("menu error: {}", e).into());
                (vec![], String::new(), String::new())
            }
        }
    });
    let menus_data = menu_data.read();
    let (menu_items, user_name, user_avatar) = match menus_data.as_ref() {
        Some((m, u, a)) => (m.clone(), u.clone(), a.clone()),
        None => (vec![], String::new(), String::new()),
    };
    let display_name = if user_name.is_empty() { "用户".to_string() } else { user_name };
    let avatar_letter = display_name.chars().next().unwrap_or('U').to_uppercase().to_string();
    let has_avatar = !user_avatar.is_empty() && (user_avatar.starts_with("http://") || user_avatar.starts_with("https://"));

    // Redirect if no token
    let nav = use_navigator();
    use_effect(move || {
        if crate::services::api::get_token().is_none() { nav.push(Route::Login {}); }
    });
    let avatar_letter = display_name.chars().next().unwrap_or('U').to_uppercase().to_string();

    // Find current page title from menus based on route
    let page_title = {
        let route_str = format!("{:?}", route);
        // The Debug format for Route is like "Route::SystemUser {}", extract path
        find_menu_title(&menu_items, &route_str)
    };

    rsx! {
        div { class: "flex h-screen overflow-hidden bg-gray-50",
            // ── Sidebar ──
            aside {
                class: format!("{} bg-sidebar-bg text-white flex flex-col transition-all duration-200 flex-shrink-0 overflow-hidden",
                    if collapsed() { "w-[64px]" } else { "w-[240px]" }),
                div { class: "flex items-center h-[56px] px-4 border-b border-white/10 flex-shrink-0 bg-gradient-to-r from-primary-700 to-primary-600",
                    div { class: "w-8 h-8 rounded-lg bg-white/20 flex items-center justify-center mr-3 flex-shrink-0",
                        span { class: "text-white font-bold text-sm", "R" }
                    }
                    span { class: "text-lg font-bold text-white whitespace-nowrap",
                        if collapsed() { "" } else { "RustSet" }
                    }
                }
                nav { class: "flex-1 overflow-y-auto py-2",
                    div { class: "text-white/30 text-[10px] px-4 py-1", "{menu_items.len()} 个菜单组" }
                    NavTree { collapsed, items: menu_items.clone(), depth: 0 }
                }
                div { class: "border-t border-white/10 flex-shrink-0",
                    div { class: "px-4 py-2",
                        span { class: "text-white/40 text-[10px]", "RustSet v0.2.0" }
                        if !collapsed() { span { class: "text-white/25 text-[10px] ml-2", "· Dioxus" } }
                    }
                    button {
                        class: "w-full text-white/60 hover:text-white text-xs py-1.5 rounded hover:bg-white/10 transition-colors",
                        onclick: move |_| collapsed.toggle(),
                        if collapsed() { "▶" } else { "◀ 收起" }
                    }
                }
            }
            // ── Main content ──
            div { class: "flex-1 flex flex-col overflow-hidden",
                header { class: "h-[56px] bg-white border-b flex items-center justify-between px-6 flex-shrink-0 shadow-sm z-10",
                    div { class: "flex items-center gap-4",
                        nav { class: "flex items-center gap-2 text-sm text-gray-500",
                            span { class: "text-gray-800 font-medium", "{page_title}" }
                        }
                    }
                    div { class: "flex items-center gap-3",
                        button {
                            class: "text-gray-400 hover:text-gray-600 transition-colors p-1.5 rounded-lg hover:bg-gray-100",
                            onclick: move |_| show_search.toggle(),
                            title: "搜索",
                            "🔍"
                        }
                        button {
                            class: "text-gray-400 hover:text-gray-600 relative transition-colors p-1.5 rounded-lg hover:bg-gray-100",
                            "🔔"
                            span { class: "absolute -top-0.5 -right-0.5 w-4 h-4 bg-red-500 text-white text-[10px] rounded-full flex items-center justify-center font-bold", "0" }
                        }
                        div { class: "relative",
                            button {
                                class: "flex items-center gap-2 text-sm text-gray-600 hover:text-gray-800 transition-colors p-1 rounded-lg hover:bg-gray-100",
                                onclick: move |_| show_user_menu.toggle(),
                                div { class: "w-7 h-7 bg-primary-500 text-white rounded-full flex items-center justify-center text-xs font-bold overflow-hidden",
                                    if has_avatar {
                                        img { src: "{user_avatar}", class: "w-full h-full object-cover", alt: "头像" }
                                    } else {
                                        span { "{avatar_letter}" }
                                    }
                                }
                                div { class: "flex items-center gap-1",
                                    span { class: "hidden sm:inline font-medium text-sm", "{display_name}" }
                                    span { class: "w-1.5 h-1.5 rounded-full bg-green-400 inline-block", title: "在线" }
                                }
                                span { class: "text-[10px] text-gray-400", if show_user_menu() { "▴" } else { "▾" } }
                            }
                            if show_user_menu() {
                                div { class: "absolute right-0 top-full mt-1 w-48 bg-white rounded-lg shadow-lg border py-1 z-50",
                                    a { href: "/system/profile", class: "block w-full text-left px-4 py-2 text-sm text-gray-700 hover:bg-gray-50 no-underline",
                                        "👤 个人中心"
                                    }
                                    div { class: "border-t my-1" }
                                    button {
                                        class: "w-full text-left px-4 py-2 text-sm text-red-500 hover:bg-red-50 flex items-center gap-2",
                                        onclick: move |_| {
                                            crate::services::api::clear_token();
                                            state::clear_state();
                                            let _ = web_sys::window().unwrap().location().set_href("/auth/login");
                                        },
                                        "🚪 退出登录"
                                    }
                                }
                            }
                        }
                    }
                }
                if show_search() {
                    div { class: "fixed inset-0 bg-black/40 z-[2000] flex items-start justify-center pt-[15vh]",
                        onclick: move |_| show_search.set(false),
                        div { class: "bg-white rounded-xl shadow-2xl w-[560px] max-w-[90vw] overflow-hidden",
                            onclick: |e| e.stop_propagation(),
                            div { class: "flex items-center gap-3 px-5 py-4 border-b",
                                span { class: "text-gray-400 text-lg", "🔍" }
                                input {
                                    class: "flex-1 border-none outline-none text-base",
                                    placeholder: "搜索页面...",
                                    autofocus: true,
                                    onkeydown: move |e| { if e.key() == Key::Escape { show_search.set(false); } }
                                }
                                span { class: "text-xs text-gray-400 bg-gray-100 px-2 py-0.5 rounded", "ESC" }
                            }
                            div { class: "px-5 py-4 text-sm text-gray-400 text-center", "输入关键词搜索页面" }
                        }
                    }
                }
                main { class: "flex-1 overflow-y-auto p-6", Outlet::<Route> {} }
            }
        }
        ToastContainer {}
    }
}

// ── Dynamic navigation from backend menu tree ──

#[component]
#[component]
fn NavTree(collapsed: Signal<bool>, items: Vec<MenuItem>, depth: u32) -> Element {
    rsx! {
        for item in items.iter().filter(|m| m.visible) {
            if item.component.to_uppercase() == "LAYOUT" || !item.children.is_empty() {
                MenuGroup { collapsed, label: item.name.clone(), icon_str: item.icon.clone(), sub_menus: item.children.clone(), depth, parent_path: item.path.clone() }
            } else if !item.path.is_empty() {
                MenuItemView { collapsed, path_str: item.path.clone(), icon_str: item.icon.clone(), label: item.name.clone(), depth }
            }
        }
    }
}

#[component]
fn MenuGroup(collapsed: Signal<bool>, label: String, icon_str: String, sub_menus: Vec<MenuItem>, depth: u32, parent_path: String) -> Element {
    let mut open = use_signal(|| false);
    let icon = if icon_str.is_empty() { "📁".to_string() } else { map_icon(&icon_str) };
    let pad = (depth as usize) * 12;
    if collapsed() {
        return rsx! { NavTree { collapsed, items: sub_menus, depth: depth + 1 } };
    }
    rsx! {
        div {
            button { class: "w-full flex items-center gap-3 px-4 py-2 text-sm text-white/55 hover:text-white/80 transition-colors",
                style: "padding-left: {pad + 16}px",
                onclick: move |_| open.toggle(),
                span { "{icon}" }
                span { class: "flex-1 text-left text-xs font-medium uppercase tracking-wider", "{label}" }
                span { class: "text-[10px]", if open() { "▾" } else { "▸" } }
            }
            if open() { div { NavTree { collapsed, items: sub_menus, depth: depth + 1 } } }
        }
    }
}

#[component]
fn MenuItemView(collapsed: Signal<bool>, path_str: String, icon_str: String, label: String, depth: u32) -> Element {
    let route: Route = use_route();
    let current_path = format!("{:?}", route);
    let active = current_path.contains(&path_str) && !path_str.is_empty();
    let bg = if active { "bg-primary-600/25 border-l-2 border-primary-400" } else { "border-l-2 border-transparent hover:bg-white/8" };
    let tc = if active { "text-white" } else { "text-white/65 hover:text-white/90" };
    let pad = (depth as usize) * 12;
    let icon = if icon_str.is_empty() { "📄".to_string() } else { map_icon(&icon_str) };
    let p = path_str.clone();
    rsx! {
        div { class: "flex items-center gap-3 px-4 py-2.5 text-sm {bg} {tc} transition-colors w-full text-left cursor-pointer",
            style: "padding-left: {pad + 16}px",
            onclick: move |_| { dioxus::router::navigator().push(p.clone()); },
            span { "{icon}" }
            if !collapsed() { span { "{label}" } }
        }
    }
}

fn NavGroup(
    collapsed: Signal<bool>,
    icon: &'static str,
    label: &'static str,
    open: bool,
    children: Element,
) -> Element {
    let mut is_open = use_signal(|| open);
    if collapsed() { return rsx! { {children} }; }
    rsx! {
        div {
            button { class: "w-full flex items-center gap-3 px-4 py-2 text-sm text-white/55 hover:text-white/80 transition-colors",
                onclick: move |_| is_open.toggle(),
                span { "{icon}" }
                span { class: "flex-1 text-left text-xs font-medium uppercase tracking-wider", "{label}" }
                span { class: "text-[10px]", if is_open() { "▾" } else { "▸" } }
            }
            if is_open() { div { class: "ml-2", {children} } }
        }
    }
}

#[component]
fn NavItem(
    collapsed: Signal<bool>,
    to: Route,
    icon: &'static str,
    label: &'static str,
) -> Element {
    let route: Route = use_route();
    let active = route == to;
    let bg = if active { "bg-primary-600/25 border-l-2 border-primary-400" } else { "border-l-2 border-transparent hover:bg-white/8" };
    let text_class = if active { "text-white" } else { "text-white/65 hover:text-white/90" };
    rsx! {
        Link { to, class: "flex items-center gap-3 px-4 py-2.5 text-sm {bg} {text_class} transition-colors w-full text-left no-underline",
            span { "{icon}" }
            if !collapsed() { span { "{label}" } }
        }
    }
}

// ── Helpers ──

fn route_to_path(r: &Route) -> String {
    // Extract path from Route debug format
    let s = format!("{:?}", r);
    // Format: "Route::SystemUser {}" or "Route::Dashboard {}" etc.
    // We need to match each variant. Use a match for the common ones.
    // For now, convert variant name to path
    let variant = s.split("::").last().unwrap_or("").split(' ').next().unwrap_or("");
    match variant {
        "Dashboard" => "/".to_string(),
        "DashboardWorkspace" => "/dashboard/workspace".to_string(),
        "Login" => "/auth/login".to_string(),
        "Register" => "/auth/register".to_string(),
        "ForgotPassword" => "/auth/forgot".to_string(),
        _ => {
            // Convert CamelCase to kebab-case path: SystemUser -> /system/user
            let path = variant
                .chars()
                .fold((String::new(), false), |(mut acc, prev_upper), c| {
                    if c.is_uppercase() {
                        if !acc.is_empty() && !prev_upper { acc.push('-'); }
                        acc.push(c.to_ascii_lowercase());
                        (acc, true)
                    } else {
                        acc.push(c);
                        (acc, false)
                    }
                }).0;
            format!("/{}", path)
        }
    }
}

fn find_menu_title(menus: &[MenuItem], route_str: &str) -> String {
    let current_path = {
        let variant = route_str.split("::").last().unwrap_or("").split(' ').next().unwrap_or("");
        if variant == "Dashboard" { return "仪表板".to_string(); }
        let path = variant
            .chars()
            .fold((String::new(), false), |(mut acc, prev_upper), c| {
                if c.is_uppercase() {
                    if !acc.is_empty() && !prev_upper { acc.push('-'); }
                    acc.push(c.to_ascii_lowercase());
                    (acc, true)
                } else { acc.push(c); (acc, false) }
            }).0;
        format!("/{}", path)
    };
    for item in menus {
        if item.path == current_path { return item.name.clone(); }
        if !item.children.is_empty() {
            if let Some(t) = find_menu_title_recursive(&item.children, &current_path) {
                return t;
            }
        }
    }
    "页面".to_string()
}

fn find_menu_title_recursive(menus: &[MenuItem], path: &str) -> Option<String> {
    for item in menus {
        if item.path == path { return Some(item.name.clone()); }
        if !item.children.is_empty() {
            if let Some(t) = find_menu_title_recursive(&item.children, path) { return Some(t); }
        }
    }
    None
}

fn map_icon(icon: &str) -> String {
    match icon {
        "lucide:settings" | "lucide:settings-2" => "⚙️".into(),
        "lucide:users" | "lucide:user-cog" => "👤".into(),
        "lucide:list-tree" | "lucide:list-checks" | "lucide:list-todo" => "📋".into(),
        "lucide:network" | "lucide:share-2" => "🏛️".into(),
        "lucide:briefcase-business" | "lucide:briefcase" => "💼".into(),
        "lucide:book-open" | "lucide:book-text" | "lucide:book-marked" => "📖".into(),
        "lucide:building-2" | "lucide:building" => "🏢".into(),
        "lucide:package" => "📦".into(),
        "lucide:map" => "🗺️".into(),
        "lucide:megaphone" | "lucide:mail" | "lucide:mail-plus" | "lucide:mails" | "lucide:mail-check" => "📢".into(),
        "lucide:key-round" | "lucide:key" | "lucide:ticket-check" => "🔑".into(),
        "lucide:file-clock" | "lucide:log-in" => "📝".into(),
        "lucide:database" | "lucide:database-zap" | "lucide:database-backup" => "🗄️".into(),
        "lucide:message-circle" | "lucide:messages-square" | "lucide:message-square-more" | "lucide:message-square-text" | "lucide:message-square-warning" => "💬".into(),
        "lucide:image" | "lucide:images" => "🖼️".into(),
        "lucide:pen-line" => "✍️".into(),
        "lucide:music" | "lucide:list-music" => "🎵".into(),
        "lucide:brain-circuit" => "🧠".into(),
        "lucide:bot" => "🤖".into(),
        "lucide:wrench" => "🔧".into(),
        "lucide:clapperboard" => "🎬".into(),
        "lucide:palette" => "🎨".into(),
        "lucide:layout-dashboard" => "📊".into(),
        "lucide:notebook-tabs" => "📖".into(),
        "lucide:file-stack" => "📄".into(),
        "lucide:wand-sparkles" => "🔮".into(),
        "lucide:workflow" => "🔀".into(),
        "lucide:folder-tree" => "📂".into(),
        "lucide:timer" => "⏰".into(),
        "lucide:scroll-text" => "📊".into(),
        "lucide:triangle-alert" => "⚠️".into(),
        "lucide:file" | "lucide:file-cog" | "lucide:file-text" | "lucide:files" => "📁".into(),
        "lucide:code-2" | "lucide:code" => "💻".into(),
        "lucide:panel-top" => "📋".into(),
        "lucide:flask-conical" | "lucide:flask-round" => "🧪".into(),
        "lucide:server" | "lucide:server-cog" => "🖥️".into(),
        "lucide:route" => "🔗".into(),
        "lucide:radio-tower" => "📡".into(),
        "lucide:blocks" => "🔧".into(),
        "lucide:sliders-horizontal" => "⚙️".into(),
        "lucide:users-round" => "👥".into(),
        "lucide:copy-check" => "✅".into(),
        "lucide:function-square" => "📐".into(),
        "lucide:radio" => "📻".into(),
        "lucide:file-plus-2" => "📄".into(),
        "lucide:file-pen-line" => "📝".into(),
        "lucide:search-check" => "🔍".into(),
        "lucide:clapperboard" => "🎬".into(),
        _ => "📄".into(),
    }
}
