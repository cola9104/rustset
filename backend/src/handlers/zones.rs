use axum::{
    extract::{State, Json, Path},
    http::{StatusCode, HeaderMap},
};
use shared::{ZoneConfig, Role};
use crate::state::AppState;
use crate::utils::{get_current_user, log_action};
use crate::database::{get_zones as db_get_zones, insert_zone_wrapper as db_insert_zone, update_zone as db_update_zone, delete_zone as db_delete_zone, db_zone_to_shared};
use crate::middleware::ApiError;
use uuid::Uuid;

pub async fn get_zones(State(state): State<AppState>, headers: HeaderMap) -> Result<Json<Vec<ZoneConfig>>, ApiError> {
    let _user = get_current_user(&headers, &state.users)
        .ok_or_else(|| ApiError::unauthorized("Unauthorized"))?;

    // Try to load from database first
    match db_get_zones().await {
        Ok(db_zones) => {
            let zones: Vec<ZoneConfig> = db_zones.into_iter().map(db_zone_to_shared).collect();
            // Update in-memory cache
            *state.zones.write()
                .map_err(|e| ApiError::internal(format!("Failed to write zones cache: {}", e)))? = zones.clone();
            return Ok(Json(zones));
        }
        Err(e) => {
            eprintln!("Error loading zones from database: {}", e);
            // Fallback to memory cache
            let zones = state.zones.read()
                .map_err(|e| ApiError::internal(format!("Failed to read zones cache: {}", e)))?;
            Ok(Json(zones.clone()))
        }
    }
}

pub async fn create_zone(State(state): State<AppState>, headers: HeaderMap, Json(req): Json<ZoneConfig>) -> Result<Json<ZoneConfig>, ApiError> {
    let user = get_current_user(&headers, &state.users)
        .ok_or_else(|| ApiError::unauthorized("Unauthorized"))?;

    if user.role != Role::SecAdmin {
        return Err(ApiError::forbidden("Access denied: SecAdmin only"));
    }

    let mut new_zone = req;
    if new_zone.id.is_empty() {
        new_zone.id = Uuid::new_v4().to_string();
    }

    // Add to in-memory storage
    {
        let mut zones = state.zones.write()
            .map_err(|e| ApiError::internal(format!("Failed to write zones: {}", e)))?;
        zones.push(new_zone.clone());
    }

    // Persist to database
    let _ = db_insert_zone(&new_zone).await;

    log_action(&state.audit_logs, &user, "CREATE_ZONE", &new_zone.name, "Created network zone");
    Ok(Json(new_zone))
}

pub async fn update_zone(State(state): State<AppState>, headers: HeaderMap, Path(id): Path<String>, Json(req): Json<ZoneConfig>) -> Result<Json<Option<ZoneConfig>>, ApiError> {
    let user = get_current_user(&headers, &state.users)
        .ok_or_else(|| ApiError::unauthorized("Unauthorized"))?;

    if user.role != Role::SecAdmin {
        return Err(ApiError::forbidden("Access denied: SecAdmin only"));
    }

    // Find and update zone, then release lock before async operations
    let (found, updated_zone) = {
        let mut zones = state.zones.write()
            .map_err(|e| ApiError::internal(format!("Failed to write zones: {}", e)))?;
        if let Some(zone) = zones.iter_mut().find(|z| z.id == id) {
            zone.name = req.name.clone();
            zone.cidr = req.cidr.clone();
            zone.priority = req.priority;
            (true, zone.clone())
        } else {
            (false, req)
        }
    };

    if found {
        // Persist to database (after releasing lock)
        let _ = db_update_zone(&id, &updated_zone).await;

        log_action(&state.audit_logs, &user, "UPDATE_ZONE", &updated_zone.name, "Updated zone config");
        Ok(Json(Some(updated_zone)))
    } else {
        Ok(Json(None))
    }
}

pub async fn delete_zone(State(state): State<AppState>, headers: HeaderMap, Path(id): Path<String>) -> Result<Json<String>, ApiError> {
    let user = get_current_user(&headers, &state.users)
        .ok_or_else(|| ApiError::unauthorized("Unauthorized"))?;

    if user.role != Role::SecAdmin {
        return Err(ApiError::forbidden("Access denied: SecAdmin only"));
    }

    let zone_name = {
        let zones = state.zones.read()
            .map_err(|e| ApiError::internal(format!("Failed to read zones: {}", e)))?;
        if let Some(z) = zones.iter().find(|z| z.id == id) {
            z.name.clone()
        } else {
            return Ok(Json("Zone not found".to_string()));
        }
    };

    // Remove from in-memory storage
    {
        let mut zones = state.zones.write()
            .map_err(|e| ApiError::internal(format!("Failed to write zones: {}", e)))?;
        zones.retain(|z| z.id != id);
    }

    // Persist to database (after releasing lock)
    let _ = db_delete_zone(&id).await;

    log_action(&state.audit_logs, &user, "DELETE_ZONE", &zone_name, "Deleted zone");
    Ok(Json("Deleted".to_string()))
}
