use web_sys::window;
use crate::app::AuthUser;

const TOKEN_KEY: &str = "auth_token";
const USER_KEY: &str = "auth_user";

/// 获取 localStorage
fn local_storage() -> Option<web_sys::Storage> {
    window()?.local_storage().ok()?
}

/// 保存认证 token
pub fn save_token(token: &str) {
    if let Some(storage) = local_storage() {
        let _ = storage.set_item(TOKEN_KEY, token);
    }
}

/// 获取认证 token
pub fn get_token() -> Option<String> {
    local_storage().and_then(|s| s.get_item(TOKEN_KEY).ok().flatten())
}

/// 清除认证 token
pub fn clear_token() {
    if let Some(storage) = local_storage() {
        let _ = storage.remove_item(TOKEN_KEY);
    }
}

/// 保存当前用户信息
pub fn save_current_user(user: &AuthUser) {
    if let Some(storage) = local_storage() {
        if let Ok(json) = serde_json::to_string(user) {
            let _ = storage.set_item(USER_KEY, &json);
        }
    }
}

/// 获取当前用户信息
pub fn get_current_user() -> Option<AuthUser> {
    local_storage().and_then(|s| {
        s.get_item(USER_KEY).ok().flatten().and_then(|json| {
            serde_json::from_str(&json).ok()
        })
    })
}

/// 清除当前用户信息
pub fn clear_current_user() {
    if let Some(storage) = local_storage() {
        let _ = storage.remove_item(USER_KEY);
    }
}

/// 清除所有认证信息
pub fn clear_auth() {
    clear_token();
    clear_current_user();
}
