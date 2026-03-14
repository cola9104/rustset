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
