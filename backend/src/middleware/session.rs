#![allow(dead_code)]

use tokio::sync::OnceCell;
use tower_sessions::{MemoryStore, SessionManagerLayer};

/// 全局共享的 Session Store
static SESSION_STORE: OnceCell<MemoryStore> = OnceCell::const_new();

/// 初始化 Session Store
async fn init_session_store() -> MemoryStore {
    SESSION_STORE
        .get_or_init(|| async { MemoryStore::default() })
        .await
        .clone()
}

/// 创建 Session Manager Layer
/// 用于在 Axum 应用中管理 Session
pub async fn create_session_layer() -> SessionManagerLayer<MemoryStore> {
    let store = init_session_store().await;
    SessionManagerLayer::new(store)
}

/// 同步版本的 Session Layer 创建
/// 用于在非 async 上下文中使用
pub fn create_session_layer_sync() -> SessionManagerLayer<MemoryStore> {
    // 注意：这会在第一次调用时初始化 store
    // 后续调用会复用同一个 store
    static STORE: std::sync::OnceLock<MemoryStore> = std::sync::OnceLock::new();
    let store = STORE.get_or_init(MemoryStore::default);
    SessionManagerLayer::new(store.clone())
}
