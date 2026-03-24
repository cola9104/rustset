use axum::{response::IntoResponse, Json};
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};

use crate::database::get_db;
use crate::middleware::{ApiError, AuthUser};

#[derive(Debug, serde::Serialize)]
pub struct DashboardSummary {
    pub asset_count: u64,
    pub scanned_asset_count: u64,
    pub task_count: u64,
    pub running_task_count: u64,
    pub risk_count: u64,
    pub open_risk_count: u64,
    pub user_count: u64,
    pub active_user_count: u64,
    pub database_connected: bool,
    pub version: &'static str,
}

pub async fn get_dashboard_summary(_user: AuthUser) -> Result<impl IntoResponse, ApiError> {
    let conn = get_db().ok_or_else(|| ApiError::internal("Database not available"))?;

    let asset_count = crate::entities::asset::Entity::find()
        .count(conn.as_ref())
        .await
        .map_err(|e| ApiError::internal(format!("Failed to load asset count: {}", e)))?;

    let scanned_asset_count = crate::entities::asset::Entity::find()
        .filter(crate::entities::asset::Column::LastScanned.is_not_null())
        .count(conn.as_ref())
        .await
        .map_err(|e| ApiError::internal(format!("Failed to load scanned asset count: {}", e)))?;

    let task_count = crate::entities::task::Entity::find()
        .count(conn.as_ref())
        .await
        .map_err(|e| ApiError::internal(format!("Failed to load task count: {}", e)))?;

    let running_task_count = crate::entities::task::Entity::find()
        .filter(crate::entities::task::Column::Status.eq("Running"))
        .count(conn.as_ref())
        .await
        .map_err(|e| ApiError::internal(format!("Failed to load running task count: {}", e)))?;

    let risk_count = crate::entities::risk::Entity::find()
        .count(conn.as_ref())
        .await
        .map_err(|e| ApiError::internal(format!("Failed to load risk count: {}", e)))?;

    let open_risk_count = crate::entities::risk::Entity::find()
        .filter(crate::entities::risk::Column::Status.eq("Open"))
        .count(conn.as_ref())
        .await
        .map_err(|e| ApiError::internal(format!("Failed to load open risk count: {}", e)))?;

    let user_count = crate::entities::user::Entity::find()
        .count(conn.as_ref())
        .await
        .map_err(|e| ApiError::internal(format!("Failed to load user count: {}", e)))?;

    let active_user_count = crate::entities::user::Entity::find()
        .filter(crate::entities::user::Column::Status.eq("active"))
        .count(conn.as_ref())
        .await
        .map_err(|e| ApiError::internal(format!("Failed to load active user count: {}", e)))?;

    Ok(Json(DashboardSummary {
        asset_count,
        scanned_asset_count,
        task_count,
        running_task_count,
        risk_count,
        open_risk_count,
        user_count,
        active_user_count,
        database_connected: true,
        version: env!("CARGO_PKG_VERSION"),
    }))
}
