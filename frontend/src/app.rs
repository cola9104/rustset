//! Main application component with routing

use yew::prelude::*;
use crate::Page;
use crate::get_auth_token;
use crate::components::{
    Login, Sidebar,
    BusinessApplication,
    CloudZoneManagement, CloudPlatformManagement, CloudProviderManagement,
    Dashboard, TaskCenter, RiskCenter, AuditLogs,
    OperationsManagement, CloudServiceAssetManagement,
    AdvancedScanning,
};

/// Main App component
#[function_component]
pub fn App() -> Html {
    let current_page = use_state(|| {
        // Check if user is logged in
        if !get_auth_token().is_empty() {
            Page::Dashboard
        } else {
            Page::Login
        }
    });

    let page = *current_page;

    html! {
        <div>
            { match page {
                Page::Login => html! {
                    <Login current_page={current_page.clone()} />
                },
                _ => html! {
                    <div class="columns is-gapless">
                        <div class="column is-2">
                            <Sidebar current_page={current_page.clone()} />
                        </div>
                        <div class="column">
                            { match page {
                                Page::Dashboard => html! { <Dashboard /> },
                                Page::TaskCenter => html! { <TaskCenter /> },
                                Page::AdvancedScanning => html! { <AdvancedScanning /> },
                                Page::BusinessAcceptance => html! { <BusinessApplication /> },
                                Page::BusinessApplication => html! { <BusinessApplication /> },
                                Page::OperationsManagement => html! { <OperationsManagement /> },
                                Page::AutomationOrchestration => html! {
                                    <div class="section">
                                        <div class="container">
                                            <h1 class="title">{"Automation Orchestration"}</h1>
                                            <p class="subtitle">{"Under development..."}</p>
                                        </div>
                                    </div>
                                },
                                Page::RiskCenter => html! { <RiskCenter /> },
                                Page::UserManagement => html! {
                                    <div class="section">
                                        <div class="container">
                                            <h1 class="title">{"User Management"}</h1>
                                            <p class="subtitle">{"Loading from lib.rs..."}</p>
                                        </div>
                                    </div>
                                },
                                Page::PermissionManagement => html! {
                                    <div class="section">
                                        <div class="container">
                                            <h1 class="title">{"Permission Management"}</h1>
                                            <p class="subtitle">{"Loading from lib.rs..."}</p>
                                        </div>
                                    </div>
                                },
                                Page::AuditLogs => html! { <AuditLogs /> },
                                Page::CloudProviderManagement => html! { <CloudProviderManagement /> },
                                Page::CloudZoneManagement => html! { <CloudZoneManagement /> },
                                Page::CloudPlatformManagement => html! { <CloudPlatformManagement /> },
                                Page::CloudServiceAssetManagement => html! { <CloudServiceAssetManagement /> },
                                Page::UserProfile => html! {
                                    <div class="section">
                                        <div class="container">
                                            <h1 class="title">{"User Profile"}</h1>
                                            <p class="subtitle">{"Loading from lib.rs..."}</p>
                                        </div>
                                    </div>
                                },
                                Page::PasswordPolicyManagement => html! {
                                    <div class="section">
                                        <div class="container">
                                            <h1 class="title">{"Password Policy Management"}</h1>
                                            <p class="subtitle">{"Loading from lib.rs..."}</p>
                                        </div>
                                    </div>
                                },
                                Page::Login => html! { <Login current_page={current_page.clone()} /> },
                            }}
                        </div>
                    </div>
                }
            }}
        </div>
    }
}
