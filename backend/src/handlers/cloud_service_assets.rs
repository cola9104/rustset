//! Cloud Service Asset / 云服务资产管理 API handlers
//!
//! 统一纳管物理机和云虚拟机的资产

use axum::{
    extract::{Query, State},
    response::Json,
};
use chrono::Utc;

use crate::state::AppState;
use shared::{
    CloudServiceAsset, CloudServiceAssetQuery, CloudServiceAssetStats,
    BusinessResource, CloudAsset,
};

/// 获取云服务资产列表（统一管理物理机和云虚拟机）
pub async fn get_cloud_service_assets(
    State(state): State<AppState>,
    Query(query): Query<CloudServiceAssetQuery>,
) -> Json<Vec<CloudServiceAsset>> {
    let business_resources = state.business_resources.lock().unwrap().clone();
    let cloud_assets = state.cloud_assets.lock().unwrap().clone();

    let mut assets = Vec::new();

    // 处理业务资源中的物理机和云虚拟机
    for br in business_resources {
        // 只处理已完成的资源（非待审批状态）
        if br.ecs_status == "待审批" || br.ecs_status == "审批中" {
            continue;
        }

        let asset_type = if br.resource_type == "physical" { "physical" } else { "virtual" };

        // 解析系统盘类型和大小
        let (system_disk_type, system_disk_size_gb) =
            if br.system_disk.contains('(') {
                let parts: Vec<&str> = br.system_disk.split('(').collect();
                let disk_type = parts.first().unwrap_or(&"").to_string();
                let size_str = parts.get(1).and_then(|s| s.strip_suffix(')')).unwrap_or("0");
                let size: u32 = size_str.parse().unwrap_or(0);
                (disk_type, size)
            } else {
                (br.system_disk.clone(), br.system_disk_size_gb)
            };

        // 计算到期时间
        let expire_time = br.completion_time
            .and_then(|t| t.checked_add_signed(chrono::Duration::days(365)))
            .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string());

        let asset = CloudServiceAsset {
            id: format!("{}-{}", asset_type, br.id.unwrap_or(0)),
            asset_type: asset_type.to_string(),
            source_type: "business_resource".to_string(),
            name: br.ecs_name.clone(),
            instance_id: br.instance_id.clone(),
            status: br.ecs_status.clone(),
            cloud_provider: br.cloud_category.clone(),
            region: br.cloud_region.clone(),
            instance_type: br.ecs_type.clone(),
            cpu_cores: br.cpu_cores,
            memory_gb: br.memory_gb,
            system_disk_type: system_disk_type.clone(),
            system_disk_size_gb,
            data_disk_info: br.data_disk.clone(),
            os_type: if br.ecs_os.to_lowercase().contains("windows") { "Windows" } else { "Linux" }.to_string(),
            os_name: br.ecs_os.clone(),
            ip_address: br.ip_address.clone(),
            public_ip: None,
            ipv6_address: None,
            customer_name: br.customer_name.clone(),
            department: br.county_city.clone(),
            project: br.vdc_name.clone(),
            application_name: br.application_name.clone(),
            contract_name: br.contract_name.clone(),
            owner_name: None,
            login_method: br.ecs_login_method.clone(),
            login_username: br.ecs_login_username.clone(),
            bastion_address: br.bastion_address.clone(),
            bastion_account: br.bastion_admin_account.clone(),
            serial_number: br.serial_number.clone(),
            rack_location: br.rack_location.clone(),
            hardware_model: br.hardware_model.clone(),
            warranty_expiry: br.warranty_expiry.map(|d| d.format("%Y-%m-%d").to_string()),
            agent_status: br.agent_status.clone(),
            ipmi_address: br.ipmi_address.clone(),
            billing_mode: None,
            expire_time,
            charge_type: None,
            created_at: br.created_at.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string()).unwrap_or_default(),
            updated_at: br.updated_at.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string()),
            last_synced: None,
            tags: None,
            remarks: br.remarks.clone(),
            business_resource_id: br.id,
            cloud_asset_id: None,
        };
        assets.push(asset);
    }

    // 处理混合云同步的云虚拟机资产
    for ca in cloud_assets {
        let asset = CloudServiceAsset {
            id: format!("virtual-{}", ca.id.unwrap_or(0)),
            asset_type: "virtual".to_string(),
            source_type: "cloud_asset".to_string(),
            name: ca.asset_name.clone(),
            instance_id: ca.instance_id.clone(),
            status: format!("{:?}", ca.status),
            cloud_provider: format!("{:?}", ca.spec.cloud_provider),
            region: format!("{:?}", ca.cloud_region.region),
            instance_type: ca.spec.instance_type.instance_type,
            cpu_cores: ca.spec.cpu_cores,
            memory_gb: ca.spec.memory_gb,
            system_disk_type: ca.system_disk.disk_type.to_string(),
            system_disk_size_gb: ca.system_disk.size_gb,
            data_disk_info: if ca.cloud_disk_count > 0 {
                Some(format!("{} disks, {} GB total", ca.cloud_disk_count, ca.cloud_disk_total_size_gb))
            } else {
                None
            },
            os_type: ca.os_type,
            os_name: ca.os_name,
            ip_address: ca.private_ip.clone(),
            public_ip: ca.public_ip.clone(),
            ipv6_address: ca.ipv6_address.clone(),
            customer_name: ca.owner.owner_name.clone(),
            department: Some(ca.department.department_name.clone()),
            project: Some(ca.project.project_name.clone()),
            application_name: None,
            contract_name: None,
            owner_name: Some(ca.owner.owner_name.clone()),
            login_method: None,
            login_username: None,
            bastion_address: None,
            bastion_account: None,
            serial_number: None,
            rack_location: None,
            hardware_model: None,
            warranty_expiry: None,
            agent_status: None,
            ipmi_address: None,
            billing_mode: format!("{:?}", ca.billing_mode),
            expire_time: ca.expire_time.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string()),
            charge_type: ca.charge_type,
            created_at: ca.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: None,
            last_synced: ca.last_synced.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string()),
            tags: if ca.tags.is_empty() { None } else { Some(ca.tags.join(",")) },
            remarks: None,
            business_resource_id: None,
            cloud_asset_id: ca.id,
        };
        assets.push(asset);
    }

    // 应用过滤
    let filtered: Vec<CloudServiceAsset> = assets
        .into_iter()
        .filter(|a| {
            if let Some(asset_type) = &query.asset_type {
                &a.asset_type == asset_type
            } else {
                true
            }
        })
        .filter(|a| {
            if let Some(source_type) = &query.source_type {
                &a.source_type == source_type
            } else {
                true
            }
        })
        .filter(|a| {
            if let Some(provider) = &query.cloud_provider {
                &a.cloud_provider == provider
            } else {
                true
            }
        })
        .filter(|a| {
            if let Some(status) = &query.status {
                &a.status == status
            } else {
                true
            }
        })
        .filter(|a| {
            if let Some(customer) = &query.customer_name {
                &a.customer_name == customer
            } else {
                true
            }
        })
        .filter(|a| {
            if let Some(dept) = &query.department {
                a.department.as_ref().map_or(false, |d| d == dept)
            } else {
                true
            }
        })
        .filter(|a| {
            if let Some(project) = &query.project {
                a.project.as_ref().map_or(false, |p| p == project)
            } else {
                true
            }
        })
        .filter(|a| {
            if let Some(keyword) = &query.search_keyword {
                let keyword_lower = keyword.to_lowercase();
                a.name.to_lowercase().contains(&keyword_lower)
                    || a.instance_id.to_lowercase().contains(&keyword_lower)
                    || a.ip_address.contains(keyword)
                    || a.customer_name.to_lowercase().contains(&keyword_lower)
            } else {
                true
            }
        })
        .collect();

    Json(filtered)
}

/// 获取云服务资产统计
pub async fn get_cloud_service_asset_stats(
    State(state): State<AppState>,
) -> Json<CloudServiceAssetStats> {
    let assets = get_cloud_service_assets(State(state), Query(CloudServiceAssetQuery {
        asset_type: None,
        source_type: None,
        cloud_provider: None,
        status: None,
        customer_name: None,
        department: None,
        project: None,
        search_keyword: None,
    })).await;

    let total_count = assets.len() as u32;
    let physical_count = assets.iter().filter(|a| a.asset_type == "physical").count() as u32;
    let virtual_count = assets.iter().filter(|a| a.asset_type == "virtual").count() as u32;
    let running_count = assets.iter().filter(|a| a.status == "运行中").count() as u32;
    let stopped_count = assets.iter().filter(|a| a.status == "已停止").count() as u32;
    let total_cpu_cores = assets.iter().map(|a| a.cpu_cores).sum();
    let total_memory_gb = assets.iter().map(|a| a.memory_gb).sum();

    // 计算即将到期（30天内）
    let now = Utc::now();
    let expiring_soon_count = assets.iter()
        .filter(|a| {
            a.expire_time.as_ref().map_or(false, |e| {
                if let Ok(expire_date) = chrono::DateTime::parse_from_rfc3339(e) {
                    let days_until = (expire_date - now).num_days();
                    days_until >= 0 && days_until <= 30
                } else {
                    false
                }
            })
        })
        .count() as u32;

    // 按厂商统计
    let mut by_provider_map = std::collections::HashMap::new();
    for a in &assets {
        *by_provider_map.entry(a.cloud_provider.clone()).or_insert(0) += 1;
    }
    let mut by_provider: Vec<(String, u32)> = by_provider_map.into_iter().collect();
    by_provider.sort_by(|a, b| b.1.cmp(&a.1));

    // 按客户统计
    let mut by_customer_map = std::collections::HashMap::new();
    for a in &assets {
        *by_customer_map.entry(a.customer_name.clone()).or_insert(0) += 1;
    }
    let mut by_customer: Vec<(String, u32)> = by_customer_map.into_iter().collect();
    by_customer.sort_by(|a, b| b.1.cmp(&a.1));

    // 按状态统计
    let mut by_status_map = std::collections::HashMap::new();
    for a in &assets {
        *by_status_map.entry(a.status.clone()).or_insert(0) += 1;
    }
    let mut by_status: Vec<(String, u32)> = by_status_map.into_iter().collect();
    by_status.sort_by(|a, b| b.1.cmp(&a.1));

    Json(CloudServiceAssetStats {
        total_count,
        physical_count,
        virtual_count,
        running_count,
        stopped_count,
        total_cpu_cores,
        total_memory_gb,
        expiring_soon_count,
        by_provider,
        by_customer,
        by_status,
    })
}
