//! Business Resource / 业务受理 API handlers
//!
//! 云资源管理业务受理单的 CRUD 操作

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::Deserialize;
use uuid::Uuid;
use chrono::Utc;

use crate::state::AppState;
use shared::{
    BusinessResource, CreateBusinessResourceRequest, UpdateBusinessResourceRequest,
    BusinessResourceQuery, BusinessResourceStats,
};

/// 审批业务资源（仅审批，不自动创建云资源）
///
/// 正确的流程是：
/// 1. 业务申请 → 2. 审批通过 → 3. 运维人员在云厂商平台创建资源 → 4. 录入云服务资源 → 5. 混合云管理对接
pub async fn approve_business_resource(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, StatusCode> {
    // 获取业务资源
    let mut resources = state.business_resources.lock().unwrap();
    let resource = resources.iter_mut().find(|r| r.id == Some(id))
        .ok_or(StatusCode::NOT_FOUND)?;

    // 检查状态，只能审批待审批或审批中的
    if resource.ecs_status != "待审批" && resource.ecs_status != "审批中" {
        return Err(StatusCode::BAD_REQUEST);
    }

    // 更新状态为待交付（等待运维人员在云厂商平台创建资源后录入）
    resource.ecs_status = "待交付".to_string();
    resource.updated_at = Some(Utc::now());

    // 记录审计日志
    let mut logs = state.audit_logs.lock().unwrap();
    logs.push(shared::AuditLog {
        id: Uuid::new_v4().to_string(),
        user_id: "system".to_string(),
        username: "system".to_string(),
        action: "APPROVE_BUSINESS_RESOURCE".to_string(),
        target: format!("BusinessResource: {}", id),
        details: format!(
            "业务申请已审批通过，待运维交付。资源名称: {}, 云厂商: {}, 区域: {}",
            resource.ecs_name, resource.cloud_category, resource.cloud_region
        ),
        timestamp: Utc::now(),
    });

    Ok(Json(serde_json::json!({
        "business_resource": resource.clone(),
        "message": "审批通过，请前往云厂商平台创建资源后录入系统"
    })))
}

/// 获取业务资源列表
pub async fn get_business_resources(
    State(state): State<AppState>,
    Query(query): Query<BusinessResourceQuery>,
) -> impl IntoResponse {
    let resources = state.business_resources.lock().unwrap().clone();

    // 简单过滤实现
    let filtered: Vec<BusinessResource> = resources
        .into_iter()
        .filter(|r| {
            if let Some(keyword) = &query.search_keyword {
                let keyword_lower = keyword.to_lowercase();
                r.ecs_name.to_lowercase().contains(&keyword_lower)
                    || r.customer_name.to_lowercase().contains(&keyword_lower)
                    || r.ip_address.contains(keyword)
                    || r.resource_id.to_lowercase().contains(&keyword_lower)
                    || r.serial_number.as_ref().map_or(false, |s| s.to_lowercase().contains(&keyword_lower))
            } else {
                true
            }
        })
        .filter(|r| {
            query.resource_type.as_ref().map_or(true, |rt| rt == &r.resource_type)
        })
        .filter(|r| {
            query.cloud_category.as_ref().map_or(true, |cat| cat == &r.cloud_category)
        })
        .filter(|r| {
            query.ecs_status.as_ref().map_or(true, |status| status == &r.ecs_status)
        })
        .filter(|r| {
            query.customer_name.as_ref().map_or(true, |cust| cust == &r.customer_name)
        })
        .filter(|r| {
            query.county_city.as_ref().map_or(true, |city| r.county_city.as_ref().map_or(false, |c| c == city))
        })
        .filter(|r| {
            query.application_name.as_ref().map_or(true, |app| r.application_name.as_ref().map_or(false, |a| a == app))
        })
        .filter(|r| {
            query.contract_name.as_ref().map_or(true, |contract| r.contract_name.as_ref().map_or(false, |c| c == contract))
        })
        .collect();

    Json(filtered)
}

/// 获取业务资源详情
pub async fn get_business_resource(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, StatusCode> {
    let resources = state.business_resources.lock().unwrap();
    let resource = resources.iter().find(|r| r.id == Some(id))
        .cloned()
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(resource))
}

/// 创建业务资源
pub async fn create_business_resource(
    State(state): State<AppState>,
    req: Json<CreateBusinessResourceRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let new_id = {
        let resources = state.business_resources.lock().unwrap();
        let max_id = resources.iter().filter_map(|r| r.id).max().unwrap_or(0);
        max_id + 1
    };

    let now = Utc::now();
    let resource = BusinessResource {
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
        // 物理机特有字段
        serial_number: req.serial_number.clone(),
        rack_location: req.rack_location.clone(),
        hardware_model: req.hardware_model.clone(),
        warranty_expiry: req.warranty_expiry,
        agent_status: None,  // 初始状态为None
        ipmi_address: req.ipmi_address.clone(),
        remarks: req.remarks.clone(),
        created_at: Some(now),
        updated_at: Some(now),
        created_by: Some("system".to_string()),
        updated_by: Some("system".to_string()),
    };

    state.business_resources.lock().unwrap().push(resource.clone());

    // 记录审计日志
    let mut logs = state.audit_logs.lock().unwrap();
    logs.push(shared::AuditLog {
        id: Uuid::new_v4().to_string(),
        user_id: "system".to_string(),
        username: "system".to_string(),
        action: "CREATE_BUSINESS_RESOURCE".to_string(),
        target: format!("BusinessResource: {}", new_id),
        details: format!("Created ECS resource: {} for customer: {}", req.ecs_name, req.customer_name),
        timestamp: now,
    });

    Ok(Json(resource))
}

/// 更新业务资源
pub async fn update_business_resource(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    req: Json<UpdateBusinessResourceRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut resources = state.business_resources.lock().unwrap();
    let resource = resources.iter_mut().find(|r| r.id == Some(id))
        .ok_or(StatusCode::NOT_FOUND)?;

    // 更新字段
    if let Some(resource_type) = &req.resource_type { resource.resource_type = resource_type.clone(); }
    if let Some(name) = &req.ecs_name { resource.ecs_name = name.clone(); }
    if let Some(status) = &req.ecs_status { resource.ecs_status = status.clone(); }
    if let Some(region) = &req.cloud_region { resource.cloud_region = region.clone(); }
    if let Some(city) = &req.county_city { resource.county_city = Some(city.clone()); }
    if let Some(vdc) = &req.vdc_name { resource.vdc_name = Some(vdc.clone()); }
    if let Some(customer) = &req.customer_name { resource.customer_name = customer.clone(); }
    if let Some(app) = &req.application_name { resource.application_name = Some(app.clone()); }
    if let Some(contract) = &req.contract_name { resource.contract_name = Some(contract.clone()); }
    if let Some(ecs_type) = &req.ecs_type { resource.ecs_type = ecs_type.clone(); }
    if let Some(os) = &req.ecs_os { resource.ecs_os = os.clone(); }
    if let Some(cpu) = req.cpu_cores { resource.cpu_cores = cpu; }
    if let Some(memory) = req.memory_gb { resource.memory_gb = memory; }
    if let Some(disk) = &req.system_disk { resource.system_disk = disk.clone(); }
    if let Some(disk_size) = &req.system_disk_size_gb { resource.system_disk_size_gb = *disk_size; }
    if let Some(data_disk) = &req.data_disk { resource.data_disk = Some(data_disk.clone()); }
    if let Some(completion) = req.completion_time { resource.completion_time = Some(completion); }
    if let Some(release) = req.release_time { resource.release_time = Some(release); }
    if let Some(security) = req.has_security_product { resource.has_security_product = security; }
    if let Some(ip) = &req.ip_address { resource.ip_address = ip.clone(); }
    if let Some(method) = &req.ecs_login_method { resource.ecs_login_method = Some(method.clone()); }
    if let Some(username) = &req.ecs_login_username { resource.ecs_login_username = Some(username.clone()); }
    if let Some(password) = &req.ecs_initial_password { resource.ecs_initial_password = Some(password.clone()); }
    if let Some(bastion_addr) = &req.bastion_address { resource.bastion_address = Some(bastion_addr.clone()); }
    if let Some(bastion_admin) = &req.bastion_admin_account { resource.bastion_admin_account = Some(bastion_admin.clone()); }
    if let Some(bastion_pwd) = &req.bastion_initial_password { resource.bastion_initial_password = Some(bastion_pwd.clone()); }
    // 物理机特有字段
    if let Some(serial_number) = &req.serial_number { resource.serial_number = Some(serial_number.clone()); }
    if let Some(rack_location) = &req.rack_location { resource.rack_location = Some(rack_location.clone()); }
    if let Some(hardware_model) = &req.hardware_model { resource.hardware_model = Some(hardware_model.clone()); }
    if let Some(warranty_expiry) = req.warranty_expiry { resource.warranty_expiry = Some(warranty_expiry); }
    if let Some(agent_status) = &req.agent_status { resource.agent_status = Some(agent_status.clone()); }
    if let Some(ipmi_address) = &req.ipmi_address { resource.ipmi_address = Some(ipmi_address.clone()); }
    if let Some(remarks) = &req.remarks { resource.remarks = Some(remarks.clone()); }

    resource.updated_at = Some(Utc::now());

    Ok(Json(resource.clone()))
}

/// 删除业务资源
pub async fn delete_business_resource(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut resources = state.business_resources.lock().unwrap();
    let idx = resources.iter().position(|r| r.id == Some(id))
        .ok_or(StatusCode::NOT_FOUND)?;
    resources.remove(idx);
    Ok(StatusCode::NO_CONTENT)
}

/// 获取业务资源统计
pub async fn get_business_resource_stats(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let resources = state.business_resources.lock().unwrap().clone();

    let total_count = resources.len() as u32;
    let running_count = resources.iter().filter(|r| r.ecs_status == "运行中").count() as u32;
    let stopped_count = resources.iter().filter(|r| r.ecs_status == "已停止").count() as u32;
    let released_count = resources.iter().filter(|r| r.ecs_status == "已释放").count() as u32;
    let total_cpu_cores = resources.iter().map(|r| r.cpu_cores).sum();
    let total_memory_gb = resources.iter().map(|r| r.memory_gb).sum();
    let with_security_product_count = resources.iter().filter(|r| r.has_security_product).count() as u32;
    let cloud_count = resources.iter().filter(|r| r.resource_type == "cloud").count() as u32;
    let physical_count = resources.iter().filter(|r| r.resource_type == "physical").count() as u32;

    // 按云类别统计
    let mut by_category_map = std::collections::HashMap::new();
    for r in &resources {
        *by_category_map.entry(r.cloud_category.clone()).or_insert(0) += 1;
    }
    let mut by_category: Vec<(String, u32)> = by_category_map.into_iter().collect();
    by_category.sort_by(|a, b| a.0.cmp(&b.0));

    // 按客户统计
    let mut by_customer_map = std::collections::HashMap::new();
    for r in &resources {
        *by_customer_map.entry(r.customer_name.clone()).or_insert(0) += 1;
    }
    let mut by_customer: Vec<(String, u32)> = by_customer_map.into_iter().collect();
    by_customer.sort_by(|a, b| b.1.cmp(&a.1)); // 按数量降序

    // 按县市区统计
    let mut by_county_map = std::collections::HashMap::new();
    for r in &resources {
        if let Some(city) = &r.county_city {
            *by_county_map.entry(city.clone()).or_insert(0) += 1;
        }
    }
    let mut by_county: Vec<(String, u32)> = by_county_map.into_iter().collect();
    by_county.sort_by(|a, b| b.1.cmp(&a.1));

    // 按应用统计
    let mut by_app_map = std::collections::HashMap::new();
    for r in &resources {
        if let Some(app) = &r.application_name {
            *by_app_map.entry(app.clone()).or_insert(0) += 1;
        }
    }
    let mut by_application: Vec<(String, u32)> = by_app_map.into_iter().collect();
    by_application.sort_by(|a, b| b.1.cmp(&a.1));

    Json(BusinessResourceStats {
        total_count,
        running_count,
        stopped_count,
        released_count,
        total_cpu_cores,
        total_memory_gb,
        with_security_product_count,
        cloud_count,
        physical_count,
        by_category,
        by_customer,
        by_county,
        by_application,
    })
}

/// 导出业务资源为 CSV
pub async fn export_business_resources(
    State(state): State<AppState>,
    Query(query): Query<BusinessResourceQuery>,
) -> impl IntoResponse {
    let resources = state.business_resources.lock().unwrap().clone();

    // 应用过滤
    let filtered: Vec<BusinessResource> = resources
        .into_iter()
        .filter(|r| {
            if let Some(keyword) = &query.search_keyword {
                r.ecs_name.contains(keyword) || r.customer_name.contains(keyword) || r.ip_address.contains(keyword)
            } else {
                true
            }
        })
        .collect();

    // 生成 CSV
    let mut csv = String::from("ECS名称,ECS状态,资源ID,云区域,云类别,县市区,VDC名称,客户名称,应用名称,合同名称,实例ID,ECS类型,ECS操作系统,CPU核数,内存(GB),系统盘,系统盘大小(GB),数据盘,完成时间,释放时间,是否创建安全产品,IP地址,远程登录方式,登录用户名,初始密码,堡垒机地址,堡垒机账号,堡垒机密码,备注\n");

    for r in filtered {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
            r.ecs_name,
            r.ecs_status,
            r.resource_id,
            r.cloud_region,
            r.cloud_category,
            r.county_city.as_ref().unwrap_or(&String::new()),
            r.vdc_name.as_ref().unwrap_or(&String::new()),
            r.customer_name,
            r.application_name.as_ref().unwrap_or(&String::new()),
            r.contract_name.as_ref().unwrap_or(&String::new()),
            r.instance_id,
            r.ecs_type,
            r.ecs_os,
            r.cpu_cores,
            r.memory_gb,
            r.system_disk,
            r.system_disk_size_gb,
            r.data_disk.as_ref().unwrap_or(&String::new()),
            r.completion_time.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string()).unwrap_or_default(),
            r.release_time.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string()).unwrap_or_default(),
            r.has_security_product,
            r.ip_address,
            r.ecs_login_method.as_ref().unwrap_or(&String::new()),
            r.ecs_login_username.as_ref().unwrap_or(&String::new()),
            r.ecs_initial_password.as_ref().unwrap_or(&String::new()),
            r.bastion_address.as_ref().unwrap_or(&String::new()),
            r.bastion_admin_account.as_ref().unwrap_or(&String::new()),
            r.bastion_initial_password.as_ref().unwrap_or(&String::new()),
            r.remarks.as_ref().unwrap_or(&String::new()),
        ));
    }

    let headers = [("content-type", "text/csv; charset=utf-8")];
    (headers, csv)
}

/// 批量导入业务资源
#[derive(Debug, Deserialize)]
pub struct BatchImportRequest {
    pub resources: Vec<CreateBusinessResourceRequest>,
}

pub async fn batch_import_business_resources(
    State(state): State<AppState>,
    Json(req): Json<BatchImportRequest>,
) -> impl IntoResponse {
    let total = req.resources.len();
    let mut imported = Vec::new();

    for resource_req in req.resources {
        // 直接创建业务资源逻辑，避免调用 handler 函数
        let new_id = {
            let resources = state.business_resources.lock().unwrap();
            let max_id = resources.iter().filter_map(|r| r.id).max().unwrap_or(0);
            max_id + 1
        };

        let now = Utc::now();
        let resource = BusinessResource {
            id: Some(new_id),
            resource_type: resource_req.resource_type.clone(),
            ecs_name: resource_req.ecs_name.clone(),
            ecs_status: resource_req.ecs_status.clone(),
            resource_id: resource_req.resource_id.clone(),
            cloud_region: resource_req.cloud_region.clone(),
            cloud_category: resource_req.cloud_category.clone(),
            cloud_provider_config_id: resource_req.cloud_provider_config_id,
            county_city: resource_req.county_city.clone(),
            vdc_name: resource_req.vdc_name.clone(),
            customer_name: resource_req.customer_name.clone(),
            application_name: resource_req.application_name.clone(),
            contract_name: resource_req.contract_name.clone(),
            instance_id: resource_req.instance_id.clone(),
            ecs_type: resource_req.ecs_type.clone(),
            ecs_os: resource_req.ecs_os.clone(),
            cpu_cores: resource_req.cpu_cores,
            memory_gb: resource_req.memory_gb,
            system_disk: resource_req.system_disk.clone(),
            system_disk_size_gb: resource_req.system_disk_size_gb,
            data_disk: resource_req.data_disk.clone(),
            completion_time: resource_req.completion_time,
            release_time: resource_req.release_time,
            has_security_product: resource_req.has_security_product,
            ip_address: resource_req.ip_address.clone(),
            ecs_login_method: resource_req.ecs_login_method.clone(),
            ecs_login_username: resource_req.ecs_login_username.clone(),
            ecs_initial_password: resource_req.ecs_initial_password.clone(),
            bastion_address: resource_req.bastion_address.clone(),
            bastion_admin_account: resource_req.bastion_admin_account.clone(),
            bastion_initial_password: resource_req.bastion_initial_password.clone(),
            // 物理机特有字段
            serial_number: resource_req.serial_number.clone(),
            rack_location: resource_req.rack_location.clone(),
            hardware_model: resource_req.hardware_model.clone(),
            warranty_expiry: resource_req.warranty_expiry,
            agent_status: None,
            ipmi_address: resource_req.ipmi_address.clone(),
            remarks: resource_req.remarks.clone(),
            created_at: Some(now),
            updated_at: Some(now),
            created_by: Some("system".to_string()),
            updated_by: Some("system".to_string()),
        };

        state.business_resources.lock().unwrap().push(resource.clone());

        // 记录审计日志
        let mut logs = state.audit_logs.lock().unwrap();
        logs.push(shared::AuditLog {
            id: Uuid::new_v4().to_string(),
            user_id: "system".to_string(),
            username: "system".to_string(),
            action: "BATCH_IMPORT_BUSINESS_RESOURCE".to_string(),
            target: format!("BusinessResource: {}", new_id),
            details: format!("Batch imported ECS resource: {} for customer: {}", resource_req.ecs_name, resource_req.customer_name),
            timestamp: now,
        });

        imported.push(resource);
    }

    Json(serde_json::json!({
        "total": total,
        "imported": imported.len(),
        "failed": 0,
        "failed_indices": [],
        "imported_resources": imported,
    }))
}
