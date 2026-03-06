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

/// 机房数据模型
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MachineRoom {
    pub id: i32,
    pub room_name: String,
    pub room_code: String,
    pub facility_type: String,
    pub address: String,
    pub provider_id: i32,
    pub room_type: String,
    pub contact_person: String,
    pub contact_phone: String,
    pub floor: Option<String>,
    pub cabinet_count: Option<i32>,
    pub area_size: Option<String>,
    pub remarks: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: Option<String>,
}

/// 创建机房请求
#[derive(Clone, Debug, Deserialize)]
pub struct CreateMachineRoomRequest {
    pub room_name: String,
    pub room_code: String,
    pub facility_type: String,
    pub address: String,
    pub provider_id: i32,
    pub room_type: String,
    pub contact_person: String,
    pub contact_phone: String,
    pub floor: Option<String>,
    pub cabinet_count: Option<i32>,
    pub area_size: Option<String>,
    pub remarks: Option<String>,
    pub status: Option<String>,
}

/// 更新机房请求
#[derive(Clone, Debug, Deserialize)]
pub struct UpdateMachineRoomRequest {
    pub room_name: Option<String>,
    pub room_code: Option<String>,
    pub facility_type: Option<String>,
    pub address: Option<String>,
    pub provider_id: Option<i32>,
    pub room_type: Option<String>,
    pub contact_person: Option<String>,
    pub contact_phone: Option<String>,
    pub floor: Option<String>,
    pub cabinet_count: Option<i32>,
    pub area_size: Option<String>,
    pub remarks: Option<String>,
    pub status: Option<String>,
}

/// 获取机房列表
pub async fn get_machine_rooms(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let _user = match get_current_user(&headers, &state.users) {
        Some(u) => u,
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()).into_response(),
    };

    match crate::database::get_db() {
        Some(conn) => {
            match crate::database::get_all_machine_rooms(&conn).await {
                Ok(rooms) => Json(rooms).into_response(),
                Err(e) => {
                    eprintln!("Error loading machine rooms from database: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()).into_response()
                }
            }
        }
        None => {
            (StatusCode::INTERNAL_SERVER_ERROR, "Database not available".to_string()).into_response()
        }
    }
}

/// 获取单个机房
pub async fn get_machine_room(
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
            match crate::database::get_machine_room_by_id(&conn, id).await {
                Ok(Some(room)) => Json(room).into_response(),
                Ok(None) => {
                    (StatusCode::NOT_FOUND, "Machine room not found".to_string()).into_response()
                }
                Err(e) => {
                    eprintln!("Error loading machine room from database: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()).into_response()
                }
            }
        }
        None => {
            (StatusCode::INTERNAL_SERVER_ERROR, "Database not available".to_string()).into_response()
        }
    }
}

/// 创建机房
pub async fn create_machine_room(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateMachineRoomRequest>,
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

            match crate::database::insert_machine_room(
                &conn,
                &req.room_name,
                &req.room_code,
                &req.facility_type,
                &req.address,
                req.provider_id,
                &req.room_type,
                &req.contact_person,
                &req.contact_phone,
                req.floor.as_deref(),
                req.cabinet_count,
                req.area_size.as_deref(),
                req.remarks.as_deref(),
                &status,
                &now,
            ).await {
                Ok(id) => {
                    let room = MachineRoom {
                        id,
                        room_name: req.room_name.clone(),
                        room_code: req.room_code.clone(),
                        facility_type: req.facility_type.clone(),
                        address: req.address.clone(),
                        provider_id: req.provider_id,
                        room_type: req.room_type.clone(),
                        contact_person: req.contact_person.clone(),
                        contact_phone: req.contact_phone.clone(),
                        floor: req.floor.clone(),
                        cabinet_count: req.cabinet_count,
                        area_size: req.area_size.clone(),
                        remarks: req.remarks.clone(),
                        status,
                        created_at: now,
                        updated_at: None,
                    };

                    log_action(
                        &state.audit_logs,
                        &user,
                        "CREATE_MACHINE_ROOM",
                        &req.room_name,
                        &format!("Created machine room: {}", req.room_name),
                    );

                    Json(json!({
                        "message": "机房创建成功",
                        "data": room
                    })).into_response()
                }
                Err(e) => {
                    eprintln!("Error inserting machine room: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create machine room".to_string()).into_response()
                }
            }
        }
        None => {
            (StatusCode::INTERNAL_SERVER_ERROR, "Database not available".to_string()).into_response()
        }
    }
}

/// 更新机房
pub async fn update_machine_room(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i32>,
    Json(req): Json<UpdateMachineRoomRequest>,
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
            match crate::database::get_machine_room_by_id(&conn, id).await {
                Ok(Some(_)) => {
                    let now = chrono::Utc::now().to_rfc3339();
                    match crate::database::update_machine_room(
                        &conn,
                        id,
                        req.room_name.as_deref(),
                        req.room_code.as_deref(),
                        req.facility_type.as_deref(),
                        req.address.as_deref(),
                        req.provider_id,
                        req.room_type.as_deref(),
                        req.contact_person.as_deref(),
                        req.contact_phone.as_deref(),
                        req.floor.as_deref(),
                        req.cabinet_count,
                        req.area_size.as_deref(),
                        req.remarks.as_deref(),
                        req.status.as_deref(),
                        Some(&now),
                    ).await {
                        Ok(_) => {
                            log_action(
                                &state.audit_logs,
                                &user,
                                "UPDATE_MACHINE_ROOM",
                                &format!("{}", id),
                                &format!("Updated machine room: {}", id),
                            );

                            match crate::database::get_machine_room_by_id(&conn, id).await {
                                Ok(Some(room)) => {
                                    Json(json!({
                                        "message": "机房更新成功",
                                        "data": room
                                    })).into_response()
                                }
                                _ => {
                                    Json(json!({ "message": "机房更新成功" })).into_response()
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Error updating machine room: {}", e);
                            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update machine room".to_string()).into_response()
                        }
                    }
                }
                Ok(None) => {
                    (StatusCode::NOT_FOUND, "Machine room not found".to_string()).into_response()
                }
                Err(e) => {
                    eprintln!("Error checking machine room existence: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()).into_response()
                }
            }
        }
        None => {
            (StatusCode::INTERNAL_SERVER_ERROR, "Database not available".to_string()).into_response()
        }
    }
}

/// 删除机房
pub async fn delete_machine_room(
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
            match crate::database::get_machine_room_by_id(&conn, id).await {
                Ok(Some(_)) => {
                    match crate::database::delete_machine_room(&conn, id).await {
                        Ok(_) => {
                            log_action(
                                &state.audit_logs,
                                &user,
                                "DELETE_MACHINE_ROOM",
                                &format!("{}", id),
                                &format!("Deleted machine room: {}", id),
                            );

                            Json(json!({ "message": "机房删除成功" })).into_response()
                        }
                        Err(e) => {
                            eprintln!("Error deleting machine room: {}", e);
                            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete machine room".to_string()).into_response()
                        }
                    }
                }
                Ok(None) => {
                    (StatusCode::NOT_FOUND, "Machine room not found".to_string()).into_response()
                }
                Err(e) => {
                    eprintln!("Error checking machine room existence: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()).into_response()
                }
            }
        }
        None => {
            (StatusCode::INTERNAL_SERVER_ERROR, "Database not available".to_string()).into_response()
        }
    }
}
