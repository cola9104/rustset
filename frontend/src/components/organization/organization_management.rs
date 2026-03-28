use crate::services::{
    create_organization, delete_organization, fetch_organizations, update_organization,
    OrganizationPayload,
};
use crate::state::organization::OrganizationRecord;
use crate::state::user_role::use_auth;
use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::{FaBuilding, FaMagnifyingGlass, FaPlus, FaTrash};
use dioxus_free_icons::Icon;

#[allow(non_snake_case)]
pub fn OrganizationManagement() -> Element {
    let auth = use_auth();
    let current_auth = auth.read().clone();
    let mut organizations = use_signal(Vec::<OrganizationRecord>::new);
    let mut search_query = use_signal(String::new);
    let mut error = use_signal(String::new);
    let mut loading = use_signal(|| true);
    let mut editing = use_signal(|| None::<OrganizationRecord>);

    let reload = move || async move {
        loading.set(true);
        match fetch_organizations().await {
            Ok(data) => {
                organizations.set(data);
                error.set(String::new());
            }
            Err(err) => error.set(err),
        }
        loading.set(false);
    };

    use_effect(move || {
        spawn(reload());
    });

    let filtered = organizations
        .read()
        .iter()
        .filter(|item| {
            let query = search_query.read().to_lowercase();
            query.is_empty()
                || item.name.to_lowercase().contains(&query)
                || item.code.to_lowercase().contains(&query)
        })
        .cloned()
        .collect::<Vec<_>>();
    let can_manage = current_auth.can_manage_permissions();

    rsx! {
        div { class: "space-y-6",
            div { class: "flex items-center justify-between",
                h1 { class: "text-2xl font-bold text-gray-800", "组织/单位管理" }
                if can_manage {
                    button {
                        class: "flex items-center gap-2 rounded-md bg-blue-600 px-4 py-2 text-white hover:bg-blue-700",
                        onclick: move |_| editing.set(Some(OrganizationRecord {
                            id: 0,
                            name: String::new(),
                            code: String::new(),
                            status: "active".to_string(),
                            remarks: None,
                            created_at: String::new(),
                            updated_at: None,
                        })),
                        Icon { icon: FaPlus, width: 16, height: 16 }
                        "新增组织"
                    }
                }
            }

            if !can_manage {
                div { class: "rounded-lg border border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-800",
                    "当前账号仅可查看组织/单位数据，新增、编辑、删除操作已禁用。"
                }
            }

            div { class: "rounded-lg bg-white p-4 shadow",
                div { class: "flex items-center gap-2",
                    Icon { icon: FaMagnifyingGlass, width: 16, height: 16, class: "text-gray-400" }
                    input {
                        class: "w-full border-0 focus:outline-none",
                        placeholder: "搜索组织名称或编码...",
                        value: search_query,
                        oninput: move |e| search_query.set(e.value()),
                    }
                }
            }

            if *loading.read() {
                div { class: "rounded-lg bg-white p-6 shadow text-gray-500", "正在加载组织数据..." }
            } else if !error.read().is_empty() {
                div { class: "rounded-lg border border-red-200 bg-red-50 p-4 text-red-700", "{error}" }
            } else {
                div { class: "overflow-hidden rounded-lg bg-white shadow",
                    table { class: "min-w-full divide-y divide-gray-200",
                        thead { class: "bg-gray-50",
                            tr {
                                th { class: "px-6 py-3 text-left text-xs font-medium uppercase tracking-wider text-gray-500", "名称" }
                                th { class: "px-6 py-3 text-left text-xs font-medium uppercase tracking-wider text-gray-500", "编码" }
                                th { class: "px-6 py-3 text-left text-xs font-medium uppercase tracking-wider text-gray-500", "状态" }
                                th { class: "px-6 py-3 text-left text-xs font-medium uppercase tracking-wider text-gray-500", "备注" }
                                th { class: "px-6 py-3 text-right text-xs font-medium uppercase tracking-wider text-gray-500", "操作" }
                            }
                        }
                        tbody { class: "divide-y divide-gray-200 bg-white",
                            for item in filtered {
                                tr { class: "hover:bg-gray-50",
                                    td { class: "px-6 py-4 text-sm font-medium text-gray-900",
                                        div { class: "flex items-center gap-3",
                                            div { class: "flex h-9 w-9 items-center justify-center rounded-full bg-blue-100 text-blue-600",
                                                Icon { icon: FaBuilding, width: 16, height: 16 }
                                            }
                                            span { "{item.name}" }
                                        }
                                    }
                                    td { class: "px-6 py-4 text-sm text-gray-600", "{item.code}" }
                                    td { class: "px-6 py-4 text-sm text-gray-600",
                                        if item.status == "active" { "启用" } else { "停用" }
                                    }
                                    td { class: "px-6 py-4 text-sm text-gray-600", "{item.remarks.clone().unwrap_or_default()}" }
                                    td { class: "px-6 py-4 text-right text-sm",
                                        if can_manage {
                                            button {
                                                class: "mr-3 text-blue-600 hover:text-blue-800",
                                                onclick: {
                                                    let item = item.clone();
                                                    move |_| editing.set(Some(item.clone()))
                                                },
                                                "编辑"
                                            }
                                            button {
                                                class: "text-red-600 hover:text-red-800",
                                                onclick: move |_| {
                                                    let id = item.id;
                                                    spawn(async move {
                                                        let _ = delete_organization(id).await;
                                                        spawn(reload());
                                                    });
                                                },
                                                Icon { icon: FaTrash, width: 14, height: 14 }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if can_manage {
                if let Some(item) = editing() {
                OrganizationModal {
                    key: "{item.id}",
                    initial: item,
                    on_close: move |_| editing.set(None),
                    on_save: move |payload: (i32, OrganizationPayload)| {
                        spawn(async move {
                            let result = if payload.0 == 0 {
                                create_organization(&payload.1).await.map(|_| ())
                            } else {
                                update_organization(payload.0, &payload.1).await.map(|_| ())
                            };

                            match result {
                                Ok(()) => {
                                    editing.set(None);
                                    spawn(reload());
                                }
                                Err(err) => error.set(err),
                            }
                        });
                    }
                }
            }
            }
        }
    }
}

#[component]
fn OrganizationModal(
    initial: OrganizationRecord,
    on_close: EventHandler<()>,
    on_save: EventHandler<(i32, OrganizationPayload)>,
) -> Element {
    let mut name = use_signal(|| initial.name.clone());
    let mut code = use_signal(|| initial.code.clone());
    let mut status = use_signal(|| initial.status.clone());
    let mut remarks = use_signal(|| initial.remarks.unwrap_or_default());

    rsx! {
        div { class: "fixed inset-0 z-50 flex items-center justify-center bg-black/50",
            div { class: "w-full max-w-lg rounded-lg bg-white shadow-xl",
                div { class: "border-b p-4 text-lg font-semibold text-gray-800",
                    if initial.id == 0 { "新增组织" } else { "编辑组织" }
                }
                div { class: "space-y-4 p-4",
                    div {
                        label { class: "mb-1 block text-sm font-medium text-gray-700", "组织名称" }
                        input {
                            class: "w-full rounded-md border border-gray-300 px-3 py-2",
                            value: name,
                            oninput: move |e| name.set(e.value()),
                        }
                    }
                    div {
                        label { class: "mb-1 block text-sm font-medium text-gray-700", "组织编码" }
                        input {
                            class: "w-full rounded-md border border-gray-300 px-3 py-2",
                            value: code,
                            oninput: move |e| code.set(e.value()),
                        }
                    }
                    div {
                        label { class: "mb-1 block text-sm font-medium text-gray-700", "状态" }
                        select {
                            class: "w-full rounded-md border border-gray-300 px-3 py-2",
                            value: status,
                            onchange: move |e| status.set(e.value()),
                            option { value: "active", "启用" }
                            option { value: "inactive", "停用" }
                        }
                    }
                    div {
                        label { class: "mb-1 block text-sm font-medium text-gray-700", "备注" }
                        textarea {
                            class: "w-full rounded-md border border-gray-300 px-3 py-2",
                            rows: 3,
                            value: remarks,
                            oninput: move |e| remarks.set(e.value()),
                        }
                    }
                }
                div { class: "flex justify-end gap-3 border-t p-4",
                    button {
                        class: "rounded-md border border-gray-300 px-4 py-2 text-gray-700",
                        onclick: move |_| on_close.call(()),
                        "取消"
                    }
                    button {
                        class: "rounded-md bg-blue-600 px-4 py-2 text-white",
                        onclick: move |_| on_save.call((initial.id, OrganizationPayload {
                            name: name.read().trim().to_string(),
                            code: code.read().trim().to_string(),
                            status: status.read().clone(),
                            remarks: if remarks.read().trim().is_empty() { None } else { Some(remarks.read().trim().to_string()) },
                        })),
                        "保存"
                    }
                }
            }
        }
    }
}
