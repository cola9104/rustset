use dioxus::prelude::*;
use dioxus_router::Router;
use gloo_net::http::Request;
use web_sys::RequestCredentials;

use crate::config::current_user_url;
use crate::router::Route;
use crate::state::user_role::AuthState as WorkflowAuthState;
use crate::state::{
    cloud_platform::CloudPlatformConfig, machine_room::MachineRoomConfig,
    network_policy::NetworkPolicyConfig, security_product::SecurityProduct,
    service_provider::ServiceProviderConfig,
};
use crate::utils::storage::{authorization_header, clear_token};

/// 全局安全产品数据状态
pub static SECURITY_PRODUCTS_STATE: GlobalSignal<Vec<SecurityProduct>> = Signal::global(Vec::new);

/// 主应用组件
#[allow(non_snake_case)]
pub fn App() -> Element {
    let mut workflow_auth_state = use_context_provider(|| Signal::new(WorkflowAuthState::guest()));

    use_effect(move || {
        spawn(async move {
            let mut request =
                Request::get(&current_user_url()).credentials(RequestCredentials::Include);
            if let Some(header) = authorization_header() {
                request = request.header("Authorization", &header);
            }

            match request.send().await {
                Ok(response) if response.ok() => match response.json::<CurrentUserResponse>().await
                {
                    Ok(current_user) => {
                        let auth_user = current_user.to_auth_user();
                        workflow_auth_state.set(WorkflowAuthState::from(&auth_user));
                        *AUTH_STATE.write() = Some(auth_user);
                    }
                    Err(_) => {
                        workflow_auth_state.set(WorkflowAuthState::guest());
                        *AUTH_STATE.write() = None;
                    }
                },
                Ok(response) if response.status() == 401 => {
                    clear_token();
                    workflow_auth_state.set(WorkflowAuthState::guest());
                    *AUTH_STATE.write() = None;
                }
                _ => {
                    workflow_auth_state.set(WorkflowAuthState::guest());
                    *AUTH_STATE.write() = None;
                }
            }

            *AUTH_READY.write() = true;
        });
    });

    rsx! {
        Router::<Route> {}
    }
}

/// 认证用户信息
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AuthUser {
    pub id: String,
    pub username: String,
    pub real_name: String,
    pub display_name: String,
    pub role: String,
    pub permissions: Vec<String>,
    pub organization_id: Option<i32>,
    pub organization_name: String,
    pub department_id: Option<i32>,
    pub department_name: String,
}

impl AuthUser {
    pub fn requester_name(&self) -> String {
        if !self.real_name.trim().is_empty() {
            self.real_name.clone()
        } else if !self.display_name.trim().is_empty() {
            self.display_name.clone()
        } else {
            self.username.clone()
        }
    }

    pub fn ticket_profile_warning(&self) -> Option<String> {
        let mut missing = Vec::new();

        if self.requester_name().trim().is_empty() {
            missing.push("姓名");
        }
        if self.organization_id.is_none() || self.organization_name.trim().is_empty() {
            missing.push("公司/组织");
        }
        if self.department_id.is_none() || self.department_name.trim().is_empty() {
            missing.push("部门");
        }

        if missing.is_empty() {
            None
        } else {
            Some(format!(
                "当前账号缺少{}信息，暂时无法提交资源申请，请先由管理员完善用户资料。",
                missing.join("、")
            ))
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
struct CurrentUserResponse {
    id: String,
    username: String,
    #[serde(default)]
    real_name: Option<String>,
    #[serde(default)]
    display_name: Option<String>,
    role: serde_json::Value,
    #[serde(default)]
    permissions: serde_json::Value,
    #[serde(default)]
    organization_id: Option<i32>,
    #[serde(default)]
    organization_name: Option<String>,
    #[serde(default)]
    department_id: Option<i32>,
    #[serde(default)]
    department_name: Option<String>,
}

impl CurrentUserResponse {
    fn to_auth_user(&self) -> AuthUser {
        let role = parse_role_value(&self.role);
        let permissions = self
            .permissions
            .as_object()
            .map(|permissions| {
                let mut items = Vec::new();
                for (key, value) in permissions {
                    if value.as_bool().is_some_and(|enabled| enabled) {
                        items.push(key.clone());
                    } else if let Some(scope) = value.as_str() {
                        items.push(format!("{key}:{scope}"));
                    }
                }
                items
            })
            .unwrap_or_default();

        AuthUser {
            id: self.id.clone(),
            username: self.username.clone(),
            real_name: self.real_name.clone().unwrap_or_default(),
            display_name: self
                .display_name
                .clone()
                .filter(|value| !value.trim().is_empty())
                .or_else(|| self.real_name.clone())
                .unwrap_or_else(|| self.username.clone()),
            role,
            permissions,
            organization_id: self.organization_id,
            organization_name: self.organization_name.clone().unwrap_or_default(),
            department_id: self.department_id,
            department_name: self.department_name.clone().unwrap_or_default(),
        }
    }
}

fn parse_role_value(role: &serde_json::Value) -> String {
    match role {
        serde_json::Value::String(value) => value.clone(),
        serde_json::Value::Object(map) => map
            .get("Custom")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string)
            .unwrap_or_else(|| "Custom".to_string()),
        _ => "Unknown".to_string(),
    }
}

/// 全局认证状态
pub static AUTH_STATE: GlobalSignal<Option<AuthUser>> = Signal::global(|| None);
pub static AUTH_READY: GlobalSignal<bool> = Signal::global(|| false);

/// 全局服务商数据状态
pub static PROVIDERS_STATE: GlobalSignal<Vec<ServiceProviderConfig>> = Signal::global(Vec::new);

/// 全局机房数据状态
pub static MACHINE_ROOMS_STATE: GlobalSignal<Vec<MachineRoomConfig>> = Signal::global(Vec::new);

/// 全局云平台数据状态
pub static CLOUD_PLATFORMS_STATE: GlobalSignal<Vec<CloudPlatformConfig>> = Signal::global(Vec::new);

/// 全局网络策略数据状态
pub static NETWORK_POLICIES_STATE: GlobalSignal<Vec<NetworkPolicyConfig>> =
    Signal::global(Vec::new);

/// 检查是否已认证
pub fn is_authenticated() -> bool {
    AUTH_STATE.read().is_some()
}

/// 登出
pub fn logout() {
    clear_token();
    *AUTH_STATE.write() = None;
    *AUTH_READY.write() = true;
}
