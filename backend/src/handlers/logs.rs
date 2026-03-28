use crate::database::{get_audit_logs as db_get_audit_logs, get_db};
use crate::middleware::{ApiError, AuthUser};
use crate::state::AppState;
use crate::utils::{ensure_user_has_any_role, require_current_user_from_auth};
use axum::{extract::State, Json};
use shared::{AuditLog, Role};

const AUDIT_LOGS_CACHE_KEY: &str = "rustset:cache:audit_logs";

async fn load_audit_logs(state: &AppState) -> Result<Vec<AuditLog>, ApiError> {
    let redis_config = crate::redis::redis_config();
    if let Some(cached) = crate::redis::cache_get_json::<Vec<AuditLog>>(AUDIT_LOGS_CACHE_KEY)
        .await
        .map_err(ApiError::internal)?
    {
        return Ok(cached);
    }

    if get_db().is_some() {
        let logs: Vec<AuditLog> = db_get_audit_logs(Some(1000))
            .await
            .map_err(|e| ApiError::internal(format!("Failed to load audit logs: {}", e)))?
            .into_iter()
            .map(|db_log| AuditLog {
                id: db_log.id,
                user_id: db_log.user_id,
                username: db_log.username,
                action: db_log.action,
                target: db_log.target,
                details: db_log.details,
                timestamp: chrono::DateTime::parse_from_rfc3339(&db_log.timestamp)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now()),
            })
            .collect();

        *state.audit_logs.write().map_err(|e| {
            ApiError::internal(format!("Failed to write audit logs cache: {}", e))
        })? = logs.clone();

        crate::redis::cache_set_json(
            AUDIT_LOGS_CACHE_KEY,
            &logs,
            redis_config.audit_logs_cache_ttl_secs,
        )
        .await
        .map_err(ApiError::internal)?;

        return Ok(logs);
    }

    state
        .audit_logs
        .read()
        .map(|logs| logs.clone())
        .map_err(|e| ApiError::internal(format!("Failed to read audit logs cache: {}", e)))
}

pub async fn get_audit_logs(
    auth_user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<AuditLog>>, ApiError> {
    let user = require_current_user_from_auth(&auth_user, &state).await?;
    ensure_user_has_any_role(
        &user,
        &[Role::SecAdmin, Role::SysAdmin, Role::Auditor],
        "Access denied: SecAdmin, SysAdmin, or Auditor only",
    )?;

    Ok(Json(load_audit_logs(&state).await?))
}
