//! Health Check Handler
//!
//! 提供 API 健康检查和指标端点

use axum::{extract::State, Json};
use chrono::Utc;
use serde::Serialize;

use crate::database::get_db;
use crate::state::AppState;

/// 健康检查响应
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub database: DatabaseHealth,
    pub memory: MemoryHealth,
}

#[derive(Debug, Serialize)]
pub struct DatabaseHealth {
    pub connected: bool,
}

#[derive(Debug, Serialize)]
pub struct MemoryHealth {
    pub users_count: usize,
    pub assets_count: usize,
    pub tasks_count: usize,
    pub audit_logs_count: usize,
}

/// 启动时间
static START_TIME: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();

/// 获取启动时间
fn get_start_time() -> std::time::Instant {
    *START_TIME.get_or_init(std::time::Instant::now)
}

/// 健康检查端点
pub async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
    // 检查数据库连接
    let db_connected = get_db().is_some();

    // 获取内存统计
    let users_count = state.users.read().map(|u| u.len()).unwrap_or(0);
    let assets_count = state.assets.read().map(|a| a.len()).unwrap_or(0);
    let tasks_count = state.tasks.read().map(|t| t.len()).unwrap_or(0);
    let audit_logs_count = state.audit_logs.read().map(|l| l.len()).unwrap_or(0);

    let status = if db_connected { "healthy" } else { "degraded" };

    Json(HealthResponse {
        status: status.to_string(),
        timestamp: Utc::now().to_rfc3339(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: get_start_time().elapsed().as_secs(),
        database: DatabaseHealth {
            connected: db_connected,
        },
        memory: MemoryHealth {
            users_count,
            assets_count,
            tasks_count,
            audit_logs_count,
        },
    })
}

/// 就绪检查端点
pub async fn readiness_check(State(state): State<AppState>) -> Json<serde_json::Value> {
    // 检查数据库连接
    let db_ready = get_db().is_some() && state.users.read().is_ok();

    Json(serde_json::json!({
        "ready": db_ready,
        "checks": {
            "database": db_ready,
            "memory": true
        }
    }))
}

/// 存活检查端点
pub async fn liveness_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "alive": true,
        "timestamp": Utc::now().to_rfc3339()
    }))
}

/// Prometheus 指标端点
pub async fn metrics(State(state): State<AppState>) -> String {
    let users_count = state.users.read().map(|u| u.len()).unwrap_or(0);
    let assets_count = state.assets.read().map(|a| a.len()).unwrap_or(0);
    let tasks_count = state.tasks.read().map(|t| t.len()).unwrap_or(0);
    let risks_count = state.risks.read().map(|r| r.len()).unwrap_or(0);
    let audit_logs_count = state.audit_logs.read().map(|l| l.len()).unwrap_or(0);

    let uptime = get_start_time().elapsed().as_secs();

    format!(
        r#"# HELP rustset_users_total Total number of users
# TYPE rustset_users_total gauge
rustset_users_total {}

# HELP rustset_assets_total Total number of assets
# TYPE rustset_assets_total gauge
rustset_assets_total {}

# HELP rustset_tasks_total Total number of tasks
# TYPE rustset_tasks_total gauge
rustset_tasks_total {}

# HELP rustset_risks_total Total number of risks
# TYPE rustset_risks_total gauge
rustset_risks_total {}

# HELP rustset_audit_logs_total Total number of audit logs
# TYPE rustset_audit_logs_total gauge
rustset_audit_logs_total {}

# HELP rustset_uptime_seconds Uptime in seconds
# TYPE rustset_uptime_seconds counter
rustset_uptime_seconds {}
"#,
        users_count, assets_count, tasks_count, risks_count, audit_logs_count, uptime
    )
}
