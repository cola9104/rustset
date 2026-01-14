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

use chrono::Utc;

pub async fn update_risk_status(State(state): State<AppState>, headers: HeaderMap, Path((id, status_str)): Path<(String, String)>) -> Result<Json<String>, (StatusCode, String)> {
    let user = get_current_user(&headers, &state.users).ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;
    if user.role != Role::SecAdmin {
        return Err((StatusCode::FORBIDDEN, "Access denied: SecAdmin only".to_string()));
    }

    let new_status = match status_str.as_str() {
        "verified" => RiskStatus::Verified,
        "resolved" => RiskStatus::Resolved,
        "ignored" => RiskStatus::Ignored,
        "open" => RiskStatus::Open,
        "false_positive" => RiskStatus::FalsePositive,
        "pending_review" => RiskStatus::PendingReview,
        _ => return Err((StatusCode::BAD_REQUEST, "Invalid status".to_string())),
    };

    let mut risks = state.risks.lock().unwrap();
    if let Some(risk) = risks.iter_mut().find(|r| r.id == id) {
        risk.status = new_status;
        risk.updated_at = Some(Utc::now());
        log_action(&state.audit_logs, &user, "UPDATE_RISK_STATUS", &id, &format!("Updated risk status to {}", status_str));
    }
    Ok(Json("Updated".to_string()))
}

pub async fn resolve_risk(State(state): State<AppState>, headers: HeaderMap, Path(id): Path<String>) -> Result<Json<String>, (StatusCode, String)> {
    let user = get_current_user(&headers, &state.users).ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;
    if user.role != Role::SecAdmin {
        return Err((StatusCode::FORBIDDEN, "Access denied: SecAdmin only".to_string()));
    }

    let mut risks = state.risks.lock().unwrap();
    if let Some(risk) = risks.iter_mut().find(|r| r.id == id) {
        risk.status = RiskStatus::Resolved;
        risk.updated_at = Some(Utc::now());
        log_action(&state.audit_logs, &user, "RESOLVE_RISK", &id, "Resolved risk");
    }
    Ok(Json("Resolved".to_string()))
}
