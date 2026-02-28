//! Authentication-related hooks

use yew::prelude::*;
use shared::{User, Role, Permissions, LoginResponse};

/// Hook to get auth token from localStorage
pub fn use_auth_token() -> String {
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

/// Hook to get authenticated user from localStorage
pub fn use_auth_user() -> Option<User> {
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

/// Hook to get user role
pub fn use_user_role() -> Option<Role> {
    use_auth_user().map(|user| user.role)
}

/// Hook to get user permissions
pub fn use_user_permissions() -> Option<Permissions> {
    use_auth_user().and_then(|user| user.permissions)
}
