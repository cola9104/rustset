use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json},
};
use serde_json::json;
use std::sync::Mutex;
use chrono::Utc;

use crate::state::AppState;
use crate::utils::{get_current_user, log_action};
use crate::database::{
    get_business_resources as db_get_business_resources,
    insert_business_resource_wrapper as db_insert_business_resource,
    update_business_resource as db_update_business_resource,
    delete_business_resource as db_delete_business_resource,
    get_business_resource_by_id as db_get_business_resource_by_id,
};
use shared::{
    BusinessResource, CreateBusinessResourceRequest, UpdateBusinessResourceRequest,
};

// 业务资源存储 (内存缓存 + 数据库持久化)
pub static BUSINESS_RESOURCES: Mutex<Vec<BusinessResource>> = Mutex::new(Vec::new());

/// 辅助函数：将 DbBusinessResource 转换为 BusinessResource
fn db_to_business_resource(db: crate::database::DbBusinessResource) -> BusinessResource {
    use chrono::TimeZone;
    BusinessResource {
        id: Some(db.id),
        resource_type: db.resource_type,
        ecs_name: db.ecs_name,
        ecs_status: db.ecs_status,
        resource_id: db.resource_id,
        cloud_region: db.cloud_region,
        cloud_category: db.cloud_category,
        cloud_provider_config_id: db.cloud_provider_config_id,
        zone_name: db.zone_name,
        platform_name: db.platform_name,
        county_city: db.county_city,
        vdc_name: db.vdc_name,
        customer_name: db.customer_name,
        application_name: db.application_name,
        contract_name: db.contract_name,
        instance_id: db.instance_id,
        ecs_type: db.ecs_type,
        ecs_os: db.ecs_os,
        cpu_cores: db.cpu_cores as u32,
        memory_gb: db.memory_gb as u32,
        system_disk: db.system_disk,
        system_disk_size_gb: db.system_disk_size_gb as u32,
        data_disk: db.data_disk,
        completion_time: db.completion_time.as_ref().and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok().map(|dt| dt.with_timezone(&Utc))),
        release_time: db.release_time.as_ref().and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok().map(|dt| dt.with_timezone(&Utc))),
        has_security_product: db.has_security_product != 0,
        ip_address: db.ip_address,
        ecs_login_method: db.ecs_login_method,
        ecs_login_username: db.ecs_login_username,
        ecs_initial_password: db.ecs_initial_password,
        bastion_address: db.bastion_address,
        bastion_admin_account: db.bastion_admin_account,
        bastion_initial_password: db.bastion_initial_password,
        serial_number: db.serial_number,
        rack_location: db.rack_location,
        hardware_model: db.hardware_model,
        warranty_expiry: db.warranty_expiry.as_ref().and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok().map(|dt| dt.with_timezone(&Utc))),
        agent_status: db.agent_status,
        ipmi_address: db.ipmi_address,
        remarks: db.remarks,
        created_at: chrono::DateTime::parse_from_rfc3339(&db.created_at).ok().map(|dt| dt.with_timezone(&Utc)),
        updated_at: db.updated_at.as_ref().and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok().map(|dt| dt.with_timezone(&Utc))),
        created_by: db.created_by,
        updated_by: db.updated_by,
    }
}

/// 获取业务资源列表
pub async fn get_business_resources(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let _user = match get_current_user(&headers, &state.users) {
        Some(u) => u,
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()).into_response(),
    };

    // 先尝试从数据库加载
    match db_get_business_resources().await {
        Ok(db_resources) => {
            let resources: Vec<BusinessResource> = db_resources
                .into_iter()
                .map(db_to_business_resource)
                .collect();

            // 更新内存缓存
            *BUSINESS_RESOURCES.lock().unwrap() = resources.clone();

            return Json(resources).into_response();
        }
        Err(_) => {
            // 数据库查询失败，回退到内存缓存
        }
    }

    // 回退到内存缓存
    let resources = BUSINESS_RESOURCES.lock().unwrap();
    Json(resources.clone()).into_response()
}

/// 创建业务资源申请
#[axum::debug_handler]
pub async fn create_business_resource(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateBusinessResourceRequest>,
) -> impl IntoResponse {
    let user = match get_current_user(&headers, &state.users) {
        Some(u) => u,
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()).into_response(),
    };

    // 检查权限
    if user.role != shared::Role::SysAdmin && user.role != shared::Role::SecAdmin {
        return (StatusCode::FORBIDDEN, "Access denied".to_string()).into_response();
    }

    let now = Utc::now();
    let created_at_str = now.to_rfc3339();

    // 先持久化到数据库，获取数据库生成的ID
    let db_id = match db_insert_business_resource(&req, &created_at_str, &user.username).await {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Error inserting business resource to database: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, json!({
                "message": "数据库保存失败",
                "error": e.to_string()
            }).to_string()).into_response();
        }
    };

    // 使用数据库生成的ID创建资源对象
    let new_resource = BusinessResource {
        id: Some(db_id),
        resource_type: req.resource_type.clone(),
        ecs_name: req.ecs_name.clone(),
        ecs_status: req.ecs_status.clone(),
        resource_id: req.resource_id.clone(),
        cloud_region: req.cloud_region.clone(),
        cloud_category: req.cloud_category.clone(),
        cloud_provider_config_id: req.cloud_provider_config_id,
        zone_name: req.zone_name.clone(),
        platform_name: req.platform_name.clone(),
        county_city: req.county_city.clone(),
        vdc_name: req.vdc_name.clone(),
        customer_name: req.customer_name.clone(),
        application_name: req.application_name.clone(),
        contract_name: req.contract_name.clone(),
        instance_id: req.instance_id.clone(),
        ecs_type: req.ecs_type.clone(),
        ecs_os: req.ecs_os.clone(),
        cpu_cores: req.cpu_cores,
        memory_gb: req.memory_gb,
        system_disk: req.system_disk.clone(),
        system_disk_size_gb: req.system_disk_size_gb,
        data_disk: req.data_disk.clone(),
        completion_time: req.completion_time,
        release_time: req.release_time,
        has_security_product: req.has_security_product,
        ip_address: req.ip_address.clone(),
        ecs_login_method: req.ecs_login_method.clone(),
        ecs_login_username: req.ecs_login_username.clone(),
        ecs_initial_password: req.ecs_initial_password.clone(),
        bastion_address: req.bastion_address.clone(),
        bastion_admin_account: req.bastion_admin_account.clone(),
        bastion_initial_password: req.bastion_initial_password.clone(),
        serial_number: req.serial_number.clone(),
        rack_location: req.rack_location.clone(),
        hardware_model: req.hardware_model.clone(),
        warranty_expiry: req.warranty_expiry,
        agent_status: None,
        ipmi_address: req.ipmi_address.clone(),
        remarks: req.remarks.clone(),
        created_at: Some(now),
        updated_at: Some(now),
        created_by: Some(user.username.clone()),
        updated_by: Some(user.username.clone()),
    };

    // 添加到内存缓存
    {
        let mut resources = BUSINESS_RESOURCES.lock().unwrap();
        resources.push(new_resource.clone());
    }

    // 记录日志
    log_action(
        &state.audit_logs,
        &user,
        "CREATE_BUSINESS_RESOURCE",
        &req.ecs_name,
        &format!("Created business resource: {} (ID: {})", req.ecs_name, db_id),
    );

    Json(json!({
        "message": "业务资源创建成功",
        "data": new_resource
    })).into_response()
}

/// 更新业务资源
#[axum::debug_handler]
pub async fn update_business_resource(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i32>,
    Json(req): Json<UpdateBusinessResourceRequest>,
) -> impl IntoResponse {
    let user = match get_current_user(&headers, &state.users) {
        Some(u) => u,
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()).into_response(),
    };

    if user.role != shared::Role::SysAdmin && user.role != shared::Role::SecAdmin {
        return (StatusCode::FORBIDDEN, "Access denied".to_string()).into_response();
    }

    // 检查资源是否存在
    let resource_exists = {
        let resources = BUSINESS_RESOURCES.lock().unwrap();
        resources.iter().any(|r| r.id == Some(id))
    };

    if !resource_exists {
        return (StatusCode::NOT_FOUND, "Business resource not found".to_string()).into_response();
    }

    // Clone the request before moving values (needed for database persistence later)
    let req_clone = req.clone();

    // 更新内存缓存中的资源
    let now = Utc::now();
    let updated_resource = {
        let mut resources = BUSINESS_RESOURCES.lock().unwrap();
        if let Some(resource) = resources.iter_mut().find(|r| r.id == Some(id)) {
            // 应用更新 - 对于 String 类型的字段，直接赋值
            if let Some(v) = req.resource_type { resource.resource_type = v; }
            if let Some(v) = req.ecs_name { resource.ecs_name = v; }
            if let Some(v) = req.ecs_status { resource.ecs_status = v; }
            if let Some(v) = req.cloud_region { resource.cloud_region = v; }
            if let Some(v) = req.cloud_category { resource.cloud_category = v; }
            if let Some(v) = req.cloud_provider_config_id { resource.cloud_provider_config_id = Some(v); }
            if let Some(v) = req.zone_name { resource.zone_name = Some(v); }
            if let Some(v) = req.platform_name { resource.platform_name = Some(v); }
            if let Some(v) = req.county_city { resource.county_city = Some(v); }
            if let Some(v) = req.vdc_name { resource.vdc_name = Some(v); }
            if let Some(v) = req.customer_name { resource.customer_name = v; }
            if let Some(v) = req.application_name { resource.application_name = Some(v); }
            if let Some(v) = req.contract_name { resource.contract_name = Some(v); }
            if let Some(v) = req.ecs_type { resource.ecs_type = v; }
            if let Some(v) = req.ecs_os { resource.ecs_os = v; }
            if let Some(v) = req.cpu_cores { resource.cpu_cores = v; }
            if let Some(v) = req.memory_gb { resource.memory_gb = v; }
            if let Some(v) = req.system_disk { resource.system_disk = v; }
            if let Some(v) = req.system_disk_size_gb { resource.system_disk_size_gb = v; }
            if let Some(v) = req.data_disk { resource.data_disk = Some(v); }
            if let Some(v) = req.completion_time { resource.completion_time = Some(v); }
            if let Some(v) = req.release_time { resource.release_time = Some(v); }
            if let Some(v) = req.has_security_product { resource.has_security_product = v; }
            if let Some(v) = req.ip_address { resource.ip_address = v; }
            if let Some(v) = req.ecs_login_method { resource.ecs_login_method = Some(v); }
            if let Some(v) = req.ecs_login_username { resource.ecs_login_username = Some(v); }
            if let Some(v) = req.ecs_initial_password { resource.ecs_initial_password = Some(v); }
            if let Some(v) = req.bastion_address { resource.bastion_address = Some(v); }
            if let Some(v) = req.bastion_admin_account { resource.bastion_admin_account = Some(v); }
            if let Some(v) = req.bastion_initial_password { resource.bastion_initial_password = Some(v); }
            if let Some(v) = req.serial_number { resource.serial_number = Some(v); }
            if let Some(v) = req.rack_location { resource.rack_location = Some(v); }
            if let Some(v) = req.hardware_model { resource.hardware_model = Some(v); }
            if let Some(v) = req.warranty_expiry { resource.warranty_expiry = Some(v); }
            if let Some(v) = req.agent_status { resource.agent_status = Some(v); }
            if let Some(v) = req.ipmi_address { resource.ipmi_address = Some(v); }
            if let Some(v) = req.remarks { resource.remarks = Some(v); }
            resource.updated_at = Some(now);
            resource.updated_by = Some(user.username.clone());
            Some(resource.clone())
        } else {
            None
        }
    };

    if let Some(resource) = updated_resource {
        // 持久化到数据库
        let _ = db_update_business_resource(id, &req_clone, &user.username).await;

        // 记录日志
        log_action(
            &state.audit_logs,
            &user,
            "UPDATE_BUSINESS_RESOURCE",
            &format!("{}", id),
            &format!("Updated business resource: {}", id),
        );

        Json(json!({
            "message": "业务资源更新成功",
            "data": resource
        })).into_response()
    } else {
        (StatusCode::NOT_FOUND, "Business resource not found".to_string()).into_response()
    }
}

/// 删除业务资源
#[axum::debug_handler]
pub async fn delete_business_resource(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    let user = match get_current_user(&headers, &state.users) {
        Some(u) => u,
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()).into_response(),
    };

    if user.role != shared::Role::SysAdmin && user.role != shared::Role::SecAdmin {
        return (StatusCode::FORBIDDEN, "Access denied".to_string()).into_response();
    }

    // 检查资源是否存在
    let found = {
        let mut resources = BUSINESS_RESOURCES.lock().unwrap();
        resources.iter().position(|r| r.id == Some(id))
    };

    if let Some(pos) = found {
        // 从内存中删除
        {
            let mut resources = BUSINESS_RESOURCES.lock().unwrap();
            resources.remove(pos);
        }

        // 持久化到数据库
        let _ = db_delete_business_resource(id).await;

        // 记录日志
        log_action(
            &state.audit_logs,
            &user,
            "DELETE_BUSINESS_RESOURCE",
            &format!("{}", id),
            &format!("Deleted business resource: {}", id),
        );

        Json(json!({
            "message": "业务资源删除成功"
        })).into_response()
    } else {
        (StatusCode::NOT_FOUND, "Business resource not found".to_string()).into_response()
    }
}
