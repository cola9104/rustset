use axum::{
    extract::{State, Json},
    http::StatusCode,
};
use shared::{LoginRequest, LoginResponse};
use crate::state::AppState;

pub async fn login(State(state): State<AppState>, Json(req): Json<LoginRequest>) -> Result<Json<LoginResponse>, (StatusCode, String)> {
    let users = state.users.lock().unwrap();
    if let Some(user) = users.iter().find(|u| u.username == req.username && u.password == req.password) {
        // Mock Token: just username
        Ok(Json(LoginResponse {
            token: user.username.clone(),
            user: user.clone(),
        }))
    } else {
        Err((StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()))
    }
}
