//! Cloud Assets / 混合云管理 API handlers
//!
//! 职责说明：
//! - 本模块负责混合云统一管理，通过 API 对接各云厂商平台
//! - 支持手动录入云资源（从运维交付流程录入）
//! - 支持自动同步云厂商资源状态
//!
//! 流程说明：
//! 1. 业务申请 → 2. 审批通过 → 3. 运维人员在云厂商平台创建资源 → 4. 录入本模块 → 5. 自动同步状态

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use shared::{
    CloudAsset, CloudRegion, VMStatus,
    InstanceSpec, SystemDisk, SnapshotInfo,
    Department, Project, ContactPerson, CreateCloudAssetRequest,
    UpdateCloudAssetRequest, CloudAssetQuery, CloudAssetStats,
};
use crate::state::AppState;
use chrono::Utc;
use uuid::Uuid;

/// 获取云资产列表
pub async fn get_cloud_assets(
    State(state): State<AppState>,
    Query(query): Query<CloudAssetQuery>,
) -> impl IntoResponse {
    let assets = state.cloud_assets.lock().unwrap().clone();

    // 过滤
    let filtered: Vec<CloudAsset> = assets
        .into_iter()
        .filter(|a| {
            query.provider.as_ref().map_or(true, |p| &a.cloud_region.provider == p)
        })
        .filter(|a| {
            query.region_id.as_ref().map_or(true, |r| &a.cloud_region.region_id == r)
        })
        .filter(|a| {
            query.status.as_ref().map_or(true, |s| &a.status == s)
        })
        .filter(|a| {
            query.search_keyword.as_ref().map_or(true, |k| {
                a.asset_name.contains(k) || a.instance_id.contains(k)
            })
        })
        .collect();

    Json(filtered)
}

/// 获取单个云资产
pub async fn get_cloud_asset(
    Path(asset_id): Path<i32>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let assets = state.cloud_assets.lock().unwrap();

    if let Some(asset) = assets.iter().find(|a| a.id == Some(asset_id)) {
        Json(asset.clone()).into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Cloud asset not found"}))
        ).into_response()
    }
}

/// 创建云资产（手动录入或从业务资源创建）
///
/// 支持两种场景：
/// 1. 直接录入：运维人员在云厂商平台创建资源后，手动录入系统
/// 2. 从业务资源创建：通过 business_resource_id 关联业务申请
pub async fn create_cloud_asset(
    State(state): State<AppState>,
    Json(asset_req): Json<CreateCloudAssetRequest>,
) -> impl IntoResponse {
    // 如果关联了业务资源，更新业务资源状态为"已交付"
    if let Some(biz_res_id) = asset_req.business_resource_id {
        let mut biz_resources = state.business_resources.lock().unwrap();
        if let Some(biz_res) = biz_resources.iter_mut().find(|r| r.id == Some(biz_res_id)) {
            if biz_res.ecs_status == "待交付" {
                biz_res.ecs_status = "已交付".to_string();
                biz_res.updated_at = Some(Utc::now());
            }
        }
    }

    let mut assets = state.cloud_assets.lock().unwrap();
    let new_id = assets.iter().filter_map(|a| a.id).max().map(|m| m + 1).unwrap_or(1);

    let new_asset = CloudAsset {
        id: Some(new_id),
        asset_name: asset_req.asset_name.clone(),
        spec: InstanceSpec {
            instance_type: asset_req.instance_type.clone(),
            cpu_cores: asset_req.cpu_cores,
            memory_gb: asset_req.memory_gb,
            cpu_arch: None,
            gpu_spec: None,
            bandwidth_mbps: asset_req.bandwidth_mbps,
        },
        system_disk: SystemDisk {
            disk_type: asset_req.system_disk_type.clone(),
            size_gb: asset_req.system_disk_size_gb,
            category: None,
            performance_level: None,
        },
        cloud_region: CloudRegion {
            provider: asset_req.cloud_provider.clone(),
            region_id: asset_req.region_id.clone(),
            zone_id: asset_req.zone_id.clone(),
            region_name: format!("{:?}", asset_req.cloud_provider),
        },
        public_ip: asset_req.public_ip.clone(),
        private_ip: asset_req.private_ip.clone(),
        ipv6_address: asset_req.ipv6_address.clone(),
        billing_mode: asset_req.billing_mode.clone(),
        expire_time: asset_req.expire_time,
        status: VMStatus::Running,
        os_type: asset_req.os_type.clone(),
        os_name: asset_req.os_name.clone(),
        os_arch: None,
        image_id: asset_req.image_id.clone(),
        image_name: asset_req.image_name.clone(),
        department: Department {
            id: asset_req.department_id.clone(),
            name: asset_req.department_name.clone(),
            parent_id: None,
            level: 1,
        },
        project: Project {
            id: asset_req.project_id.clone(),
            name: asset_req.project_name.clone(),
            code: asset_req.project_code.clone(),
            department_id: Some(asset_req.department_id.clone()),
        },
        owner: ContactPerson {
            id: asset_req.owner_id.clone(),
            name: asset_req.owner_name.clone(),
            email: asset_req.owner_email.clone(),
            phone: asset_req.owner_phone.clone(),
            department: Some(asset_req.department_name.clone()),
        },
        created_at: Utc::now(),
        last_synced: Some(Utc::now()),
        cloud_disks: asset_req.cloud_disks.clone(),
        cloud_disk_count: asset_req.cloud_disks.len() as u32,
        cloud_disk_total_size_gb: asset_req.cloud_disks.iter().map(|d| d.size_gb).sum(),
        snapshot_info: SnapshotInfo {
            has_snapshot: asset_req.has_snapshot,
            snapshot_count: asset_req.snapshot_count,
            latest_snapshot_time: None,
            total_snapshot_size_gb: 0,
        },
        instance_id: asset_req.instance_id.clone(),
        tags: asset_req.tags.clone(),
        charge_type: None,
    };

    assets.push(new_asset.clone());

    // 记录审计日志
    let mut logs = state.audit_logs.lock().unwrap();
    logs.push(shared::AuditLog {
        id: Uuid::new_v4().to_string(),
        user_id: "system".to_string(),
        username: "system".to_string(),
        action: "CREATE_CLOUD_ASSET".to_string(),
        target: format!("CloudAsset: {}", new_id),
        details: format!(
            "录入云资产: {} ({:?})",
            new_asset.asset_name, new_asset.cloud_region.provider
        ),
        timestamp: Utc::now(),
    });

    (StatusCode::CREATED, Json(new_asset)).into_response()
}

/// 从业务资源快速创建云资产
///
/// 这个接口用于运维人员在云厂商平台创建资源后，快速录入系统
/// 自动填充业务申请中的信息，减少重复录入
pub async fn create_cloud_asset_from_business(
    State(state): State<AppState>,
    Path(business_resource_id): Path<i32>,
    Json(mut asset_req): Json<CreateCloudAssetRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    // 获取业务资源信息并检查状态
    {
        let biz_resources = state.business_resources.lock().unwrap();
        let biz_res = biz_resources.iter().find(|r| r.id == Some(business_resource_id))
            .ok_or(StatusCode::NOT_FOUND)?;

        // 检查状态
        if biz_res.ecs_status != "待交付" && biz_res.ecs_status != "已交付" {
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    // 自动设置 business_resource_id
    asset_req.business_resource_id = Some(business_resource_id);

    // 调用创建接口（这里直接复制逻辑避免类型问题）
    // 如果关联了业务资源，更新业务资源状态为"已交付"
    {
        let mut biz_resources = state.business_resources.lock().unwrap();
        if let Some(biz_res) = biz_resources.iter_mut().find(|r| r.id == Some(business_resource_id)) {
            if biz_res.ecs_status == "待交付" {
                biz_res.ecs_status = "已交付".to_string();
                biz_res.updated_at = Some(Utc::now());
            }
        }
    }

    let mut assets = state.cloud_assets.lock().unwrap();
    let new_id = assets.iter().filter_map(|a| a.id).max().map(|m| m + 1).unwrap_or(1);

    let new_asset = CloudAsset {
        id: Some(new_id),
        asset_name: asset_req.asset_name.clone(),
        spec: InstanceSpec {
            instance_type: asset_req.instance_type.clone(),
            cpu_cores: asset_req.cpu_cores,
            memory_gb: asset_req.memory_gb,
            cpu_arch: None,
            gpu_spec: None,
            bandwidth_mbps: asset_req.bandwidth_mbps,
        },
        system_disk: SystemDisk {
            disk_type: asset_req.system_disk_type.clone(),
            size_gb: asset_req.system_disk_size_gb,
            category: None,
            performance_level: None,
        },
        cloud_region: CloudRegion {
            provider: asset_req.cloud_provider.clone(),
            region_id: asset_req.region_id.clone(),
            zone_id: asset_req.zone_id.clone(),
            region_name: format!("{:?}", asset_req.cloud_provider),
        },
        public_ip: asset_req.public_ip.clone(),
        private_ip: asset_req.private_ip.clone(),
        ipv6_address: asset_req.ipv6_address.clone(),
        billing_mode: asset_req.billing_mode.clone(),
        expire_time: asset_req.expire_time,
        status: VMStatus::Running,
        os_type: asset_req.os_type.clone(),
        os_name: asset_req.os_name.clone(),
        os_arch: None,
        image_id: asset_req.image_id.clone(),
        image_name: asset_req.image_name.clone(),
        department: Department {
            id: asset_req.department_id.clone(),
            name: asset_req.department_name.clone(),
            parent_id: None,
            level: 1,
        },
        project: Project {
            id: asset_req.project_id.clone(),
            name: asset_req.project_name.clone(),
            code: asset_req.project_code.clone(),
            department_id: Some(asset_req.department_id.clone()),
        },
        owner: ContactPerson {
            id: asset_req.owner_id.clone(),
            name: asset_req.owner_name.clone(),
            email: asset_req.owner_email.clone(),
            phone: asset_req.owner_phone.clone(),
            department: Some(asset_req.department_name.clone()),
        },
        created_at: Utc::now(),
        last_synced: Some(Utc::now()),
        cloud_disks: asset_req.cloud_disks.clone(),
        cloud_disk_count: asset_req.cloud_disks.len() as u32,
        cloud_disk_total_size_gb: asset_req.cloud_disks.iter().map(|d| d.size_gb).sum(),
        snapshot_info: SnapshotInfo {
            has_snapshot: asset_req.has_snapshot,
            snapshot_count: asset_req.snapshot_count,
            latest_snapshot_time: None,
            total_snapshot_size_gb: 0,
        },
        instance_id: asset_req.instance_id.clone(),
        tags: asset_req.tags.clone(),
        charge_type: None,
    };

    assets.push(new_asset.clone());

    // 记录审计日志
    let mut logs = state.audit_logs.lock().unwrap();
    logs.push(shared::AuditLog {
        id: Uuid::new_v4().to_string(),
        user_id: "system".to_string(),
        username: "system".to_string(),
        action: "CREATE_CLOUD_ASSET_FROM_BUSINESS".to_string(),
        target: format!("CloudAsset: {} <- BusinessResource: {}", new_id, business_resource_id),
        details: format!(
            "从业务申请录入云资产: {} ({:?})",
            new_asset.asset_name, new_asset.cloud_region.provider
        ),
        timestamp: Utc::now(),
    });

    Ok((StatusCode::CREATED, Json(new_asset)))
}

/// 更新云资产
pub async fn update_cloud_asset(
    Path(asset_id): Path<i32>,
    State(state): State<AppState>,
    Json(asset_req): Json<UpdateCloudAssetRequest>,
) -> impl IntoResponse {
    let mut assets = state.cloud_assets.lock().unwrap();

    if let Some(asset) = assets.iter_mut().find(|a| a.id == Some(asset_id)) {
        if let Some(name) = &asset_req.asset_name {
            asset.asset_name = name.clone();
        }
        if let Some(status) = &asset_req.status {
            asset.status = status.clone();
        }
        if let Some(expire) = &asset_req.expire_time {
            asset.expire_time = Some(*expire);
        }
        if let Some(owner_id) = &asset_req.owner_id {
            asset.owner.id = owner_id.clone();
        }
        if let Some(owner_name) = &asset_req.owner_name {
            asset.owner.name = owner_name.clone();
        }
        if let Some(dept_id) = &asset_req.department_id {
            asset.department.id = dept_id.clone();
        }
        if let Some(proj_id) = &asset_req.project_id {
            asset.project.id = proj_id.clone();
        }
        if let Some(tags) = &asset_req.tags {
            asset.tags = tags.clone();
        }

        Json(asset.clone()).into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Cloud asset not found"}))
        ).into_response()
    }
}

/// 删除云资产
pub async fn delete_cloud_asset(
    Path(asset_id): Path<i32>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let mut assets = state.cloud_assets.lock().unwrap();

    let original_len = assets.len();
    assets.retain(|a| a.id != Some(asset_id));

    if assets.len() < original_len {
        // 记录审计日志
        let mut logs = state.audit_logs.lock().unwrap();
        logs.push(shared::AuditLog {
            id: Uuid::new_v4().to_string(),
            user_id: "system".to_string(),
            username: "system".to_string(),
            action: "DELETE_CLOUD_ASSET".to_string(),
            target: format!("CloudAsset: {}", asset_id),
            details: format!("删除云资产: {}", asset_id),
            timestamp: Utc::now(),
        });

        Json(json!({"message": "Cloud asset deleted successfully"})).into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Cloud asset not found"}))
        ).into_response()
    }
}

/// 获取云资产统计
pub async fn get_cloud_asset_stats(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let assets = state.cloud_assets.lock().unwrap();

    let total_count = assets.len() as u32;
    let running_count = assets.iter().filter(|a| a.status == VMStatus::Running).count() as u32;
    let stopped_count = assets.iter().filter(|a| a.status == VMStatus::Stopped).count() as u32;
    let total_cpu_cores: u32 = assets.iter().map(|a| a.spec.cpu_cores).sum();
    let total_memory_gb: u32 = assets.iter().map(|a| a.spec.memory_gb).sum();
    let total_disk_size_gb: u32 = assets.iter().map(|a| a.cloud_disk_total_size_gb).sum();

    // Expire in 7 days
    let seven_days_from_now = Utc::now() + chrono::Duration::days(7);
    let expiring_soon_count = assets.iter()
        .filter(|a| {
            a.expire_time.map_or(false, |e| e <= seven_days_from_now)
        })
        .count() as u32;

    // 按厂商统计
    let mut by_provider = vec![];
    let mut seen_providers = std::collections::HashSet::new();
    for asset in assets.iter() {
        if seen_providers.insert(asset.cloud_region.provider.clone()) {
            let count = assets.iter().filter(|a| a.cloud_region.provider == asset.cloud_region.provider).count() as u32;
            by_provider.push((asset.cloud_region.provider.clone(), count));
        }
    }

    // 按部门统计
    let mut by_department = vec![];
    let mut seen_depts = std::collections::HashSet::new();
    for asset in assets.iter() {
        if seen_depts.insert(asset.department.name.clone()) {
            let count = assets.iter().filter(|a| a.department.name == asset.department.name).count() as u32;
            by_department.push((asset.department.name.clone(), count));
        }
    }

    // 按项目统计
    let mut by_project = vec![];
    let mut seen_projs = std::collections::HashSet::new();
    for asset in assets.iter() {
        if seen_projs.insert(asset.project.name.clone()) {
            let count = assets.iter().filter(|a| a.project.name == asset.project.name).count() as u32;
            by_project.push((asset.project.name.clone(), count));
        }
    }

    let stats = CloudAssetStats {
        total_count,
        running_count,
        stopped_count,
        total_cpu_cores,
        total_memory_gb,
        total_disk_size_gb,
        expiring_soon_count,
        by_provider,
        by_department,
        by_project,
    };

    Json(stats).into_response()
}

/// 同步云资产（从云厂商平台同步）
///
/// 说明：
/// - 本接口用于触发从云厂商平台同步资源状态
/// - 实际实现需要对接各云厂商的 API（Aliyun SDK、Tencent SDK 等）
/// - 同步内容包括：实例状态、IP地址、磁盘信息等
///
/// 云厂商 API 对接示例：
/// - 阿里云: https://help.aliyun.com/document_detail/25484.html
/// - 腾讯云: https://cloud.tencent.com/document/api/213/15728
/// - 华为云: https://support.huaweicloud.com/api-ecs/ecs_02_0101.html
pub async fn sync_cloud_assets(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let assets = state.cloud_assets.lock().unwrap();
    let count = assets.len();

    // 记录审计日志
    let mut logs = state.audit_logs.lock().unwrap();
    logs.push(shared::AuditLog {
        id: Uuid::new_v4().to_string(),
        user_id: "system".to_string(),
        username: "system".to_string(),
        action: "SYNC_CLOUD_ASSETS".to_string(),
        target: "cloud_assets".to_string(),
        details: format!("触发云资产同步，当前资产数: {}", count),
        timestamp: Utc::now(),
    });

    // TODO: 实际对接云厂商 API
    // 1. 根据资产中的 instance_id 和 provider 调用对应云厂商 API
    // 2. 获取最新状态并更新
    // 3. 对于不存在的资源标记为 deleted

    Json(json!({
        "message": "Cloud sync triggered",
        "status": "completed",
        "synced_count": count,
        "note": "实际生产环境需要对接云厂商 API (Aliyun/Tencent/Huawei/AWS SDK)"
    })).into_response()
}
