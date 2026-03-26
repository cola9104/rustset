use crate::database::{
    db_risk_to_shared, get_db, get_risk_by_id as db_get_risk_by_id, get_risks as db_get_risks,
    update_risk as db_update_risk,
};
use crate::middleware::{ApiError, AuthUser};
use crate::state::AppState;
use crate::utils::{ensure_user_has_any_role, log_action, require_current_user_from_auth};
use axum::extract::{Json, Path, State};
use chrono::Utc;
use shared::{Risk, RiskStatus, Role};

fn sync_risk_cache(state: &AppState, risk: &Risk) -> Result<(), ApiError> {
    let mut risks = state
        .risks
        .write()
        .map_err(|e| ApiError::internal(format!("Failed to write risks cache: {}", e)))?;

    if let Some(existing) = risks.iter_mut().find(|existing| existing.id == risk.id) {
        *existing = risk.clone();
    } else {
        risks.push(risk.clone());
    }

    Ok(())
}

async fn load_all_risks(state: &AppState) -> Result<Vec<Risk>, ApiError> {
    if get_db().is_some() {
        let risks: Vec<Risk> = db_get_risks()
            .await
            .map_err(|e| ApiError::internal(format!("Failed to load risks from database: {}", e)))?
            .into_iter()
            .map(db_risk_to_shared)
            .collect();

        *state
            .risks
            .write()
            .map_err(|e| ApiError::internal(format!("Failed to write risks cache: {}", e)))? =
            risks.clone();

        return Ok(risks);
    }

    state
        .risks
        .read()
        .map(|risks| risks.clone())
        .map_err(|e| ApiError::internal(format!("Failed to read risks cache: {}", e)))
}

async fn load_risk_by_id(state: &AppState, id: &str) -> Result<Option<Risk>, ApiError> {
    if get_db().is_some() {
        let risk = db_get_risk_by_id(id)
            .await
            .map_err(|e| ApiError::internal(format!("Failed to load risk: {}", e)))?
            .map(db_risk_to_shared);

        if let Some(risk) = &risk {
            sync_risk_cache(state, risk)?;
        }

        return Ok(risk);
    }

    state
        .risks
        .read()
        .map(|risks| risks.iter().find(|risk| risk.id == id).cloned())
        .map_err(|e| ApiError::internal(format!("Failed to read risks cache: {}", e)))
}

async fn persist_risk(state: &AppState, risk: &Risk) -> Result<(), ApiError> {
    if get_db().is_some() {
        db_update_risk(&risk.id, risk)
            .await
            .map_err(|e| ApiError::internal(format!("Failed to persist risk: {}", e)))?;
    }

    sync_risk_cache(state, risk)?;
    Ok(())
}

pub async fn get_risks(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<Vec<Risk>>, ApiError> {
    let _user = require_current_user_from_auth(&auth_user, &state).await?;

    Ok(Json(load_all_risks(&state).await?))
}

pub async fn update_risk_status(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((id, status_str)): Path<(String, String)>,
) -> Result<Json<String>, ApiError> {
    let user = require_current_user_from_auth(&auth_user, &state).await?;
    ensure_user_has_any_role(&user, &[Role::SecAdmin], "Access denied: SecAdmin only")?;

    let new_status = match status_str.as_str() {
        "verified" => RiskStatus::Verified,
        "resolved" => RiskStatus::Resolved,
        "ignored" => RiskStatus::Ignored,
        "open" => RiskStatus::Open,
        "false_positive" => RiskStatus::FalsePositive,
        "pending_review" => RiskStatus::PendingReview,
        _ => return Err(ApiError::bad_request("Invalid status. Valid values: verified, resolved, ignored, open, false_positive, pending_review")),
    };

    let mut risk = load_risk_by_id(&state, &id)
        .await?
        .ok_or_else(|| ApiError::not_found(format!("Risk with ID '{}' not found", id)))?;
    risk.status = new_status;
    risk.updated_at = Some(Utc::now());

    persist_risk(&state, &risk).await?;

    log_action(
        &state.audit_logs,
        &user,
        "UPDATE_RISK_STATUS",
        &id,
        &format!("Updated risk status to {}", status_str),
    );

    Ok(Json("Updated".to_string()))
}

pub async fn resolve_risk(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<String>, ApiError> {
    let user = require_current_user_from_auth(&auth_user, &state).await?;
    ensure_user_has_any_role(&user, &[Role::SecAdmin], "Access denied: SecAdmin only")?;

    let mut risk = load_risk_by_id(&state, &id)
        .await?
        .ok_or_else(|| ApiError::not_found(format!("Risk with ID '{}' not found", id)))?;
    risk.status = RiskStatus::Resolved;
    risk.updated_at = Some(Utc::now());

    persist_risk(&state, &risk).await?;

    log_action(
        &state.audit_logs,
        &user,
        "RESOLVE_RISK",
        &id,
        "Resolved risk",
    );

    Ok(Json("Resolved".to_string()))
}
