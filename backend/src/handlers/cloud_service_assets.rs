//! Cloud Service Assets API handlers
//!
//! Provides REST API endpoints for unified cloud service assets (physical + virtual machines).

use axum::{
    extract::{Query, State},
    response::{IntoResponse, Json},
};

use crate::state::AppState;
use crate::database::get_business_resources;
use crate::handlers::cloud_providers::PROVIDER_CONFIGS;
use shared::{CloudServiceAsset, CloudServiceAssetQuery, CloudServiceAssetStats};

/// Get all cloud service assets (unified view of physical and virtual machines)
pub async fn get_cloud_service_assets(
    State(_state): State<AppState>,
    Query(query): Query<CloudServiceAssetQuery>,
) -> impl IntoResponse {
    // Aggregate from business resources (physical machines) from database
    let business_resources = match get_business_resources().await {
        Ok(resources) => resources,
        Err(e) => {
            eprintln!("Error loading business resources: {}", e);
            return Json(vec![]);
        }
    };

    // Load cloud provider configs for provider name lookup
    let provider_configs = PROVIDER_CONFIGS.lock().unwrap().clone();

    let mut assets: Vec<CloudServiceAsset> = business_resources.into_iter().map(|br| {
        CloudServiceAsset {
            // 基础标识
            id: format!("{}-{}", br.resource_type, br.id),
            asset_type: if br.resource_type == "cloud" { "virtual".to_string() } else { br.resource_type.clone() },
            source_type: "business_resource".to_string(),

            // 基本信息
            name: br.ecs_name,
            instance_id: br.instance_id,
            status: br.ecs_status,
            cloud_platform: br.cloud_category,
            cloud_zone: br.platform_name.clone().unwrap_or_else(|| "-".to_string()),
            supplier_name: br.zone_name,

            // 实例配置
            instance_type: br.ecs_type,
            cpu_cores: br.cpu_cores as u32,
            memory_gb: br.memory_gb as u32,
            system_disk_type: br.system_disk,
            system_disk_size_gb: br.system_disk_size_gb as u32,
            data_disk_info: br.data_disk,

            // 操作系统
            os_type: "Linux".to_string(), // Default, could be parsed from ecs_os
            os_name: br.ecs_os,

            // 网络信息
            ip_address: br.ip_address,
            public_ip: None,
            ipv6_address: None,

            // 业务信息
            customer_name: br.customer_name,
            department: None,
            project: br.application_name.clone(),
            application_name: br.application_name,
            contract_name: br.contract_name,
            owner_name: None, // Not directly available in business_resource

            // 访问信息
            login_method: br.ecs_login_method,
            login_username: br.ecs_login_username,
            bastion_address: br.bastion_address,
            bastion_account: br.bastion_admin_account,
            bastion_initial_password: br.bastion_initial_password,

            // 物理机特有信息
            serial_number: br.serial_number,
            rack_location: br.rack_location,
            hardware_model: br.hardware_model,
            warranty_expiry: br.warranty_expiry,
            agent_status: br.agent_status,
            ipmi_address: br.ipmi_address,

            // 云虚拟机特有信息
            billing_mode: None,
            expire_time: br.release_time,
            charge_type: None,

            // 时间信息
            created_at: br.created_at,
            updated_at: br.updated_at,
            last_synced: None,

            // 其他
            tags: None,
            remarks: br.remarks,

            // 关联ID
            business_resource_id: Some(br.id),
            cloud_asset_id: None,
        }
    }).collect();

    // Apply filters
    if let Some(asset_type) = &query.asset_type {
        assets.retain(|a| &a.asset_type == asset_type);
    }
    if let Some(source_type) = &query.source_type {
        assets.retain(|a| &a.source_type == source_type);
    }
    if let Some(cloud_platform) = &query.cloud_platform {
        assets.retain(|a| &a.cloud_platform == cloud_platform);
    }
    if let Some(status) = &query.status {
        assets.retain(|a| &a.status == status);
    }
    if let Some(customer_name) = &query.customer_name {
        assets.retain(|a| a.customer_name.contains(customer_name));
    }
    if let Some(department) = &query.department {
        assets.retain(|a| a.department.as_ref().map_or(false, |d| d.contains(department)));
    }
    if let Some(project) = &query.project {
        assets.retain(|a| a.project.as_ref().map_or(false, |p| p.contains(project)));
    }
    if let Some(keyword) = &query.search_keyword {
        assets.retain(|a| {
            a.name.contains(keyword)
                || a.ip_address.contains(keyword)
                || a.instance_id.contains(keyword)
        });
    }

    Json(assets)
}

/// Get cloud service asset statistics
pub async fn get_cloud_service_stats(
    State(_state): State<AppState>,
) -> impl IntoResponse {
    // Load from database
    let business_resources = match get_business_resources().await {
        Ok(resources) => resources,
        Err(e) => {
            eprintln!("Error loading business resources: {}", e);
            return Json(CloudServiceAssetStats {
                total_count: 0,
                physical_count: 0,
                virtual_count: 0,
                running_count: 0,
                stopped_count: 0,
                total_cpu_cores: 0,
                total_memory_gb: 0,
                expiring_soon_count: 0,
                by_provider: vec![],
                by_customer: vec![],
                by_status: vec![],
            });
        }
    };

    // Load cloud provider configs for provider name lookup
    let provider_configs = PROVIDER_CONFIGS.lock().unwrap().clone();

    let total = business_resources.len() as u32;
    let running = business_resources.iter().filter(|r| r.ecs_status == "运行中").count() as u32;
    let stopped = business_resources.iter().filter(|r| r.ecs_status == "已释放").count() as u32;
    let total_cpu: u32 = business_resources.iter().map(|r| r.cpu_cores as u32).sum();
    let total_memory: u32 = business_resources.iter().map(|r| r.memory_gb as u32).sum();

    // Group by provider
    let mut by_provider = std::collections::HashMap::new();
    for r in &business_resources {
        let provider = if let Some(config_id) = r.cloud_provider_config_id {
            provider_configs.iter()
                .find(|c| c.id == Some(config_id))
                .map(|c| c.provider.as_str().to_string())
                .unwrap_or_else(|| "Unknown".to_string())
        } else {
            "Unknown".to_string()
        };
        *by_provider.entry(provider).or_insert(0) += 1;
    }

    // Group by customer
    let mut by_customer = std::collections::HashMap::new();
    for r in &business_resources {
        let customer = r.customer_name.clone();
        *by_customer.entry(customer).or_insert(0) += 1;
    }

    // Group by status
    let mut by_status = std::collections::HashMap::new();
    for r in &business_resources {
        let status = r.ecs_status.clone();
        *by_status.entry(status).or_insert(0) += 1;
    }

    Json(CloudServiceAssetStats {
        total_count: total,
        physical_count: total,
        virtual_count: 0, // Would come from cloud assets
        running_count: running,
        stopped_count: stopped,
        total_cpu_cores: total_cpu,
        total_memory_gb: total_memory,
        expiring_soon_count: 0, // TODO: calculate from warranty_expiry
        by_provider: by_provider.into_iter().collect(),
        by_customer: by_customer.into_iter().collect(),
        by_status: by_status.into_iter().collect(),
    })
}
