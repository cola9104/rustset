use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json},
};
use serde_json::json;
use std::sync::RwLock;
use chrono::Utc;

use crate::state::AppState;
use crate::utils::{get_current_user, log_action};
use crate::database::{
    get_business_resources as db_get_business_resources,
    insert_business_resource_wrapper as db_insert_business_resource,
    update_business_resource as db_update_business_resource,
    delete_business_resource as db_delete_business_resource,
    get_physical_machine_by_business_resource_id,
    get_cloud_virtual_machine_by_business_resource_id,
    DbPhysicalMachine, DbCloudVirtualMachine,
};
use shared::{
    BusinessResource, CreateBusinessResourceRequest, UpdateBusinessResourceRequest,
    PhysicalMachineInfo, CloudVirtualMachineInfo,
};

// 业务资源存储 (内存缓存 + 数据库持久化)
pub static BUSINESS_RESOURCES: RwLock<Vec<BusinessResource>> = RwLock::new(Vec::new());

/// 辅助函数：将 DbPhysicalMachine 转换为 PhysicalMachineInfo
fn db_to_physical_machine_info(db: DbPhysicalMachine) -> PhysicalMachineInfo {
    PhysicalMachineInfo {
        id: Some(db.id),
        business_resource_id: Some(db.business_resource_id),
        serial_number: db.serial_number,
        rack_location: db.rack_location,
        hardware_model: db.hardware_model,
        warranty_expiry: db.warranty_expiry.as_ref().and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok().map(|dt| dt.with_timezone(&Utc))),
        agent_status: db.agent_status,
        ipmi_address: db.ipmi_address,
    }
}

/// 辅助函数：将 DbCloudVirtualMachine 转换为 CloudVirtualMachineInfo
fn db_to_cloud_vm_info(db: DbCloudVirtualMachine) -> CloudVirtualMachineInfo {
    CloudVirtualMachineInfo {
        id: Some(db.id),
        business_resource_id: Some(db.business_resource_id),
        billing_mode: db.billing_mode,
        expire_time: db.expire_time.as_ref().and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok().map(|dt| dt.with_timezone(&Utc))),
        charge_type: db.charge_type,
        instance_charge_type: db.instance_charge_type,
        internet_charge_type: db.internet_charge_type,
        internet_max_bandwidth_out: db.internet_max_bandwidth_out,
        image_id: db.image_id,
        v_switch_id: db.v_switch_id,
        vpc_id: db.vpc_id,
        security_group_ids: db.security_group_ids.and_then(|s| serde_json::from_str(&s).ok()),
    }
}

/// 辅助函数：将 DbBusinessResource 转换为 BusinessResource（包含详情信息）
async fn db_to_business_resource_with_details(
    db: crate::database::DbBusinessResource,
    conn: &sea_orm::DatabaseConnection,
) -> BusinessResource {
    

    // Load detail information based on resource type
    let physical_machine_info = if db.resource_type == "physical" {
        if let Ok(Some(pm)) = get_physical_machine_by_business_resource_id(conn, db.id).await {
            Some(db_to_physical_machine_info(pm))
        } else {
            None
        }
    } else {
        None
    };

    let cloud_vm_info = if db.resource_type == "cloud" {
        if let Ok(Some(cvm)) = get_cloud_virtual_machine_by_business_resource_id(conn, db.id).await {
            Some(db_to_cloud_vm_info(cvm))
        } else {
            None
        }
    } else {
        None
    };

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
        physical_machine_info,
        cloud_vm_info,
        remarks: db.remarks,
        created_at: chrono::DateTime::parse_from_rfc3339(&db.created_at).ok().map(|dt| dt.with_timezone(&Utc)),
        updated_at: db.updated_at.as_ref().and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok().map(|dt| dt.with_timezone(&Utc))),
        created_by: db.created_by,
        updated_by: db.updated_by,
        // 申请流程相关
        applicant: db.applicant,
        department: db.department,
        approver: db.approver,
        approval_time: db.approval_time.as_ref().and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok().map(|dt| dt.with_timezone(&Utc))),
        approval_remarks: db.approval_remarks,
        rejection_reason: db.rejection_reason,
        // 资源配置相关
        bandwidth_mbps: db.bandwidth_mbps.map(|v| v as u32),
        bandwidth_type: db.bandwidth_type,
        public_ip_count: db.public_ip_count.map(|v| v as u32),
        network_type: db.network_type,
        // 业务关联相关
        project_name: db.project_name,
        project_code: db.project_code,
        business_owner: db.business_owner,
        tech_owner: db.tech_owner,
        contact_phone: db.contact_phone,
        // 费用相关
        billing_method: db.billing_method,
        purchase_duration: db.purchase_duration.map(|v| v as u32),
        cost_center: db.cost_center,
        // 合规相关
        security_level: db.security_level,
        data_sensitivity: db.data_sensitivity,
        // 其他
        purpose: db.purpose,
        expected_delivery_time: db.expected_delivery_time.as_ref().and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok().map(|dt| dt.with_timezone(&Utc))),
        // 状态管理
        application_status: db.application_status,
        delivery_status: db.delivery_status,
        delivery_confirmed_at: db.delivery_confirmed_at.as_ref().and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok().map(|dt| dt.with_timezone(&Utc))),
        delivery_confirmed_by: db.delivery_confirmed_by,
    }
}

/// 简化版本：不包含详情信息的转换（用于兼容旧代码）
fn db_to_business_resource(db: crate::database::DbBusinessResource) -> BusinessResource {
    
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
        physical_machine_info: None,
        cloud_vm_info: None,
        remarks: db.remarks,
        created_at: chrono::DateTime::parse_from_rfc3339(&db.created_at).ok().map(|dt| dt.with_timezone(&Utc)),
        updated_at: db.updated_at.as_ref().and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok().map(|dt| dt.with_timezone(&Utc))),
        created_by: db.created_by,
        updated_by: db.updated_by,
        // 申请流程相关
        applicant: db.applicant,
        department: db.department,
        approver: db.approver,
        approval_time: db.approval_time.as_ref().and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok().map(|dt| dt.with_timezone(&Utc))),
        approval_remarks: db.approval_remarks,
        rejection_reason: db.rejection_reason,
        // 资源配置相关
        bandwidth_mbps: db.bandwidth_mbps.map(|v| v as u32),
        bandwidth_type: db.bandwidth_type,
        public_ip_count: db.public_ip_count.map(|v| v as u32),
        network_type: db.network_type,
        // 业务关联相关
        project_name: db.project_name,
        project_code: db.project_code,
        business_owner: db.business_owner,
        tech_owner: db.tech_owner,
        contact_phone: db.contact_phone,
        // 费用相关
        billing_method: db.billing_method,
        purchase_duration: db.purchase_duration.map(|v| v as u32),
        cost_center: db.cost_center,
        // 合规相关
        security_level: db.security_level,
        data_sensitivity: db.data_sensitivity,
        // 其他
        purpose: db.purpose,
        expected_delivery_time: db.expected_delivery_time.as_ref().and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok().map(|dt| dt.with_timezone(&Utc))),
        // 状态管理
        application_status: db.application_status,
        delivery_status: db.delivery_status,
        delivery_confirmed_at: db.delivery_confirmed_at.as_ref().and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok().map(|dt| dt.with_timezone(&Utc))),
        delivery_confirmed_by: db.delivery_confirmed_by,
    }
}

/// 获取业务资源列表
/// 注意：只返回未完成交付的资源（delivery_status != "已交付"）
/// 已交付的资源将在云服务资产中显示
pub async fn get_business_resources(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let _user = match get_current_user(&headers, &state.users) {
        Some(u) => u,
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()).into_response(),
    };

    // 先尝试从数据库加载
    if let Some(conn) = crate::database::get_db() {
        match db_get_business_resources().await {
            Ok(db_resources) => {
                // Load with details，过滤掉已交付的资源
                let mut resources = Vec::new();
                for db_res in db_resources {
                    // 跳过已交付的资源，它们应该在云服务资产中显示
                    if db_res.delivery_status.as_ref().map(|s| s == "已交付").unwrap_or(false) {
                        continue;
                    }
                    let res = db_to_business_resource_with_details(db_res, &conn).await;
                    resources.push(res);
                }

                // 更新内存缓存
                *BUSINESS_RESOURCES.write().unwrap() = resources.clone();

                return Json(resources).into_response();
            }
            Err(_) => {
                // 数据库查询失败，回退到内存缓存
            }
        }
    }

    // 回退到内存缓存，也过滤已交付的资源
    let resources = BUSINESS_RESOURCES.read().unwrap();
    let filtered: Vec<_> = resources.iter()
        .filter(|r| r.delivery_status.as_ref().map(|s| s != "已交付").unwrap_or(true))
        .cloned()
        .collect();
    Json(filtered).into_response()
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

    // Convert CreatePhysicalMachineInfo to PhysicalMachineInfo
    let physical_machine_info = req.physical_machine_info.as_ref().map(|info| {
        shared::PhysicalMachineInfo {
            id: None,
            business_resource_id: Some(db_id),
            serial_number: info.serial_number.clone(),
            rack_location: info.rack_location.clone(),
            hardware_model: info.hardware_model.clone(),
            warranty_expiry: info.warranty_expiry,
            agent_status: info.agent_status.clone(),
            ipmi_address: info.ipmi_address.clone(),
        }
    });

    // Convert CreateCloudVirtualMachineInfo to CloudVirtualMachineInfo
    let cloud_vm_info = req.cloud_vm_info.as_ref().map(|info| {
        shared::CloudVirtualMachineInfo {
            id: None,
            business_resource_id: Some(db_id),
            billing_mode: info.billing_mode.clone(),
            expire_time: info.expire_time,
            charge_type: info.charge_type.clone(),
            instance_charge_type: info.instance_charge_type.clone(),
            internet_charge_type: info.internet_charge_type.clone(),
            internet_max_bandwidth_out: info.internet_max_bandwidth_out,
            image_id: info.image_id.clone(),
            v_switch_id: info.v_switch_id.clone(),
            vpc_id: info.vpc_id.clone(),
            security_group_ids: info.security_group_ids.clone(),
        }
    });

    // 使用数据库生成的ID创建资源对象
    let new_resource = BusinessResource {
        id: Some(db_id),
        resource_type: req.resource_type.clone(),
        ecs_name: req.ecs_name.clone(),
        ecs_status: req.ecs_status.clone(),
        resource_id: req.resource_id.clone().unwrap_or_else(|| "".to_string()),
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
        instance_id: req.instance_id.clone().unwrap_or_else(|| "".to_string()),
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
        physical_machine_info,
        cloud_vm_info,
        remarks: req.remarks.clone(),
        created_at: Some(now),
        updated_at: Some(now),
        created_by: Some(user.username.clone()),
        updated_by: Some(user.username.clone()),
        // 申请流程相关
        applicant: req.applicant.clone(),
        department: req.department.clone(),
        approver: req.approver.clone(),
        approval_time: req.approval_time,
        approval_remarks: req.approval_remarks.clone(),
        rejection_reason: req.rejection_reason.clone(),
        // 资源配置相关
        bandwidth_mbps: req.bandwidth_mbps,
        bandwidth_type: req.bandwidth_type.clone(),
        public_ip_count: req.public_ip_count,
        network_type: req.network_type.clone(),
        // 业务关联相关
        project_name: req.project_name.clone(),
        project_code: req.project_code.clone(),
        business_owner: req.business_owner.clone(),
        tech_owner: req.tech_owner.clone(),
        contact_phone: req.contact_phone.clone(),
        // 费用相关
        billing_method: req.billing_method.clone(),
        purchase_duration: req.purchase_duration,
        cost_center: req.cost_center.clone(),
        // 合规相关
        security_level: req.security_level.clone(),
        data_sensitivity: req.data_sensitivity.clone(),
        // 其他
        purpose: req.purpose.clone(),
        expected_delivery_time: req.expected_delivery_time,
        // 状态管理
        application_status: req.application_status.clone(),
        delivery_status: req.delivery_status.clone(),
        delivery_confirmed_at: req.delivery_confirmed_at,
        delivery_confirmed_by: req.delivery_confirmed_by.clone(),
    };

    // 添加到内存缓存
    {
        let mut resources = BUSINESS_RESOURCES.write().unwrap();
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
        let resources = BUSINESS_RESOURCES.read().unwrap();
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
        let mut resources = BUSINESS_RESOURCES.write().unwrap();
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
            // 云平台资源ID和实例ID - 由运维或自动化编排分配
            if let Some(v) = req.resource_id { resource.resource_id = v; }
            if let Some(v) = req.instance_id { resource.instance_id = v; }
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

            // 更新详情信息
            if let Some(pm_info) = &req.physical_machine_info {
                if resource.physical_machine_info.is_none() {
                    resource.physical_machine_info = Some(shared::PhysicalMachineInfo {
                        id: None,
                        business_resource_id: Some(id),
                        serial_number: pm_info.serial_number.clone(),
                        rack_location: pm_info.rack_location.clone(),
                        hardware_model: pm_info.hardware_model.clone(),
                        warranty_expiry: pm_info.warranty_expiry,
                        agent_status: pm_info.agent_status.clone(),
                        ipmi_address: pm_info.ipmi_address.clone(),
                    });
                } else if let Some(ref mut existing) = resource.physical_machine_info {
                    if let Some(v) = &pm_info.serial_number { existing.serial_number = Some(v.clone()); }
                    if let Some(v) = &pm_info.rack_location { existing.rack_location = Some(v.clone()); }
                    if let Some(v) = &pm_info.hardware_model { existing.hardware_model = Some(v.clone()); }
                    if let Some(v) = pm_info.warranty_expiry { existing.warranty_expiry = Some(v); }
                    if let Some(v) = &pm_info.agent_status { existing.agent_status = Some(v.clone()); }
                    if let Some(v) = &pm_info.ipmi_address { existing.ipmi_address = Some(v.clone()); }
                }
            }

            if let Some(cvm_info) = &req.cloud_vm_info {
                if resource.cloud_vm_info.is_none() {
                    resource.cloud_vm_info = Some(shared::CloudVirtualMachineInfo {
                        id: None,
                        business_resource_id: Some(id),
                        billing_mode: cvm_info.billing_mode.clone(),
                        expire_time: cvm_info.expire_time,
                        charge_type: cvm_info.charge_type.clone(),
                        instance_charge_type: cvm_info.instance_charge_type.clone(),
                        internet_charge_type: cvm_info.internet_charge_type.clone(),
                        internet_max_bandwidth_out: cvm_info.internet_max_bandwidth_out,
                        image_id: cvm_info.image_id.clone(),
                        v_switch_id: cvm_info.v_switch_id.clone(),
                        vpc_id: cvm_info.vpc_id.clone(),
                        security_group_ids: cvm_info.security_group_ids.clone(),
                    });
                } else if let Some(ref mut existing) = resource.cloud_vm_info {
                    if let Some(v) = &cvm_info.billing_mode { existing.billing_mode = Some(v.clone()); }
                    if let Some(v) = cvm_info.expire_time { existing.expire_time = Some(v); }
                    if let Some(v) = &cvm_info.charge_type { existing.charge_type = Some(v.clone()); }
                    if let Some(v) = &cvm_info.instance_charge_type { existing.instance_charge_type = Some(v.clone()); }
                    if let Some(v) = &cvm_info.internet_charge_type { existing.internet_charge_type = Some(v.clone()); }
                    if let Some(v) = cvm_info.internet_max_bandwidth_out { existing.internet_max_bandwidth_out = Some(v); }
                    if let Some(v) = &cvm_info.image_id { existing.image_id = Some(v.clone()); }
                    if let Some(v) = &cvm_info.v_switch_id { existing.v_switch_id = Some(v.clone()); }
                    if let Some(v) = &cvm_info.vpc_id { existing.vpc_id = Some(v.clone()); }
                    if let Some(v) = &cvm_info.security_group_ids { existing.security_group_ids = Some(v.clone()); }
                }
            }

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

        // 同时更新详情表
        if let Some(conn) = crate::database::get_db() {
            if let Some(pm_info) = &req_clone.physical_machine_info {
                let _ = crate::database::update_physical_machine(&conn, id, pm_info, &now.to_rfc3339()).await;
            }
            if let Some(cvm_info) = &req_clone.cloud_vm_info {
                let _ = crate::database::update_cloud_virtual_machine(&conn, id, cvm_info, &now.to_rfc3339()).await;
            }
        }

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
        let resources = BUSINESS_RESOURCES.read().unwrap();
        resources.iter().position(|r| r.id == Some(id))
    };

    if let Some(pos) = found {
        // 从内存中删除
        {
            let mut resources = BUSINESS_RESOURCES.write().unwrap();
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
