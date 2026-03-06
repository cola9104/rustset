use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use crate::state::machine_room::MachineRoomConfig;
use crate::utils::storage::get_token;

const API_BASE: &str = "http://localhost:3003/api";

fn auth_header() -> Result<String, String> {
    get_token().ok_or_else(|| "未登录，请先登录".to_string())
}

/// 后端返回的机房结构
#[derive(Clone, Debug, Serialize, Deserialize)]
struct BackendMachineRoom {
    id: i32,
    room_name: String,
    room_code: String,
    facility_type: String,
    address: String,
    provider_id: i32,
    room_type: String,
    contact_person: String,
    contact_phone: String,
    floor: Option<String>,
    cabinet_count: Option<i32>,
    area_size: Option<String>,
    remarks: Option<String>,
    status: String,
    created_at: String,
    updated_at: Option<String>,
}

impl BackendMachineRoom {
    fn to_frontend(&self) -> MachineRoomConfig {
        MachineRoomConfig {
            id: self.id,
            room_name: self.room_name.clone(),
            room_code: self.room_code.clone(),
            facility_type: self.facility_type.clone(),
            address: self.address.clone(),
            provider_id: self.provider_id,
            room_type: self.room_type.clone(),
            contact_person: self.contact_person.clone(),
            contact_phone: self.contact_phone.clone(),
            floor: self.floor.clone(),
            cabinet_count: self.cabinet_count,
            area_size: self.area_size.clone(),
            remarks: self.remarks.clone(),
            status: self.status.clone(),
            created_at: self.created_at.clone(),
            updated_at: self.updated_at.clone(),
        }
    }
}

/// 获取机房列表
pub async fn fetch_machine_rooms() -> Result<Vec<MachineRoomConfig>, String> {
    let token = auth_header()?;
    let response = Request::get(&format!("{}/machine-rooms", API_BASE))
        .header("Authorization", &token)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    if !response.ok() {
        return Err(format!("服务器错误: {}", response.status()));
    }

    let backend_rooms: Vec<BackendMachineRoom> = response
        .json()
        .await
        .map_err(|e| format!("解析失败: {}", e))?;

    Ok(backend_rooms.iter().map(|r| r.to_frontend()).collect())
}

/// 创建机房请求体
#[derive(Clone, Debug, Serialize)]
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

impl From<&MachineRoomConfig> for CreateMachineRoomRequest {
    fn from(config: &MachineRoomConfig) -> Self {
        Self {
            room_name: config.room_name.clone(),
            room_code: config.room_code.clone(),
            facility_type: config.facility_type.clone(),
            address: config.address.clone(),
            provider_id: config.provider_id,
            room_type: config.room_type.clone(),
            contact_person: config.contact_person.clone(),
            contact_phone: config.contact_phone.clone(),
            floor: config.floor.clone(),
            cabinet_count: config.cabinet_count,
            area_size: config.area_size.clone(),
            remarks: config.remarks.clone(),
            status: Some(config.status.clone()),
        }
    }
}

/// 创建机房
pub async fn create_machine_room(config: &MachineRoomConfig) -> Result<MachineRoomConfig, String> {
    let token = auth_header()?;
    let request_body = CreateMachineRoomRequest::from(config);

    let response = Request::post(&format!("{}/machine-rooms", API_BASE))
        .header("Authorization", &token)
        .json(&request_body)
        .map_err(|e| format!("构建请求失败: {}", e))?
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    if !response.ok() {
        return Err(format!("服务器错误: {}", response.status()));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("解析失败: {}", e))?;

    if let Some(data) = json.get("data") {
        let created: BackendMachineRoom = serde_json::from_value(data.clone())
            .map_err(|e| format!("解析响应失败: {}", e))?;
        Ok(created.to_frontend())
    } else {
        Err("响应格式错误".to_string())
    }
}

/// 更新机房
pub async fn update_machine_room(id: i32, config: &MachineRoomConfig) -> Result<MachineRoomConfig, String> {
    let token = auth_header()?;
    let request_body = CreateMachineRoomRequest::from(config);

    let response = Request::put(&format!("{}/machine-rooms/{}", API_BASE, id))
        .header("Authorization", &token)
        .json(&request_body)
        .map_err(|e| format!("构建请求失败: {}", e))?
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    if !response.ok() {
        return Err(format!("服务器错误: {}", response.status()));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("解析失败: {}", e))?;

    if let Some(data) = json.get("data") {
        let updated: BackendMachineRoom = serde_json::from_value(data.clone())
            .map_err(|e| format!("解析响应失败: {}", e))?;
        Ok(updated.to_frontend())
    } else {
        Err("响应格式错误".to_string())
    }
}

/// 删除机房
pub async fn delete_machine_room(id: i32) -> Result<(), String> {
    let token = auth_header()?;
    let response = Request::delete(&format!("{}/machine-rooms/{}", API_BASE, id))
        .header("Authorization", &token)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    if !response.ok() {
        return Err(format!("服务器错误: {}", response.status()));
    }

    Ok(())
}
