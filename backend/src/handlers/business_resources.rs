use axum::{
    extract::{State, Json, Path},
    http::{StatusCode, HeaderMap},
};
use std::sync::Mutex;
use chrono::Utc;

use crate::state::AppState;
use crate::utils::{get_current_user, log_action};
use shared::{
    BusinessResource, CreateBusinessResourceRequest, UpdateBusinessResourceRequest,
    Role,
};

// 简单的内存存储（pub 以便其他模块访问）
pub static BUSINESS_RESOURCES: Mutex<Vec<BusinessResource>> = Mutex::new(Vec::new());

/// 获取业务资源列表
pub async fn get_business_resources(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<BusinessResource>>, (StatusCode, String)> {
    let _user = get_current_user(&headers, &state.users)
        .ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;

    let resources = BUSINESS_RESOURCES.lock().unwrap();
    Ok(Json(resources.clone()))
}

/// 创建业务资源申请
pub async fn create_business_resource(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateBusinessResourceRequest>,
) -> Result<Json<BusinessResource>, (StatusCode, String)> {
    let user = get_current_user(&headers, &state.users)
        .ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;

    // 检查权限
    if user.role != Role::SecAdmin && user.role != Role::SysAdmin {
        return Err((StatusCode::FORBIDDEN, "Access denied".to_string()));
    }

    let mut resources = BUSINESS_RESOURCES.lock().unwrap();

    // 创建新的业务资源
    let new_id = resources.len() as i32 + 1;
    let now = Utc::now();
    let new_resource = BusinessResource {
        id: Some(new_id),
        resource_type: req.resource_type.clone(),
        ecs_name: req.ecs_name.clone(),
        ecs_status: req.ecs_status.clone(),
        resource_id: req.resource_id.clone(),
        cloud_region: req.cloud_region.clone(),
        cloud_category: req.cloud_category.clone(),
        cloud_provider_config_id: req.cloud_provider_config_id,
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

    resources.push(new_resource.clone());

    // 记录日志
    log_action(
        &state.audit_logs,
        &user,
        "CREATE_BUSINESS_RESOURCE",
        &req.ecs_name,
        &format!("Created business resource application: {}", req.ecs_name),
    );

    Ok(Json(new_resource))
}

/// 更新业务资源
pub async fn update_business_resource(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i32>,
    Json(_req): Json<UpdateBusinessResourceRequest>,
) -> Result<Json<Option<BusinessResource>>, (StatusCode, String)> {
    let user = get_current_user(&headers, &state.users)
        .ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;

    if user.role != Role::SecAdmin {
        return Err((StatusCode::FORBIDDEN, "Access denied".to_string()));
    }

    let mut resources = BUSINESS_RESOURCES.lock().unwrap();

    if let Some(resource) = resources.iter_mut().find(|r| r.id == Some(id)) {
        // 更新字段
        if let Some(status) = &_req.ecs_status {
            resource.ecs_status = status.clone();
        }

        log_action(
            &state.audit_logs,
            &user,
            "UPDATE_BUSINESS_RESOURCE",
            &format!("{}", id),
            &format!("Updated business resource: {}", id),
        );

        return Ok(Json(Some(resource.clone())));
    }

    Ok(Json(None))
}

/// 删除业务资源
pub async fn delete_business_resource(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i32>,
) -> Result<Json<String>, (StatusCode, String)> {
    let user = get_current_user(&headers, &state.users)
        .ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))?;

    if user.role != Role::SecAdmin {
        return Err((StatusCode::FORBIDDEN, "Access denied".to_string()));
    }

    let mut resources = BUSINESS_RESOURCES.lock().unwrap();

    if let Some(pos) = resources.iter().position(|r| r.id == Some(id)) {
        resources.remove(pos);

        log_action(
            &state.audit_logs,
            &user,
            "DELETE_BUSINESS_RESOURCE",
            &format!("{}", id),
            &format!("Deleted business resource: {}", id),
        );

        return Ok(Json("Deleted".to_string()));
    }

    Err((StatusCode::NOT_FOUND, "Business resource not found".to_string()))
}
