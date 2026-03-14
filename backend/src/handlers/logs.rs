use axum::{
    extract::State,
    http::HeaderMap,
    Json,
};
use shared::{AuditLog, Role};
use crate::state::AppState;
use crate::utils::get_current_user;
use crate::database::get_audit_logs as db_get_audit_logs;
use crate::middleware::ApiError;

pub async fn get_audit_logs(State(state): State<AppState>, headers: HeaderMap) -> Result<Json<Vec<AuditLog>>, ApiError> {
    let user = get_current_user(&headers, &state.users)
        .ok_or_else(|| ApiError::unauthorized("Unauthorized"))?;

    // SecAdmin、SysAdmin 和 Auditor 可以查看审计日志
    if user.role != Role::SecAdmin && user.role != Role::SysAdmin && user.role != Role::Auditor {
        return Err(ApiError::forbidden("Access denied: SecAdmin, SysAdmin, or Auditor only"));
    }

    // Try to load from database first
    match db_get_audit_logs(Some(1000)).await {
        Ok(db_logs) => {
            let logs: Vec<AuditLog> = db_logs.into_iter().map(|db_log| {
                AuditLog {
                    id: db_log.id,
                    user_id: db_log.user_id,
                    username: db_log.username,
                    action: db_log.action,
                    target: db_log.target,
                    details: db_log.details,
                    timestamp: chrono::DateTime::parse_from_rfc3339(&db_log.timestamp)
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                        .unwrap_or_else(|_| chrono::Utc::now()),
                }
            }).collect();
            // Update in-memory cache
            *state.audit_logs.write()
                .map_err(|e| ApiError::internal(format!("Failed to write audit logs cache: {}", e)))? = logs.clone();
            return Ok(Json(logs));
        }
        Err(e) => {
            eprintln!("Error loading audit logs from database: {}", e);
            // Fallback to memory cache
        }
    }

    // Fallback to memory cache
    let logs = state.audit_logs.read()
        .map_err(|e| ApiError::internal(format!("Failed to read audit logs cache: {}", e)))?;
    Ok(Json(logs.clone()))
}
