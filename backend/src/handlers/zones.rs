use axum::{
    extract::{State, Json, Path},
    http::{StatusCode, HeaderMap},
};
use shared::{ZoneConfig, Role};
use crate::state::AppState;
use crate::utils::{get_current_user, log_action};
use uuid::Uuid;

pub async fn get_zones(State(state): State<AppState>, headers: HeaderMap) -> Result<Json<Vec<ZoneConfig>>, (StatusCode, String)> {
    let _user = get_current_user(&headers, &state.users).ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;
    let zones = state.zones.lock().unwrap();
    Ok(Json(zones.clone()))
}

pub async fn create_zone(State(state): State<AppState>, headers: HeaderMap, Json(req): Json<ZoneConfig>) -> Result<Json<ZoneConfig>, (StatusCode, String)> {
    let user = get_current_user(&headers, &state.users).ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;
    if user.role != Role::SecAdmin {
        return Err((StatusCode::FORBIDDEN, "Access denied: SecAdmin only".to_string()));
    }

    let mut zones = state.zones.lock().unwrap();
    let mut new_zone = req;
    if new_zone.id.is_empty() {
        new_zone.id = Uuid::new_v4().to_string();
    }
    zones.push(new_zone.clone());
    log_action(&state.audit_logs, &user, "CREATE_ZONE", &new_zone.name, "Created network zone");
    Ok(Json(new_zone))
}

pub async fn update_zone(State(state): State<AppState>, headers: HeaderMap, Path(id): Path<String>, Json(req): Json<ZoneConfig>) -> Result<Json<Option<ZoneConfig>>, (StatusCode, String)> {
    let user = get_current_user(&headers, &state.users).ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;
    if user.role != Role::SecAdmin {
        return Err((StatusCode::FORBIDDEN, "Access denied: SecAdmin only".to_string()));
    }

    let mut zones = state.zones.lock().unwrap();
    if let Some(zone) = zones.iter_mut().find(|z| z.id == id) {
        zone.name = req.name.clone();
        zone.cidr = req.cidr.clone();
        zone.priority = req.priority;
        log_action(&state.audit_logs, &user, "UPDATE_ZONE", &zone.name, "Updated zone config");
        return Ok(Json(Some(zone.clone())));
    }
    Ok(Json(None))
}

pub async fn delete_zone(State(state): State<AppState>, headers: HeaderMap, Path(id): Path<String>) -> Result<Json<String>, (StatusCode, String)> {
    let user = get_current_user(&headers, &state.users).ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;
    if user.role != Role::SecAdmin {
        return Err((StatusCode::FORBIDDEN, "Access denied: SecAdmin only".to_string()));
    }

    let mut zones = state.zones.lock().unwrap();
    let zone_name = if let Some(z) = zones.iter().find(|z| z.id == id) {
        z.name.clone()
    } else {
        return Ok(Json("Zone not found".to_string()));
    };

    zones.retain(|z| z.id != id);
    log_action(&state.audit_logs, &user, "DELETE_ZONE", &zone_name, "Deleted zone");
    Ok(Json("Deleted".to_string()))
}
