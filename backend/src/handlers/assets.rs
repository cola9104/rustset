use axum::{
    extract::{State, Json, Path},
    http::{StatusCode, HeaderMap},
};
use shared::{Asset, PortInfo, PortBindingRequest, Role};
use crate::state::AppState;
use crate::utils::{get_current_user, log_action, determine_zone};

pub async fn get_assets(State(state): State<AppState>, headers: HeaderMap) -> Result<Json<Vec<Asset>>, (StatusCode, String)> {
    let _user = get_current_user(&headers, &state.users).ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;
    let assets = state.assets.lock().unwrap();
    Ok(Json(assets.clone()))
}

pub async fn add_asset(State(state): State<AppState>, headers: HeaderMap, Json(mut asset): Json<Asset>) -> Result<Json<Asset>, (StatusCode, String)> {
    let user = get_current_user(&headers, &state.users).ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;
    if user.role != Role::SecAdmin {
        return Err((StatusCode::FORBIDDEN, "Access denied: SecAdmin only".to_string()));
    }

    {
        let zones = state.zones.lock().unwrap();
        asset.zone = determine_zone(&asset.ip, &zones);
    }

    let mut assets = state.assets.lock().unwrap();
    let new_id = assets.len() as i32 + 1;
    asset.id = Some(new_id);
    asset.created_by = Some(user.username.clone());
    assets.push(asset.clone());

    log_action(&state.audit_logs, &user, "CREATE_ASSET", &asset.name, &format!("IP: {}", asset.ip));

    Ok(Json(asset))
}

pub async fn update_asset(State(state): State<AppState>, headers: HeaderMap, Path(id): Path<i32>, Json(req): Json<Asset>) -> Result<Json<Option<Asset>>, (StatusCode, String)> {
    let user = get_current_user(&headers, &state.users).ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;
    if user.role != Role::SecAdmin {
        return Err((StatusCode::FORBIDDEN, "Access denied: SecAdmin only".to_string()));
    }

    let zones = state.zones.lock().unwrap();
    let mut assets = state.assets.lock().unwrap();
    
    if let Some(asset) = assets.iter_mut().find(|a| a.id == Some(id)) {
        asset.name = req.name;
        if asset.ip != req.ip {
             asset.ip = req.ip.clone();
             asset.zone = determine_zone(&asset.ip, &zones);
        }
        asset.contact_person = req.contact_person;
        asset.contact_phone = req.contact_phone;
        asset.updated_by = Some(user.username.clone());

        log_action(&state.audit_logs, &user, "UPDATE_ASSET", &asset.name, "Updated asset details");
        return Ok(Json(Some(asset.clone())));
    }
    Ok(Json(None))
}

pub async fn delete_asset(State(state): State<AppState>, headers: HeaderMap, Path(id): Path<i32>) -> Result<Json<String>, (StatusCode, String)> {
    let user = get_current_user(&headers, &state.users).ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;
    if user.role != Role::SecAdmin {
        return Err((StatusCode::FORBIDDEN, "Access denied: SecAdmin only".to_string()));
    }

    let mut assets = state.assets.lock().unwrap();
    assets.retain(|a| a.id != Some(id));
    
    log_action(&state.audit_logs, &user, "DELETE_ASSET", &id.to_string(), "Deleted asset");
    Ok(Json("Deleted".to_string()))
}

pub async fn add_asset_port(State(state): State<AppState>, headers: HeaderMap, Path(id): Path<i32>, Json(mut port_info): Json<PortInfo>) -> Result<Json<Option<Asset>>, (StatusCode, String)> {
    let user = get_current_user(&headers, &state.users).ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;
    if user.role != Role::SecAdmin {
        return Err((StatusCode::FORBIDDEN, "Access denied: SecAdmin only".to_string()));
    }

    let mut assets = state.assets.lock().unwrap();
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

    let mut assets = state.assets.lock().unwrap();
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

    let mut assets = state.assets.lock().unwrap();
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

    let mut assets = state.assets.lock().unwrap();
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
