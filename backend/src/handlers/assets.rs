use axum::{
    extract::{State, Json, Path},
    http::{StatusCode, HeaderMap},
};
use std::net::IpAddr;
use shared::{Asset, PortInfo, PortBindingRequest, Role};
use crate::state::AppState;
use crate::utils::{get_current_user, log_action, determine_zone};
use crate::database::{get_assets as db_get_assets, insert_asset_wrapper as db_insert_asset, update_asset as db_update_asset, delete_asset as db_delete_asset, db_asset_to_shared};
use crate::middleware::ApiError;

pub async fn get_assets(State(state): State<AppState>, headers: HeaderMap) -> Result<Json<Vec<Asset>>, ApiError> {
    let _user = get_current_user(&headers, &state.users)
        .ok_or_else(|| ApiError::unauthorized("Unauthorized"))?;

    // Try to load from database first
    match db_get_assets().await {
        Ok(db_assets) => {
            let assets: Vec<Asset> = db_assets.into_iter().map(db_asset_to_shared).collect();
            // Update in-memory cache
            *state.assets.write().map_err(|e| ApiError::internal(format!("Failed to write assets cache: {}", e)))? = assets.clone();
            return Ok(Json(assets));
        }
        Err(e) => {
            eprintln!("Error loading assets from database: {}", e);
            // Fallback to memory cache
            let assets = state.assets.read()
                .map_err(|e| ApiError::internal(format!("Failed to read assets cache: {}", e)))?;
            Ok(Json(assets.clone()))
        }
    }
}

pub async fn add_asset(State(state): State<AppState>, headers: HeaderMap, Json(mut asset): Json<Asset>) -> Result<Json<Asset>, ApiError> {
    let user = get_current_user(&headers, &state.users)
        .ok_or_else(|| ApiError::unauthorized("Unauthorized"))?;

    if user.role != Role::SecAdmin {
        return Err(ApiError::forbidden("Access denied: SecAdmin only"));
    }

    if asset.ip.parse::<IpAddr>().is_err() {
        return Err(ApiError::bad_request("Invalid IP address format"));
    }

    {
        let zones = state.zones.read()
            .map_err(|e| ApiError::internal(format!("Failed to read zones: {}", e)))?;
        asset.zone = determine_zone(&asset.ip, &zones);
    }

    let new_asset = {
        let mut assets = state.assets.write()
            .map_err(|e| ApiError::internal(format!("Failed to write assets: {}", e)))?;
        let new_id = assets.len() as i32 + 1;
        asset.id = Some(new_id);
        asset.created_by = Some(user.username.clone());
        assets.push(asset.clone());
        asset
    };

    // Persist to database (after releasing lock)
    let _ = db_insert_asset(&new_asset).await;

    log_action(&state.audit_logs, &user, "CREATE_ASSET", &new_asset.name, &format!("IP: {}", new_asset.ip));

    Ok(Json(new_asset))
}

pub async fn update_asset(State(state): State<AppState>, headers: HeaderMap, Path(id): Path<i32>, Json(req): Json<Asset>) -> Result<Json<Option<Asset>>, ApiError> {
    let user = get_current_user(&headers, &state.users)
        .ok_or_else(|| ApiError::unauthorized("Unauthorized"))?;

    if user.role != Role::SecAdmin {
        return Err(ApiError::forbidden("Access denied: SecAdmin only"));
    }

    if req.ip.parse::<IpAddr>().is_err() {
        return Err(ApiError::bad_request("Invalid IP address format"));
    }

    // Clone zones data before acquiring assets lock
    let zones = {
        let zones_lock = state.zones.read()
            .map_err(|e| ApiError::internal(format!("Failed to read zones: {}", e)))?;
        zones_lock.clone()
    };

    // Find and update asset, then release lock before async operations
    let (found, updated_asset) = {
        let mut assets = state.assets.write().unwrap();
        if let Some(asset) = assets.iter_mut().find(|a| a.id == Some(id)) {
            asset.name = req.name.clone();
            if asset.ip != req.ip {
                 asset.ip = req.ip.clone();
                 asset.zone = determine_zone(&asset.ip, &zones);
            }
            asset.contact_person = req.contact_person.clone();
            asset.contact_phone = req.contact_phone.clone();
            asset.owner = req.owner;
            asset.weight = req.weight;
            asset.labels = req.labels.clone();
            asset.os = req.os.clone();
            asset.device_type = req.device_type.clone();
            asset.updated_by = Some(user.username.clone());
            (true, asset.clone())
        } else {
            (false, req)
        }
    };

    if found {
        // Persist to database (after releasing lock)
        let _ = db_update_asset(id, &updated_asset).await;

        log_action(&state.audit_logs, &user, "UPDATE_ASSET", &updated_asset.name, "Updated asset details");
        Ok(Json(Some(updated_asset)))
    } else {
        Ok(Json(None))
    }
}

pub async fn delete_asset(State(state): State<AppState>, headers: HeaderMap, Path(id): Path<i32>) -> Result<Json<String>, ApiError> {
    let user = get_current_user(&headers, &state.users)
        .ok_or_else(|| ApiError::unauthorized("Unauthorized"))?;

    if user.role != Role::SecAdmin {
        return Err(ApiError::forbidden("Access denied: SecAdmin only"));
    }

    // Remove from in-memory storage
    {
        let mut assets = state.assets.write()
            .map_err(|e| ApiError::internal(format!("Failed to write assets: {}", e)))?;
        assets.retain(|a| a.id != Some(id));
    }

    // Persist to database (after releasing lock)
    let _ = db_delete_asset(id).await;

    log_action(&state.audit_logs, &user, "DELETE_ASSET", &id.to_string(), "Deleted asset");
    Ok(Json("Deleted".to_string()))
}

pub async fn add_asset_port(State(state): State<AppState>, headers: HeaderMap, Path(id): Path<i32>, Json(mut port_info): Json<PortInfo>) -> Result<Json<Option<Asset>>, (StatusCode, String)> {
    let user = get_current_user(&headers, &state.users).ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;
    if user.role != Role::SecAdmin {
        return Err((StatusCode::FORBIDDEN, "Access denied: SecAdmin only".to_string()));
    }

    let mut assets = state.assets.write().unwrap();
    if let Some(asset) = assets.iter_mut().find(|a| a.id == Some(id)) {
        if !asset.ports.iter().any(|p| p.port == port_info.port) {
            port_info.created_by = Some(user.username.clone());
            asset.ports.push(port_info.clone());
            asset.ports.sort_by(|a, b| a.port.cmp(&b.port));
            
            log_action(&state.audit_logs, &user, "ADD_PORT", &format!("{}:{}", asset.ip, port_info.port), "Added port");
            return Ok(Json(Some(asset.clone())));
        }
    }
    Ok(Json(None))
}

pub async fn update_asset_port(State(state): State<AppState>, headers: HeaderMap, Path((id, port)): Path<(i32, u16)>, Json(port_info): Json<PortInfo>) -> Result<Json<Option<Asset>>, (StatusCode, String)> {
    let user = get_current_user(&headers, &state.users).ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;
    if user.role != Role::SecAdmin {
        return Err((StatusCode::FORBIDDEN, "Access denied: SecAdmin only".to_string()));
    }

    let mut assets = state.assets.write().unwrap();
    if let Some(asset) = assets.iter_mut().find(|a| a.id == Some(id)) {
        if let Some(p) = asset.ports.iter_mut().find(|p| p.port == port) {
            *p = port_info;
            p.port = port;
            p.updated_by = Some(user.username.clone());
            
            log_action(&state.audit_logs, &user, "UPDATE_PORT", &format!("{}:{}", asset.ip, port), "Updated port");
        }
        return Ok(Json(Some(asset.clone())));
    }
    Ok(Json(None))
}

pub async fn delete_asset_port(State(state): State<AppState>, headers: HeaderMap, Path((id, port)): Path<(i32, u16)>) -> Result<Json<Option<Asset>>, (StatusCode, String)> {
    let user = get_current_user(&headers, &state.users).ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;
    if user.role != Role::SecAdmin {
        return Err((StatusCode::FORBIDDEN, "Access denied: SecAdmin only".to_string()));
    }

    let mut assets = state.assets.write().unwrap();
    if let Some(asset) = assets.iter_mut().find(|a| a.id == Some(id)) {
        asset.ports.retain(|p| p.port != port);
        log_action(&state.audit_logs, &user, "DELETE_PORT", &format!("{}:{}", asset.ip, port), "Deleted port");
        return Ok(Json(Some(asset.clone())));
    }
    Ok(Json(None))
}

pub async fn bind_port(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((ip, port)): Path<(String, u16)>,
    Json(req): Json<PortBindingRequest>
) -> Result<Json<Option<Asset>>, (StatusCode, String)> {
    let user = get_current_user(&headers, &state.users).ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;
    if user.role != Role::SecAdmin {
        return Err((StatusCode::FORBIDDEN, "Access denied: SecAdmin only".to_string()));
    }

    let mut assets = state.assets.write().unwrap();
    if let Some(asset) = assets.iter_mut().find(|a| a.ip == ip) {
        if let Some(p) = asset.ports.iter_mut().find(|p| p.port == port) {
            p.is_bound = true;
            p.system_name = Some(req.system_name.clone());
            p.middleware = Some(req.middleware.clone());
            p.updated_by = Some(user.username.clone());
            
            log_action(&state.audit_logs, &user, "BIND_PORT", &format!("{}:{}", ip, port), &format!("Bound to {}", req.system_name));
        }
        return Ok(Json(Some(asset.clone())));
    }
    Ok(Json(None))
}
