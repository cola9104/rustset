use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json},
};
use serde_json::json;
use serde::{Deserialize, Serialize};
use chrono::Utc;

use crate::state::AppState;
use crate::utils::{get_current_user, log_action};
use shared::Role;

/// 服务商数据模型
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServiceProvider {
    pub id: i32,
    pub provider_name: String,
    pub provider_code: String,
    pub short_name: String,
    pub logo_url: Option<String>,
    pub contact_person: String,
    pub contact_phone: String,
    pub contact_email: String,
    pub headquarters: String,
    pub service_area: String,
    pub business_license: String,
    pub remarks: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: Option<String>,
}

/// 创建服务商请求
#[derive(Clone, Debug, Deserialize)]
pub struct CreateServiceProviderRequest {
    pub provider_name: String,
    pub provider_code: String,
    pub short_name: String,
    pub logo_url: Option<String>,
    pub contact_person: String,
    pub contact_phone: String,
    pub contact_email: String,
    pub headquarters: String,
    pub service_area: String,
    pub business_license: String,
    pub remarks: Option<String>,
    pub status: Option<String>,
}

/// 更新服务商请求
#[derive(Clone, Debug, Deserialize)]
pub struct UpdateServiceProviderRequest {
    pub provider_name: Option<String>,
    pub provider_code: Option<String>,
    pub short_name: Option<String>,
    pub logo_url: Option<String>,
    pub contact_person: Option<String>,
    pub contact_phone: Option<String>,
    pub contact_email: Option<String>,
    pub headquarters: Option<String>,
    pub service_area: Option<String>,
    pub business_license: Option<String>,
    pub remarks: Option<String>,
    pub status: Option<String>,
}

/// 获取服务商列表
pub async fn get_service_providers(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let _user = match get_current_user(&headers, &state.users) {
        Some(u) => u,
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()).into_response(),
    };

    match crate::database::get_db() {
        Some(conn) => {
            match crate::database::get_all_service_providers(&conn).await {
                Ok(providers) => Json(providers).into_response(),
                Err(e) => {
                    eprintln!("Error loading service providers from database: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()).into_response()
                }
            }
        }
        None => {
            (StatusCode::INTERNAL_SERVER_ERROR, "Database not available".to_string()).into_response()
        }
    }
}

/// 获取单个服务商
pub async fn get_service_provider(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    let _user = match get_current_user(&headers, &state.users) {
        Some(u) => u,
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()).into_response(),
    };

    match crate::database::get_db() {
        Some(conn) => {
            match crate::database::get_service_provider_by_id(&conn, id).await {
                Ok(Some(provider)) => Json(provider).into_response(),
                Ok(None) => {
                    (StatusCode::NOT_FOUND, "Service provider not found".to_string()).into_response()
                }
                Err(e) => {
                    eprintln!("Error loading service provider from database: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()).into_response()
                }
            }
        }
        None => {
            (StatusCode::INTERNAL_SERVER_ERROR, "Database not available".to_string()).into_response()
        }
    }
}

/// 创建服务商
pub async fn create_service_provider(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateServiceProviderRequest>,
) -> impl IntoResponse {
    let user = match get_current_user(&headers, &state.users) {
        Some(u) => u,
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()).into_response(),
    };

    if user.role != Role::SysAdmin && user.role != Role::SecAdmin {
        return (StatusCode::FORBIDDEN, "Access denied".to_string()).into_response();
    }

    match crate::database::get_db() {
        Some(conn) => {
            let now = Utc::now().to_rfc3339();
            let status = req.status.clone().unwrap_or_else(|| "active".to_string());

            match crate::database::insert_service_provider(
                &conn,
                &req.provider_name,
                &req.provider_code,
                &req.short_name,
                req.logo_url.as_deref(),
                &req.contact_person,
                &req.contact_phone,
                &req.contact_email,
                &req.headquarters,
                &req.service_area,
                &req.business_license,
                req.remarks.as_deref(),
                &status,
                &now,
            ).await {
                Ok(id) => {
                    let provider = ServiceProvider {
                        id,
                        provider_name: req.provider_name.clone(),
                        provider_code: req.provider_code.clone(),
                        short_name: req.short_name.clone(),
                        logo_url: req.logo_url.clone(),
                        contact_person: req.contact_person.clone(),
                        contact_phone: req.contact_phone.clone(),
                        contact_email: req.contact_email.clone(),
                        headquarters: req.headquarters.clone(),
                        service_area: req.service_area.clone(),
                        business_license: req.business_license.clone(),
                        remarks: req.remarks.clone(),
                        status,
                        created_at: now,
                        updated_at: None,
                    };

                    log_action(
                        &state.audit_logs,
                        &user,
                        "CREATE_SERVICE_PROVIDER",
                        &req.provider_name,
                        &format!("Created service provider: {}", req.provider_name),
                    );

                    Json(json!({
                        "message": "服务商创建成功",
                        "data": provider
                    })).into_response()
                }
                Err(e) => {
                    eprintln!("Error inserting service provider: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create service provider".to_string()).into_response()
                }
            }
        }
        None => {
            (StatusCode::INTERNAL_SERVER_ERROR, "Database not available".to_string()).into_response()
        }
    }
}

/// 更新服务商
pub async fn update_service_provider(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i32>,
    Json(req): Json<UpdateServiceProviderRequest>,
) -> impl IntoResponse {
    let user = match get_current_user(&headers, &state.users) {
        Some(u) => u,
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()).into_response(),
    };

    if user.role != Role::SysAdmin && user.role != Role::SecAdmin {
        return (StatusCode::FORBIDDEN, "Access denied".to_string()).into_response();
    }

    match crate::database::get_db() {
        Some(conn) => {
            // 检查是否存在
            match crate::database::get_service_provider_by_id(&conn, id).await {
                Ok(Some(_)) => {
                    let now = Utc::now().to_rfc3339();
                    match crate::database::update_service_provider(
                        &conn,
                        id,
                        req.provider_name.as_deref(),
                        req.provider_code.as_deref(),
                        req.short_name.as_deref(),
                        req.logo_url.as_deref(),
                        req.contact_person.as_deref(),
                        req.contact_phone.as_deref(),
                        req.contact_email.as_deref(),
                        req.headquarters.as_deref(),
                        req.service_area.as_deref(),
                        req.business_license.as_deref(),
                        req.remarks.as_deref(),
                        req.status.as_deref(),
                        Some(&now),
                    ).await {
                        Ok(_) => {
                            log_action(
                                &state.audit_logs,
                                &user,
                                "UPDATE_SERVICE_PROVIDER",
                                &format!("{}", id),
                                &format!("Updated service provider: {}", id),
                            );

                            // 返回更新后的数据
                            match crate::database::get_service_provider_by_id(&conn, id).await {
                                Ok(Some(provider)) => {
                                    Json(json!({
                                        "message": "服务商更新成功",
                                        "data": provider
                                    })).into_response()
                                }
                                _ => {
                                    Json(json!({ "message": "服务商更新成功" })).into_response()
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Error updating service provider: {}", e);
                            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update service provider".to_string()).into_response()
                        }
                    }
                }
                Ok(None) => {
                    (StatusCode::NOT_FOUND, "Service provider not found".to_string()).into_response()
                }
                Err(e) => {
                    eprintln!("Error checking service provider existence: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()).into_response()
                }
            }
        }
        None => {
            (StatusCode::INTERNAL_SERVER_ERROR, "Database not available".to_string()).into_response()
        }
    }
}

/// 删除服务商
pub async fn delete_service_provider(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    let user = match get_current_user(&headers, &state.users) {
        Some(u) => u,
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()).into_response(),
    };

    if user.role != Role::SysAdmin && user.role != Role::SecAdmin {
        return (StatusCode::FORBIDDEN, "Access denied".to_string()).into_response();
    }

    match crate::database::get_db() {
        Some(conn) => {
            match crate::database::get_service_provider_by_id(&conn, id).await {
                Ok(Some(_)) => {
                    match crate::database::delete_service_provider(&conn, id).await {
                        Ok(_) => {
                            log_action(
                                &state.audit_logs,
                                &user,
                                "DELETE_SERVICE_PROVIDER",
                                &format!("{}", id),
                                &format!("Deleted service provider: {}", id),
                            );

                            Json(json!({ "message": "服务商删除成功" })).into_response()
                        }
                        Err(e) => {
                            eprintln!("Error deleting service provider: {}", e);
                            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete service provider".to_string()).into_response()
                        }
                    }
                }
                Ok(None) => {
                    (StatusCode::NOT_FOUND, "Service provider not found".to_string()).into_response()
                }
                Err(e) => {
                    eprintln!("Error checking service provider existence: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()).into_response()
                }
            }
        }
        None => {
            (StatusCode::INTERNAL_SERVER_ERROR, "Database not available".to_string()).into_response()
        }
    }
}
