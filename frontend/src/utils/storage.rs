//! localStorage operations for authentication

use shared::{User, LoginResponse};

/// Get auth token from localStorage
pub fn get_auth_token() -> String {
    match web_sys::window() {
        Some(window) => match window.local_storage() {
            Ok(Some(storage)) => match storage.get_item("auth_token") {
                Ok(token) => token.unwrap_or_default().trim().to_string(),
                _ => String::new(),
            },
            _ => String::new(),
        },
        _ => String::new(),
    }
}

/// Get authenticated user from localStorage
pub fn get_auth_user() -> Option<User> {
    match web_sys::window() {
        Some(window) => match window.local_storage() {
            Ok(Some(storage)) => match storage.get_item("auth_user") {
                Ok(Some(user_str)) => serde_json::from_str::<LoginResponse>(&user_str).ok().map(|r| r.user),
                _ => None,
            },
            _ => None,
        },
        _ => None,
    }
}

/// Get user role from localStorage
pub fn get_user_role() -> Option<shared::Role> {
    get_auth_user().map(|user| user.role)
}

/// Get user permissions from localStorage
pub fn get_user_permissions() -> Option<shared::Permissions> {
    get_auth_user().and_then(|user| user.permissions)
}

/// Set auth data in localStorage
pub fn set_auth(token: &str, user_str: &str) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.set_item("auth_token", token);
            let _ = storage.set_item("auth_user", user_str);
        }
    }
}

/// Clear auth data from localStorage
pub fn clear_auth() {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.remove_item("auth_token");
            let _ = storage.remove_item("auth_user");
        }
    }
}
