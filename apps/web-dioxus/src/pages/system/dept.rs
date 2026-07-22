use dioxus::prelude::*;
use serde_json::{Value, json};
use crate::components::*;
use crate::components::tree::{Tree, TreeNode};
use crate::services::api;

// ── Department Management (tree + detail + CRUD) ──

#[component]
pub fn SystemDeptTree() -> Element {
    let mut depts = use_signal(Vec::<Value>::new);
    let mut selected_id = use_signal(|| None::<String>);
    let mut loading = use_signal(|| true);
    let mut show_form = use_signal(|| false);
    let mut edit_id = use_signal(|| None::<String>);
    let mut form = use_signal(|| json!({ "status": "0" }));
    let mut saving = use_signal(|| false);

    let load = {
        move || {
            spawn(async move {
                loading.set(true);
                if let Ok(list) = api::sys_list("dept").await { depts.set(list); }
                loading.set(false);
            });
        }
    };
    use_effect(load);

    let tree = build_dept_tree(&depts(), 0i64);
    let sel_id = selected_id.read().clone();
    let selected_dept = sel_id.as_ref().and_then(|id| depts().iter().find(|d| get_id(d) == *id).cloned());
    let dept_status = selected_dept.as_ref().map(|d| if d["status"].as_str() == Some("0") { "启用" } else { "禁用" }).unwrap_or("");
    let dept_sort = selected_dept.as_ref().and_then(|d| d["sort"].as_i64()).unwrap_or(0).to_string();
    let dept_name = selected_dept.as_ref().and_then(|d| d["name"].as_str()).unwrap_or("-").to_string();
    let dept_leader = selected_dept.as_ref().and_then(|d| d["leader"].as_str()).unwrap_or("-").to_string();
    let dept_phone = selected_dept.as_ref().and_then(|d| d["phone"].as_str()).unwrap_or("-").to_string();
    let dept_email = selected_dept.as_ref().and_then(|d| d["email"].as_str()).unwrap_or("-").to_string();

    rsx! {
        div {
            h1 { class: "text-xl font-bold text-gray-800 mb-4", "部门管理" }
            div { class: "grid grid-cols-3 gap-4",
                div { class: "col-span-1 bg-white rounded-lg shadow-sm border",
                    div { class: "flex items-center justify-between px-4 py-3 border-b",
                        h3 { class: "text-sm font-semibold", "部门树" }
                        button { class: "ant-btn ant-btn-primary text-xs py-0.5 px-2",
                            onclick: move |_| {
                                form.set(json!({"status":"0","parentId":0}));
                                edit_id.set(None);
                                show_form.set(true);
                            },
                            "+ 新增"
                        }
                    }
                    div { class: "p-2 max-h-[60vh] overflow-y-auto",
                        if loading() { Spinner { text: Some("加载中...".into()) } }
                        else if tree.is_empty() { EmptyState { text: Some("暂无部门".into()) } }
                        else {
                            Tree {
                                nodes: tree,
                                selected: selected_id.read().clone(),
                                on_select: EventHandler::new(move |id: String| selected_id.set(Some(id))),
                            }
                        }
                    }
                }
                div { class: "col-span-2",
                    if selected_dept.is_some() {
                        div { class: "bg-white rounded-lg shadow-sm border p-6",
                            h3 { class: "text-base font-semibold mb-4", "部门信息" }
                            div { class: "grid grid-cols-2 gap-3 text-sm",
                                InfoRow { label: "名称".to_string(), val: dept_name }
                                InfoRow { label: "负责人".to_string(), val: dept_leader }
                                InfoRow { label: "电话".to_string(), val: dept_phone }
                                InfoRow { label: "邮箱".to_string(), val: dept_email }
                                InfoRow { label: "排序".to_string(), val: dept_sort }
                                InfoRow { label: "状态".to_string(), val: dept_status.to_string() }
                            }
                            div { class: "flex gap-2 mt-6 pt-4 border-t",
                                button { class: "ant-btn ant-btn-primary text-sm",
                                    onclick: {
                                        let d = selected_dept.clone();
                                        move |_| {
                                            if let Some(ref d) = d {
                                                form.set(d.clone());
                                                edit_id.set(Some(get_id(d)));
                                                show_form.set(true);
                                            }
                                        }
                                    },
                                    "编辑"
                                }
                                button { class: "ant-btn ant-btn-danger text-sm",
                                    onclick: {
                                        let id = sel_id.clone();
                                        move |_| {
                                            if let Some(ref id) = id {
                                                let id = id.clone();
                                                if web_sys::window().map(|w| w.confirm_with_message("确认删除？").unwrap_or(false)).unwrap_or(false) {
                                                    spawn(async move {
                                                        let _ = api::sys_remove("dept", &id).await;
                                                        toast("删除成功", ToastKind::Success);
                                                        selected_id.set(None);
                                                        if let Ok(list) = api::sys_list("dept").await { depts.set(list); }
                                                    });
                                                }
                                            }
                                        }
                                    },
                                    "删除"
                                }
                            }
                        }
                    } else {
                        div { class: "bg-white rounded-lg shadow-sm border p-12 text-center text-gray-400",
                            span { class: "text-5xl block mb-4", "🏛️" }
                            p { class: "text-lg", "选择一个部门查看详情" }
                        }
                    }
                }
            }
            // Form Modal
            Modal {
                title: if edit_id().is_some() { "编辑部门".to_string() } else { "新增部门".to_string() },
                visible: show_form(),
                width: None,
                loading: saving(),
                on_close: EventHandler::new(move |_| show_form.set(false)),
                on_save: Some(EventHandler::new({
                    let mut edit_id = edit_id;
                    let mut show_form = show_form;
                    let mut saving = saving;
                    let mut form = form;
                    let mut depts = depts;
                    move |_| {
                        let f = form.read().clone();
                        saving.set(true);
                        let is_edit = edit_id.read().is_some();
                        spawn(async move {
                            let r = if is_edit { api::sys_update("dept", f).await }
                            else { api::sys_create("dept", f).await.map(|_| ()) };
                            saving.set(false);
                            if r.is_ok() { toast("保存成功", ToastKind::Success); show_form.set(false);
                                if let Ok(list) = api::sys_list("dept").await { depts.set(list); } }
                            else { toast(&format!("失败:{}", r.unwrap_err()), ToastKind::Error); }
                        });
                    }
                })),
                children: rsx! {
                    FormItem { label: "名称".to_string(), required: true, children: rsx! {
                        input { class: "ant-input", value: form().get("name").and_then(|v|v.as_str()).unwrap_or("").to_string(),
                            oninput: move |e| { let mut f = form(); f["name"] = json!(e.value()); form.set(f); } }
                    }}
                    FormItem { label: "上级部门".to_string(), required: false, children: rsx! {
                        select { class: "ant-input",
                            onchange: move |e| { let mut f = form(); f["parentId"] = json!(e.value().parse::<i64>().unwrap_or(0)); form.set(f); },
                            option { value: "0", "根部门" }
                            for d in depts() {
                                option {
                                    value: "{d[\"id\"].as_i64().unwrap_or(0)}",
                                    selected: form().get("parentId").and_then(|v|v.as_i64()).unwrap_or(0) == d["id"].as_i64().unwrap_or(0),
                                    "{d[\"name\"].as_str().unwrap_or(\"\")}"
                                }
                            }
                        }
                    }}
                    FormItem { label: "负责人".to_string(), required: false, children: rsx! {
                        input { class: "ant-input", value: form().get("leader").and_then(|v|v.as_str()).unwrap_or("").to_string(),
                            oninput: move |e| { let mut f = form(); f["leader"] = json!(e.value()); form.set(f); } }
                    }}
                    FormItem { label: "电话".to_string(), required: false, children: rsx! {
                        input { class: "ant-input", value: form().get("phone").and_then(|v|v.as_str()).unwrap_or("").to_string(),
                            oninput: move |e| { let mut f = form(); f["phone"] = json!(e.value()); form.set(f); } }
                    }}
                    FormItem { label: "邮箱".to_string(), required: false, children: rsx! {
                        input { class: "ant-input", value: form().get("email").and_then(|v|v.as_str()).unwrap_or("").to_string(),
                            oninput: move |e| { let mut f = form(); f["email"] = json!(e.value()); form.set(f); } }
                    }}
                    FormItem { label: "排序".to_string(), required: false, children: rsx! {
                        input { class: "ant-input", r#type: "number",
                            value: "{form().get(\"sort\").and_then(|v|v.as_i64()).unwrap_or(0)}",
                            oninput: move |e| { let mut f = form(); f["sort"] = json!(e.value().parse::<i64>().unwrap_or(0)); form.set(f); } }
                    }}
                    FormItem { label: "状态".to_string(), required: false, children: rsx! {
                        select { class: "ant-input",
                            onchange: move |e| { let mut f = form(); f["status"] = json!(e.value()); form.set(f); },
                            option { value: "0", selected: form().get("status").and_then(|v|v.as_str()) == Some("0"), "启用" }
                            option { value: "1", selected: form().get("status").and_then(|v|v.as_str()) == Some("1"), "禁用" }
                        }
                    }}
                },
            }
        }
    }
}

#[component]
fn InfoRow(label: String, val: String) -> Element {
    rsx! {
        div { class: "flex py-2 border-b border-gray-50",
            span { class: "w-20 text-gray-400 flex-shrink-0", "{label}" }
            span { class: "text-gray-800", "{val}" }
        }
    }
}

fn build_dept_tree(depts: &[Value], parent_id: i64) -> Vec<TreeNode> {
    depts.iter()
        .filter(|d| d.get("parentId").and_then(|v| v.as_i64()).unwrap_or(0) == parent_id)
        .map(|d| {
            let id = d["id"].as_i64().unwrap_or(0);
            TreeNode {
                id: id.to_string(),
                label: d["name"].as_str().unwrap_or("未命名").to_string(),
                children: build_dept_tree(depts, id),
            }
        })
        .collect()
}

// ── Menu Management (tree + detail + CRUD) ──

#[component]
pub fn SystemMenuTree() -> Element {
    let mut menus = use_signal(Vec::<Value>::new);
    let mut selected_id = use_signal(|| None::<String>);
    let mut loading = use_signal(|| true);
    let mut show_form = use_signal(|| false);
    let mut edit_id = use_signal(|| None::<String>);
    let mut form = use_signal(|| json!({"status":"0","type":2,"visible":true,"keepAlive":true,"alwaysShow":true}));
    let mut saving = use_signal(|| false);

    let load = { move || { spawn(async move { loading.set(true);
        if let Ok(list) = api::sys_list("menu").await { menus.set(list); }
        loading.set(false); }); } };
    use_effect(load);

    let tree = build_menu_tree(&menus(), 0i64);
    let sel_id = selected_id.read().clone();
    let selected_menu = sel_id.as_ref().and_then(|id| menus().iter().find(|d| get_id(d) == *id).cloned());
    let mtype = selected_menu.as_ref().and_then(|m| m["type"].as_i64()).unwrap_or(2);
    let type_label = match mtype { 1 => "目录", 2 => "菜单", 3 => "按钮", _ => "未知" };
    let menu_name = selected_menu.as_ref().and_then(|m| m["name"].as_str()).unwrap_or("-").to_string();
    let menu_path = selected_menu.as_ref().and_then(|m| m["path"].as_str()).unwrap_or("-").to_string();
    let menu_comp = selected_menu.as_ref().and_then(|m| m["component"].as_str()).unwrap_or("-").to_string();
    let menu_perm = selected_menu.as_ref().and_then(|m| m["permission"].as_str()).unwrap_or("-").to_string();
    let menu_icon = selected_menu.as_ref().and_then(|m| m["icon"].as_str()).unwrap_or("-").to_string();
    let menu_sort = selected_menu.as_ref().and_then(|m| m["sort"].as_i64()).unwrap_or(0).to_string();
    let menu_visible = selected_menu.as_ref().map(|m| if m["visible"].as_bool().unwrap_or(true) { "是" } else { "否" }).unwrap_or("");
    let menu_status = selected_menu.as_ref().map(|m| if m["status"].as_str() == Some("0") { "启用" } else { "禁用" }).unwrap_or("");

    rsx! {
        div {
            h1 { class: "text-xl font-bold text-gray-800 mb-4", "菜单管理" }
            div { class: "grid grid-cols-3 gap-4",
                div { class: "col-span-1 bg-white rounded-lg shadow-sm border",
                    div { class: "flex items-center justify-between px-4 py-3 border-b",
                        h3 { class: "text-sm font-semibold", "菜单树" }
                        button { class: "ant-btn ant-btn-primary text-xs py-0.5 px-2",
                            onclick: move |_| {
                                form.set(json!({"status":"0","type":2,"visible":true,"parentId":0}));
                                edit_id.set(None); show_form.set(true);
                            },
                            "+ 新增"
                        }
                    }
                    div { class: "p-2 max-h-[60vh] overflow-y-auto",
                        if loading() { Spinner { text: Some("加载中...".into()) } }
                        else {
                            Tree {
                                nodes: tree,
                                selected: selected_id.read().clone(),
                                on_select: EventHandler::new(move |id: String| selected_id.set(Some(id))),
                            }
                        }
                    }
                }
                div { class: "col-span-2",
                    if selected_menu.is_some() {
                        div { class: "bg-white rounded-lg shadow-sm border p-6",
                            h3 { class: "text-base font-semibold mb-4", "菜单详情" }
                            div { class: "grid grid-cols-2 gap-3 text-sm",
                                InfoRow { label: "名称".to_string(), val: menu_name }
                                InfoRow { label: "类型".to_string(), val: type_label.to_string() }
                                InfoRow { label: "路径".to_string(), val: menu_path }
                                InfoRow { label: "组件".to_string(), val: menu_comp }
                                InfoRow { label: "权限标识".to_string(), val: menu_perm }
                                InfoRow { label: "图标".to_string(), val: menu_icon }
                                InfoRow { label: "排序".to_string(), val: menu_sort }
                                InfoRow { label: "可见".to_string(), val: menu_visible.to_string() }
                                InfoRow { label: "状态".to_string(), val: menu_status.to_string() }
                            }
                            div { class: "flex gap-2 mt-6 pt-4 border-t",
                                button { class: "ant-btn ant-btn-primary text-sm",
                                    onclick: {
                                        let d = selected_menu.clone();
                                        move |_| { if let Some(ref d) = d { form.set(d.clone()); edit_id.set(Some(get_id(d))); show_form.set(true); } }
                                    },
                                    "编辑"
                                }
                                button { class: "ant-btn ant-btn-danger text-sm",
                                    onclick: {
                                        let id = sel_id.clone();
                                        move |_| {
                                            if let Some(ref id) = id {
                                                let id = id.clone();
                                                if web_sys::window().map(|w| w.confirm_with_message("确认删除？子菜单也会删除。").unwrap_or(false)).unwrap_or(false) {
                                                    spawn(async move {
                                                        let _ = api::sys_remove("menu", &id).await;
                                                        toast("删除成功", ToastKind::Success);
                                                        selected_id.set(None);
                                                        if let Ok(list) = api::sys_list("menu").await { menus.set(list); }
                                                    });
                                                }
                                            }
                                        }
                                    },
                                    "删除"
                                }
                            }
                        }
                    } else {
                        div { class: "bg-white rounded-lg shadow-sm border p-12 text-center text-gray-400",
                            span { class: "text-5xl block mb-4", "📋" }
                            p { class: "text-lg", "选择一个菜单查看详情" }
                        }
                    }
                }
            }
            Modal {
                title: if edit_id().is_some() { "编辑菜单".to_string() } else { "新增菜单".to_string() },
                visible: show_form(),
                width: Some("640px".into()),
                loading: saving(),
                on_close: EventHandler::new(move |_| show_form.set(false)),
                on_save: Some(EventHandler::new(move |_| {
                    let f = form.read().clone();
                    saving.set(true);
                    let is_edit = edit_id.read().is_some();
                    spawn(async move {
                        let r = if is_edit { api::sys_update("menu", f).await }
                        else { api::sys_create("menu", f).await.map(|_| ()) };
                        saving.set(false);
                        if r.is_ok() { toast("保存成功", ToastKind::Success); show_form.set(false);
                            if let Ok(list) = api::sys_list("menu").await { menus.set(list); } }
                        else { toast(&format!("失败:{}", r.unwrap_err()), ToastKind::Error); }
                    });
                })),
                children: {
                    let fv = form();
                    rsx! {
                        FormItem { label: "名称".to_string(), required: true, children: rsx! {
                            input { class: "ant-input", value: fv.get("name").and_then(|v|v.as_str()).unwrap_or("").to_string(),
                                oninput: move |e| { let mut f = form(); f["name"] = json!(e.value()); form.set(f); } }
                        }}
                        FormItem { label: "类型".to_string(), required: true, children: rsx! {
                            select { class: "ant-input",
                                onchange: move |e| { let mut f = form(); f["type"] = json!(e.value().parse::<i64>().unwrap_or(2)); form.set(f); },
                                option { value: "1", selected: fv.get("type").and_then(|v|v.as_i64()) == Some(1), "目录" }
                                option { value: "2", selected: fv.get("type").and_then(|v|v.as_i64()).unwrap_or(2) == 2, "菜单" }
                                option { value: "3", selected: fv.get("type").and_then(|v|v.as_i64()) == Some(3), "按钮" }
                            }
                        }}
                        FormItem { label: "上级菜单".to_string(), required: false, children: rsx! {
                            select { class: "ant-input",
                                onchange: move |e| { let mut f = form(); f["parentId"] = json!(e.value().parse::<i64>().unwrap_or(0)); form.set(f); },
                                option { value: "0", "根节点" }
                                for m in menus() {
                                    if m["type"].as_i64().unwrap_or(2) <= 1 {
                                        option {
                                            value: "{m[\"id\"].as_i64().unwrap_or(0)}",
                                            selected: fv.get("parentId").and_then(|v|v.as_i64()).unwrap_or(0) == m["id"].as_i64().unwrap_or(0),
                                            "{m[\"name\"].as_str().unwrap_or(\"\")}"
                                        }
                                    }
                                }
                            }
                        }}
                        FormItem { label: "路径".to_string(), required: false, children: rsx! {
                            input { class: "ant-input", value: fv.get("path").and_then(|v|v.as_str()).unwrap_or("").to_string(),
                                oninput: move |e| { let mut f = form(); f["path"] = json!(e.value()); form.set(f); } }
                        }}
                        FormItem { label: "组件".to_string(), required: false, children: rsx! {
                            input { class: "ant-input", value: fv.get("component").and_then(|v|v.as_str()).unwrap_or("").to_string(),
                                oninput: move |e| { let mut f = form(); f["component"] = json!(e.value()); form.set(f); } }
                        }}
                        FormItem { label: "图标".to_string(), required: false, children: rsx! {
                            input { class: "ant-input", value: fv.get("icon").and_then(|v|v.as_str()).unwrap_or("").to_string(),
                                oninput: move |e| { let mut f = form(); f["icon"] = json!(e.value()); form.set(f); } }
                        }}
                        FormItem { label: "排序".to_string(), required: false, children: rsx! {
                            input { class: "ant-input", r#type: "number",
                                value: "{fv.get(\"sort\").and_then(|v|v.as_i64()).unwrap_or(0)}",
                                oninput: move |e| { let mut f = form(); f["sort"] = json!(e.value().parse::<i64>().unwrap_or(0)); form.set(f); } }
                        }}
                        FormItem { label: "可见".to_string(), required: false, children: rsx! {
                            select { class: "ant-input",
                                onchange: move |e| { let mut f = form(); f["visible"] = json!(e.value() == "true"); form.set(f); },
                                option { value: "true", selected: fv.get("visible").and_then(|v|v.as_bool()).unwrap_or(true), "是" }
                                option { value: "false", selected: fv.get("visible").and_then(|v|v.as_bool()) == Some(false), "否" }
                            }
                        }}
                        FormItem { label: "状态".to_string(), required: false, children: rsx! {
                            select { class: "ant-input",
                                onchange: move |e| { let mut f = form(); f["status"] = json!(e.value()); form.set(f); },
                                option { value: "0", selected: fv.get("status").and_then(|v|v.as_str()) == Some("0"), "启用" }
                                option { value: "1", selected: fv.get("status").and_then(|v|v.as_str()) == Some("1"), "禁用" }
                            }
                        }}
                    }
                },
            }
        }
    }
}

fn build_menu_tree(menus: &[Value], parent_id: i64) -> Vec<TreeNode> {
    menus.iter()
        .filter(|m| m.get("parentId").and_then(|v| v.as_i64()).unwrap_or(0) == parent_id)
        .map(|m| {
            let id = m["id"].as_i64().unwrap_or(0);
            let t = match m.get("type").and_then(|v| v.as_i64()).unwrap_or(2) { 1=>"D", 2=>"M", 3=>"B", _=>"?" };
            TreeNode {
                id: id.to_string(),
                label: m["name"].as_str().unwrap_or("?").to_string(),
                children: build_menu_tree(menus, id),
            }
        })
        .collect()
}
