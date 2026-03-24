use tower_sessions::{MemoryStore, SessionManagerLayer};

/// 同步版本的 Session Layer 创建
/// 用于在非 async 上下文中使用
pub fn create_session_layer_sync() -> SessionManagerLayer<MemoryStore> {
    // 注意：这会在第一次调用时初始化 store
    // 后续调用会复用同一个 store
    static STORE: std::sync::OnceLock<MemoryStore> = std::sync::OnceLock::new();
    let store = STORE.get_or_init(MemoryStore::default);
    SessionManagerLayer::new(store.clone())
}
