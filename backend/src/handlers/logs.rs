use crate::database::{get_audit_logs as db_get_audit_logs, get_db};
use crate::middleware::ApiError;
use crate::state::AppState;
use crate::utils::get_current_user_from_headers;
use axum::{extract::State, http::HeaderMap, Json};
use shared::{AuditLog, Role};

async fn load_audit_logs(state: &AppState) -> Result<Vec<AuditLog>, ApiError> {
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

        return Ok(logs);
    }

    state
        .audit_logs
        .read()
        .map(|logs| logs.clone())
        .map_err(|e| ApiError::internal(format!("Failed to read audit logs cache: {}", e)))
}

pub async fn get_audit_logs(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<AuditLog>>, ApiError> {
    let user = get_current_user_from_headers(&headers, &state.users)
        .await
        .ok_or_else(|| ApiError::unauthorized("Unauthorized"))?;

    // SecAdmin、SysAdmin 和 Auditor 可以查看审计日志
    if user.role != Role::SecAdmin && user.role != Role::SysAdmin && user.role != Role::Auditor {
        return Err(ApiError::forbidden(
            "Access denied: SecAdmin, SysAdmin, or Auditor only",
        ));
    }

    Ok(Json(load_audit_logs(&state).await?))
}
