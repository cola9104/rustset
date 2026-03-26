use crate::auth::token_expiration_hours;
use tower_sessions::{cookie::SameSite, Expiry, MemoryStore, SessionManagerLayer};

fn bool_env(name: &str, default: bool) -> bool {
    match std::env::var(name) {
        Ok(value) => match value.trim().to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => true,
            "0" | "false" | "no" | "off" => false,
            _ => default,
        },
        Err(_) => default,
    }
}

fn duration_hours_env(name: &str, default: i64) -> i64 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
        .filter(|hours| *hours > 0)
        .unwrap_or(default)
}

fn session_cookie_name() -> String {
    std::env::var("SESSION_COOKIE_NAME")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "rustset.sid".to_string())
}

fn session_cookie_secure() -> bool {
    bool_env("SESSION_COOKIE_SECURE", !cfg!(debug_assertions))
}

fn session_cookie_same_site() -> SameSite {
    match std::env::var("SESSION_COOKIE_SAME_SITE")
        .ok()
        .unwrap_or_else(|| "lax".to_string())
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "strict" => SameSite::Strict,
        "none" => SameSite::None,
        _ => SameSite::Lax,
    }
}

fn session_expiry() -> Expiry {
    let hours = duration_hours_env("SESSION_IDLE_TIMEOUT_HOURS", token_expiration_hours());
    Expiry::OnInactivity(tower_sessions::cookie::time::Duration::hours(hours))
}

/// 同步版本的 Session Layer 创建
/// 用于在非 async 上下文中使用
pub fn create_session_layer_sync() -> SessionManagerLayer<MemoryStore> {
    // 注意：这会在第一次调用时初始化 store
    // 后续调用会复用同一个 store
    static STORE: std::sync::OnceLock<MemoryStore> = std::sync::OnceLock::new();
    let store = STORE.get_or_init(MemoryStore::default);
    SessionManagerLayer::new(store.clone())
        .with_name(session_cookie_name())
        .with_http_only(true)
        .with_same_site(session_cookie_same_site())
        .with_secure(session_cookie_secure())
        .with_expiry(session_expiry())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request, response::IntoResponse, routing::get, Router};
    use std::sync::{Mutex, OnceLock};
    use tower::ServiceExt;
    use tower_cookies::CookieManagerLayer;
    use tower_sessions::Session;

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
    }

    async fn create_session(session: Session) -> impl IntoResponse {
        session.insert("user", "admin").await.unwrap();
        "ok"
    }

    #[tokio::test]
    async fn session_cookie_uses_expected_defaults() {
        let _guard = env_lock();
        std::env::remove_var("SESSION_COOKIE_NAME");
        std::env::remove_var("SESSION_COOKIE_SAME_SITE");
        std::env::remove_var("SESSION_COOKIE_SECURE");
        std::env::set_var("SESSION_IDLE_TIMEOUT_HOURS", "24");

        let app = Router::new()
            .route("/login", get(create_session))
            .layer(create_session_layer_sync())
            .layer(CookieManagerLayer::new());

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/login")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let set_cookie = response
            .headers()
            .get(axum::http::header::SET_COOKIE)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        assert!(set_cookie.contains("rustset.sid="));
        assert!(set_cookie.contains("HttpOnly"));
        assert!(set_cookie.contains("SameSite=Lax"));
        assert!(set_cookie.contains("Max-Age=86400"));
    }

    #[tokio::test]
    async fn session_cookie_honors_env_overrides() {
        let _guard = env_lock();
        std::env::set_var("SESSION_COOKIE_NAME", "custom.sid");
        std::env::set_var("SESSION_COOKIE_SAME_SITE", "strict");
        std::env::set_var("SESSION_COOKIE_SECURE", "true");
        std::env::set_var("SESSION_IDLE_TIMEOUT_HOURS", "1");

        let app = Router::new()
            .route("/login", get(create_session))
            .layer(create_session_layer_sync())
            .layer(CookieManagerLayer::new());

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/login")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let set_cookie = response
            .headers()
            .get(axum::http::header::SET_COOKIE)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        assert!(set_cookie.contains("custom.sid="));
        assert!(set_cookie.contains("HttpOnly"));
        assert!(set_cookie.contains("SameSite=Strict"));
        assert!(set_cookie.contains("Secure"));
        assert!(set_cookie.contains("Max-Age=3600"));
    }
}
