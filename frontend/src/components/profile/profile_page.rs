use crate::app::AUTH_STATE;
use crate::services::user_api::{update_current_user_profile, UpdateCurrentUserProfilePayload};
use crate::state::user_role::{use_auth, AuthState as WorkflowAuthState};
use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::{FaAddressCard, FaFloppyDisk, FaPenToSquare};
use dioxus_free_icons::Icon;

fn scope_label(value: Option<&str>) -> &'static str {
    match value {
        Some("self") => "仅本人",
        Some("department") => "本部门",
        Some("organization") => "本公司/组织",
        Some("all") => "全部数据",
        _ => "未配置",
    }
}

#[allow(non_snake_case)]
pub fn ProfilePage() -> Element {
    let mut workflow_auth_state = use_auth();
    let workflow_auth = workflow_auth_state.read().clone();
    let current_user = AUTH_STATE.read().clone();

    let initial_real_name = current_user
        .as_ref()
        .map(|user| user.real_name.clone())
        .unwrap_or_default();
    let initial_email = current_user
        .as_ref()
        .map(|user| user.email.clone())
        .unwrap_or_default();
    let initial_phone = current_user
        .as_ref()
        .map(|user| user.phone.clone())
        .unwrap_or_default();

    let mut real_name = use_signal({
        let initial_real_name = initial_real_name.clone();
        move || initial_real_name.clone()
    });
    let mut email = use_signal({
        let initial_email = initial_email.clone();
        move || initial_email.clone()
    });
    let mut phone = use_signal({
        let initial_phone = initial_phone.clone();
        move || initial_phone.clone()
    });
    let mut saving = use_signal(|| false);
    let mut save_error = use_signal(String::new);
    let mut save_success = use_signal(String::new);

    let real_name_value = real_name.read().clone();
    let email_value = email.read().clone();
    let phone_value = phone.read().clone();

    let display_name = if !real_name_value.trim().is_empty() {
        real_name_value.clone()
    } else {
        current_user
            .as_ref()
            .map(|user| user.display_name.clone())
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| workflow_auth.username.clone())
    };
    let organization = current_user
        .as_ref()
        .map(|user| user.organization_name.clone())
        .unwrap_or_default();
    let department = current_user
        .as_ref()
        .map(|user| user.department_name.clone())
        .unwrap_or_default();
    let profile_warning = current_user
        .as_ref()
        .and_then(|user| user.ticket_profile_warning());
    let scope = scope_label(workflow_auth.scope_value("resource_ticket_scope"));
    let has_changes = current_user
        .as_ref()
        .map(|user| {
            user.real_name != real_name_value
                || user.email != email_value
                || user.phone != phone_value
        })
        .unwrap_or(false);
    let can_save = !*saving.read() && has_changes;

    let handle_save = move |_| {
        if !can_save {
            return;
        }

        let next_real_name = real_name.read().clone();
        let next_email = email.read().clone();
        let next_phone = phone.read().clone();

        spawn(async move {
            saving.set(true);
            save_error.set(String::new());
            save_success.set(String::new());

            let payload = UpdateCurrentUserProfilePayload {
                real_name: Some(next_real_name),
                email: Some(next_email),
                phone: Some(next_phone),
            };

            match update_current_user_profile(&payload).await {
                Ok(auth_user) => {
                    let saved_real_name = auth_user.real_name.clone();
                    let saved_email = auth_user.email.clone();
                    let saved_phone = auth_user.phone.clone();

                    workflow_auth_state.set(WorkflowAuthState::from(&auth_user));
                    *AUTH_STATE.write() = Some(auth_user);
                    real_name.set(saved_real_name);
                    email.set(saved_email);
                    phone.set(saved_phone);
                    save_success.set("个人资料已更新".to_string());
                }
                Err(err) => {
                    save_error.set(err);
                }
            }

            saving.set(false);
        });
    };

    rsx! {
        div { class: "mx-auto max-w-5xl space-y-6",
            div { class: "rounded-2xl bg-gradient-to-r from-slate-900 via-slate-800 to-blue-900 p-6 text-white shadow-lg",
                div { class: "flex flex-col gap-5 md:flex-row md:items-end md:justify-between",
                    div {
                        div { class: "mb-3 inline-flex h-12 w-12 items-center justify-center rounded-xl bg-white/10 ring-1 ring-white/15",
                            Icon { icon: FaAddressCard, width: 20, height: 20, class: "text-blue-200" }
                        }
                        h1 { class: "text-2xl font-bold tracking-tight", "个人资料" }
                        p { class: "mt-2 text-sm text-slate-200",
                            "这里维护当前登录账号的姓名和联系方式；公司/组织、部门仍由管理员维护。"
                        }
                    }
                    div { class: "rounded-xl bg-white/10 px-4 py-3 text-sm ring-1 ring-white/15",
                        p { class: "text-slate-300", "当前角色" }
                        p { class: "mt-1 font-semibold", "{workflow_auth.display_role_label()}" }
                    }
                }
            }

            if let Some(message) = profile_warning.clone() {
                div { class: "rounded-xl border border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-800",
                    "{message}"
                }
            } else {
                div { class: "rounded-xl border border-emerald-200 bg-emerald-50 px-4 py-3 text-sm text-emerald-800",
                    "当前账号资料完整，可以正常提交资源工单。"
                }
            }

            if !save_error.read().is_empty() {
                div { class: "rounded-xl border border-rose-200 bg-rose-50 px-4 py-3 text-sm text-rose-700",
                    "{save_error.read()}"
                }
            }

            if !save_success.read().is_empty() {
                div { class: "rounded-xl border border-emerald-200 bg-emerald-50 px-4 py-3 text-sm text-emerald-700",
                    "{save_success.read()}"
                }
            }

            div { class: "grid grid-cols-1 gap-6 lg:grid-cols-[1.2fr,0.8fr]",
                div { class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm",
                    div { class: "flex items-start justify-between gap-4",
                        div {
                            h2 { class: "text-lg font-semibold text-slate-900", "可维护信息" }
                            p { class: "mt-1 text-sm text-slate-500",
                                "姓名会作为资源工单默认申请人，邮箱和手机号用于联络展示。"
                            }
                        }
                        div { class: "inline-flex h-11 w-11 items-center justify-center rounded-xl bg-slate-100 text-slate-700",
                            Icon { icon: FaPenToSquare, width: 18, height: 18 }
                        }
                    }

                    div { class: "mt-6 grid grid-cols-1 gap-4",
                        FormField {
                            label: "登录账号",
                            value: workflow_auth.username.clone(),
                            placeholder: "",
                            readonly: true,
                            oninput: |_| {},
                        }
                        FormField {
                            label: "姓名",
                            value: real_name_value.clone(),
                            placeholder: "请输入姓名",
                            readonly: false,
                            oninput: move |event: FormEvent| real_name.set(event.value()),
                        }
                        FormField {
                            label: "邮箱",
                            value: email_value.clone(),
                            placeholder: "用于接收联络信息",
                            readonly: false,
                            oninput: move |event: FormEvent| email.set(event.value()),
                        }
                        FormField {
                            label: "手机号",
                            value: phone_value.clone(),
                            placeholder: "用于紧急联系",
                            readonly: false,
                            oninput: move |event: FormEvent| phone.set(event.value()),
                        }
                    }

                    div { class: "mt-6 flex flex-wrap items-center gap-3",
                        button {
                            class: if can_save {
                                "inline-flex items-center gap-2 rounded-xl bg-blue-600 px-4 py-2.5 text-sm font-medium text-white shadow-sm transition hover:bg-blue-700"
                            } else {
                                "inline-flex items-center gap-2 rounded-xl bg-slate-200 px-4 py-2.5 text-sm font-medium text-slate-500"
                            },
                            disabled: !can_save,
                            onclick: handle_save,
                            Icon { icon: FaFloppyDisk, width: 16, height: 16 }
                            if *saving.read() {
                                "保存中..."
                            } else if has_changes {
                                "保存修改"
                            } else {
                                "暂无变更"
                            }
                        }
                        p { class: "text-sm text-slate-500",
                            "公司/组织和部门由管理员维护，当前页面不会修改权限、角色或归属。"
                        }
                    }
                }

                div { class: "space-y-6",
                    div { class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm",
                        h2 { class: "mb-4 text-lg font-semibold text-slate-900", "归属与权限" }
                        div { class: "grid grid-cols-1 gap-4",
                            InfoCard { icon_class: "fa-id-card", label: "当前显示名称", value: display_name }
                            InfoCard { icon_class: "fa-building", label: "公司/组织", value: empty_to_dash(&organization) }
                            InfoCard { icon_class: "fa-building", label: "部门", value: empty_to_dash(&department) }
                            InfoCard { icon_class: "fa-shield-halved", label: "工单数据范围", value: scope.to_string() }
                            InfoCard {
                                icon_class: "fa-shield-halved",
                                label: "工单提交",
                                value: if profile_warning.is_some() {
                                    "资料不完整".to_string()
                                } else if workflow_auth.can_submit() {
                                    "允许提交".to_string()
                                } else {
                                    "无提交权限".to_string()
                                }
                            }
                        }
                    }

                    div { class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm",
                        h2 { class: "mb-3 text-lg font-semibold text-slate-900", "说明" }
                        p { class: "text-sm leading-7 text-slate-600",
                            "如果资源工单申请仍提示资料不完整，通常是组织或部门尚未绑定。这两项需要管理员在用户管理中维护，保存个人资料后会立即同步到当前登录态和前端权限视图。"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn FormField(
    label: &'static str,
    value: String,
    placeholder: &'static str,
    readonly: bool,
    oninput: EventHandler<FormEvent>,
) -> Element {
    rsx! {
        label { class: "block space-y-2",
            span { class: "text-sm font-medium text-slate-700", "{label}" }
            input {
                r#type: "text",
                class: if readonly {
                    "w-full rounded-xl border border-slate-200 bg-slate-50 px-4 py-3 text-sm text-slate-500 outline-none"
                } else {
                    "w-full rounded-xl border border-slate-200 bg-white px-4 py-3 text-sm text-slate-900 outline-none transition focus:border-blue-500 focus:ring-2 focus:ring-blue-100"
                },
                value: value,
                placeholder: placeholder,
                readonly: readonly,
                oninput: move |event| oninput.call(event),
            }
        }
    }
}

#[component]
fn InfoCard(icon_class: &'static str, label: &'static str, value: String) -> Element {
    rsx! {
        div { class: "rounded-xl border border-slate-200 bg-slate-50 p-4",
            div { class: "flex items-center gap-3",
                div { class: "flex h-10 w-10 items-center justify-center rounded-xl bg-white text-slate-700 ring-1 ring-slate-200",
                    i { class: "fa {icon_class} text-sm" }
                }
                div {
                    p { class: "text-xs font-medium uppercase tracking-wide text-slate-500", "{label}" }
                    p { class: "mt-1 break-all text-sm font-medium text-slate-900", "{value}" }
                }
            }
        }
    }
}

fn empty_to_dash(value: &str) -> String {
    if value.trim().is_empty() {
        "-".to_string()
    } else {
        value.to_string()
    }
}
