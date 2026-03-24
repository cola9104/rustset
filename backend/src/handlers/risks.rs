use crate::database::{
    db_risk_to_shared, get_risks as db_get_risks, update_risk as db_update_risk,
};
use crate::middleware::ApiError;
use crate::state::AppState;
use crate::utils::{get_current_user, log_action};
use axum::{
    extract::{Json, Path, State},
    http::HeaderMap,
};
use chrono::Utc;
use shared::{Risk, RiskStatus, Role};

pub async fn get_risks(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<Risk>>, ApiError> {
    let _user = get_current_user(&headers, &state.users)
        .ok_or_else(|| ApiError::unauthorized("Unauthorized"))?;

    // Try to load from database first
    match db_get_risks().await {
        Ok(db_risks) => {
            let risks: Vec<Risk> = db_risks.into_iter().map(db_risk_to_shared).collect();
            // Update in-memory cache
            *state
                .risks
                .write()
                .map_err(|e| ApiError::internal(format!("Failed to write risks cache: {}", e)))? =
                risks.clone();
            Ok(Json(risks))
        }
        Err(e) => {
            eprintln!("Error loading risks from database: {}", e);
            // Fallback to memory cache
            let risks = state
                .risks
                .read()
                .map_err(|e| ApiError::internal(format!("Failed to read risks cache: {}", e)))?;
            Ok(Json(risks.clone()))
        }
    }
}

pub async fn update_risk_status(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((id, status_str)): Path<(String, String)>,
) -> Result<Json<String>, ApiError> {
    let user = get_current_user(&headers, &state.users)
        .ok_or_else(|| ApiError::unauthorized("Unauthorized"))?;

    if user.role != Role::SecAdmin {
        return Err(ApiError::forbidden("Access denied: SecAdmin only"));
    }

    let new_status = match status_str.as_str() {
        "verified" => RiskStatus::Verified,
        "resolved" => RiskStatus::Resolved,
        "ignored" => RiskStatus::Ignored,
        "open" => RiskStatus::Open,
        "false_positive" => RiskStatus::FalsePositive,
        "pending_review" => RiskStatus::PendingReview,
        _ => return Err(ApiError::bad_request("Invalid status. Valid values: verified, resolved, ignored, open, false_positive, pending_review")),
    };

    let (found, updated_risk) = {
        let mut risks = state
            .risks
            .write()
            .map_err(|e| ApiError::internal(format!("Failed to write risks: {}", e)))?;
        if let Some(risk) = risks.iter_mut().find(|r| r.id == id) {
            risk.status = new_status;
            risk.updated_at = Some(Utc::now());
            (true, risk.clone())
        } else {
            (
                false,
                shared::Risk {
                    id: id.clone(),
                    asset_ip: String::new(),
                    port: 0,
                    severity: String::new(),
                    description: String::new(),
                    solution: None,
                    status: new_status,
                    created_at: None,
                    updated_at: Some(Utc::now()),
                    assigned_to: None,
                },
            )
        }
    };

    if found {
        // Persist to database (after releasing lock)
        let _ = db_update_risk(&id, &updated_risk).await;

        log_action(
            &state.audit_logs,
            &user,
            "UPDATE_RISK_STATUS",
            &id,
            &format!("Updated risk status to {}", status_str),
        );
    }
    Ok(Json("Updated".to_string()))
}

pub async fn resolve_risk(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<String>, ApiError> {
    let user = get_current_user(&headers, &state.users)
        .ok_or_else(|| ApiError::unauthorized("Unauthorized"))?;

    if user.role != Role::SecAdmin {
        return Err(ApiError::forbidden("Access denied: SecAdmin only"));
    }

    let (found, updated_risk) = {
        let mut risks = state
            .risks
            .write()
            .map_err(|e| ApiError::internal(format!("Failed to write risks: {}", e)))?;
        if let Some(risk) = risks.iter_mut().find(|r| r.id == id) {
            risk.status = RiskStatus::Resolved;
            risk.updated_at = Some(Utc::now());
            (true, risk.clone())
        } else {
            (
                false,
                shared::Risk {
                    id: id.clone(),
                    asset_ip: String::new(),
                    port: 0,
                    severity: String::new(),
                    description: String::new(),
                    solution: None,
                    status: RiskStatus::Resolved,
                    created_at: None,
                    updated_at: Some(Utc::now()),
                    assigned_to: None,
                },
            )
        }
    };

    if found {
        // Persist to database (after releasing lock)
        let _ = db_update_risk(&id, &updated_risk).await;

        log_action(
            &state.audit_logs,
            &user,
            "RESOLVE_RISK",
            &id,
            "Resolved risk",
        );
    }
    Ok(Json("Resolved".to_string()))
}
