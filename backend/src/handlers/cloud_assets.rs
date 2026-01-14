use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::sync::Mutex;
use shared::{
    CloudAsset, CloudProvider, CloudRegion, BillingMode, VMStatus,
    InstanceSpec, SystemDisk, CloudDisk, SnapshotInfo,
    Department, Project, ContactPerson, CreateCloudAssetRequest,
    UpdateCloudAssetRequest,
};
use crate::state::AppState;
use chrono::Utc;

// In-memory storage for cloud assets (replace with real database in production)
lazy_static::lazy_static! {
    static ref CLOUD_ASSETS: Mutex<Vec<CloudAsset>> = Mutex::new(Vec::new());
}

// Initialize with sample data
fn init_sample_data() {
    let mut assets = CLOUD_ASSETS.lock().unwrap();
    if assets.is_empty() {
        // Sample: Aliyun ECS instance
        assets.push(CloudAsset {
            id: Some(1),
            asset_name: "Web-Server-01".to_string(),
            spec: InstanceSpec {
                instance_type: "ecs.g6.large".to_string(),
                cpu_cores: 2,
                memory_gb: 8,
                cpu_arch: Some("x86_64".to_string()),
                gpu_spec: None,
                bandwidth_mbps: Some(5),
            },
            system_disk: SystemDisk {
                disk_type: "cloud_essd".to_string(),
                size_gb: 40,
                category: Some("ESSD_PL1".to_string()),
                performance_level: Some("PL1".to_string()),
            },
            cloud_region: CloudRegion {
                provider: CloudProvider::Aliyun,
                region_id: "cn-hangzhou".to_string(),
                zone_id: Some("cn-hangzhou-i".to_string()),
                region_name: "华东1(杭州)".to_string(),
            },
            public_ip: Some("47.97.123.45".to_string()),
            private_ip: "172.16.0.10".to_string(),
            ipv6_address: Some("2001:db8::1".to_string()),
            billing_mode: BillingMode::Subscription,
            expire_time: Some(Utc::now() + chrono::Duration::days(30)),
            status: VMStatus::Running,
            os_type: "Linux".to_string(),
            os_name: "CentOS 7.9".to_string(),
            os_arch: Some("x86_64".to_string()),
            image_id: "centos_7_9_x64_20G_alibase_20230710.vhd".to_string(),
            image_name: Some("CentOS 7.9 64位".to_string()),
            department: Department {
                id: "dept-001".to_string(),
                name: "技术部".to_string(),
                parent_id: None,
                level: 1,
            },
            project: Project {
                id: "proj-001".to_string(),
                name: "电商平台".to_string(),
                code: "ECOMMERCE".to_string(),
                department_id: Some("dept-001".to_string()),
            },
            owner: ContactPerson {
                id: "user-001".to_string(),
                name: "张三".to_string(),
                email: Some("zhangsan@example.com".to_string()),
                phone: Some("13800138000".to_string()),
                department: Some("技术部".to_string()),
            },
            created_at: Utc::now() - chrono::Duration::days(60),
            last_synced: Some(Utc::now()),
            cloud_disks: vec![
                CloudDisk {
                    disk_id: "d-12345678".to_string(),
                    disk_name: "data-disk-01".to_string(),
                    disk_type: "cloud_essd".to_string(),
                    size_gb: 100,
                    status: "in_use".to_string(),
                    category: Some("ESSD_PL1".to_string()),
                    iops: Some(3000),
                    throughput_mb: Some(125.0),
                    is_snapshot: false,
                }
            ],
            cloud_disk_count: 1,
            cloud_disk_total_size_gb: 100,
            snapshot_info: SnapshotInfo {
                has_snapshot: true,
                snapshot_count: 3,
                latest_snapshot_time: Some(Utc::now() - chrono::Duration::days(1)),
                total_snapshot_size_gb: 15,
            },
            instance_id: "i-1234567890abcdef0".to_string(),
            tags: vec!["web".to_string(), "production".to_string()],
            charge_type: Some("PrePaid".to_string()),
        });

        // Sample: Tencent Cloud CVM
        assets.push(CloudAsset {
            id: Some(2),
            asset_name: "API-Gateway-01".to_string(),
            spec: InstanceSpec {
                instance_type: "S5.MEDIUM4".to_string(),
                cpu_cores: 4,
                memory_gb: 16,
                cpu_arch: Some("x86_64".to_string()),
                gpu_spec: None,
                bandwidth_mbps: Some(10),
            },
            system_disk: SystemDisk {
                disk_type: "CLOUD_PREMIUM".to_string(),
                size_gb: 50,
                category: Some("premium".to_string()),
                performance_level: None,
            },
            cloud_region: CloudRegion {
                provider: CloudProvider::Tencent,
                region_id: "ap-guangzhou".to_string(),
                zone_id: Some("ap-guangzhou-4".to_string()),
                region_name: "华南地区(广州)".to_string(),
            },
            public_ip: Some("119.29.1.100".to_string()),
            private_ip: "10.0.1.5".to_string(),
            ipv6_address: None,
            billing_mode: BillingMode::PayAsYouGo,
            expire_time: None,
            status: VMStatus::Running,
            os_type: "Linux".to_string(),
            os_name: "Ubuntu 20.04".to_string(),
            os_arch: Some("x86_64".to_string()),
            image_id: "img-8toqc6ts".to_string(),
            image_name: Some("Ubuntu 20.04 LTS".to_string()),
            department: Department {
                id: "dept-001".to_string(),
                name: "技术部".to_string(),
                parent_id: None,
                level: 1,
            },
            project: Project {
                id: "proj-002".to_string(),
                name: "微服务架构".to_string(),
                code: "MICROSERVICE".to_string(),
                department_id: Some("dept-001".to_string()),
            },
            owner: ContactPerson {
                id: "user-002".to_string(),
                name: "李四".to_string(),
                email: Some("lisi@example.com".to_string()),
                phone: Some("13900139000".to_string()),
                department: Some("技术部".to_string()),
            },
            created_at: Utc::now() - chrono::Duration::days(30),
            last_synced: Some(Utc::now()),
            cloud_disks: vec![],
            cloud_disk_count: 0,
            cloud_disk_total_size_gb: 50,
            snapshot_info: SnapshotInfo {
                has_snapshot: false,
                snapshot_count: 0,
                latest_snapshot_time: None,
                total_snapshot_size_gb: 0,
            },
            instance_id: "ins-12345678".to_string(),
            tags: vec!["api".to_string(), "gateway".to_string()],
            charge_type: Some("PostPaid".to_string()),
        });

        // Sample: Huawei Cloud ECS
        assets.push(CloudAsset {
            id: Some(3),
            asset_name: "Database-Master".to_string(),
            spec: InstanceSpec {
                instance_type: "s6.xlarge.4".to_string(),
                cpu_cores: 16,
                memory_gb: 64,
                cpu_arch: Some("x86_64".to_string()),
                gpu_spec: None,
                bandwidth_mbps: Some(20),
            },
            system_disk: SystemDisk {
                disk_type: "SATA".to_string(),
                size_gb: 100,
                category: None,
                performance_level: None,
            },
            cloud_region: CloudRegion {
                provider: CloudProvider::Huawei,
                region_id: "cn-north-4".to_string(),
                zone_id: Some("cn-north-4a".to_string()),
                region_name: "华北-北京四".to_string(),
            },
            public_ip: Some("114.115.1.200".to_string()),
            private_ip: "192.168.1.10".to_string(),
            ipv6_address: None,
            billing_mode: BillingMode::Subscription,
            expire_time: Some(Utc::now() + chrono::Duration::days(90)),
            status: VMStatus::Running,
            os_type: "Linux".to_string(),
            os_name: "EulerOS 2.0".to_string(),
            os_arch: Some("x86_64".to_string()),
            image_id: "img-market-0000001".to_string(),
            image_name: Some("EulerOS 2.0 SP8".to_string()),
            department: Department {
                id: "dept-002".to_string(),
                name: "运维部".to_string(),
                parent_id: None,
                level: 1,
            },
            project: Project {
                id: "proj-003".to_string(),
                name: "数据库服务".to_string(),
                code: "DATABASE".to_string(),
                department_id: Some("dept-002".to_string()),
            },
            owner: ContactPerson {
                id: "user-003".to_string(),
                name: "王五".to_string(),
                email: Some("wangwu@example.com".to_string()),
                phone: Some("13700137000".to_string()),
                department: Some("运维部".to_string()),
            },
            created_at: Utc::now() - chrono::Duration::days(90),
            last_synced: Some(Utc::now()),
            cloud_disks: vec![
                CloudDisk {
                    disk_id: "disk-98765432".to_string(),
                    disk_name: "db-data-01".to_string(),
                    disk_type: "SSD".to_string(),
                    size_gb: 500,
                    status: "in_use".to_string(),
                    category: None,
                    iops: Some(5000),
                    throughput_mb: None,
                    is_snapshot: true,
                }
            ],
            cloud_disk_count: 1,
            cloud_disk_total_size_gb: 500,
            snapshot_info: SnapshotInfo {
                has_snapshot: true,
                snapshot_count: 7,
                latest_snapshot_time: Some(Utc::now() - chrono::Duration::hours(6)),
                total_snapshot_size_gb: 80,
            },
            instance_id: "i-9876543210fedcba0".to_string(),
            tags: vec!["database".to_string(), "mysql".to_string()],
            charge_type: Some("PrePaid".to_string()),
        });
    }
}

pub async fn get_cloud_assets(
    State(_state): State<AppState>,
) -> impl IntoResponse {
    init_sample_data();

    let assets = CLOUD_ASSETS.lock().unwrap();
    Json(assets.clone()).into_response()
}

pub async fn get_cloud_asset(
    Path(asset_id): Path<i32>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    init_sample_data();

    let assets = CLOUD_ASSETS.lock().unwrap();

    if let Some(asset) = assets.iter().find(|a| a.id == Some(asset_id)) {
        Json(asset.clone()).into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Cloud asset not found"}))
        ).into_response()
    }
}

pub async fn create_cloud_asset(
    State(_state): State<AppState>,
    Json(asset_req): Json<CreateCloudAssetRequest>,
) -> impl IntoResponse {
    init_sample_data();

    let mut assets = CLOUD_ASSETS.lock().unwrap();

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

    (StatusCode::CREATED, Json(new_asset)).into_response()
}

pub async fn update_cloud_asset(
    Path(asset_id): Path<i32>,
    State(_state): State<AppState>,
    Json(asset_req): Json<UpdateCloudAssetRequest>,
) -> impl IntoResponse {
    init_sample_data();

    let mut assets = CLOUD_ASSETS.lock().unwrap();

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

pub async fn delete_cloud_asset(
    Path(asset_id): Path<i32>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    init_sample_data();

    let mut assets = CLOUD_ASSETS.lock().unwrap();

    let original_len = assets.len();
    assets.retain(|a| a.id != Some(asset_id));

    if assets.len() < original_len {
        Json(json!({"message": "Cloud asset deleted successfully"})).into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Cloud asset not found"}))
        ).into_response()
    }
}

pub async fn get_cloud_asset_stats(
    State(_state): State<AppState>,
) -> impl IntoResponse {
    init_sample_data();

    let assets = CLOUD_ASSETS.lock().unwrap();

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

    // Use simple Vec instead of HashMap for by_provider
    let mut by_provider = vec![];
    let mut seen_providers = std::collections::HashSet::new();
    for asset in assets.iter() {
        if seen_providers.insert(asset.cloud_region.provider.clone()) {
            let count = assets.iter().filter(|a| a.cloud_region.provider == asset.cloud_region.provider).count() as u32;
            by_provider.push((asset.cloud_region.provider.clone(), count));
        }
    }

    // Simple Vec for by_department
    let mut by_department = vec![];
    let mut seen_depts = std::collections::HashSet::new();
    for asset in assets.iter() {
        if seen_depts.insert(asset.department.name.clone()) {
            let count = assets.iter().filter(|a| a.department.name == asset.department.name).count() as u32;
            by_department.push((asset.department.name.clone(), count));
        }
    }

    // Simple Vec for by_project
    let mut by_project = vec![];
    let mut seen_projs = std::collections::HashSet::new();
    for asset in assets.iter() {
        if seen_projs.insert(asset.project.name.clone()) {
            let count = assets.iter().filter(|a| a.project.name == asset.project.name).count() as u32;
            by_project.push((asset.project.name.clone(), count));
        }
    }

    use shared::CloudAssetStats;
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

pub async fn sync_cloud_assets(
    State(_state): State<AppState>,
) -> impl IntoResponse {
    // Placeholder for cloud asset sync from actual cloud providers
    Json(json!({
        "message": "Cloud sync started",
        "status": "in_progress",
        "providers": ["aliyun", "tencent", "huawei", "aws", "azure", "gcp"]
    })).into_response()
}
