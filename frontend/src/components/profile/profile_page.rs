use crate::app::AUTH_STATE;
use crate::state::user_role::use_auth;
use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::FaAddressCard;
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
    let workflow_auth = use_auth().read().clone();
    let current_user = AUTH_STATE.read().clone();

    let display_name = current_user
        .as_ref()
        .map(|user| user.requester_name())
        .unwrap_or_else(|| workflow_auth.username.clone());
    let email = current_user
        .as_ref()
        .map(|user| user.email.clone())
        .unwrap_or_default();
    let phone = current_user
        .as_ref()
        .map(|user| user.phone.clone())
        .unwrap_or_default();
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
                            "这里展示当前登录账号的身份、组织归属和资源工单申请资格。"
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

            div { class: "grid grid-cols-1 gap-6 lg:grid-cols-2",
                div { class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm",
                    h2 { class: "mb-4 text-lg font-semibold text-slate-900", "身份信息" }
                    div { class: "grid grid-cols-1 gap-4 sm:grid-cols-2",
                        InfoCard { icon_class: "fa-user", label: "登录账号", value: workflow_auth.username.clone() }
                        InfoCard { icon_class: "fa-id-card", label: "姓名", value: display_name }
                        InfoCard { icon_class: "fa-building", label: "公司/组织", value: empty_to_dash(&organization) }
                        InfoCard { icon_class: "fa-building", label: "部门", value: empty_to_dash(&department) }
                    }
                }

                div { class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm",
                    h2 { class: "mb-4 text-lg font-semibold text-slate-900", "联系与权限" }
                    div { class: "grid grid-cols-1 gap-4 sm:grid-cols-2",
                        InfoCard { icon_class: "fa-envelope", label: "邮箱", value: empty_to_dash(&email) }
                        InfoCard { icon_class: "fa-phone", label: "手机号", value: empty_to_dash(&phone) }
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
            }

            div { class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm",
                h2 { class: "mb-3 text-lg font-semibold text-slate-900", "说明" }
                p { class: "text-sm leading-7 text-slate-600",
                    "姓名、公司/组织、部门是资源申请默认带出的基础资料。邮箱和手机号用于联络展示。当前页面先提供只读确认；如果组织或部门缺失，需要管理员在用户管理中补齐后，资源工单申请入口才会自动恢复可用。"
                }
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
                    p { class: "mt-1 text-sm font-medium text-slate-900 break-all", "{value}" }
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
