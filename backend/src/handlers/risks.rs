use axum::{
    extract::{State, Json, Path},
    http::{StatusCode, HeaderMap},
};
use shared::{Risk, RiskStatus, Role};
use crate::state::AppState;
use crate::utils::{get_current_user, log_action};

pub async fn get_risks(State(state): State<AppState>, headers: HeaderMap) -> Result<Json<Vec<Risk>>, (StatusCode, String)> {
    let _user = get_current_user(&headers, &state.users).ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;
    let risks = state.risks.lock().unwrap();
    Ok(Json(risks.clone()))
}

pub async fn resolve_risk(State(state): State<AppState>, headers: HeaderMap, Path(id): Path<String>) -> Result<Json<String>, (StatusCode, String)> {
    let user = get_current_user(&headers, &state.users).ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;
    if user.role != Role::SecAdmin {
        return Err((StatusCode::FORBIDDEN, "Access denied: SecAdmin only".to_string()));
    }

    let mut risks = state.risks.lock().unwrap();
    if let Some(risk) = risks.iter_mut().find(|r| r.id == id) {
        risk.status = RiskStatus::Resolved;
        log_action(&state.audit_logs, &user, "RESOLVE_RISK", &id, "Resolved risk");
    }
    Ok(Json("Resolved".to_string()))
}
