//! Health Check Handler
//!
//! 提供 API 健康检查和指标端点

use axum::{extract::State, Json};
use chrono::Utc;
use serde::Serialize;

use crate::database::{
    get_assets as db_get_assets, get_audit_logs as db_get_audit_logs, get_db,
    get_risks as db_get_risks, get_tasks as db_get_tasks, get_users as db_get_users,
};
use crate::redis::RedisHealth;
use crate::state::AppState;

/// 健康检查响应
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub database: DatabaseHealth,
    pub redis: RedisHealth,
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

#[derive(Debug, Clone, Copy)]
struct RuntimeCounts {
    users: usize,
    assets: usize,
    tasks: usize,
    risks: usize,
    audit_logs: usize,
}

async fn collect_runtime_counts(state: &AppState) -> RuntimeCounts {
    let cache_counts = RuntimeCounts {
        users: state.users.read().map(|u| u.len()).unwrap_or(0),
        assets: state.assets.read().map(|a| a.len()).unwrap_or(0),
        tasks: state.tasks.read().map(|t| t.len()).unwrap_or(0),
        risks: state.risks.read().map(|r| r.len()).unwrap_or(0),
        audit_logs: state.audit_logs.read().map(|l| l.len()).unwrap_or(0),
    };

    if get_db().is_none() {
        return cache_counts;
    }

    RuntimeCounts {
        users: db_get_users()
            .await
            .map(|items| items.len())
            .unwrap_or(cache_counts.users),
        assets: db_get_assets()
            .await
            .map(|items| items.len())
            .unwrap_or(cache_counts.assets),
        tasks: db_get_tasks()
            .await
            .map(|items| items.len())
            .unwrap_or(cache_counts.tasks),
        risks: db_get_risks()
            .await
            .map(|items| items.len())
            .unwrap_or(cache_counts.risks),
        audit_logs: db_get_audit_logs(None)
            .await
            .map(|items| items.len())
            .unwrap_or(cache_counts.audit_logs),
    }
}

/// 健康检查端点
pub async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
    // 检查数据库连接
    let db_connected = get_db().is_some();
    let redis = crate::redis::redis_health();
    let counts = collect_runtime_counts(&state).await;

    let redis_ok = !redis.enabled || redis.connected;
    let status = if db_connected && redis_ok {
        "healthy"
    } else {
        "degraded"
    };

    Json(HealthResponse {
        status: status.to_string(),
        timestamp: Utc::now().to_rfc3339(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: get_start_time().elapsed().as_secs(),
        database: DatabaseHealth {
            connected: db_connected,
        },
        redis,
        memory: MemoryHealth {
            users_count: counts.users,
            assets_count: counts.assets,
            tasks_count: counts.tasks,
            audit_logs_count: counts.audit_logs,
        },
    })
}

/// 就绪检查端点
pub async fn readiness_check(State(state): State<AppState>) -> Json<serde_json::Value> {
    let cache_ready = state.users.read().is_ok();
    let db_ready = if get_db().is_some() {
        db_get_users().await.is_ok()
    } else {
        false
    };
    let redis = crate::redis::redis_health();
    let redis_ready = !redis.enabled || redis.connected;

    Json(serde_json::json!({
        "ready": (db_ready || cache_ready) && redis_ready,
        "checks": {
            "database": db_ready,
            "memory": cache_ready,
            "redis": redis_ready
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
    let counts = collect_runtime_counts(&state).await;
    let uptime = get_start_time().elapsed().as_secs();
    let redis = crate::redis::redis_health();
    let redis_connected = if redis.connected { 1 } else { 0 };

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

# HELP rustset_redis_connected Redis connectivity status
# TYPE rustset_redis_connected gauge
rustset_redis_connected {}
"#,
        counts.users,
        counts.assets,
        counts.tasks,
        counts.risks,
        counts.audit_logs,
        uptime,
        redis_connected
    )
}
