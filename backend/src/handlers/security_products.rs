use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json},
};
use serde_json::json;
use serde::{Deserialize, Serialize};

use crate::state::AppState;
use crate::utils::{get_current_user, log_action};
use shared::Role;

/// 安全产品数据模型
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SecurityProduct {
    pub id: i32,
    pub name: String,
    pub category: String,
    pub vendor: String,
    pub model: String,
    pub version: String,
    pub serial_number: Option<String>,
    pub license_type: String,
    pub license_expiry: Option<String>,
    pub management_ip: Option<String>,
    pub deployment_mode: String,
    pub cloud_platform_id: Option<i32>,
    pub machine_room_id: Option<i32>,
    pub provider_id: Option<i32>,
    pub status: String,
    pub features: Option<String>,
    pub throughput: Option<String>,
    pub contact_person: String,
    pub contact_phone: String,
    pub remarks: Option<String>,
    pub created_at: String,
}

/// 创建安全产品请求
#[derive(Clone, Debug, Deserialize)]
pub struct CreateSecurityProductRequest {
    pub name: String,
    pub category: String,
    pub vendor: String,
    pub model: String,
    pub version: String,
    pub serial_number: Option<String>,
    pub license_type: String,
    pub license_expiry: Option<String>,
    pub management_ip: Option<String>,
    pub deployment_mode: String,
    pub cloud_platform_id: Option<i32>,
    pub machine_room_id: Option<i32>,
    pub provider_id: Option<i32>,
    pub status: Option<String>,
    pub features: Option<Vec<String>>,
    pub throughput: Option<String>,
    pub contact_person: String,
    pub contact_phone: String,
    pub remarks: Option<String>,
}

/// 更新安全产品请求
#[derive(Clone, Debug, Deserialize)]
pub struct UpdateSecurityProductRequest {
    pub name: Option<String>,
    pub category: Option<String>,
    pub vendor: Option<String>,
    pub model: Option<String>,
    pub version: Option<String>,
    pub serial_number: Option<String>,
    pub license_type: Option<String>,
    pub license_expiry: Option<String>,
    pub management_ip: Option<String>,
    pub deployment_mode: Option<String>,
    pub cloud_platform_id: Option<i32>,
    pub machine_room_id: Option<i32>,
    pub provider_id: Option<i32>,
    pub status: Option<String>,
    pub features: Option<Vec<String>>,
    pub throughput: Option<String>,
    pub contact_person: Option<String>,
    pub contact_phone: Option<String>,
    pub remarks: Option<String>,
}

/// 获取安全产品列表
pub async fn get_security_products(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let _user = match get_current_user(&headers, &state.users) {
        Some(u) => u,
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()).into_response(),
    };

    match crate::database::get_db() {
        Some(conn) => {
            match crate::database::get_all_security_products(&conn).await {
                Ok(products) => Json(products).into_response(),
                Err(e) => {
                    eprintln!("Error loading security products from database: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()).into_response()
                }
            }
        }
        None => {
            (StatusCode::INTERNAL_SERVER_ERROR, "Database not available".to_string()).into_response()
        }
    }
}

/// 获取单个安全产品
pub async fn get_security_product(
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
            match crate::database::get_security_product_by_id(&conn, id).await {
                Ok(Some(product)) => Json(product).into_response(),
                Ok(None) => {
                    (StatusCode::NOT_FOUND, "Security product not found".to_string()).into_response()
                }
                Err(e) => {
                    eprintln!("Error loading security product from database: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()).into_response()
                }
            }
        }
        None => {
            (StatusCode::INTERNAL_SERVER_ERROR, "Database not available".to_string()).into_response()
        }
    }
}

/// 创建安全产品
pub async fn create_security_product(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateSecurityProductRequest>,
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
            let now = chrono::Utc::now().to_rfc3339();
            let status = req.status.clone().unwrap_or_else(|| "active".to_string());
            let features = req.features.as_ref()
                .map(|f| serde_json::to_string(f).unwrap_or_else(|_| "[]".to_string()));

            match crate::database::insert_security_product(
                &conn,
                &req.name,
                &req.category,
                &req.vendor,
                &req.model,
                &req.version,
                req.serial_number.as_deref(),
                &req.license_type,
                req.license_expiry.as_deref(),
                req.management_ip.as_deref(),
                &req.deployment_mode,
                req.cloud_platform_id,
                req.machine_room_id,
                req.provider_id,
                &status,
                features.as_deref(),
                req.throughput.as_deref(),
                &req.contact_person,
                &req.contact_phone,
                req.remarks.as_deref(),
                &now,
            ).await {
                Ok(id) => {
                    let product = SecurityProduct {
                        id,
                        name: req.name.clone(),
                        category: req.category.clone(),
                        vendor: req.vendor.clone(),
                        model: req.model.clone(),
                        version: req.version.clone(),
                        serial_number: req.serial_number.clone(),
                        license_type: req.license_type.clone(),
                        license_expiry: req.license_expiry.clone(),
                        management_ip: req.management_ip.clone(),
                        deployment_mode: req.deployment_mode.clone(),
                        cloud_platform_id: req.cloud_platform_id,
                        machine_room_id: req.machine_room_id,
                        provider_id: req.provider_id,
                        status,
                        features,
                        throughput: req.throughput.clone(),
                        contact_person: req.contact_person.clone(),
                        contact_phone: req.contact_phone.clone(),
                        remarks: req.remarks.clone(),
                        created_at: now,
                    };

                    log_action(
                        &state.audit_logs,
                        &user,
                        "CREATE_SECURITY_PRODUCT",
                        &req.name,
                        &format!("Created security product: {}", req.name),
                    );

                    Json(json!({
                        "message": "安全产品创建成功",
                        "data": product
                    })).into_response()
                }
                Err(e) => {
                    eprintln!("Error inserting security product: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create security product".to_string()).into_response()
                }
            }
        }
        None => {
            (StatusCode::INTERNAL_SERVER_ERROR, "Database not available".to_string()).into_response()
        }
    }
}

/// 更新安全产品
pub async fn update_security_product(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i32>,
    Json(req): Json<UpdateSecurityProductRequest>,
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
            match crate::database::get_security_product_by_id(&conn, id).await {
                Ok(Some(_)) => {
                    let features = req.features.as_ref()
                        .map(|f| serde_json::to_string(f).unwrap_or_else(|_| "[]".to_string()));

                    match crate::database::update_security_product(
                        &conn,
                        id,
                        req.name.as_deref(),
                        req.category.as_deref(),
                        req.vendor.as_deref(),
                        req.model.as_deref(),
                        req.version.as_deref(),
                        req.serial_number.as_deref(),
                        req.license_type.as_deref(),
                        req.license_expiry.as_deref(),
                        req.management_ip.as_deref(),
                        req.deployment_mode.as_deref(),
                        req.cloud_platform_id,
                        req.machine_room_id,
                        req.provider_id,
                        req.status.as_deref(),
                        features.as_deref(),
                        req.throughput.as_deref(),
                        req.contact_person.as_deref(),
                        req.contact_phone.as_deref(),
                        req.remarks.as_deref(),
                    ).await {
                        Ok(_) => {
                            log_action(
                                &state.audit_logs,
                                &user,
                                "UPDATE_SECURITY_PRODUCT",
                                &format!("{}", id),
                                &format!("Updated security product: {}", id),
                            );

                            match crate::database::get_security_product_by_id(&conn, id).await {
                                Ok(Some(product)) => {
                                    Json(json!({
                                        "message": "安全产品更新成功",
                                        "data": product
                                    })).into_response()
                                }
                                _ => {
                                    Json(json!({ "message": "安全产品更新成功" })).into_response()
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Error updating security product: {}", e);
                            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update security product".to_string()).into_response()
                        }
                    }
                }
                Ok(None) => {
                    (StatusCode::NOT_FOUND, "Security product not found".to_string()).into_response()
                }
                Err(e) => {
                    eprintln!("Error checking security product existence: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()).into_response()
                }
            }
        }
        None => {
            (StatusCode::INTERNAL_SERVER_ERROR, "Database not available".to_string()).into_response()
        }
    }
}

/// 删除安全产品
pub async fn delete_security_product(
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
            match crate::database::get_security_product_by_id(&conn, id).await {
                Ok(Some(_)) => {
                    match crate::database::delete_security_product(&conn, id).await {
                        Ok(_) => {
                            log_action(
                                &state.audit_logs,
                                &user,
                                "DELETE_SECURITY_PRODUCT",
                                &format!("{}", id),
                                &format!("Deleted security product: {}", id),
                            );

                            Json(json!({ "message": "安全产品删除成功" })).into_response()
                        }
                        Err(e) => {
                            eprintln!("Error deleting security product: {}", e);
                            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete security product".to_string()).into_response()
                        }
                    }
                }
                Ok(None) => {
                    (StatusCode::NOT_FOUND, "Security product not found".to_string()).into_response()
                }
                Err(e) => {
                    eprintln!("Error checking security product existence: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()).into_response()
                }
            }
        }
        None => {
            (StatusCode::INTERNAL_SERVER_ERROR, "Database not available".to_string()).into_response()
        }
    }
}
