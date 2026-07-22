pub mod cascader;
pub mod description;
pub mod steps;
pub mod timeline;
pub mod transfer;
pub mod tree;
pub mod upload;

use dioxus::prelude::*;
use serde_json::Value;

// ── Toast ──
#[derive(Clone, PartialEq)]
pub struct Toast { pub message: String, pub kind: ToastKind }
#[derive(Clone, PartialEq)]
pub enum ToastKind { Success, Error, Info, Warning }

pub fn toast(msg: &str, kind: ToastKind) {
    TOASTS.with_mut(|v| v.push(Toast { message: msg.into(), kind }));
}
static TOASTS: GlobalSignal<Vec<Toast>> = GlobalSignal::new(|| vec![]);

#[component]
pub fn ToastContainer() -> Element {
    let toasts = TOASTS();
    let items: Vec<_> = toasts.iter().take(5).map(|t| {
        let bg = match t.kind {
            ToastKind::Success => "bg-green-500",
            ToastKind::Error => "bg-red-500",
            ToastKind::Info => "bg-blue-500",
            ToastKind::Warning => "bg-orange-500",
        };
        let icon = match t.kind {
            ToastKind::Success => "✅", ToastKind::Error => "❌",
            ToastKind::Info => "ℹ️", ToastKind::Warning => "⚠️",
        };
        (bg, icon, t.message.clone())
    }).collect();
    rsx! {
        div { class: "fixed top-4 right-4 z-[2000] flex flex-col gap-2",
            for (bg, icon, msg) in &items {
                div { class: "{bg} text-white px-5 py-3 rounded-lg shadow-lg text-sm flex items-center gap-2",
                    span { "{icon}" }
                    "{msg}"
                }
            }
        }
    }
}

// ── Loading / Empty ──
#[component]
pub fn Spinner(text: Option<String>) -> Element {
    rsx! {
        div { class: "flex flex-col items-center justify-center py-12 text-gray-400",
            div { class: "w-8 h-8 border-2 border-primary-200 border-t-primary-600 rounded-full animate-spin mb-3" }
            if let Some(t) = text { p { class: "text-sm", "{t}" } }
        }
    }
}

#[component]
pub fn FullLoading() -> Element {
    rsx! {
        div { class: "w-full min-h-[300px] flex items-center justify-center",
            Spinner { text: Some("加载中...".into()) }
        }
    }
}

#[component]
pub fn EmptyState(text: Option<String>) -> Element {
    rsx! {
        div { class: "text-center py-12 text-gray-400",
            span { class: "text-4xl block mb-3", "📭" }
            p { class: "text-sm", { text.unwrap_or_else(|| "暂无数据".into()) } }
        }
    }
}

// ── StatCard ──
#[component]
pub fn StatCard(title: String, value: String, icon: String, color: String) -> Element {
    rsx! {
        div { class: "bg-white rounded-lg p-6 shadow-sm border hover:shadow-md transition-shadow",
            div { class: "flex items-center justify-between",
                div {
                    p { class: "text-sm text-gray-500 mb-1", "{title}" }
                    p { class: "text-2xl font-bold text-gray-800", "{value}" }
                }
                span { class: "text-2xl p-3 rounded-lg {color}", "{icon}" }
            }
        }
    }
}

// ── SearchBar ──
#[component]
pub fn SearchBar(placeholder: String, on_search: EventHandler<String>) -> Element {
    let mut val = use_signal(String::new);
    rsx! {
        div { class: "relative",
            input {
                class: "ant-input w-[260px] pl-9",
                placeholder: "{placeholder}",
                value: "{val}",
                oninput: move |e| {
                    let v = e.value();
                    val.set(v.clone());
                    on_search.call(v);
                }
            }
            span { class: "absolute left-3 top-1/2 -translate-y-1/2 text-gray-400 text-sm", "🔍" }
        }
    }
}

// ── PageHeader ──
#[component]
pub fn PageHeader(title: String, actions: Option<Element>) -> Element {
    rsx! {
        div { class: "flex items-center justify-between mb-4",
            h1 { class: "text-xl font-bold text-gray-800", "{title}" }
            if let Some(a) = actions {
                div { class: "flex items-center gap-3", {a} }
            }
        }
    }
}

// ── Column ──
#[derive(Clone, PartialEq)]
pub struct Column {
    pub key: String,
    pub title: String,
    pub width: Option<String>,
}

// ── DataTable ──
#[component]
pub fn DataTable(
    columns: Vec<Column>,
    data: Vec<Value>,
    loading: bool,
    page: i64,
    page_size: i64,
    total: i64,
    on_page_change: EventHandler<i64>,
    on_edit: Option<EventHandler<String>>,
    on_delete: Option<EventHandler<String>>,
    extra_row: Option<EventHandler<(String, Callback<Element>)>>,
) -> Element {
    let tp = ((total + page_size - 1) / page_size).max(1);
    let ac = on_edit.is_some() || on_delete.is_some() || extra_row.is_some();
    let col_count = columns.len() + if ac { 1 } else { 0 };
    let total_display = total;

    rsx! {
        div { class: "bg-white rounded-lg shadow-sm border overflow-hidden",
            table { class: "ant-table",
                thead {
                    tr {
                        for c in &columns {
                            th {
                                class: c.width.as_deref().map(|w| format!("w-[{w}]")),
                                "{c.title}"
                            }
                        }
                        if ac { th { class: "w-[200px]", "操作" } }
                    }
                }
                tbody {
                    if loading {
                        for _ in 0..5 {
                            tr { class: "animate-pulse",
                                td { colspan: "{col_count}",
                                    div { class: "h-4 bg-gray-200 rounded w-full" }
                                }
                            }
                        }
                    } else if data.is_empty() {
                        tr {
                            td { colspan: "{col_count}", EmptyState { text: None } }
                        }
                    } else {
                        for row in &data {
                            { let row_id = get_id(row);
                            let cols_c = columns.clone();
                            let edit_h = on_edit.clone();
                            let delete_h = on_delete.clone();
                            let row_clone = row.clone();
                            rsx! {
                                tr { class: "hover:bg-gray-100 even:bg-gray-50",
                                    for c in &cols_c {
                                        td { class: "text-sm py-3", { render_cell(&c.key, &row_clone) } }
                                    }
                                    if ac {
                                        td {
                                            div { class: "flex gap-1.5",
                                                if let Some(ref eh) = edit_h {
                                                    { let id = row_id.clone(); let h = eh.clone(); rsx! {
                                                        button {
                                                            class: "ant-btn text-xs py-0.5 px-2",
                                                            onclick: move |_| h.call(id.clone()),
                                                            "编辑"
                                                        }
                                                    }}
                                                }
                                                if let Some(ref dh) = delete_h {
                                                    { let id = row_id.clone(); let h = dh.clone(); rsx! {
                                                        button {
                                                            class: "ant-btn ant-btn-danger text-xs py-0.5 px-2",
                                                            onclick: move |_| {
                                                                if web_sys::window().map(|w| w.confirm_with_message("确认删除？").unwrap_or(false)).unwrap_or(false) {
                                                                    h.call(id.clone())
                                                                }
                                                            },
                                                            "删除"
                                                        }
                                                    }}
                                                }
                                            }
                                        }
                                    }
                                }
                            }}
                        }
                    }
                }
            }
            div { class: "ant-pagination border-t flex items-center justify-between px-4 py-3",
                span { class: "text-sm text-gray-500", "共 {total_display} 条" }
                if total_display > page_size {
                    Pagination { page, total_pages: tp, on_change: on_page_change }
                }
            }
        }
    }
}

pub fn get_id(row: &Value) -> String {
    row.get("id")
        .and_then(|v| {
            v.as_i64()
                .map(|n| n.to_string())
                .or_else(|| v.as_str().map(String::from))
        })
        .unwrap_or_default()
}

fn render_cell(key: &str, row: &Value) -> Element {
    let v = row.get(key);
    let s = match v {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Number(n)) => n.to_string(),
        Some(Value::Bool(b)) => b.to_string(),
        _ => v.and_then(|x| x.as_str()).unwrap_or("").to_string(),
    };

    let status_keys = [
        "status", "ticketStatus", "ecsStatus", "deliveryStatus",
        "applicationStatus", "processStatus",
    ];
    if status_keys.contains(&key) {
        let c = tag_color(&s);
        if !c.is_empty() {
            return rsx! { span { class: "ant-tag {c}", "{s}" } };
        }
    }
    if key == "severity" {
        let c = match s.as_str() {
            "Critical" | "critical" => "ant-tag-red",
            "High" | "high" => "ant-tag-orange",
            "Medium" | "medium" => "ant-tag-blue",
            _ => "",
        };
        if !c.is_empty() {
            return rsx! { span { class: "ant-tag {c}", "{s}" } };
        }
    }

    rsx! { span { class: "text-sm text-gray-700 max-w-[200px] truncate inline-block", title: "{s}", "{s}" } }
}

fn tag_color(s: &str) -> &'static str {
    match s {
        "active" | "delivered" | "已交付" | "resolved" | "completed" | "运行中" | "success"
        | "正常" | "启用" => "ant-tag-green",
        "pending" | "pending_approval" | "pending_provision" | "pending_delivery"
        | "待审核" | "open" | "待交付" | "处理中" => "ant-tag-blue",
        "rejected" | "error" | "failed" | "已拒绝" | "禁用" | "锁定" => "ant-tag-red",
        "inactive" | "stopped" | "disabled" | "已停止" | "停用" => "ant-tag-orange",
        "ignored" | "false_positive" | "verified" | "测试中" => "ant-tag-purple",
        _ => "",
    }
}

// ── Pagination ──
#[component]
pub fn Pagination(page: i64, total_pages: i64, on_change: EventHandler<i64>) -> Element {
    let prev_disabled = page <= 1; let next_disabled = page >= total_pages;
    let first_label = "«".to_string(); let prev_label = "‹".to_string();
    let next_label = "›".to_string(); let last_label = "»".to_string();
    rsx! {
        div { class: "flex items-center gap-1 p-2",
            PageBtn { disabled: prev_disabled, label: first_label,
                on_click: { let h = on_change.clone(); move |_| h.call(1) }
            }
            PageBtn { disabled: prev_disabled, label: prev_label,
                on_click: { let h = on_change.clone(); move |_| h.call(page - 1) }
            }
            for p in page_range(page, total_pages) {
                button {
                    class: format!("min-w-[32px] h-8 px-2 rounded text-sm {}",
                        if p == page { "bg-primary-600 text-white" } else { "hover:bg-gray-100" }),
                    onclick: { let h = on_change.clone(); move |_| h.call(p) },
                    "{p}"
                }
            }
            PageBtn { disabled: next_disabled, label: next_label,
                on_click: { let h = on_change.clone(); move |_| h.call(page + 1) }
            }
            PageBtn { disabled: next_disabled, label: last_label,
                on_click: { let h = on_change.clone(); move |_| h.call(total_pages) }
            }
        }
    }
}

#[component]
fn PageBtn(disabled: bool, label: String, on_click: EventHandler<()>) -> Element {
    rsx! {
        button {
            class: "min-w-[32px] h-8 px-2 rounded text-sm disabled:opacity-30 hover:bg-gray-100",
            disabled,
            onclick: move |_| on_click.call(()),
            "{label}"
        }
    }
}

fn page_range(c: i64, t: i64) -> Vec<i64> {
    ((c - 2).max(1)..=(c + 2).min(t)).collect()
}

// ── Modal ──
#[component]
pub fn Modal(
    title: String, visible: bool, width: Option<String>, loading: bool,
    on_close: EventHandler<()>, on_save: Option<EventHandler<()>>, children: Element,
) -> Element {
    if !visible { return rsx! {}; }
    let w = width.unwrap_or_else(|| "520px".into());
    rsx! {
        div { class: "ant-modal-overlay", onclick: move |_| on_close.call(()),
            div { class: "ant-modal", style: "min-width:{w};max-width:90vw",
                onclick: |e| e.stop_propagation(),
                div { class: "flex items-center justify-between mb-5",
                    h3 { class: "text-lg font-semibold", "{title}" }
                    button { class: "text-gray-400 hover:text-gray-600 text-xl leading-none",
                        onclick: move |_| on_close.call(()), "×"
                    }
                }
                div { class: "max-h-[55vh] overflow-y-auto", {children} }
                if on_save.is_some() {
                    div { class: "flex justify-end gap-2 mt-6 pt-4 border-t",
                        button { class: "ant-btn",
                            onclick: move |_| on_close.call(()), "取消"
                        }
                        button { class: "ant-btn ant-btn-primary", disabled: loading,
                            onclick: move |_| on_save.as_ref().unwrap().call(()),
                            if loading { "保存中..." } else { "确定" }
                        }
                    }
                }
            }
        }
    }
}

// ── Drawer ──
#[component]
pub fn Drawer(
    title: String, visible: bool, width: Option<String>,
    on_close: EventHandler<()>, children: Element,
) -> Element {
    if !visible { return rsx! {}; }
    let w = width.unwrap_or_else(|| "400px".into());
    let style_val = format!("width:{}", w);
    rsx! {
        div { class: "fixed inset-0 z-[1100]",
            div { class: "absolute inset-0 bg-black/40", onclick: move |_| on_close.call(()) }
            div { class: "absolute right-0 top-0 h-full bg-white shadow-2xl overflow-y-auto",
                style: "{style_val}",
                div { class: "flex items-center justify-between p-4 border-b sticky top-0 bg-white z-10",
                    h3 { class: "text-lg font-semibold", "{title}" }
                    button { class: "text-gray-400 hover:text-gray-600 text-xl leading-none",
                        onclick: move |_| on_close.call(()), "×"
                    }
                }
                div { class: "p-4", {children} }
            }
        }
    }
}

// ── Tabs ──
#[component]
pub fn Tabs(active: String, items: Vec<(String, String)>, on_change: EventHandler<String>) -> Element {
    rsx! {
        div { class: "flex gap-1 border-b mb-4",
            for (key, label) in &items {
                button {
                    class: format!("px-4 py-2 text-sm font-medium transition-colors border-b-2 {}",
                        if active == *key { "border-primary-600 text-primary-600" }
                        else { "border-transparent text-gray-500 hover:text-gray-700" }),
                    onclick: {
                        let h = on_change.clone();
                        let k = key.clone();
                        move |_| h.call(k.clone())
                    },
                    "{label}"
                }
            }
        }
    }
}

// ── Form Fields ──
#[component]
pub fn FormItem(label: String, required: bool, children: Element) -> Element {
    rsx! {
        div { class: "mb-4 flex items-start gap-3",
            label { class: "w-24 flex-shrink-0 pt-2 text-right text-sm font-medium text-gray-700",
                if required { span { class: "text-red-500 mr-0.5", "*" } }
                "{label}"
            }
            div { class: "flex-1", {children} }
        }
    }
}

#[component]
pub fn InputField(value: String, placeholder: String, on_change: EventHandler<String>) -> Element {
    rsx! {
        input {
            class: "ant-input", r#type: "text",
            placeholder: "{placeholder}", value: "{value}",
            oninput: move |e| on_change.call(e.value())
        }
    }
}

#[component]
pub fn SelectField(value: String, options: Vec<(String, String)>, on_change: EventHandler<String>) -> Element {
    rsx! {
        select { class: "ant-input", value: "{value}",
            onchange: move |e| on_change.call(e.value()),
            for (val, label) in &options {
                option { value: "{val}", "{label}" }
            }
        }
    }
}

#[component]
pub fn TextareaField(value: String, rows: i32, on_change: EventHandler<String>) -> Element {
    rsx! {
        textarea {
            class: "ant-input", rows: "{rows}",
            value: "{value}",
            oninput: move |e| on_change.call(e.value())
        }
    }
}

// ── Card (inline for Dioxus compat) ──
// Card is not a #[component] because of Dioxus 0.7 completions name resolution issues.
// Use card() inline function or the CardSection macro pattern below instead.
pub fn render_card(header: Option<&str>, extra: Option<Element>, body: Element) -> Element {
    rsx! {
        div { class: "bg-white rounded-lg shadow-sm border",
            if header.is_some() || extra.is_some() {
                div { class: "flex items-center justify-between px-6 py-4 border-b",
                    if let Some(h) = header { h3 { class: "text-base font-semibold", "{h}" } }
                    if let Some(e) = extra { {e} }
                }
            }
            div { class: "p-6", {body} }
        }
    }
}

/// Reusable CRUD data page — proper #[component] so hooks work safely
#[component]
pub fn CrudPage(title: String, module: String, prefix: String, columns: Vec<Column>) -> Element {
    let pfx = prefix;
    let m = module.clone();
    let mut data = use_signal(Vec::<Value>::new);
    let mut total = use_signal(|| 0i64);
    let mut pg = use_signal(|| 1i64);
    let mut loading = use_signal(|| true);
    let mut show = use_signal(|| false);
    let mut edit_id = use_signal(|| None::<String>);
    let mut form = use_signal(|| serde_json::json!({}));
    let mut saving = use_signal(|| false);
    let mut keyword = use_signal(String::new);

    let load = {
        let m = m.clone();
        let pfx = pfx.clone();
        let cols = columns.clone();
        move || {
            let m = m.clone();
            let pfx = pfx.clone();
            let kw = keyword();
            let sk = cols.first().map(|c| c.key.clone()).unwrap_or_else(|| "name".to_string());
            spawn(async move {
                loading.set(true);
                if !kw.is_empty() {
                    let path = format!("/{}/{}/page?pageNo={}&pageSize=10&{}={}", pfx, m, pg(), sk, kw);
                    if let Ok(v) = crate::services::api::get_path(&path).await {
                        if let Some(arr) = v["data"]["list"].as_array() { data.set(arr.clone()); }
                        total.set(v["data"]["total"].as_i64().unwrap_or(0));
                    }
                } else {
                    let api_pg = if pfx == "system" {
                        crate::services::api::sys_page(&m, pg(), 10).await
                    } else {
                        crate::services::api::page(&m, pg(), 10).await
                    };
                    if let Ok((l, t)) = api_pg { data.set(l); total.set(t); }
                }
                loading.set(false);
            });
        }
    };
    use_effect(load);

    let on_save = {
        let m = m.clone();
        let pfx = pfx.clone();
        move |_| {
            let f = form();
            let m = m.clone();
            let pfx = pfx.clone();
            saving.set(true);
            spawn(async move {
                let r = if edit_id().is_some() {
                    if pfx == "system" { crate::services::api::sys_update(&m, f).await }
                    else { crate::services::api::update(&m, f).await }
                } else {
                    if pfx == "system" { crate::services::api::sys_create(&m, f).await.map(|_| ()) }
                    else { crate::services::api::create(&m, f).await.map(|_| ()) }
                };
                saving.set(false);
                if r.is_ok() { toast("保存成功", ToastKind::Success); show.set(false); pg.set(1); }
                else { toast(&format!("失败:{}", r.unwrap_err()), ToastKind::Error); }
            });
        }
    };

    let dt = data();
    let tot = total();
    let p = pg();
    let ld = loading();
    let sv = saving();
    let sh = show();
    let kw = keyword();
    let m2 = m.clone();
    let pfx2 = pfx.clone();
    let cols2 = columns.clone();
    let m3 = m.clone();
    let pfx3 = pfx.clone();
    let cols3 = columns.clone();
    let m4 = m.clone();
    let pfx4 = pfx.clone();
    let t = title.clone();
    let cols4 = columns.clone();

    rsx! {
        PageHeader {
            title: t.clone(),
            actions: rsx! {
                button {
                    class: "ant-btn ant-btn-primary",
                    onclick: move |_| { form.set(serde_json::json!({})); edit_id.set(None); show.set(true); },
                    "+ 新增"
                }
            }
        }
        div { class: "flex gap-2 mb-4",
            input {
                class: "ant-input w-[260px]",
                placeholder: "搜索...",
                value: "{kw}",
                oninput: move |e| keyword.set(e.value()),
                onkeydown: {
                    let sk = cols2.first().map(|c| c.key.clone()).unwrap_or_else(|| "name".to_string());
                    let m = m2.clone();
                    let pfx = pfx2.clone();
                    move |e| {
                        if e.key() == Key::Enter {
                            let m = m.clone();
                            let pfx = pfx.clone();
                            let sk = sk.clone();
                            let kw2 = keyword();
                            spawn(async move {
                                loading.set(true);
                                if !kw2.is_empty() {
                                    let path = format!("/{}/{}/page?pageNo={}&pageSize=10&{}={}", pfx, m, pg(), sk, kw2);
                                    if let Ok(v) = crate::services::api::get_path(&path).await {
                                        if let Some(arr) = v["data"]["list"].as_array() { data.set(arr.clone()); }
                                        total.set(v["data"]["total"].as_i64().unwrap_or(0));
                                    }
                                } else {
                                    let api_pg = if pfx == "system" {
                                        crate::services::api::sys_page(&m, pg(), 10).await
                                    } else {
                                        crate::services::api::page(&m, pg(), 10).await
                                    };
                                    if let Ok((l, t)) = api_pg { data.set(l); total.set(t); }
                                }
                                loading.set(false);
                            });
                        }
                    }
                }
            }
            button {
                class: "ant-btn ant-btn-primary text-sm",
                onclick: {
                    let sk2 = cols3.first().map(|c| c.key.clone()).unwrap_or_else(|| "name".to_string());
                    let m = m3.clone();
                    let pfx = pfx3.clone();
                    move |_| {
                        let m = m.clone();
                        let pfx = pfx.clone();
                        let sk2 = sk2.clone();
                        let kw2 = keyword();
                        spawn(async move {
                            loading.set(true);
                            if !kw2.is_empty() {
                                let path = format!("/{}/{}/page?pageNo={}&pageSize=10&{}={}", pfx, m, pg(), sk2, kw2);
                                if let Ok(v) = crate::services::api::get_path(&path).await {
                                    if let Some(arr) = v["data"]["list"].as_array() { data.set(arr.clone()); }
                                    total.set(v["data"]["total"].as_i64().unwrap_or(0));
                                }
                            } else {
                                let api_pg = if pfx == "system" {
                                    crate::services::api::sys_page(&m, pg(), 10).await
                                } else {
                                    crate::services::api::page(&m, pg(), 10).await
                                };
                                if let Ok((l, t)) = api_pg { data.set(l); total.set(t); }
                            }
                            loading.set(false);
                        });
                    }
                },
                "🔍 搜索"
            }
            button {
                class: "ant-btn text-sm",
                onclick: {
                    let m = m4.clone();
                    let pfx = pfx4.clone();
                    move |_| {
                        keyword.set(String::new());
                        let m = m.clone();
                        let pfx = pfx.clone();
                        spawn(async move {
                            loading.set(true);
                            let api_pg = if pfx == "system" {
                                crate::services::api::sys_page(&m, pg(), 10).await
                            } else {
                                crate::services::api::page(&m, pg(), 10).await
                            };
                            if let Ok((l, t)) = api_pg { data.set(l); total.set(t); }
                            loading.set(false);
                        });
                    }
                },
                "↺ 重置"
            }
        }
        DataTable {
            columns: columns.clone(),
            data: dt,
            loading: ld,
            page: p,
            page_size: 10,
            total: tot,
            on_page_change: EventHandler::new(move |p: i64| pg.set(p)),
            on_edit: Some(EventHandler::new(move |id: String| {
                if let Some(i) = data().iter().find(|v| get_id(v) == id) {
                    form.set(i.clone());
                    edit_id.set(Some(id));
                    show.set(true);
                }
            })),
            on_delete: Some(EventHandler::new({
                let m = m.clone();
                let pfx = pfx.clone();
                move |id: String| {
                    let m = m.clone();
                    let pfx = pfx.clone();
                    spawn(async move {
                        let _ = if pfx == "system" {
                            crate::services::api::sys_remove(&m, &id).await
                        } else {
                            crate::services::api::remove(&m, &id).await
                        };
                        toast("删除成功", ToastKind::Success);
                    });
                }
            })),
            extra_row: None,
        }
        Modal {
            title: if edit_id().is_some() { "编辑".to_string() } else { "新增".to_string() },
            visible: sh,
            width: None,
            loading: sv,
            on_close: EventHandler::new(move |_| show.set(false)),
            on_save: Some(EventHandler::new(on_save)),
            children: {
                let cols = columns.clone();
                let items: Vec<_> = cols.iter().map(|c| {
                    let k = c.key.clone();
                    let l = c.title.clone();
                    let ft = form_type(&k);
                    match ft {
                        0 => rsx! {
                            FormItem { label: l.clone(), required: false,
                                children: {
                                    let k = k.clone();
                                    rsx! {
                                        select {
                                            class: "ant-input",
                                            onchange: move |e| { let mut f = form(); f[k.clone()] = serde_json::json!(e.value()); form.set(f); },
                                            option { value: "", "请选择" }
                                            option { value: "0", "启用" }
                                            option { value: "1", "禁用" }
                                        }
                                    }
                                }
                            }
                        },
                        1 => rsx! {
                            FormItem { label: l.clone(), required: false,
                                children: {
                                    let k = k.clone();
                                    rsx! {
                                        input {
                                            class: "ant-input",
                                            r#type: "number",
                                            placeholder: "请输入{l}",
                                            oninput: move |e| {
                                                let mut f = form();
                                                f[k.clone()] = serde_json::json!(e.value().parse::<i64>().unwrap_or(0));
                                                form.set(f);
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        2 => rsx! {
                            FormItem { label: l.clone(), required: false,
                                children: {
                                    let k = k.clone();
                                    rsx! {
                                        textarea {
                                            class: "ant-input",
                                            rows: "3",
                                            placeholder: "请输入{l}",
                                            oninput: move |e| { let mut f = form(); f[k.clone()] = serde_json::json!(e.value()); form.set(f); }
                                        }
                                    }
                                }
                            }
                        },
                        _ => rsx! {
                            FormItem { label: l.clone(), required: false,
                                children: {
                                    let k = k.clone();
                                    rsx! {
                                        input {
                                            class: "ant-input",
                                            placeholder: "请输入{l}",
                                            oninput: move |e| { let mut f = form(); f[k.clone()] = serde_json::json!(e.value()); form.set(f); }
                                        }
                                    }
                                }
                            }
                        },
                    }
                }).collect::<Vec<_>>();
                rsx! { for item in items { {item} } }
            },
        }
    }
}

fn form_type(key: &str) -> u8 {
    let k = key.to_lowercase();
    if k == "status" || k.ends_with("status") { 0 }
    else if k == "sort" || k.contains("sort") || k == "order" || k.contains("age") || k == "type" || (k.contains("id") && k != "id") { 1 }
    else if k.contains("remark") || k.contains("desc") || k.contains("content") || k.contains("note") || k.contains("address") { 2 }
    else { 3 }
}

#[component]
pub fn Breadcrumb(items: Vec<(String, Option<String>)>) -> Element {
    rsx! {
        nav { class: "flex items-center gap-2 text-sm text-gray-500 mb-4",
            for (i, (label, _href)) in items.iter().enumerate() {
                if i > 0 { span { class: "text-gray-300", "/" } }
                span {
                    class: if i == items.len() - 1 { "text-gray-800 font-medium" },
                    "{label}"
                }
            }
        }
    }
}
