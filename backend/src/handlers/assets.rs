use crate::database::{
    db_asset_to_shared, db_zone_to_shared, delete_asset as db_delete_asset,
    get_asset_by_id as db_get_asset_by_id, get_asset_by_ip as db_get_asset_by_ip,
    get_assets as db_get_assets, get_db, get_zones as db_get_zones,
    insert_asset_wrapper as db_insert_asset, update_asset as db_update_asset,
};
use crate::middleware::{ApiError, AuthUser};
use crate::state::AppState;
use crate::utils::{
    determine_zone, ensure_user_has_any_role, log_action, require_current_user_from_auth,
};
use axum::extract::{Json, Path, State};
use shared::{Asset, PortBindingRequest, PortInfo, Role, User, ZoneConfig};
use std::net::IpAddr;

fn sync_asset_cache(state: &AppState, asset: &Asset) -> Result<(), ApiError> {
    let mut assets = state
        .assets
        .write()
        .map_err(|e| ApiError::internal(format!("Failed to write assets cache: {}", e)))?;
    if let Some(id) = asset.id {
        if let Some(existing) = assets.iter_mut().find(|existing| existing.id == Some(id)) {
            *existing = asset.clone();
        } else {
            assets.push(asset.clone());
        }
    } else {
        assets.push(asset.clone());
    }
    Ok(())
}

fn remove_asset_from_cache(state: &AppState, id: i32) -> Result<(), ApiError> {
    let mut assets = state
        .assets
        .write()
        .map_err(|e| ApiError::internal(format!("Failed to write assets cache: {}", e)))?;
    assets.retain(|asset| asset.id != Some(id));
    Ok(())
}

async fn load_all_assets(state: &AppState) -> Result<Vec<Asset>, ApiError> {
    if get_db().is_some() {
        let assets: Vec<Asset> = db_get_assets()
            .await
            .map_err(|e| ApiError::internal(format!("Failed to load assets from database: {}", e)))?
            .into_iter()
            .map(db_asset_to_shared)
            .collect();
        let mut cache = state
            .assets
            .write()
            .map_err(|e| ApiError::internal(format!("Failed to write assets cache: {}", e)))?;
        *cache = assets.clone();
        return Ok(assets);
    }

    state
        .assets
        .read()
        .map(|assets| assets.clone())
        .map_err(|e| ApiError::internal(format!("Failed to read assets cache: {}", e)))
}

async fn load_asset_by_id(state: &AppState, id: i32) -> Result<Option<Asset>, ApiError> {
    if let Some(conn) = get_db() {
        let asset = db_get_asset_by_id(&conn, id)
            .await
            .map_err(|e| ApiError::internal(format!("Failed to load asset: {}", e)))?
            .map(db_asset_to_shared);
        if let Some(asset) = &asset {
            sync_asset_cache(state, asset)?;
        }
        return Ok(asset);
    }

    state
        .assets
        .read()
        .map(|assets| assets.iter().find(|asset| asset.id == Some(id)).cloned())
        .map_err(|e| ApiError::internal(format!("Failed to read assets cache: {}", e)))
}

async fn load_asset_by_ip(state: &AppState, ip: &str) -> Result<Option<Asset>, ApiError> {
    if let Some(conn) = get_db() {
        let asset = db_get_asset_by_ip(&conn, ip)
            .await
            .map_err(|e| ApiError::internal(format!("Failed to load asset: {}", e)))?
            .map(db_asset_to_shared);
        if let Some(asset) = &asset {
            sync_asset_cache(state, asset)?;
        }
        return Ok(asset);
    }

    state
        .assets
        .read()
        .map(|assets| assets.iter().find(|asset| asset.ip == ip).cloned())
        .map_err(|e| ApiError::internal(format!("Failed to read assets cache: {}", e)))
}

async fn load_zones(state: &AppState) -> Result<Vec<ZoneConfig>, ApiError> {
    if get_db().is_some() {
        let zones: Vec<ZoneConfig> = db_get_zones()
            .await
            .map_err(|e| ApiError::internal(format!("Failed to load zones from database: {}", e)))?
            .into_iter()
            .map(db_zone_to_shared)
            .collect();
        let mut cache = state
            .zones
            .write()
            .map_err(|e| ApiError::internal(format!("Failed to write zones cache: {}", e)))?;
        *cache = zones.clone();
        return Ok(zones);
    }

    state
        .zones
        .read()
        .map(|zones| zones.clone())
        .map_err(|e| ApiError::internal(format!("Failed to read zones cache: {}", e)))
}

async fn require_current_user(auth_user: &AuthUser, state: &AppState) -> Result<User, ApiError> {
    require_current_user_from_auth(auth_user, state).await
}

pub async fn get_assets(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<Vec<Asset>>, ApiError> {
    let _user = require_current_user(&auth_user, &state).await?;
    Ok(Json(load_all_assets(&state).await?))
}

pub async fn add_asset(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(mut asset): Json<Asset>,
) -> Result<Json<Asset>, ApiError> {
    let user = require_current_user(&auth_user, &state).await?;
    ensure_user_has_any_role(&user, &[Role::SecAdmin], "Access denied: SecAdmin only")?;

    if asset.ip.parse::<IpAddr>().is_err() {
        return Err(ApiError::bad_request("Invalid IP address format"));
    }

    let zones = load_zones(&state).await?;
    asset.zone = determine_zone(&asset.ip, &zones);

    asset.created_by = Some(user.username.clone());
    asset.updated_by = None;

    if get_db().is_some() {
        let new_id = db_insert_asset(&asset)
            .await
            .map_err(|e| ApiError::internal(format!("Failed to persist asset: {}", e)))?;
        asset.id = Some(new_id as i32);
    } else {
        let next_id = state
            .assets
            .read()
            .map_err(|e| ApiError::internal(format!("Failed to read assets cache: {}", e)))?
            .iter()
            .filter_map(|existing| existing.id)
            .max()
            .unwrap_or(0)
            + 1;
        asset.id = Some(next_id);
    }

    sync_asset_cache(&state, &asset)?;

    log_action(
        &state.audit_logs,
        &user,
        "CREATE_ASSET",
        &asset.name,
        &format!("IP: {}", asset.ip),
    );

    Ok(Json(asset))
}

pub async fn update_asset(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<i32>,
    Json(req): Json<Asset>,
) -> Result<Json<Option<Asset>>, ApiError> {
    let user = require_current_user(&auth_user, &state).await?;
    ensure_user_has_any_role(&user, &[Role::SecAdmin], "Access denied: SecAdmin only")?;

    if req.ip.parse::<IpAddr>().is_err() {
        return Err(ApiError::bad_request("Invalid IP address format"));
    }

    let zones = load_zones(&state).await?;

    let mut updated_asset = match load_asset_by_id(&state, id).await? {
        Some(asset) => asset,
        None => return Ok(Json(None)),
    };

    updated_asset.name = req.name.clone();
    if updated_asset.ip != req.ip {
        updated_asset.ip = req.ip.clone();
        updated_asset.zone = determine_zone(&updated_asset.ip, &zones);
    }
    updated_asset.contact_person = req.contact_person.clone();
    updated_asset.contact_phone = req.contact_phone.clone();
    updated_asset.owner = req.owner;
    updated_asset.weight = req.weight;
    updated_asset.labels = req.labels.clone();
    updated_asset.os = req.os.clone();
    updated_asset.device_type = req.device_type.clone();
    updated_asset.updated_by = Some(user.username.clone());

    if get_db().is_some() {
        db_update_asset(id, &updated_asset)
            .await
            .map_err(|e| ApiError::internal(format!("Failed to update asset: {}", e)))?;
    }
    sync_asset_cache(&state, &updated_asset)?;

    log_action(
        &state.audit_logs,
        &user,
        "UPDATE_ASSET",
        &updated_asset.name,
        "Updated asset details",
    );
    Ok(Json(Some(updated_asset)))
}

pub async fn delete_asset(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<String>, ApiError> {
    let user = require_current_user(&auth_user, &state).await?;
    ensure_user_has_any_role(&user, &[Role::SecAdmin], "Access denied: SecAdmin only")?;

    if load_asset_by_id(&state, id).await?.is_none() {
        return Err(ApiError::not_found(format!(
            "Asset with ID '{}' not found",
            id
        )));
    }

    if get_db().is_some() {
        db_delete_asset(id)
            .await
            .map_err(|e| ApiError::internal(format!("Failed to delete asset: {}", e)))?;
    }
    remove_asset_from_cache(&state, id)?;

    log_action(
        &state.audit_logs,
        &user,
        "DELETE_ASSET",
        &id.to_string(),
        "Deleted asset",
    );
    Ok(Json("Deleted".to_string()))
}

pub async fn add_asset_port(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<i32>,
    Json(mut port_info): Json<PortInfo>,
) -> Result<Json<Option<Asset>>, ApiError> {
    let user = require_current_user(&auth_user, &state).await?;
    ensure_user_has_any_role(&user, &[Role::SecAdmin], "Access denied: SecAdmin only")?;

    let mut asset = match load_asset_by_id(&state, id).await? {
        Some(asset) => asset,
        None => return Ok(Json(None)),
    };

    if !asset.ports.iter().any(|p| p.port == port_info.port) {
        port_info.created_by = Some(user.username.clone());
        port_info.updated_by = None;
        asset.ports.push(port_info.clone());
        asset.ports.sort_by(|a, b| a.port.cmp(&b.port));
        asset.updated_by = Some(user.username.clone());

        if get_db().is_some() {
            db_update_asset(id, &asset)
                .await
                .map_err(|e| ApiError::internal(format!("Failed to add port: {}", e)))?;
        }
        sync_asset_cache(&state, &asset)?;

        log_action(
            &state.audit_logs,
            &user,
            "ADD_PORT",
            &format!("{}:{}", asset.ip, port_info.port),
            "Added port",
        );
        return Ok(Json(Some(asset)));
    }

    Ok(Json(None))
}

pub async fn update_asset_port(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((id, port)): Path<(i32, u16)>,
    Json(port_info): Json<PortInfo>,
) -> Result<Json<Option<Asset>>, ApiError> {
    let user = require_current_user(&auth_user, &state).await?;
    ensure_user_has_any_role(&user, &[Role::SecAdmin], "Access denied: SecAdmin only")?;

    let mut asset = match load_asset_by_id(&state, id).await? {
        Some(asset) => asset,
        None => return Ok(Json(None)),
    };

    if let Some(p) = asset.ports.iter_mut().find(|p| p.port == port) {
        *p = port_info;
        p.port = port;
        p.updated_by = Some(user.username.clone());
        asset.updated_by = Some(user.username.clone());

        if get_db().is_some() {
            db_update_asset(id, &asset)
                .await
                .map_err(|e| ApiError::internal(format!("Failed to update port: {}", e)))?;
        }
        sync_asset_cache(&state, &asset)?;

        log_action(
            &state.audit_logs,
            &user,
            "UPDATE_PORT",
            &format!("{}:{}", asset.ip, port),
            "Updated port",
        );
    }
    Ok(Json(Some(asset)))
}

pub async fn delete_asset_port(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((id, port)): Path<(i32, u16)>,
) -> Result<Json<Option<Asset>>, ApiError> {
    let user = require_current_user(&auth_user, &state).await?;
    ensure_user_has_any_role(&user, &[Role::SecAdmin], "Access denied: SecAdmin only")?;

    let mut asset = match load_asset_by_id(&state, id).await? {
        Some(asset) => asset,
        None => return Ok(Json(None)),
    };

    let original_len = asset.ports.len();
    asset.ports.retain(|p| p.port != port);
    if asset.ports.len() != original_len {
        asset.updated_by = Some(user.username.clone());
        if get_db().is_some() {
            db_update_asset(id, &asset)
                .await
                .map_err(|e| ApiError::internal(format!("Failed to delete port: {}", e)))?;
        }
        sync_asset_cache(&state, &asset)?;
        log_action(
            &state.audit_logs,
            &user,
            "DELETE_PORT",
            &format!("{}:{}", asset.ip, port),
            "Deleted port",
        );
    }
    Ok(Json(Some(asset)))
}

pub async fn bind_port(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((ip, port)): Path<(String, u16)>,
    Json(req): Json<PortBindingRequest>,
) -> Result<Json<Option<Asset>>, ApiError> {
    let user = require_current_user(&auth_user, &state).await?;
    ensure_user_has_any_role(&user, &[Role::SecAdmin], "Access denied: SecAdmin only")?;

    let mut asset = match load_asset_by_ip(&state, &ip).await? {
        Some(asset) => asset,
        None => return Ok(Json(None)),
    };

    if let Some(p) = asset.ports.iter_mut().find(|p| p.port == port) {
        p.is_bound = true;
        p.system_name = Some(req.system_name.clone());
        p.middleware = Some(req.middleware.clone());
        p.updated_by = Some(user.username.clone());
        asset.updated_by = Some(user.username.clone());

        if let Some(id) = asset.id {
            if get_db().is_some() {
                db_update_asset(id, &asset)
                    .await
                    .map_err(|e| ApiError::internal(format!("Failed to bind port: {}", e)))?;
            }
        }
        sync_asset_cache(&state, &asset)?;

        log_action(
            &state.audit_logs,
            &user,
            "BIND_PORT",
            &format!("{}:{}", ip, port),
            &format!("Bound to {}", req.system_name),
        );
    }
    Ok(Json(Some(asset)))
}
