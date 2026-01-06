use axum::{
    extract::State,
    http::{StatusCode, HeaderMap},
    Json,
};
use shared::{AuditLog, Role};
use crate::state::AppState;
use crate::utils::get_current_user;

pub async fn get_audit_logs(State(state): State<AppState>, headers: HeaderMap) -> Result<Json<Vec<AuditLog>>, (StatusCode, String)> {
    let user = get_current_user(&headers, &state.users).ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;
    if user.role != Role::Auditor {
        return Err((StatusCode::FORBIDDEN, "Access denied".to_string()));
    }
    let logs = state.audit_logs.lock().unwrap();
    Ok(Json(logs.clone()))
}
