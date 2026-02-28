//! Sidebar navigation component

use yew::prelude::*;
use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use shared::{User, Role, Permissions, LoginResponse};
use crate::{api_url, Language, Page, get_auth_token, get_auth_user, set_auth, clear_auth};

#[derive(Properties, PartialEq)]
pub struct SidebarProps {
    pub current_page: UseStateHandle<Page>,
}

#[function_component]
pub fn Sidebar(props: &SidebarProps) -> Html {
    let current_page = props.current_page.clone();
    let lang = use_state(|| Language::Zh);

    // Get user from localStorage
    let user_role = get_auth_user().map(|u| u.role);
    let user_permissions = get_auth_user().and_then(|u| u.permissions);

    // Real-time permissions from backend
    let latest_permissions = use_state(|| None as Option<Permissions>);
    let loading_permissions = use_state(|| false);

    // Get token
    let token = get_auth_token();

    // Effect to fetch latest permissions on mount
    let latest_permissions_clone = latest_permissions.clone();
    let loading_permissions_clone = loading_permissions.clone();
    let token_clone = token.clone();

    use_effect_with((), move |_| {
        let token = token_clone.clone();
        if !token.is_empty() {
            loading_permissions_clone.set(true);
            spawn_local(async move {
                match Request::get(&api_url("users/me"))
                    .header("Authorization", &token)
                    .send()
                    .await
                {
                    Ok(resp) if resp.ok() => {
                        if let Ok(updated_user) = resp.json::<User>().await {
                            // Update localStorage
                            if let Ok(user_str) = serde_json::to_string(&updated_user) {
                                let login_response = LoginResponse {
                                    token: token.clone(),
                                    user: updated_user.clone(),
                                };
                                if let Ok(login_str) = serde_json::to_string(&login_response) {
                                    set_auth(&token, &login_str);
                                }
                            }
                            // Update permissions state
                            latest_permissions_clone.set(updated_user.permissions);
                        }
                    }
                    _ => {}
                }
                loading_permissions_clone.set(false);
            });
        }
        || ()
    });

    // Use real-time permissions, fallback to cached
    let effective_permissions = latest_permissions.as_ref().or(user_permissions.as_ref());

    // Debug output
    web_sys::console::log_1(&format!("Sidebar debug - user_role: {:?}", user_role).into());
    if let Some(perms) = effective_permissions {
        web_sys::console::log_1(&format!("Sidebar debug - permissions: can_access_general={}, can_access_assets_risks={}, can_access_cloud={}, can_access_audit={}",
            perms.can_access_general, perms.can_access_assets_risks, perms.can_access_cloud, perms.can_access_audit).into());
    } else {
        web_sys::console::log_1(&"Sidebar debug - user_permissions is None!".into());
    }

    let toggle_lang = {
        let lang = lang.clone();
        Callback::from(move |_| {
            lang.set(if *lang == Language::Zh {
                Language::En
            } else {
                Language::Zh
            });
        })
    };

    let on_logout: Callback<web_sys::Event> = {
        let current_page = current_page.clone();
        Callback::from(move |_| {
            clear_auth();
            current_page.set(Page::Login);
        })
    };

    let navigate = |page: Page| -> Callback<MouseEvent> {
        let current_page = current_page.clone();
        Callback::from(move |_| current_page.set(page))
    };

    html! {
        <aside class="menu p-4" style="height: 100vh; background-color: #f5f5f5; overflow-y: auto; position: sticky; top: 0;">
            <div class="level is-mobile mb-4">
                <div class="level-left">
                    <h1 class="title is-4">{ "RustSet" }</h1>
                </div>
                <div class="level-right">
                    <button class="button is-small is-white" onclick={toggle_lang}>
                        { if *lang == Language::Zh { "中文" } else { "English" } }
                    </button>
                </div>
            </div>
            <div class="level is-mobile">
                <div class="level-left">
                    <p class="menu-label">{ lang.t("general") }</p>
                </div>
            </div>
            <ul class="menu-list">
                <li><a onclick={navigate(Page::Dashboard)}>{ lang.t("dashboard") }</a></li>
                // General module - uses top-level permission can_access_general
                if user_role == Some(Role::SecAdmin) || user_role == Some(Role::Auditor) || effective_permissions.map(|p| p.can_access_general).unwrap_or(false) {
                    <li><a onclick={navigate(Page::TaskCenter)}>{ lang.t("task_center") }</a></li>
                    <li><a onclick={navigate(Page::AdvancedScanning)}>{ lang.t("advanced_scanning") }</a></li>
                }
                // Risk monitoring - uses permission can_view_risks
                if user_role == Some(Role::SecAdmin) || effective_permissions.map(|p| p.can_view_risks).unwrap_or(false) {
                    <li><a onclick={navigate(Page::RiskCenter)}>{ lang.t("risk_monitoring") }</a></li>
                }
            </ul>
            // Business process - uses top-level permission can_access_assets_risks
            if user_role == Some(Role::SecAdmin) || effective_permissions.map(|p| p.can_view_business_process).unwrap_or(false) {
                <p class="menu-label">{ "业务流程" }</p>
                <ul class="menu-list">
                    <li><a onclick={navigate(Page::BusinessApplication)}>{ lang.t("business_application") }</a></li>
                    <li><a onclick={navigate(Page::OperationsManagement)}>{ lang.t("operations_management") }</a></li>
                    <li><a onclick={navigate(Page::AutomationOrchestration)}>{ lang.t("automation_orchestration") }</a></li>
                </ul>
            }
            // Cloud management module - uses top-level permission can_access_cloud
            if user_role == Some(Role::SecAdmin) || effective_permissions.map(|p| p.can_access_cloud).unwrap_or(false) {
                <p class="menu-label">{ "云管理" }</p>
                <ul class="menu-list">
                    if user_role == Some(Role::SecAdmin) || effective_permissions.map(|p| p.can_view_cloud_providers).unwrap_or(false) {
                        <li><a onclick={navigate(Page::CloudZoneManagement)}>{ lang.t("cloud_zone_management") }</a></li>
                        <li><a onclick={navigate(Page::CloudPlatformManagement)}>{ lang.t("cloud_platform_management") }</a></li>
                        <li><a onclick={navigate(Page::CloudProviderManagement)}>{ lang.t("cloud_provider_management") }</a></li>
                    }
                    if user_role == Some(Role::SecAdmin) || effective_permissions.map(|p| p.can_view_cloud_assets).unwrap_or(false) {
                        <li><a onclick={navigate(Page::CloudServiceAssetManagement)}>{ lang.t("cloud_service_asset_management") }</a></li>
                    }

                </ul>
            }
            // User management module - uses top-level permission can_access_user_management
            if user_role == Some(Role::SecAdmin) || effective_permissions.map(|p| p.can_access_user_management).unwrap_or(false) {
                <p class="menu-label">{ lang.t("user_management") }</p>
                <ul class="menu-list">
                    if user_role == Some(Role::SecAdmin) || effective_permissions.map(|p| p.can_view_users).unwrap_or(false) {
                        <li><a onclick={navigate(Page::UserManagement)}>{ lang.t("user_management") }</a></li>
                    }
                    if user_role == Some(Role::SecAdmin) || effective_permissions.map(|p| p.can_manage_permissions).unwrap_or(false) {
                        <li><a onclick={navigate(Page::PermissionManagement)}>{ lang.t("permission_management") }</a></li>
                    }
                    if user_role == Some(Role::SecAdmin) || effective_permissions.map(|p| p.can_view_password_policy).unwrap_or(false) {
                        <li><a onclick={navigate(Page::PasswordPolicyManagement)}>{ lang.t("password_policy_management") }</a></li>
                    }
                </ul>
            }
            // Audit module - uses top-level permission can_access_audit
            if user_role == Some(Role::Auditor) || effective_permissions.map(|p| p.can_access_audit).unwrap_or(false) {
                <p class="menu-label">{ lang.t("audit_logs") }</p>
                <ul class="menu-list">
                    if user_role == Some(Role::Auditor) || effective_permissions.map(|p| p.can_view_audit_logs).unwrap_or(false) {
                        <li><a onclick={navigate(Page::AuditLogs)}>{ lang.t("audit_logs") }</a></li>
                    }
                </ul>
            }
            <p class="menu-label">{ "Account" }</p>
            <ul class="menu-list">
                <li><a onclick={navigate(Page::UserProfile)}>{ lang.t("user_profile") }</a></li>
            </ul>
        </aside>
    }
}
