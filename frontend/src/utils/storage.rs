use web_sys::window;

const TOKEN_KEY: &str = "auth_token";

/// 获取 localStorage
fn local_storage() -> Option<web_sys::Storage> {
    window()?.local_storage().ok()?
}

/// 获取认证 token
pub fn get_token() -> Option<String> {
    local_storage().and_then(|s| s.get_item(TOKEN_KEY).ok().flatten())
}

/// 存储认证 token
pub fn set_token(token: &str) {
    if let Some(storage) = local_storage() {
        let _ = storage.set_item(TOKEN_KEY, token);
    }
}

/// 清除认证 token
pub fn clear_token() {
    if let Some(storage) = local_storage() {
        let _ = storage.remove_item(TOKEN_KEY);
    }
}
