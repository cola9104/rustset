use axum::{
    extract::{State, Json, Path},
    http::{StatusCode, HeaderMap},
};
use shared::{User, CreateUserRequest, Role};
use crate::state::AppState;
use crate::utils::{get_current_user, log_action};
use uuid::Uuid;
use chrono::Utc;

pub async fn get_users(State(state): State<AppState>, headers: HeaderMap) -> Result<Json<Vec<User>>, (StatusCode, String)> {
    let user = get_current_user(&headers, &state.users).ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;
    if user.role != Role::SysAdmin {
        return Err((StatusCode::FORBIDDEN, "Access denied".to_string()));
    }
    let users = state.users.lock().unwrap();
    Ok(Json(users.clone()))
}

pub async fn create_user(State(state): State<AppState>, headers: HeaderMap, Json(req): Json<CreateUserRequest>) -> Result<Json<User>, (StatusCode, String)> {
    let current_user = get_current_user(&headers, &state.users).ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;
    if current_user.role != Role::SysAdmin {
        return Err((StatusCode::FORBIDDEN, "Access denied".to_string()));
    }

    let mut users = state.users.lock().unwrap();
    if users.iter().any(|u| u.username == req.username) {
        return Err((StatusCode::BAD_REQUEST, "Username exists".to_string()));
    }

    let new_user = User {
        id: Uuid::new_v4().to_string(),
        username: req.username.clone(),
        password: req.password,
        role: req.role,
        created_at: Utc::now(),
    };

    users.push(new_user.clone());
    log_action(&state.audit_logs, &current_user, "CREATE_USER", &new_user.username, "Created new user");
    
    Ok(Json(new_user))
}

pub async fn delete_user(State(state): State<AppState>, headers: HeaderMap, Path(id): Path<String>) -> Result<Json<String>, (StatusCode, String)> {
    let current_user = get_current_user(&headers, &state.users).ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;
    if current_user.role != Role::SysAdmin {
        return Err((StatusCode::FORBIDDEN, "Access denied".to_string()));
    }

    let mut users = state.users.lock().unwrap();
    if let Some(idx) = users.iter().position(|u| u.id == id) {
        let removed = users.remove(idx);
        log_action(&state.audit_logs, &current_user, "DELETE_USER", &removed.username, "Deleted user");
        Ok(Json("Deleted".to_string()))
    } else {
        Err((StatusCode::NOT_FOUND, "User not found".to_string()))
    }
}
