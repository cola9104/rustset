use crate::services::api::{self, MenuItem, UserInfo};
use dioxus::prelude::*;

#[derive(Clone, Debug)]
pub struct AppState {
    pub user: Option<UserInfo>,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub menus: Vec<MenuItem>,
    pub loading: bool,
    pub error: Option<String>,
}

impl AppState {
    pub fn is_authenticated() -> bool { APP_STATE.read().user.is_some() }
    pub fn menus() -> Vec<MenuItem> { APP_STATE.read().menus.clone() }
    pub fn user() -> Option<UserInfo> { APP_STATE.read().user.clone() }
}

pub static APP_STATE: GlobalSignal<AppState> = GlobalSignal::new(|| AppState {
    user: None, roles: Vec::new(), permissions: Vec::new(), menus: Vec::new(),
    loading: false, error: None,
});

pub async fn load_permission_info() {
    APP_STATE.write().loading = true;
    APP_STATE.write().error = None;
    let token = api::get_token();
    web_sys::console::log_1(&format!("load_permission_info: token={}", token.is_some()).into());
    match api::get_permission_info().await {
        Ok(info) => {
            web_sys::console::log_1(&format!("menus loaded: {} items, {} top-level", info.menus.len(), info.menus.len()).into());
            APP_STATE.write().user = Some(info.user);
            APP_STATE.write().roles = info.roles;
            APP_STATE.write().permissions = info.permissions;
            APP_STATE.write().menus = info.menus;
            APP_STATE.write().loading = false;
        }
        Err(e) => { APP_STATE.write().error = Some(e); APP_STATE.write().loading = false; }
    }
}

pub fn clear_state() {
    APP_STATE.write().user = None;
    APP_STATE.write().roles = Vec::new();
    APP_STATE.write().permissions = Vec::new();
    APP_STATE.write().menus = Vec::new();
    APP_STATE.write().error = None;
}
