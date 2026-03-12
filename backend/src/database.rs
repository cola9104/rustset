//! Database module using SeaORM
//!
//! This module provides database connectivity and CRUD operations using SeaORM.

use sea_orm::{Database as SeaDatabase, DatabaseConnection, DbErr, EntityTrait, ActiveModelTrait, Set, NotSet, ConnectionTrait, Statement, QuerySelect, QueryOrder, ColumnTrait};
use crate::entities::{
    cloud_zone, cloud_service, cloud_provider_config, business_resource,
    physical_machine, cloud_virtual_machine,
    user, audit_log, asset, task, risk, network_zone, custom_role, advanced_scan_task, quick_scan_result,
    CloudZone, CloudService, CloudProviderConfig, BusinessResource,
    PhysicalMachine, CloudVirtualMachine,
    User, AuditLog, Asset, Task, Risk, NetworkZone, CustomRole, AdvancedScanTask,
};
use shared::User as SharedUser;
use std::sync::Arc;
use chrono::Utc;
use uuid;

// Re-export entities for convenience
pub use crate::entities::prelude::*;

// Legacy type aliases for compatibility with handlers
pub type DbUser = user::Model;
pub type DbAuditLog = audit_log::Model;
pub type DbBusinessResource = business_resource::Model;
pub type DbPhysicalMachine = physical_machine::Model;
pub type DbCloudVirtualMachine = cloud_virtual_machine::Model;
pub type DbAsset = asset::Model;
pub type DbTask = task::Model;
pub type DbRisk = risk::Model;
pub type DbZone = network_zone::Model;

/// Global database connection (Arc-wrapped for sharing across threads)
pub static DB: std::sync::OnceLock<Arc<DatabaseConnection>> = std::sync::OnceLock::new();

/// Initialize the global database connection
pub async fn init_db(connection_string: &str) -> Result<(), DbErr> {
    let conn = SeaDatabase::connect(connection_string).await?;

    // Run migrations
    use sea_orm_migration::prelude::*;
    use crate::migration::{Migrator, MigratorTrait};
    Migrator::up(&conn, None).await?;

    DB.set(Arc::new(conn))
        .map_err(|_| DbErr::Custom("Database already initialized".to_string()))?;
    Ok(())
}

/// Get the global database connection
pub fn get_db() -> Option<Arc<DatabaseConnection>> {
    DB.get().cloned()
}

/// Database type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseType {
    SQLite,
    PostgreSQL,
    MySQL,
}

impl DatabaseType {
    /// From connection string
    pub fn from_connection_string(conn_str: &str) -> Self {
        let lower = conn_str.to_lowercase();
        if lower.starts_with("sqlite://") || lower.starts_with("sqlite:") {
            DatabaseType::SQLite
        } else if lower.starts_with("postgres://") || lower.starts_with("postgresql://") {
            DatabaseType::PostgreSQL
        } else if lower.starts_with("mysql://") || lower.starts_with("mariadb://") {
            DatabaseType::MySQL
        } else {
            DatabaseType::SQLite
        }
    }
}

pub struct Database {
    conn: DatabaseConnection,
    db_type: DatabaseType,
}

impl Database {
    pub async fn new(connection_string: &str) -> Result<Self, DbErr> {
        let db_type = DatabaseType::from_connection_string(connection_string);

        // Create connection
        let conn = SeaDatabase::connect(connection_string).await?;

        let db = Database { conn, db_type };

        // Run migrations
        db.run_migrations().await?;

        Ok(db)
    }

    pub fn conn(&self) -> &DatabaseConnection {
        &self.conn
    }

    pub fn db_type(&self) -> DatabaseType {
        self.db_type
    }

    async fn run_migrations(&self) -> Result<(), DbErr> {
        // Run SeaORM migrations
        use sea_orm_migration::prelude::*;
        use crate::migration::{Migrator, MigratorTrait};

        Migrator::up(&self.conn, None).await?;
        Ok(())
    }

    /// Execute a raw SQL query
    pub async fn execute(&self, sql: &str) -> Result<u64, DbErr> {
        let stmt = Statement::from_string(self.conn.get_database_backend(), sql.to_string());
        let result = self.conn.execute_raw(stmt).await?;
        Ok(result.rows_affected())
    }
}

// ============== Helper functions for converting between entities and shared types ==============

/// Convert DbUser (entity) to shared User
pub fn db_user_to_shared(db: user::Model) -> SharedUser {
    use shared::Role;
    use chrono::Utc;

    let role = match db.role.as_str() {
        "SysAdmin" => Role::SysAdmin,
        "SecAdmin" => Role::SecAdmin,
        "Auditor" => Role::Auditor,
        "Custom" => Role::Custom("Custom".to_string()),
        _ => Role::Custom(db.role),
    };

    let permissions = db.permissions.as_ref().and_then(|p| serde_json::from_str(p).ok());

    SharedUser {
        id: db.id,
        username: db.username,
        password: db.password,
        role,
        permissions,
        created_at: chrono::DateTime::parse_from_rfc3339(&db.created_at)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now()),
        password_changed_at: db.password_changed_at.as_ref().and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok().map(|dt| dt.with_timezone(&Utc))),
        password_strength: db.password_strength,
        force_password_change: Some(db.force_password_change != 0),
        last_login_at: db.last_login_at.as_ref().and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok().map(|dt| dt.with_timezone(&Utc))),
        email: db.email,
        phone: db.phone,
        status: db.status,
        failed_login_attempts: db.failed_login_attempts.map(|v| v as u32),
        locked_until: db.locked_until.as_ref().and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok().map(|dt| dt.with_timezone(&Utc))),
    }
}

/// Convert shared User to DbUser active model
pub fn shared_to_db_user(user: &SharedUser) -> user::ActiveModel {
    let permissions_json = user.permissions.as_ref().map(|p| serde_json::to_string(p).ok()).flatten();

    user::ActiveModel {
        id: Set(user.id.clone()),
        username: Set(user.username.clone()),
        password: Set(user.password.clone()),
        role: Set(format!("{:?}", user.role)),
        permissions: Set(permissions_json),
        created_at: Set(user.created_at.to_rfc3339()),
        password_changed_at: Set(user.password_changed_at.map(|d| d.to_rfc3339())),
        password_strength: Set(user.password_strength.clone()),
        force_password_change: Set(user.force_password_change.unwrap_or(false) as i32),
        last_login_at: Set(user.last_login_at.map(|d| d.to_rfc3339())),
        email: Set(user.email.clone()),
        phone: Set(user.phone.clone()),
        status: Set(user.status.clone()),
        failed_login_attempts: Set(user.failed_login_attempts.map(|v| v as i32)),
        locked_until: Set(user.locked_until.map(|d| d.to_rfc3339())),
    }
}

/// Convert DbAsset (entity) to shared Asset
pub fn db_asset_to_shared(db: asset::Model) -> shared::Asset {
    use shared::{Asset, NetworkZone, PortInfo};
    use std::string::String as String;

    let zone = match db.zone.as_str() {
        "Intranet" => NetworkZone::Intranet,
        "DMZ" => NetworkZone::DMZ,
        "Internet" => NetworkZone::Internet,
        _ => NetworkZone::Intranet,
    };

    let ports: Vec<PortInfo> = serde_json::from_str(&db.ports).unwrap_or_default();
    let labels: Vec<String> = db.labels.as_deref().map(|s| serde_json::from_str(s).unwrap_or_default()).unwrap_or_default();

    Asset {
        id: Some(db.id),
        name: db.name,
        ip: db.ip,
        zone,
        ports,
        last_scanned: db.last_scanned.as_ref().and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok().map(|dt| dt.with_timezone(&Utc))),
        contact_person: db.contact_person,
        contact_phone: db.contact_phone,
        created_by: db.created_by,
        updated_by: db.updated_by,
        owner: db.owner,
        weight: db.weight,
        labels,
        os: db.os,
        device_type: db.device_type,
    }
}

/// Convert DbNetworkZone (entity) to shared ZoneConfig
pub fn db_zone_to_shared(db: network_zone::Model) -> shared::ZoneConfig {
    shared::ZoneConfig {
        id: db.id,
        name: db.name,
        cidr: db.cidr,
        priority: db.priority,
    }
}

/// Convert DbRisk (entity) to shared Risk
pub fn db_risk_to_shared(db: risk::Model) -> shared::Risk {
    use shared::{Risk, RiskStatus};

    let status = match db.status.as_str() {
        "Open" => RiskStatus::Open,
        "Resolved" => RiskStatus::Resolved,
        "Verified" => RiskStatus::Verified,
        "Ignored" => RiskStatus::Ignored,
        "FalsePositive" => RiskStatus::FalsePositive,
        "PendingReview" => RiskStatus::PendingReview,
        _ => RiskStatus::Open,
    };

    Risk {
        id: db.id,
        asset_ip: db.asset_ip,
        port: db.port as u16,
        severity: db.severity,
        description: db.description,
        solution: db.solution,
        status,
        created_at: db.created_at.as_ref().and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok().map(|dt| dt.with_timezone(&Utc))),
        updated_at: db.updated_at.as_ref().and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok().map(|dt| dt.with_timezone(&Utc))),
        assigned_to: db.assigned_to,
    }
}

/// Convert DbTask (entity) to shared Task
pub fn db_task_to_shared(db: task::Model) -> shared::Task {
    use shared::{Task, TaskStatus};

    let status = match db.status.as_str() {
        "Pending" => TaskStatus::Pending,
        "Running" => TaskStatus::Running,
        "Completed" => TaskStatus::Completed,
        "Failed" => TaskStatus::Failed,
        _ => TaskStatus::Pending,
    };

    Task {
        id: db.id,
        name: db.name,
        target: db.target,
        status,
        start_time: db.start_time,
        end_time: db.end_time,
        found_assets: db.found_assets as usize,
        found_risks: db.found_risks as usize,
        port_policy: db.port_policy,
        domain_brute: db.domain_brute != 0,
        service_detection: db.service_detection != 0,
        os_detection: db.os_detection != 0,
        site_identify: db.site_identify != 0,
        created_by: db.created_by,
    }
}

// ============== User CRUD ==============

pub async fn get_users_with_conn(conn: &DatabaseConnection) -> Result<Vec<SharedUser>, DbErr> {
    let users = User::find().all(conn).await?;
    Ok(users.into_iter().map(db_user_to_shared).collect())
}

pub async fn insert_user_with_conn(conn: &DatabaseConnection, user: &SharedUser) -> Result<(), DbErr> {
    let db_user = shared_to_db_user(user);
    db_user.insert(conn).await?;
    Ok(())
}

pub async fn update_user_by_id(conn: &DatabaseConnection, user: &SharedUser) -> Result<(), DbErr> {
    let db_user = shared_to_db_user(user);
    User::update(db_user).exec(conn).await?;
    Ok(())
}

pub async fn delete_user_by_id(conn: &DatabaseConnection, id: &str) -> Result<(), DbErr> {
    let user = User::find_by_id(id.to_string()).one(conn).await?;
    if let Some(user) = user {
        user.delete(conn).await?;
    }
    Ok(())
}

pub async fn get_user_by_id(conn: &DatabaseConnection, id: &str) -> Result<Option<SharedUser>, DbErr> {
    let user = User::find_by_id(id.to_string()).one(conn).await?;
    Ok(user.map(db_user_to_shared))
}

pub async fn get_user_by_username(conn: &DatabaseConnection, username: &str) -> Result<Option<SharedUser>, DbErr> {
    let user = User::find()
        .filter(user::Column::Username.eq(username))
        .one(conn)
        .await?;
    Ok(user.map(db_user_to_shared))
}

// ============== AuditLog CRUD ==============

pub async fn insert_audit_log(conn: &DatabaseConnection, log: &shared::AuditLog) -> Result<(), DbErr> {
    let db_log = audit_log::ActiveModel {
        id: Set(log.id.clone()),
        user_id: Set(log.user_id.clone()),
        username: Set(log.username.clone()),
        action: Set(log.action.clone()),
        target: Set(log.target.clone()),
        details: Set(log.details.clone()),
        timestamp: Set(log.timestamp.to_rfc3339()),
    };
    db_log.insert(conn).await?;
    Ok(())
}

// ============== Asset CRUD ==============

pub async fn insert_asset(conn: &DatabaseConnection, asset: &shared::Asset) -> Result<i64, DbErr> {
    let ports_json = serde_json::to_string(&asset.ports).unwrap_or_default();
    let labels_json = serde_json::to_string(&asset.labels).unwrap_or_default();

    let db_asset = asset::ActiveModel {
        id: NotSet,
        name: Set(asset.name.clone()),
        ip: Set(asset.ip.clone()),
        zone: Set(format!("{:?}", asset.zone)),
        ports: Set(ports_json),
        last_scanned: Set(asset.last_scanned.map(|d| d.to_rfc3339())),
        contact_person: Set(asset.contact_person.clone()),
        contact_phone: Set(asset.contact_phone.clone()),
        created_by: Set(asset.created_by.clone()),
        updated_by: Set(asset.updated_by.clone()),
        owner: Set(asset.owner.clone()),
        weight: Set(asset.weight),
        labels: Set(Some(labels_json)),
        os: Set(asset.os.clone()),
        device_type: Set(asset.device_type.clone()),
    };

    let result = db_asset.insert(conn).await?;
    Ok(result.id as i64)
}

pub async fn update_asset_by_id(conn: &DatabaseConnection, id: i32, asset: &shared::Asset) -> Result<(), DbErr> {
    let ports_json = serde_json::to_string(&asset.ports).unwrap_or_default();
    let labels_json = serde_json::to_string(&asset.labels).unwrap_or_default();

    let db_asset = asset::ActiveModel {
        id: Set(id),
        name: Set(asset.name.clone()),
        ip: Set(asset.ip.clone()),
        zone: Set(format!("{:?}", asset.zone)),
        ports: Set(ports_json),
        last_scanned: Set(asset.last_scanned.map(|d| d.to_rfc3339())),
        contact_person: Set(asset.contact_person.clone()),
        contact_phone: Set(asset.contact_phone.clone()),
        updated_by: Set(asset.updated_by.clone()),
        owner: Set(asset.owner.clone()),
        weight: Set(asset.weight),
        labels: Set(Some(labels_json)),
        os: Set(asset.os.clone()),
        device_type: Set(asset.device_type.clone()),
        ..Default::default()
    };

    Asset::update(db_asset).exec(conn).await?;
    Ok(())
}

pub async fn delete_asset_by_id(conn: &DatabaseConnection, id: i32) -> Result<(), DbErr> {
    let asset = Asset::find_by_id(id).one(conn).await?;
    if let Some(asset) = asset {
        asset.delete(conn).await?;
    }
    Ok(())
}

pub async fn get_all_assets(conn: &DatabaseConnection) -> Result<Vec<asset::Model>, DbErr> {
    Asset::find().order_by_desc(asset::Column::Id).all(conn).await
}

pub async fn get_asset_by_id(conn: &DatabaseConnection, id: i32) -> Result<Option<asset::Model>, DbErr> {
    Asset::find_by_id(id).one(conn).await
}

// ============== Task CRUD ==============

pub async fn insert_task(conn: &DatabaseConnection, task: &shared::Task) -> Result<(), DbErr> {
    let db_task = task::ActiveModel {
        id: Set(task.id.clone()),
        name: Set(task.name.clone()),
        target: Set(task.target.clone()),
        status: Set(format!("{:?}", task.status)),
        start_time: Set(task.start_time.clone()),
        end_time: Set(task.end_time.clone()),
        found_assets: Set(task.found_assets as i32),
        found_risks: Set(task.found_risks as i32),
        port_policy: Set(task.port_policy.clone()),
        domain_brute: Set(task.domain_brute as i32),
        service_detection: Set(task.service_detection as i32),
        os_detection: Set(task.os_detection as i32),
        site_identify: Set(task.site_identify as i32),
        created_by: Set(task.created_by.clone()),
    };
    db_task.insert(conn).await?;
    Ok(())
}

pub async fn update_task_by_id(conn: &DatabaseConnection, id: &str, task: &shared::Task) -> Result<(), DbErr> {
    let db_task = task::ActiveModel {
        id: Set(id.to_string()),
        name: Set(task.name.clone()),
        target: Set(task.target.clone()),
        status: Set(format!("{:?}", task.status)),
        start_time: Set(task.start_time.clone()),
        end_time: Set(task.end_time.clone()),
        found_assets: Set(task.found_assets as i32),
        found_risks: Set(task.found_risks as i32),
        ..Default::default()
    };
    Task::update(db_task).exec(conn).await?;
    Ok(())
}

pub async fn delete_task_by_id(conn: &DatabaseConnection, id: &str) -> Result<(), DbErr> {
    let task = Task::find_by_id(id.to_string()).one(conn).await?;
    if let Some(task) = task {
        task.delete(conn).await?;
    }
    Ok(())
}

pub async fn get_all_tasks(conn: &DatabaseConnection) -> Result<Vec<task::Model>, DbErr> {
    Task::find().order_by_desc(task::Column::Id).all(conn).await
}

// ============== Risk CRUD ==============

pub async fn insert_risk(conn: &DatabaseConnection, risk: &shared::Risk) -> Result<(), DbErr> {
    let db_risk = risk::ActiveModel {
        id: Set(risk.id.clone()),
        asset_ip: Set(risk.asset_ip.clone()),
        port: Set(risk.port as i32),
        severity: Set(risk.severity.clone()),
        description: Set(risk.description.clone()),
        solution: Set(risk.solution.clone()),
        status: Set(format!("{:?}", risk.status)),
        created_at: Set(risk.created_at.map(|d| d.to_rfc3339())),
        updated_at: Set(risk.updated_at.map(|d| d.to_rfc3339())),
        assigned_to: Set(risk.assigned_to.clone()),
    };
    db_risk.insert(conn).await?;
    Ok(())
}

pub async fn update_risk_by_id(conn: &DatabaseConnection, id: &str, risk: &shared::Risk) -> Result<(), DbErr> {
    let db_risk = risk::ActiveModel {
        id: Set(id.to_string()),
        severity: Set(risk.severity.clone()),
        description: Set(risk.description.clone()),
        solution: Set(risk.solution.clone()),
        status: Set(format!("{:?}", risk.status)),
        updated_at: Set(risk.updated_at.map(|d| d.to_rfc3339())),
        assigned_to: Set(risk.assigned_to.clone()),
        ..Default::default()
    };
    Risk::update(db_risk).exec(conn).await?;
    Ok(())
}

pub async fn delete_risk_by_id(conn: &DatabaseConnection, id: &str) -> Result<(), DbErr> {
    let risk = Risk::find_by_id(id.to_string()).one(conn).await?;
    if let Some(risk) = risk {
        risk.delete(conn).await?;
    }
    Ok(())
}

pub async fn get_all_risks(conn: &DatabaseConnection) -> Result<Vec<risk::Model>, DbErr> {
    Risk::find().order_by_desc(risk::Column::CreatedAt).all(conn).await
}

// ============== NetworkZone CRUD ==============

pub async fn insert_zone(conn: &DatabaseConnection, zone: &shared::ZoneConfig) -> Result<(), DbErr> {
    let db_zone = network_zone::ActiveModel {
        id: Set(zone.id.clone()),
        name: Set(zone.name.clone()),
        cidr: Set(zone.cidr.clone()),
        priority: Set(zone.priority),
    };
    db_zone.insert(conn).await?;
    Ok(())
}

pub async fn update_zone_by_id(conn: &DatabaseConnection, id: &str, zone: &shared::ZoneConfig) -> Result<(), DbErr> {
    let db_zone = network_zone::ActiveModel {
        id: Set(id.to_string()),
        name: Set(zone.name.clone()),
        cidr: Set(zone.cidr.clone()),
        priority: Set(zone.priority),
    };
    NetworkZone::update(db_zone).exec(conn).await?;
    Ok(())
}

pub async fn delete_zone_by_id(conn: &DatabaseConnection, id: &str) -> Result<(), DbErr> {
    let zone = NetworkZone::find_by_id(id.to_string()).one(conn).await?;
    if let Some(zone) = zone {
        zone.delete(conn).await?;
    }
    Ok(())
}

pub async fn get_all_zones(conn: &DatabaseConnection) -> Result<Vec<network_zone::Model>, DbErr> {
    NetworkZone::find().order_by_asc(network_zone::Column::Priority).all(conn).await
}

// ============== CloudZone CRUD ==============

pub async fn insert_cloud_zone(
    conn: &DatabaseConnection,
    zone_name: &str,
    zone_code: &str,
    description: Option<&str>,
    created_at: &str,
) -> Result<i32, DbErr> {
    let db_zone = cloud_zone::ActiveModel {
        id: NotSet,
        zone_name: Set(zone_name.to_string()),
        zone_code: Set(zone_code.to_string()),
        description: Set(Some(description.map(|s| s.to_string()).unwrap_or_default())),
        created_at: Set(created_at.to_string()),
    };
    let result = db_zone.insert(conn).await?;
    Ok(result.id)
}

pub async fn update_cloud_zone_by_id(
    conn: &DatabaseConnection,
    id: i32,
    zone_name: Option<&str>,
    zone_code: Option<&str>,
    description: Option<&str>,
) -> Result<(), DbErr> {
    let mut db_zone = cloud_zone::ActiveModel {
        id: Set(id),
        ..Default::default()
    };

    if let Some(name) = zone_name {
        db_zone.zone_name = Set(name.to_string());
    }
    if let Some(code) = zone_code {
        db_zone.zone_code = Set(code.to_string());
    }
    if let Some(desc) = description {
        db_zone.description = Set(Some(desc.to_string()));
    }

    CloudZone::update(db_zone).exec(conn).await?;
    Ok(())
}

pub async fn delete_cloud_zone_by_id(conn: &DatabaseConnection, id: i32) -> Result<(), DbErr> {
    let zone = CloudZone::find_by_id(id).one(conn).await?;
    if let Some(zone) = zone {
        zone.delete(conn).await?;
    }
    Ok(())
}

pub async fn get_all_cloud_zones(conn: &DatabaseConnection) -> Result<Vec<cloud_zone::Model>, DbErr> {
    CloudZone::find().order_by_asc(cloud_zone::Column::Id).all(conn).await
}

pub async fn get_cloud_zone_by_id(conn: &DatabaseConnection, id: i32) -> Result<Option<cloud_zone::Model>, DbErr> {
    CloudZone::find_by_id(id).one(conn).await
}

// ============== CloudPlatform CRUD ==============

pub async fn insert_cloud_platform(
    conn: &DatabaseConnection,
    zone_id: i32,
    platform_name: &str,
    platform_code: &str,
    description: Option<&str>,
    created_at: &str,
) -> Result<i32, DbErr> {
    let db_service = cloud_service::ActiveModel {
        id: NotSet,
        zone_id: Set(zone_id),
        service_name: Set(platform_name.to_string()),
        service_code: Set(platform_code.to_string()),
        description: Set(Some(description.map(|s| s.to_string()).unwrap_or_default())),
        created_at: Set(created_at.to_string()),
    };
    let result = db_service.insert(conn).await?;
    Ok(result.id)
}

pub async fn update_cloud_platform_by_id(
    conn: &DatabaseConnection,
    id: i32,
    zone_id: Option<i32>,
    platform_name: Option<&str>,
    platform_code: Option<&str>,
    description: Option<&str>,
) -> Result<(), DbErr> {
    let mut db_service = cloud_service::ActiveModel {
        id: Set(id),
        ..Default::default()
    };

    if let Some(zid) = zone_id {
        db_service.zone_id = Set(zid);
    }
    if let Some(name) = platform_name {
        db_service.service_name = Set(name.to_string());
    }
    if let Some(code) = platform_code {
        db_service.service_code = Set(code.to_string());
    }
    if let Some(desc) = description {
        db_service.description = Set(Some(desc.to_string()));
    }

    CloudService::update(db_service).exec(conn).await?;
    Ok(())
}

pub async fn delete_cloud_platform_by_id(conn: &DatabaseConnection, id: i32) -> Result<(), DbErr> {
    let service = CloudService::find_by_id(id).one(conn).await?;
    if let Some(service) = service {
        service.delete(conn).await?;
    }
    Ok(())
}

pub async fn get_all_cloud_platforms(conn: &DatabaseConnection) -> Result<Vec<cloud_service::Model>, DbErr> {
    CloudService::find().order_by_asc(cloud_service::Column::Id).all(conn).await
}

pub async fn get_cloud_platform_by_id(conn: &DatabaseConnection, id: i32) -> Result<Option<cloud_service::Model>, DbErr> {
    CloudService::find_by_id(id).one(conn).await
}

pub async fn get_platforms_by_zone_id(conn: &DatabaseConnection, zone_id: i32) -> Result<Vec<cloud_service::Model>, DbErr> {
    CloudService::find()
        .filter(cloud_service::Column::ZoneId.eq(zone_id))
        .order_by_asc(cloud_service::Column::Id)
        .all(conn)
        .await
}

// ============== CloudProviderConfig CRUD ==============

pub async fn insert_cloud_provider_config(
    conn: &DatabaseConnection,
    zone_id: i32,
    platform_id: i32,
    provider: &str,
    region_id: &str,
    region_name: &str,
    account_name: &str,
    access_key_id: &str,
    access_key_secret: &str,
    remarks: Option<&str>,
    created_at: &str,
) -> Result<i32, DbErr> {
    let db_config = cloud_provider_config::ActiveModel {
        id: NotSet,
        zone_id: Set(zone_id),
        platform_id: Set(platform_id),
        provider: Set(provider.to_string()),
        region_id: Set(region_id.to_string()),
        region_name: Set(region_name.to_string()),
        account_name: Set(account_name.to_string()),
        access_key_id: Set(access_key_id.to_string()),
        access_key_secret: Set(access_key_secret.to_string()),
        remarks: Set(remarks.map(|s| s.to_string())),
        status: Set("active".to_string()),
        last_test_time: Set(None),
        last_test_result: Set(None),
        created_at: Set(created_at.to_string()),
        updated_at: Set(None),
    };
    let result = db_config.insert(conn).await?;
    Ok(result.id)
}

pub async fn update_cloud_provider_config_by_id(
    conn: &DatabaseConnection,
    id: i32,
    zone_id: i32,
    platform_id: i32,
    region_id: &str,
    region_name: &str,
    account_name: &str,
    access_key_id: &str,
    access_key_secret: &str,
    remarks: Option<&str>,
    status: &str,
    updated_at: Option<&str>,
) -> Result<(), DbErr> {
    let db_config = cloud_provider_config::ActiveModel {
        id: Set(id),
        zone_id: Set(zone_id),
        platform_id: Set(platform_id),
        region_id: Set(region_id.to_string()),
        region_name: Set(region_name.to_string()),
        account_name: Set(account_name.to_string()),
        access_key_id: Set(access_key_id.to_string()),
        access_key_secret: Set(access_key_secret.to_string()),
        remarks: Set(remarks.map(|s| s.to_string())),
        status: Set(status.to_string()),
        updated_at: Set(updated_at.map(|s| s.to_string())),
        ..Default::default()
    };
    CloudProviderConfig::update(db_config).exec(conn).await?;
    Ok(())
}

pub async fn delete_cloud_provider_config_by_id(conn: &DatabaseConnection, id: i32) -> Result<(), DbErr> {
    let config = CloudProviderConfig::find_by_id(id).one(conn).await?;
    if let Some(config) = config {
        config.delete(conn).await?;
    }
    Ok(())
}

pub async fn get_all_cloud_provider_configs(conn: &DatabaseConnection) -> Result<Vec<cloud_provider_config::Model>, DbErr> {
    CloudProviderConfig::find().order_by_desc(cloud_provider_config::Column::Id).all(conn).await
}

pub async fn get_cloud_provider_config_by_id(conn: &DatabaseConnection, id: i32) -> Result<Option<cloud_provider_config::Model>, DbErr> {
    CloudProviderConfig::find_by_id(id).one(conn).await
}

pub async fn get_active_cloud_provider_configs(conn: &DatabaseConnection) -> Result<Vec<cloud_provider_config::Model>, DbErr> {
    CloudProviderConfig::find()
        .filter(cloud_provider_config::Column::Status.eq("active"))
        .order_by_desc(cloud_provider_config::Column::Id)
        .all(conn)
        .await
}

// ============== BusinessResource CRUD ==============

pub async fn insert_business_resource(
    conn: &DatabaseConnection,
    req: &shared::CreateBusinessResourceRequest,
    created_at: &str,
    created_by: &str,
) -> Result<i32, DbErr> {
    let db_resource = business_resource::ActiveModel {
        id: NotSet,
        resource_type: Set(req.resource_type.clone()),
        ecs_name: Set(req.ecs_name.clone()),
        ecs_status: Set(req.ecs_status.clone()),
        resource_id: Set(req.resource_id.clone().unwrap_or_else(|| "".to_string())),
        cloud_region: Set(req.cloud_region.clone()),
        cloud_category: Set(req.cloud_category.clone()),
        cloud_provider_config_id: Set(req.cloud_provider_config_id),
        zone_name: Set(req.zone_name.clone()),
        platform_name: Set(req.platform_name.clone()),
        county_city: Set(req.county_city.clone()),
        vdc_name: Set(req.vdc_name.clone()),
        customer_name: Set(req.customer_name.clone()),
        application_name: Set(req.application_name.clone()),
        contract_name: Set(req.contract_name.clone()),
        instance_id: Set(req.instance_id.clone().unwrap_or_else(|| "".to_string())),
        ecs_type: Set(req.ecs_type.clone()),
        ecs_os: Set(req.ecs_os.clone()),
        cpu_cores: Set(req.cpu_cores as i32),
        memory_gb: Set(req.memory_gb as i32),
        system_disk: Set(req.system_disk.clone()),
        system_disk_size_gb: Set(req.system_disk_size_gb as i32),
        data_disk: Set(req.data_disk.clone()),
        completion_time: Set(req.completion_time.map(|d| d.to_rfc3339())),
        release_time: Set(req.release_time.map(|d| d.to_rfc3339())),
        has_security_product: Set(req.has_security_product as i32),
        ip_address: Set(req.ip_address.clone()),
        ecs_login_method: Set(req.ecs_login_method.clone()),
        ecs_login_username: Set(req.ecs_login_username.clone()),
        ecs_initial_password: Set(req.ecs_initial_password.clone()),
        bastion_address: Set(req.bastion_address.clone()),
        bastion_admin_account: Set(req.bastion_admin_account.clone()),
        bastion_initial_password: Set(req.bastion_initial_password.clone()),
        serial_number: Set(None),     // Legacy field - kept for compatibility but not used
        rack_location: Set(None),     // Legacy field - kept for compatibility but not used
        hardware_model: Set(None),    // Legacy field - kept for compatibility but not used
        warranty_expiry: Set(None),   // Legacy field - kept for compatibility but not used
        agent_status: Set(None),      // Legacy field - kept for compatibility but not used
        ipmi_address: Set(None),      // Legacy field - kept for compatibility but not used
        remarks: Set(req.remarks.clone()),
        created_at: Set(created_at.to_string()),
        updated_at: Set(None),
        created_by: Set(Some(created_by.to_string())),
        updated_by: Set(None),
        // 申请与交付状态管理
        application_status: Set(req.application_status.clone()),
        delivery_status: Set(req.delivery_status.clone()),
        delivery_confirmed_at: Set(req.delivery_confirmed_at.map(|d| d.to_rfc3339())),
        delivery_confirmed_by: Set(req.delivery_confirmed_by.clone()),
        // 申请流程相关
        applicant: Set(req.applicant.clone()),
        department: Set(req.department.clone()),
        approver: Set(req.approver.clone()),
        approval_time: Set(req.approval_time.map(|d| d.to_rfc3339())),
        approval_remarks: Set(req.approval_remarks.clone()),
        rejection_reason: Set(req.rejection_reason.clone()),
        // 资源配置相关
        bandwidth_mbps: Set(req.bandwidth_mbps.map(|v| v as i32)),
        bandwidth_type: Set(req.bandwidth_type.clone()),
        public_ip_count: Set(req.public_ip_count.map(|v| v as i32)),
        network_type: Set(req.network_type.clone()),
        // 业务关联相关
        project_name: Set(req.project_name.clone()),
        project_code: Set(req.project_code.clone()),
        business_owner: Set(req.business_owner.clone()),
        tech_owner: Set(req.tech_owner.clone()),
        contact_phone: Set(req.contact_phone.clone()),
        // 费用相关
        billing_method: Set(req.billing_method.clone()),
        purchase_duration: Set(req.purchase_duration.map(|v| v as i32)),
        cost_center: Set(req.cost_center.clone()),
        // 合规相关
        security_level: Set(req.security_level.clone()),
        data_sensitivity: Set(req.data_sensitivity.clone()),
        // 其他
        purpose: Set(req.purpose.clone()),
        expected_delivery_time: Set(req.expected_delivery_time.map(|d| d.to_rfc3339())),
    };
    let result = db_resource.insert(conn).await?;
    let business_resource_id = result.id;

    // Insert detail record based on resource type
    match req.resource_type.as_str() {
        "physical" => {
            if let Some(info) = &req.physical_machine_info {
                insert_physical_machine(conn, business_resource_id, info, created_at).await?;
            }
        }
        "cloud" => {
            if let Some(info) = &req.cloud_vm_info {
                insert_cloud_virtual_machine(conn, business_resource_id, info, created_at).await?;
            }
        }
        _ => {}
    }

    Ok(business_resource_id)
}

pub async fn update_business_resource_by_id(
    conn: &DatabaseConnection,
    id: i32,
    req: &shared::UpdateBusinessResourceRequest,
    updated_by: &str,
) -> Result<(), DbErr> {
    let mut db_resource = business_resource::ActiveModel {
        id: Set(id),
        updated_at: Set(Some(chrono::Utc::now().to_rfc3339())),
        updated_by: Set(Some(updated_by.to_string())),
        ..Default::default()
    };

    // Update fields that are Some
    if let Some(v) = &req.resource_type { db_resource.resource_type = Set(v.clone()); }
    if let Some(v) = &req.ecs_name { db_resource.ecs_name = Set(v.clone()); }
    if let Some(v) = &req.ecs_status { db_resource.ecs_status = Set(v.clone()); }
    if let Some(v) = &req.cloud_region { db_resource.cloud_region = Set(v.clone()); }
    if let Some(v) = &req.cloud_category { db_resource.cloud_category = Set(v.clone()); }
    if let Some(v) = req.cloud_provider_config_id { db_resource.cloud_provider_config_id = Set(Some(v)); }
    if let Some(v) = &req.zone_name { db_resource.zone_name = Set(Some(v.clone())); }
    if let Some(v) = &req.platform_name { db_resource.platform_name = Set(Some(v.clone())); }
    if let Some(v) = &req.county_city { db_resource.county_city = Set(Some(v.clone())); }
    if let Some(v) = &req.vdc_name { db_resource.vdc_name = Set(Some(v.clone())); }
    if let Some(v) = &req.customer_name { db_resource.customer_name = Set(v.clone()); }
    if let Some(v) = &req.application_name { db_resource.application_name = Set(Some(v.clone())); }
    if let Some(v) = &req.contract_name { db_resource.contract_name = Set(Some(v.clone())); }
    if let Some(v) = &req.ecs_type { db_resource.ecs_type = Set(v.clone()); }
    if let Some(v) = &req.ecs_os { db_resource.ecs_os = Set(v.clone()); }
    if let Some(v) = req.cpu_cores { db_resource.cpu_cores = Set(v as i32); }
    if let Some(v) = req.memory_gb { db_resource.memory_gb = Set(v as i32); }
    if let Some(v) = &req.system_disk { db_resource.system_disk = Set(v.clone()); }
    if let Some(v) = req.system_disk_size_gb { db_resource.system_disk_size_gb = Set(v as i32); }
    if let Some(v) = &req.data_disk { db_resource.data_disk = Set(Some(v.clone())); }
    if let Some(v) = &req.completion_time { db_resource.completion_time = Set(Some(v.to_rfc3339())); }
    if let Some(v) = &req.release_time { db_resource.release_time = Set(Some(v.to_rfc3339())); }
    if let Some(v) = req.has_security_product { db_resource.has_security_product = Set(v as i32); }
    if let Some(v) = &req.ip_address { db_resource.ip_address = Set(v.clone()); }
    if let Some(v) = &req.ecs_login_method { db_resource.ecs_login_method = Set(Some(v.clone())); }
    if let Some(v) = &req.ecs_login_username { db_resource.ecs_login_username = Set(Some(v.clone())); }
    if let Some(v) = &req.ecs_initial_password { db_resource.ecs_initial_password = Set(Some(v.clone())); }
    if let Some(v) = &req.bastion_address { db_resource.bastion_address = Set(Some(v.clone())); }
    if let Some(v) = &req.bastion_admin_account { db_resource.bastion_admin_account = Set(Some(v.clone())); }
    if let Some(v) = &req.bastion_initial_password { db_resource.bastion_initial_password = Set(Some(v.clone())); }
    // Physical machine specific fields
    if let Some(ref pm_info) = req.physical_machine_info {
        if let Some(v) = &pm_info.serial_number { db_resource.serial_number = Set(Some(v.clone())); }
        if let Some(v) = &pm_info.rack_location { db_resource.rack_location = Set(Some(v.clone())); }
        if let Some(v) = &pm_info.hardware_model { db_resource.hardware_model = Set(Some(v.clone())); }
        if let Some(v) = &pm_info.warranty_expiry { db_resource.warranty_expiry = Set(Some(v.to_rfc3339())); }
        if let Some(v) = &pm_info.agent_status { db_resource.agent_status = Set(Some(v.clone())); }
        if let Some(v) = &pm_info.ipmi_address { db_resource.ipmi_address = Set(Some(v.clone())); }
    }
    if let Some(v) = &req.remarks { db_resource.remarks = Set(Some(v.clone())); }
    // 申请流程相关
    if let Some(v) = &req.applicant { db_resource.applicant = Set(Some(v.clone())); }
    if let Some(v) = &req.department { db_resource.department = Set(Some(v.clone())); }
    if let Some(v) = &req.approver { db_resource.approver = Set(Some(v.clone())); }
    if let Some(v) = &req.approval_time { db_resource.approval_time = Set(Some(v.to_rfc3339())); }
    if let Some(v) = &req.approval_remarks { db_resource.approval_remarks = Set(Some(v.clone())); }
    if let Some(v) = &req.rejection_reason { db_resource.rejection_reason = Set(Some(v.clone())); }
    // 资源配置相关
    if let Some(v) = req.bandwidth_mbps { db_resource.bandwidth_mbps = Set(Some(v as i32)); }
    if let Some(v) = &req.bandwidth_type { db_resource.bandwidth_type = Set(Some(v.clone())); }
    if let Some(v) = req.public_ip_count { db_resource.public_ip_count = Set(Some(v as i32)); }
    if let Some(v) = &req.network_type { db_resource.network_type = Set(Some(v.clone())); }
    // 业务关联相关
    if let Some(v) = &req.project_name { db_resource.project_name = Set(Some(v.clone())); }
    if let Some(v) = &req.project_code { db_resource.project_code = Set(Some(v.clone())); }
    if let Some(v) = &req.business_owner { db_resource.business_owner = Set(Some(v.clone())); }
    if let Some(v) = &req.tech_owner { db_resource.tech_owner = Set(Some(v.clone())); }
    if let Some(v) = &req.contact_phone { db_resource.contact_phone = Set(Some(v.clone())); }
    // 费用相关
    if let Some(v) = &req.billing_method { db_resource.billing_method = Set(Some(v.clone())); }
    if let Some(v) = req.purchase_duration { db_resource.purchase_duration = Set(Some(v as i32)); }
    if let Some(v) = &req.cost_center { db_resource.cost_center = Set(Some(v.clone())); }
    // 合规相关
    if let Some(v) = &req.security_level { db_resource.security_level = Set(Some(v.clone())); }
    if let Some(v) = &req.data_sensitivity { db_resource.data_sensitivity = Set(Some(v.clone())); }
    // 其他
    if let Some(v) = &req.purpose { db_resource.purpose = Set(Some(v.clone())); }
    if let Some(v) = &req.expected_delivery_time { db_resource.expected_delivery_time = Set(Some(v.to_rfc3339())); }
    // 状态管理
    if let Some(v) = &req.application_status { db_resource.application_status = Set(Some(v.clone())); }
    if let Some(v) = &req.delivery_status { db_resource.delivery_status = Set(Some(v.clone())); }
    if let Some(v) = &req.delivery_confirmed_at { db_resource.delivery_confirmed_at = Set(Some(v.to_rfc3339())); }
    if let Some(v) = &req.delivery_confirmed_by { db_resource.delivery_confirmed_by = Set(Some(v.clone())); }

    BusinessResource::update(db_resource).exec(conn).await?;
    Ok(())
}

pub async fn delete_business_resource_by_id(conn: &DatabaseConnection, id: i32) -> Result<(), DbErr> {
    let resource = BusinessResource::find_by_id(id).one(conn).await?;
    if let Some(resource) = resource {
        resource.delete(conn).await?;
    }
    Ok(())
}

pub async fn get_all_business_resources(conn: &DatabaseConnection) -> Result<Vec<business_resource::Model>, DbErr> {
    BusinessResource::find().order_by_desc(business_resource::Column::Id).all(conn).await
}

pub async fn get_business_resource_by_id(conn: &DatabaseConnection, id: i32) -> Result<Option<business_resource::Model>, DbErr> {
    BusinessResource::find_by_id(id).one(conn).await
}

// ============== Legacy wrapper functions for handlers ==============
// These provide compatibility with the old sqlx-based API by using the global DB connection

// User wrappers (for handlers/users.rs)
pub async fn get_users() -> Result<Vec<SharedUser>, DbErr> {
    let conn = get_db().ok_or(DbErr::Custom("Database not initialized".to_string()))?;
    get_users_with_conn(&conn).await
}

pub async fn insert_user(user: &SharedUser) -> Result<(), DbErr> {
    let conn = get_db().ok_or(DbErr::Custom("Database not initialized".to_string()))?;
    insert_user_with_conn(&conn, user).await
}

pub async fn delete_user(id: &str) -> Result<(), DbErr> {
    let conn = get_db().ok_or(DbErr::Custom("Database not initialized".to_string()))?;
    delete_user_by_id(&conn, id).await
}

pub async fn update_user(_id: &str, user: &SharedUser) -> Result<(), DbErr> {
    let conn = get_db().ok_or(DbErr::Custom("Database not initialized".to_string()))?;
    update_user_by_id(&conn, user).await
}

// Business resource wrappers
pub async fn get_business_resources() -> Result<Vec<business_resource::Model>, DbErr> {
    let conn = get_db().ok_or(DbErr::Custom("Database not initialized".to_string()))?;
    get_all_business_resources(&conn).await
}

pub async fn insert_business_resource_wrapper(req: &shared::CreateBusinessResourceRequest, created_at: &str, created_by: &str) -> Result<i32, DbErr> {
    let conn = get_db().ok_or(DbErr::Custom("Database not initialized".to_string()))?;
    insert_business_resource(&conn, req, created_at, created_by).await
}

pub async fn update_business_resource(id: i32, req: &shared::UpdateBusinessResourceRequest, updated_by: &str) -> Result<(), DbErr> {
    let conn = get_db().ok_or(DbErr::Custom("Database not initialized".to_string()))?;
    update_business_resource_by_id(&conn, id, req, updated_by).await
}

pub async fn delete_business_resource(id: i32) -> Result<(), DbErr> {
    let conn = get_db().ok_or(DbErr::Custom("Database not initialized".to_string()))?;
    delete_business_resource_by_id(&conn, id).await
}

// Cloud provider config wrappers
pub async fn insert_provider_config(
    zone_id: i32,
    platform_id: i32,
    provider: &str,
    region_id: &str,
    region_name: &str,
    account_name: &str,
    access_key_id: &str,
    access_key_secret: &str,
    remarks: Option<&str>,
    created_at: &str,
) -> Result<i32, DbErr> {
    let conn = get_db().ok_or(DbErr::Custom("Database not initialized".to_string()))?;
    insert_cloud_provider_config(&conn, zone_id, platform_id, provider, region_id, region_name, account_name, access_key_id, access_key_secret, remarks, created_at).await
}

pub async fn update_provider_config(
    id: i32,
    zone_id: i32,
    platform_id: i32,
    region_id: &str,
    region_name: &str,
    account_name: &str,
    access_key_id: &str,
    access_key_secret: &str,
    remarks: Option<&str>,
    status: &str,
    updated_at: Option<&str>,
) -> Result<(), DbErr> {
    let conn = get_db().ok_or(DbErr::Custom("Database not initialized".to_string()))?;
    update_cloud_provider_config_by_id(&conn, id, zone_id, platform_id, region_id, region_name, account_name, access_key_id, access_key_secret, remarks, status, updated_at).await
}

pub async fn delete_provider_config(id: i32) -> Result<(), DbErr> {
    let conn = get_db().ok_or(DbErr::Custom("Database not initialized".to_string()))?;
    delete_cloud_provider_config_by_id(&conn, id).await
}

// Cloud zone wrappers
pub async fn update_cloud_zone(
    id: i32,
    zone_name: Option<&str>,
    zone_code: Option<&str>,
    description: Option<&str>,
) -> Result<(), DbErr> {
    let conn = get_db().ok_or(DbErr::Custom("Database not initialized".to_string()))?;
    update_cloud_zone_by_id(&conn, id, zone_name, zone_code, description).await
}

pub async fn delete_cloud_zone(id: i32) -> Result<(), DbErr> {
    let conn = get_db().ok_or(DbErr::Custom("Database not initialized".to_string()))?;
    delete_cloud_zone_by_id(&conn, id).await
}

// Cloud platform wrappers
pub async fn update_cloud_platform(
    id: i32,
    zone_id: Option<i32>,
    platform_name: Option<&str>,
    platform_code: Option<&str>,
    description: Option<&str>,
) -> Result<(), DbErr> {
    let conn = get_db().ok_or(DbErr::Custom("Database not initialized".to_string()))?;
    update_cloud_platform_by_id(&conn, id, zone_id, platform_name, platform_code, description).await
}

pub async fn delete_cloud_platform(id: i32) -> Result<(), DbErr> {
    let conn = get_db().ok_or(DbErr::Custom("Database not initialized".to_string()))?;
    delete_cloud_platform_by_id(&conn, id).await
}

// Insert wrappers
pub async fn insert_cloud_zone_wrapper(
    zone_name: &str,
    zone_code: &str,
    description: Option<&str>,
    created_at: &str,
) -> Result<i32, DbErr> {
    let conn = get_db().ok_or(DbErr::Custom("Database not initialized".to_string()))?;
    insert_cloud_zone(&conn, zone_name, zone_code, description, created_at).await
}

pub async fn insert_cloud_platform_wrapper(
    zone_id: i32,
    platform_name: &str,
    platform_code: &str,
    description: Option<&str>,
    created_at: &str,
) -> Result<i32, DbErr> {
    let conn = get_db().ok_or(DbErr::Custom("Database not initialized".to_string()))?;
    insert_cloud_platform(&conn, zone_id, platform_name, platform_code, description, created_at).await
}

// AuditLog wrapper
pub async fn get_audit_logs(limit: Option<u64>) -> Result<Vec<audit_log::Model>, DbErr> {
    let conn = get_db().ok_or(DbErr::Custom("Database not initialized".to_string()))?;
    get_audit_logs_with_conn(&conn, limit).await
}

pub async fn get_audit_logs_with_conn(conn: &DatabaseConnection, limit: Option<u64>) -> Result<Vec<audit_log::Model>, DbErr> {
    let mut query = AuditLog::find();
    if let Some(limit) = limit {
        query = query.limit(limit);
    }
    query.order_by_desc(audit_log::Column::Timestamp).all(conn).await
}

pub async fn insert_audit_log_wrapper(log: &shared::AuditLog) -> Result<(), DbErr> {
    let conn = get_db().ok_or(DbErr::Custom("Database not initialized".to_string()))?;
    insert_audit_log(&conn, log).await
}

// Asset wrappers
pub async fn get_assets() -> Result<Vec<asset::Model>, DbErr> {
    let conn = get_db().ok_or(DbErr::Custom("Database not initialized".to_string()))?;
    get_all_assets(&conn).await
}

pub async fn insert_asset_wrapper(asset: &shared::Asset) -> Result<i64, DbErr> {
    let conn = get_db().ok_or(DbErr::Custom("Database not initialized".to_string()))?;
    insert_asset(&conn, asset).await
}

pub async fn update_asset(id: i32, asset: &shared::Asset) -> Result<(), DbErr> {
    let conn = get_db().ok_or(DbErr::Custom("Database not initialized".to_string()))?;
    update_asset_by_id(&conn, id, asset).await
}

pub async fn delete_asset(id: i32) -> Result<(), DbErr> {
    let conn = get_db().ok_or(DbErr::Custom("Database not initialized".to_string()))?;
    delete_asset_by_id(&conn, id).await
}

// Task wrappers
pub async fn get_tasks() -> Result<Vec<task::Model>, DbErr> {
    let conn = get_db().ok_or(DbErr::Custom("Database not initialized".to_string()))?;
    get_all_tasks(&conn).await
}

pub async fn insert_task_wrapper(task: &shared::Task) -> Result<(), DbErr> {
    let conn = get_db().ok_or(DbErr::Custom("Database not initialized".to_string()))?;
    insert_task(&conn, task).await
}

pub async fn update_task(id: &str, task: &shared::Task) -> Result<(), DbErr> {
    let conn = get_db().ok_or(DbErr::Custom("Database not initialized".to_string()))?;
    update_task_by_id(&conn, id, task).await
}

pub async fn delete_task(id: &str) -> Result<(), DbErr> {
    let conn = get_db().ok_or(DbErr::Custom("Database not initialized".to_string()))?;
    delete_task_by_id(&conn, id).await
}

// Risk wrappers
pub async fn get_risks() -> Result<Vec<risk::Model>, DbErr> {
    let conn = get_db().ok_or(DbErr::Custom("Database not initialized".to_string()))?;
    get_all_risks(&conn).await
}

pub async fn insert_risk_wrapper(risk: &shared::Risk) -> Result<(), DbErr> {
    let conn = get_db().ok_or(DbErr::Custom("Database not initialized".to_string()))?;
    insert_risk(&conn, risk).await
}

pub async fn update_risk(id: &str, risk: &shared::Risk) -> Result<(), DbErr> {
    let conn = get_db().ok_or(DbErr::Custom("Database not initialized".to_string()))?;
    update_risk_by_id(&conn, id, risk).await
}

pub async fn delete_risk(id: &str) -> Result<(), DbErr> {
    let conn = get_db().ok_or(DbErr::Custom("Database not initialized".to_string()))?;
    delete_risk_by_id(&conn, id).await
}

// NetworkZone wrappers (ZoneConfig)
pub async fn get_zones() -> Result<Vec<network_zone::Model>, DbErr> {
    let conn = get_db().ok_or(DbErr::Custom("Database not initialized".to_string()))?;
    get_all_zones(&conn).await
}

pub async fn insert_zone_wrapper(zone: &shared::ZoneConfig) -> Result<(), DbErr> {
    let conn = get_db().ok_or(DbErr::Custom("Database not initialized".to_string()))?;
    insert_zone(&conn, zone).await
}

pub async fn update_zone(id: &str, zone: &shared::ZoneConfig) -> Result<(), DbErr> {
    let conn = get_db().ok_or(DbErr::Custom("Database not initialized".to_string()))?;
    update_zone_by_id(&conn, id, zone).await
}

pub async fn delete_zone(id: &str) -> Result<(), DbErr> {
    let conn = get_db().ok_or(DbErr::Custom("Database not initialized".to_string()))?;
    delete_zone_by_id(&conn, id).await
}

// ============== Custom Role CRUD ==============

pub async fn get_all_custom_roles(conn: &DatabaseConnection) -> Result<Vec<custom_role::Model>, DbErr> {
    CustomRole::find().order_by_asc(custom_role::Column::Id).all(conn).await
}

pub async fn get_custom_role_by_id(conn: &DatabaseConnection, id: i32) -> Result<Option<custom_role::Model>, DbErr> {
    CustomRole::find_by_id(id).one(conn).await
}

pub async fn insert_custom_role(conn: &DatabaseConnection, role: &shared::CustomRole) -> Result<i32, DbErr> {
    let permissions_json = serde_json::to_string(&role.permissions).unwrap_or_default();

    let db_role = custom_role::ActiveModel {
        id: NotSet,
        name: Set(role.name.clone()),
        description: Set(role.description.clone()),
        permissions: Set(permissions_json),
        created_at: Set(role.created_at.clone()),
        updated_at: Set(role.updated_at.clone()),
    };

    let result = db_role.insert(conn).await?;
    Ok(result.id)
}

pub async fn update_custom_role_by_id(conn: &DatabaseConnection, id: i32, role: &shared::CustomRole) -> Result<(), DbErr> {
    let permissions_json = serde_json::to_string(&role.permissions).unwrap_or_default();

    let db_role = custom_role::ActiveModel {
        id: Set(id),
        name: Set(role.name.clone()),
        description: Set(role.description.clone()),
        permissions: Set(permissions_json),
        created_at: Set(role.created_at.clone()),
        updated_at: Set(role.updated_at.clone()),
    };

    CustomRole::update(db_role).exec(conn).await?;
    Ok(())
}

pub async fn delete_custom_role_by_id(conn: &DatabaseConnection, id: i32) -> Result<(), DbErr> {
    let role = CustomRole::find_by_id(id).one(conn).await?
        .ok_or_else(|| DbErr::Custom("Role not found".to_string()))?;
    role.delete(conn).await?;
    Ok(())
}

// Convert DbCustomRole to shared CustomRole
pub fn db_custom_role_to_shared(db: custom_role::Model) -> shared::CustomRole {
    let permissions = serde_json::from_str(&db.permissions).ok();

    shared::CustomRole {
        id: Some(db.id),
        name: db.name,
        description: db.description,
        permissions: permissions.unwrap_or_default(),
        created_at: db.created_at,
        updated_at: db.updated_at,
    }
}

// CustomRole wrappers for handlers
pub async fn get_custom_roles() -> Result<Vec<custom_role::Model>, DbErr> {
    let conn = get_db().ok_or(DbErr::Custom("Database not initialized".to_string()))?;
    get_all_custom_roles(&conn).await
}

pub async fn insert_custom_role_wrapper(role: &shared::CustomRole) -> Result<i32, DbErr> {
    let conn = get_db().ok_or(DbErr::Custom("Database not initialized".to_string()))?;
    insert_custom_role(&conn, role).await
}

pub async fn update_custom_role(id: i32, role: &shared::CustomRole) -> Result<(), DbErr> {
    let conn = get_db().ok_or(DbErr::Custom("Database not initialized".to_string()))?;
    update_custom_role_by_id(&conn, id, role).await
}

pub async fn delete_custom_role(id: i32) -> Result<(), DbErr> {
    let conn = get_db().ok_or(DbErr::Custom("Database not initialized".to_string()))?;
    delete_custom_role_by_id(&conn, id).await
}

// ============== Advanced Scan Task CRUD ==============

pub async fn get_all_advanced_scan_tasks(conn: &DatabaseConnection) -> Result<Vec<advanced_scan_task::Model>, DbErr> {
    AdvancedScanTask::find().order_by_desc(advanced_scan_task::Column::CreatedAt).all(conn).await
}

pub async fn get_advanced_scan_task_by_id(conn: &DatabaseConnection, id: &str) -> Result<Option<advanced_scan_task::Model>, DbErr> {
    AdvancedScanTask::find_by_id(id).one(conn).await
}

pub async fn insert_advanced_scan_task(conn: &DatabaseConnection, task: &shared::AdvancedScanTask) -> Result<(), DbErr> {
    let targets_json = serde_json::to_string(&task.targets).unwrap_or_default();
    let config_json = serde_json::to_string(&task.config).unwrap_or_default();
    let now = Utc::now().to_rfc3339();

    let db_task = advanced_scan_task::ActiveModel {
        id: Set(task.id.clone()),
        name: Set(task.name.clone()),
        targets: Set(targets_json),
        config: Set(config_json),
        status: Set(format!("{:?}", task.status)),
        progress: Set(task.progress),
        current_target: Set(task.current_target.clone()),
        scanned_count: Set(task.scanned_count as i32),
        total_count: Set(task.total_count as i32),
        start_time: Set(task.start_time.map(|d| d.to_rfc3339())),
        end_time: Set(task.end_time.map(|d| d.to_rfc3339())),
        created_by: Set(task.created_by.clone()),
        error_message: Set(task.error_message.clone()),
        created_at: Set(Some(now.clone())),
        updated_at: Set(Some(now)),
    };

    db_task.insert(conn).await?;
    Ok(())
}

pub async fn update_advanced_scan_task_by_id(conn: &DatabaseConnection, task: &shared::AdvancedScanTask) -> Result<(), DbErr> {
    let targets_json = serde_json::to_string(&task.targets).unwrap_or_default();
    let config_json = serde_json::to_string(&task.config).unwrap_or_default();
    let now = Utc::now().to_rfc3339();

    let db_task = advanced_scan_task::ActiveModel {
        id: Set(task.id.clone()),
        name: Set(task.name.clone()),
        targets: Set(targets_json),
        config: Set(config_json),
        status: Set(format!("{:?}", task.status)),
        progress: Set(task.progress),
        current_target: Set(task.current_target.clone()),
        scanned_count: Set(task.scanned_count as i32),
        total_count: Set(task.total_count as i32),
        start_time: Set(task.start_time.map(|d| d.to_rfc3339())),
        end_time: Set(task.end_time.map(|d| d.to_rfc3339())),
        created_by: Set(task.created_by.clone()),
        error_message: Set(task.error_message.clone()),
        created_at: NotSet,
        updated_at: Set(Some(now)),
    };

    AdvancedScanTask::update(db_task).exec(conn).await?;
    Ok(())
}

pub async fn delete_advanced_scan_task_by_id(conn: &DatabaseConnection, id: &str) -> Result<(), DbErr> {
    let task = AdvancedScanTask::find_by_id(id).one(conn).await?
        .ok_or_else(|| DbErr::Custom("Scan task not found".to_string()))?;
    task.delete(conn).await?;
    Ok(())
}

pub fn db_advanced_scan_task_to_shared(db: advanced_scan_task::Model) -> shared::AdvancedScanTask {
    use shared::{TaskStatus, AdvancedScanConfig};

    let targets: Vec<String> = serde_json::from_str(&db.targets).unwrap_or_default();
    let config: AdvancedScanConfig = serde_json::from_str(&db.config).ok().unwrap_or_default();

    let status = match db.status.as_str() {
        "Pending" => TaskStatus::Pending,
        "Running" => TaskStatus::Running,
        "Completed" => TaskStatus::Completed,
        "Failed" => TaskStatus::Failed,
        _ => TaskStatus::Pending,
    };

    shared::AdvancedScanTask {
        id: db.id,
        name: db.name,
        targets,
        config,
        status,
        progress: db.progress,
        current_target: db.current_target,
        scanned_count: db.scanned_count as u32,
        total_count: db.total_count as u32,
        start_time: db.start_time.as_ref().and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok().map(|dt| dt.with_timezone(&Utc))),
        end_time: db.end_time.as_ref().and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok().map(|dt| dt.with_timezone(&Utc))),
        results: vec![], // Loaded separately
        cloud_mappings: vec![],
        error_message: db.error_message,
        created_by: db.created_by,
    }
}

// ============== Quick Scan Result CRUD ==============

pub async fn insert_quick_scan_result(conn: &DatabaseConnection, result: &shared::QuickScanResult, task_id: &str) -> Result<(), DbErr> {
    let ports_json = serde_json::to_string(&result.open_ports).unwrap_or_default();

    let db_result = quick_scan_result::ActiveModel {
        id: Set(uuid::Uuid::new_v4().to_string()),
        task_id: Set(task_id.to_string()),
        ip: Set(result.ip.clone()),
        is_alive: Set(result.is_alive),
        open_ports: Set(ports_json),
        fingerprint: Set(result.fingerprint.clone()),
        scanned_at: Set(result.scanned_at.to_rfc3339()),
    };

    db_result.insert(conn).await?;
    Ok(())
}

pub async fn get_quick_scan_results_by_task(conn: &DatabaseConnection, task_id: &str) -> Result<Vec<quick_scan_result::Model>, DbErr> {
    quick_scan_result::Entity::find()
        .filter(quick_scan_result::Column::TaskId.eq(task_id))
        .order_by_asc(quick_scan_result::Column::ScannedAt)
        .all(conn)
        .await
}

pub fn db_quick_scan_result_to_shared(db: quick_scan_result::Model) -> shared::QuickScanResult {
    let open_ports: Vec<shared::PortInfo> = serde_json::from_str(&db.open_ports).unwrap_or_default();

    let scanned_at: chrono::DateTime<Utc> = chrono::DateTime::parse_from_rfc3339(&db.scanned_at)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(Utc::now);

    shared::QuickScanResult {
        ip: db.ip,
        is_alive: db.is_alive,
        open_ports,
        fingerprint: db.fingerprint,
        scanned_at,
    }
}

// Advanced scan wrappers for handlers
pub async fn get_advanced_scan_tasks() -> Result<Vec<advanced_scan_task::Model>, DbErr> {
    let conn = get_db().ok_or(DbErr::Custom("Database not initialized".to_string()))?;
    get_all_advanced_scan_tasks(&conn).await
}

pub async fn insert_advanced_scan_task_wrapper(task: &shared::AdvancedScanTask) -> Result<(), DbErr> {
    let conn = get_db().ok_or(DbErr::Custom("Database not initialized".to_string()))?;
    insert_advanced_scan_task(&conn, task).await
}

pub async fn update_advanced_scan_task(task: &shared::AdvancedScanTask) -> Result<(), DbErr> {
    let conn = get_db().ok_or(DbErr::Custom("Database not initialized".to_string()))?;
    update_advanced_scan_task_by_id(&conn, task).await
}

pub async fn delete_advanced_scan_task(id: &str) -> Result<(), DbErr> {
    let conn = get_db().ok_or(DbErr::Custom("Database not initialized".to_string()))?;
    delete_advanced_scan_task_by_id(&conn, id).await
}

pub async fn insert_quick_scan_result_wrapper(result: &shared::QuickScanResult, task_id: &str) -> Result<(), DbErr> {
    let conn = get_db().ok_or(DbErr::Custom("Database not initialized".to_string()))?;
    insert_quick_scan_result(&conn, result, task_id).await
}

pub async fn get_quick_scan_results(task_id: &str) -> Result<Vec<quick_scan_result::Model>, DbErr> {
    let conn = get_db().ok_or(DbErr::Custom("Database not initialized".to_string()))?;
    get_quick_scan_results_by_task(&conn, task_id).await
}

// ============== PhysicalMachine CRUD ==============

pub async fn insert_physical_machine(
    conn: &DatabaseConnection,
    business_resource_id: i32,
    info: &shared::CreatePhysicalMachineInfo,
    created_at: &str,
) -> Result<i32, DbErr> {
    let db_pm = physical_machine::ActiveModel {
        id: NotSet,
        business_resource_id: Set(business_resource_id),
        serial_number: Set(info.serial_number.clone()),
        rack_location: Set(info.rack_location.clone()),
        hardware_model: Set(info.hardware_model.clone()),
        warranty_expiry: Set(info.warranty_expiry.map(|d| d.to_rfc3339())),
        agent_status: Set(info.agent_status.clone()),
        ipmi_address: Set(info.ipmi_address.clone()),
        created_at: Set(created_at.to_string()),
        updated_at: Set(None),
    };
    let result = db_pm.insert(conn).await?;
    Ok(result.id)
}

pub async fn get_physical_machine_by_business_resource_id(
    conn: &DatabaseConnection,
    business_resource_id: i32,
) -> Result<Option<physical_machine::Model>, DbErr> {
    let pm = PhysicalMachine::find()
        .filter(physical_machine::Column::BusinessResourceId.eq(business_resource_id))
        .one(conn)
        .await?;
    Ok(pm)
}

pub async fn update_physical_machine(
    conn: &DatabaseConnection,
    business_resource_id: i32,
    info: &shared::UpdatePhysicalMachineInfo,
    updated_at: &str,
) -> Result<(), DbErr> {
    if let Some(pm) = get_physical_machine_by_business_resource_id(conn, business_resource_id).await? {
        let mut pm_active: physical_machine::ActiveModel = pm.into();

        if let Some(v) = &info.serial_number { pm_active.serial_number = Set(Some(v.clone())); }
        if let Some(v) = &info.rack_location { pm_active.rack_location = Set(Some(v.clone())); }
        if let Some(v) = &info.hardware_model { pm_active.hardware_model = Set(Some(v.clone())); }
        if let Some(v) = info.warranty_expiry { pm_active.warranty_expiry = Set(Some(v.to_rfc3339())); }
        if let Some(v) = &info.agent_status { pm_active.agent_status = Set(Some(v.clone())); }
        if let Some(v) = &info.ipmi_address { pm_active.ipmi_address = Set(Some(v.clone())); }
        pm_active.updated_at = Set(Some(updated_at.to_string()));

        pm_active.update(conn).await?;
    }
    Ok(())
}

pub async fn delete_physical_machine(
    conn: &DatabaseConnection,
    business_resource_id: i32,
) -> Result<(), DbErr> {
    PhysicalMachine::delete_many()
        .filter(physical_machine::Column::BusinessResourceId.eq(business_resource_id))
        .exec(conn)
        .await?;
    Ok(())
}

// ============== CloudVirtualMachine CRUD ==============

pub async fn insert_cloud_virtual_machine(
    conn: &DatabaseConnection,
    business_resource_id: i32,
    info: &shared::CreateCloudVirtualMachineInfo,
    created_at: &str,
) -> Result<i32, DbErr> {
    let security_group_ids_json = info.security_group_ids.as_ref()
        .map(|ids| serde_json::to_string(ids).unwrap_or_default());

    let db_cvm = cloud_virtual_machine::ActiveModel {
        id: NotSet,
        business_resource_id: Set(business_resource_id),
        billing_mode: Set(info.billing_mode.clone()),
        expire_time: Set(info.expire_time.map(|d| d.to_rfc3339())),
        charge_type: Set(info.charge_type.clone()),
        instance_charge_type: Set(info.instance_charge_type.clone()),
        internet_charge_type: Set(info.internet_charge_type.clone()),
        internet_max_bandwidth_out: Set(info.internet_max_bandwidth_out),
        image_id: Set(info.image_id.clone()),
        v_switch_id: Set(info.v_switch_id.clone()),
        vpc_id: Set(info.vpc_id.clone()),
        security_group_ids: Set(security_group_ids_json),
        created_at: Set(created_at.to_string()),
        updated_at: Set(None),
    };
    let result = db_cvm.insert(conn).await?;
    Ok(result.id)
}

pub async fn get_cloud_virtual_machine_by_business_resource_id(
    conn: &DatabaseConnection,
    business_resource_id: i32,
) -> Result<Option<cloud_virtual_machine::Model>, DbErr> {
    let cvm = CloudVirtualMachine::find()
        .filter(cloud_virtual_machine::Column::BusinessResourceId.eq(business_resource_id))
        .one(conn)
        .await?;
    Ok(cvm)
}

pub async fn update_cloud_virtual_machine(
    conn: &DatabaseConnection,
    business_resource_id: i32,
    info: &shared::UpdateCloudVirtualMachineInfo,
    updated_at: &str,
) -> Result<(), DbErr> {
    if let Some(cvm) = get_cloud_virtual_machine_by_business_resource_id(conn, business_resource_id).await? {
        let mut cvm_active: cloud_virtual_machine::ActiveModel = cvm.into();

        if let Some(v) = &info.billing_mode { cvm_active.billing_mode = Set(Some(v.clone())); }
        if let Some(v) = info.expire_time { cvm_active.expire_time = Set(Some(v.to_rfc3339())); }
        if let Some(v) = &info.charge_type { cvm_active.charge_type = Set(Some(v.clone())); }
        if let Some(v) = &info.instance_charge_type { cvm_active.instance_charge_type = Set(Some(v.clone())); }
        if let Some(v) = &info.internet_charge_type { cvm_active.internet_charge_type = Set(Some(v.clone())); }
        if let Some(v) = info.internet_max_bandwidth_out { cvm_active.internet_max_bandwidth_out = Set(Some(v)); }
        if let Some(v) = &info.image_id { cvm_active.image_id = Set(Some(v.clone())); }
        if let Some(v) = &info.v_switch_id { cvm_active.v_switch_id = Set(Some(v.clone())); }
        if let Some(v) = &info.vpc_id { cvm_active.vpc_id = Set(Some(v.clone())); }
        if let Some(v) = &info.security_group_ids {
            cvm_active.security_group_ids = Set(Some(serde_json::to_string(v).unwrap_or_default()));
        }
        cvm_active.updated_at = Set(Some(updated_at.to_string()));

        cvm_active.update(conn).await?;
    }
    Ok(())
}

pub async fn delete_cloud_virtual_machine(
    conn: &DatabaseConnection,
    business_resource_id: i32,
) -> Result<(), DbErr> {
    CloudVirtualMachine::delete_many()
        .filter(cloud_virtual_machine::Column::BusinessResourceId.eq(business_resource_id))
        .exec(conn)
        .await?;
    Ok(())
}

// ==================== Service Provider CRUD ====================

use crate::entities::{service_provider, machine_room, cloud_platform_config, security_product};
use sea_orm::QueryFilter;

/// 服务商数据返回结构
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DbServiceProvider {
    pub id: i32,
    pub provider_name: String,
    pub provider_code: String,
    pub short_name: String,
    pub logo_url: Option<String>,
    pub contact_person: String,
    pub contact_phone: String,
    pub contact_email: String,
    pub headquarters: String,
    pub service_area: String,
    pub business_license: String,
    pub remarks: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: Option<String>,
}

impl From<service_provider::Model> for DbServiceProvider {
    fn from(m: service_provider::Model) -> Self {
        Self {
            id: m.id,
            provider_name: m.provider_name,
            provider_code: m.provider_code,
            short_name: m.short_name,
            logo_url: m.logo_url,
            contact_person: m.contact_person,
            contact_phone: m.contact_phone,
            contact_email: m.contact_email,
            headquarters: m.headquarters,
            service_area: m.service_area,
            business_license: m.business_license,
            remarks: m.remarks,
            status: m.status,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

pub async fn get_all_service_providers(conn: &DatabaseConnection) -> Result<Vec<DbServiceProvider>, DbErr> {
    let providers = service_provider::Entity::find()
        .all(conn)
        .await?;
    Ok(providers.into_iter().map(|p| p.into()).collect())
}

pub async fn get_service_provider_by_id(conn: &DatabaseConnection, id: i32) -> Result<Option<DbServiceProvider>, DbErr> {
    let provider = service_provider::Entity::find_by_id(id)
        .one(conn)
        .await?;
    Ok(provider.map(|p| p.into()))
}

pub async fn insert_service_provider(
    conn: &DatabaseConnection,
    provider_name: &str,
    provider_code: &str,
    short_name: &str,
    logo_url: Option<&str>,
    contact_person: &str,
    contact_phone: &str,
    contact_email: &str,
    headquarters: &str,
    service_area: &str,
    business_license: &str,
    remarks: Option<&str>,
    status: &str,
    created_at: &str,
) -> Result<i32, DbErr> {
    let provider = service_provider::ActiveModel {
        id: NotSet,
        provider_name: Set(provider_name.to_string()),
        provider_code: Set(provider_code.to_string()),
        short_name: Set(short_name.to_string()),
        logo_url: Set(logo_url.map(|s| s.to_string())),
        contact_person: Set(contact_person.to_string()),
        contact_phone: Set(contact_phone.to_string()),
        contact_email: Set(contact_email.to_string()),
        headquarters: Set(headquarters.to_string()),
        service_area: Set(service_area.to_string()),
        business_license: Set(business_license.to_string()),
        remarks: Set(remarks.map(|s| s.to_string())),
        status: Set(status.to_string()),
        created_at: Set(created_at.to_string()),
        updated_at: Set(None),
    };
    let result = provider.insert(conn).await?;
    Ok(result.id)
}

pub async fn update_service_provider(
    conn: &DatabaseConnection,
    id: i32,
    provider_name: Option<&str>,
    provider_code: Option<&str>,
    short_name: Option<&str>,
    logo_url: Option<&str>,
    contact_person: Option<&str>,
    contact_phone: Option<&str>,
    contact_email: Option<&str>,
    headquarters: Option<&str>,
    service_area: Option<&str>,
    business_license: Option<&str>,
    remarks: Option<&str>,
    status: Option<&str>,
    updated_at: Option<&str>,
) -> Result<(), DbErr> {
    if let Some(provider) = service_provider::Entity::find_by_id(id).one(conn).await? {
        let mut provider_active: service_provider::ActiveModel = provider.into();
        if let Some(v) = provider_name { provider_active.provider_name = Set(v.to_string()); }
        if let Some(v) = provider_code { provider_active.provider_code = Set(v.to_string()); }
        if let Some(v) = short_name { provider_active.short_name = Set(v.to_string()); }
        if let Some(v) = logo_url { provider_active.logo_url = Set(Some(v.to_string())); }
        if let Some(v) = contact_person { provider_active.contact_person = Set(v.to_string()); }
        if let Some(v) = contact_phone { provider_active.contact_phone = Set(v.to_string()); }
        if let Some(v) = contact_email { provider_active.contact_email = Set(v.to_string()); }
        if let Some(v) = headquarters { provider_active.headquarters = Set(v.to_string()); }
        if let Some(v) = service_area { provider_active.service_area = Set(v.to_string()); }
        if let Some(v) = business_license { provider_active.business_license = Set(v.to_string()); }
        if let Some(v) = remarks { provider_active.remarks = Set(Some(v.to_string())); }
        if let Some(v) = status { provider_active.status = Set(v.to_string()); }
        if let Some(v) = updated_at { provider_active.updated_at = Set(Some(v.to_string())); }
        provider_active.update(conn).await?;
    }
    Ok(())
}

pub async fn delete_service_provider(conn: &DatabaseConnection, id: i32) -> Result<(), DbErr> {
    service_provider::Entity::delete_by_id(id)
        .exec(conn)
        .await?;
    Ok(())
}

// ==================== Machine Room CRUD ====================

/// 机房数据返回结构
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DbMachineRoom {
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

impl From<machine_room::Model> for DbMachineRoom {
    fn from(m: machine_room::Model) -> Self {
        Self {
            id: m.id,
            room_name: m.room_name,
            room_code: m.room_code,
            facility_type: m.facility_type,
            address: m.address,
            provider_id: m.provider_id,
            room_type: m.room_type,
            contact_person: m.contact_person,
            contact_phone: m.contact_phone,
            floor: m.floor,
            cabinet_count: m.cabinet_count,
            area_size: m.area_size,
            remarks: m.remarks,
            status: m.status,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

pub async fn get_all_machine_rooms(conn: &DatabaseConnection) -> Result<Vec<DbMachineRoom>, DbErr> {
    let rooms = machine_room::Entity::find()
        .all(conn)
        .await?;
    Ok(rooms.into_iter().map(|r| r.into()).collect())
}

pub async fn get_machine_room_by_id(conn: &DatabaseConnection, id: i32) -> Result<Option<DbMachineRoom>, DbErr> {
    let room = machine_room::Entity::find_by_id(id)
        .one(conn)
        .await?;
    Ok(room.map(|r| r.into()))
}

pub async fn insert_machine_room(
    conn: &DatabaseConnection,
    room_name: &str,
    room_code: &str,
    facility_type: &str,
    address: &str,
    provider_id: i32,
    room_type: &str,
    contact_person: &str,
    contact_phone: &str,
    floor: Option<&str>,
    cabinet_count: Option<i32>,
    area_size: Option<&str>,
    remarks: Option<&str>,
    status: &str,
    created_at: &str,
) -> Result<i32, DbErr> {
    let room = machine_room::ActiveModel {
        id: NotSet,
        room_name: Set(room_name.to_string()),
        room_code: Set(room_code.to_string()),
        facility_type: Set(facility_type.to_string()),
        address: Set(address.to_string()),
        provider_id: Set(provider_id),
        room_type: Set(room_type.to_string()),
        contact_person: Set(contact_person.to_string()),
        contact_phone: Set(contact_phone.to_string()),
        floor: Set(floor.map(|s| s.to_string())),
        cabinet_count: Set(cabinet_count),
        area_size: Set(area_size.map(|s| s.to_string())),
        remarks: Set(remarks.map(|s| s.to_string())),
        status: Set(status.to_string()),
        created_at: Set(created_at.to_string()),
        updated_at: Set(None),
    };
    let result = room.insert(conn).await?;
    Ok(result.id)
}

pub async fn update_machine_room(
    conn: &DatabaseConnection,
    id: i32,
    room_name: Option<&str>,
    room_code: Option<&str>,
    facility_type: Option<&str>,
    address: Option<&str>,
    provider_id: Option<i32>,
    room_type: Option<&str>,
    contact_person: Option<&str>,
    contact_phone: Option<&str>,
    floor: Option<&str>,
    cabinet_count: Option<i32>,
    area_size: Option<&str>,
    remarks: Option<&str>,
    status: Option<&str>,
    updated_at: Option<&str>,
) -> Result<(), DbErr> {
    if let Some(room) = machine_room::Entity::find_by_id(id).one(conn).await? {
        let mut room_active: machine_room::ActiveModel = room.into();
        if let Some(v) = room_name { room_active.room_name = Set(v.to_string()); }
        if let Some(v) = room_code { room_active.room_code = Set(v.to_string()); }
        if let Some(v) = facility_type { room_active.facility_type = Set(v.to_string()); }
        if let Some(v) = address { room_active.address = Set(v.to_string()); }
        if let Some(v) = provider_id { room_active.provider_id = Set(v); }
        if let Some(v) = room_type { room_active.room_type = Set(v.to_string()); }
        if let Some(v) = contact_person { room_active.contact_person = Set(v.to_string()); }
        if let Some(v) = contact_phone { room_active.contact_phone = Set(v.to_string()); }
        if let Some(v) = floor { room_active.floor = Set(Some(v.to_string())); }
        if let Some(v) = cabinet_count { room_active.cabinet_count = Set(Some(v)); }
        if let Some(v) = area_size { room_active.area_size = Set(Some(v.to_string())); }
        if let Some(v) = remarks { room_active.remarks = Set(Some(v.to_string())); }
        if let Some(v) = status { room_active.status = Set(v.to_string()); }
        if let Some(v) = updated_at { room_active.updated_at = Set(Some(v.to_string())); }
        room_active.update(conn).await?;
    }
    Ok(())
}

pub async fn delete_machine_room(conn: &DatabaseConnection, id: i32) -> Result<(), DbErr> {
    machine_room::Entity::delete_by_id(id)
        .exec(conn)
        .await?;
    Ok(())
}

// ==================== Cloud Platform Config CRUD ====================

/// 云平台配置数据返回结构
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DbCloudPlatformConfig {
    pub id: i32,
    pub platform_name: String,
    pub provider_id: i32,
    pub cloud_type: String,
    pub foundation: String,
    pub region_id: String,
    pub machine_room_id: i32,
    pub access_key_id: String,
    pub access_key_secret: String,
    pub remarks: Option<String>,
    pub status: String,
    pub last_test_time: Option<String>,
    pub last_test_result: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
}

impl From<cloud_platform_config::Model> for DbCloudPlatformConfig {
    fn from(m: cloud_platform_config::Model) -> Self {
        Self {
            id: m.id,
            platform_name: m.platform_name,
            provider_id: m.provider_id,
            cloud_type: m.cloud_type,
            foundation: m.foundation,
            region_id: m.region_id,
            machine_room_id: m.machine_room_id,
            access_key_id: m.access_key_id,
            access_key_secret: m.access_key_secret,
            remarks: m.remarks,
            status: m.status,
            last_test_time: m.last_test_time,
            last_test_result: m.last_test_result,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

pub async fn get_all_cloud_platform_configs(conn: &DatabaseConnection) -> Result<Vec<DbCloudPlatformConfig>, DbErr> {
    let configs = cloud_platform_config::Entity::find()
        .all(conn)
        .await?;
    Ok(configs.into_iter().map(|c| c.into()).collect())
}

pub async fn get_cloud_platform_config_by_id(conn: &DatabaseConnection, id: i32) -> Result<Option<DbCloudPlatformConfig>, DbErr> {
    let config = cloud_platform_config::Entity::find_by_id(id)
        .one(conn)
        .await?;
    Ok(config.map(|c| c.into()))
}

pub async fn insert_cloud_platform_config(
    conn: &DatabaseConnection,
    platform_name: &str,
    provider_id: i32,
    cloud_type: &str,
    foundation: &str,
    region_id: &str,
    machine_room_id: i32,
    access_key_id: &str,
    access_key_secret: &str,
    remarks: Option<&str>,
    status: &str,
    created_at: &str,
) -> Result<i32, DbErr> {
    let config = cloud_platform_config::ActiveModel {
        id: NotSet,
        platform_name: Set(platform_name.to_string()),
        provider_id: Set(provider_id),
        cloud_type: Set(cloud_type.to_string()),
        foundation: Set(foundation.to_string()),
        region_id: Set(region_id.to_string()),
        machine_room_id: Set(machine_room_id),
        access_key_id: Set(access_key_id.to_string()),
        access_key_secret: Set(access_key_secret.to_string()),
        remarks: Set(remarks.map(|s| s.to_string())),
        status: Set(status.to_string()),
        last_test_time: Set(None),
        last_test_result: Set(None),
        created_at: Set(created_at.to_string()),
        updated_at: Set(None),
    };
    let result = config.insert(conn).await?;
    Ok(result.id)
}

pub async fn update_cloud_platform_config(
    conn: &DatabaseConnection,
    id: i32,
    platform_name: Option<&str>,
    provider_id: Option<i32>,
    cloud_type: Option<&str>,
    foundation: Option<&str>,
    region_id: Option<&str>,
    machine_room_id: Option<i32>,
    access_key_id: Option<&str>,
    access_key_secret: Option<&str>,
    remarks: Option<&str>,
    status: Option<&str>,
    last_test_time: Option<&str>,
    last_test_result: Option<&str>,
    updated_at: Option<&str>,
) -> Result<(), DbErr> {
    if let Some(config) = cloud_platform_config::Entity::find_by_id(id).one(conn).await? {
        let mut config_active: cloud_platform_config::ActiveModel = config.into();
        if let Some(v) = platform_name { config_active.platform_name = Set(v.to_string()); }
        if let Some(v) = provider_id { config_active.provider_id = Set(v); }
        if let Some(v) = cloud_type { config_active.cloud_type = Set(v.to_string()); }
        if let Some(v) = foundation { config_active.foundation = Set(v.to_string()); }
        if let Some(v) = region_id { config_active.region_id = Set(v.to_string()); }
        if let Some(v) = machine_room_id { config_active.machine_room_id = Set(v); }
        if let Some(v) = access_key_id { config_active.access_key_id = Set(v.to_string()); }
        if let Some(v) = access_key_secret { config_active.access_key_secret = Set(v.to_string()); }
        if let Some(v) = remarks { config_active.remarks = Set(Some(v.to_string())); }
        if let Some(v) = status { config_active.status = Set(v.to_string()); }
        if let Some(v) = last_test_time { config_active.last_test_time = Set(Some(v.to_string())); }
        if let Some(v) = last_test_result { config_active.last_test_result = Set(Some(v.to_string())); }
        if let Some(v) = updated_at { config_active.updated_at = Set(Some(v.to_string())); }
        config_active.update(conn).await?;
    }
    Ok(())
}

pub async fn delete_cloud_platform_config(conn: &DatabaseConnection, id: i32) -> Result<(), DbErr> {
    cloud_platform_config::Entity::delete_by_id(id)
        .exec(conn)
        .await?;
    Ok(())
}

// ==================== Security Product CRUD ====================

/// 安全产品数据返回结构
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DbSecurityProduct {
    pub id: i32,
    pub name: String,
    pub category: String,
    pub vendor: String,
    pub model: String,
    pub version: String,
    pub serial_number: Option<String>,
    pub license_type: String,
    pub license_expiry: Option<String>,
    pub management_ip: Option<String>,
    pub deployment_mode: String,
    pub cloud_platform_id: Option<i32>,
    pub machine_room_id: Option<i32>,
    pub provider_id: Option<i32>,
    pub status: String,
    pub features: Option<String>,
    pub throughput: Option<String>,
    pub contact_person: String,
    pub contact_phone: String,
    pub remarks: Option<String>,
    pub created_at: String,
}

impl From<security_product::Model> for DbSecurityProduct {
    fn from(m: security_product::Model) -> Self {
        Self {
            id: m.id,
            name: m.name,
            category: m.category,
            vendor: m.vendor,
            model: m.model,
            version: m.version,
            serial_number: m.serial_number,
            license_type: m.license_type,
            license_expiry: m.license_expiry,
            management_ip: m.management_ip,
            deployment_mode: m.deployment_mode,
            cloud_platform_id: m.cloud_platform_id,
            machine_room_id: m.machine_room_id,
            provider_id: m.provider_id,
            status: m.status,
            features: m.features,
            throughput: m.throughput,
            contact_person: m.contact_person,
            contact_phone: m.contact_phone,
            remarks: m.remarks,
            created_at: m.created_at,
        }
    }
}

pub async fn get_all_security_products(conn: &DatabaseConnection) -> Result<Vec<DbSecurityProduct>, DbErr> {
    let products = security_product::Entity::find()
        .all(conn)
        .await?;
    Ok(products.into_iter().map(|p| p.into()).collect())
}

pub async fn get_security_product_by_id(conn: &DatabaseConnection, id: i32) -> Result<Option<DbSecurityProduct>, DbErr> {
    let product = security_product::Entity::find_by_id(id)
        .one(conn)
        .await?;
    Ok(product.map(|p| p.into()))
}

pub async fn insert_security_product(
    conn: &DatabaseConnection,
    name: &str,
    category: &str,
    vendor: &str,
    model: &str,
    version: &str,
    serial_number: Option<&str>,
    license_type: &str,
    license_expiry: Option<&str>,
    management_ip: Option<&str>,
    deployment_mode: &str,
    cloud_platform_id: Option<i32>,
    machine_room_id: Option<i32>,
    provider_id: Option<i32>,
    status: &str,
    features: Option<&str>,
    throughput: Option<&str>,
    contact_person: &str,
    contact_phone: &str,
    remarks: Option<&str>,
    created_at: &str,
) -> Result<i32, DbErr> {
    let product = security_product::ActiveModel {
        id: NotSet,
        name: Set(name.to_string()),
        category: Set(category.to_string()),
        vendor: Set(vendor.to_string()),
        model: Set(model.to_string()),
        version: Set(version.to_string()),
        serial_number: Set(serial_number.map(|s| s.to_string())),
        license_type: Set(license_type.to_string()),
        license_expiry: Set(license_expiry.map(|s| s.to_string())),
        management_ip: Set(management_ip.map(|s| s.to_string())),
        deployment_mode: Set(deployment_mode.to_string()),
        cloud_platform_id: Set(cloud_platform_id),
        machine_room_id: Set(machine_room_id),
        provider_id: Set(provider_id),
        status: Set(status.to_string()),
        features: Set(features.map(|s| s.to_string())),
        throughput: Set(throughput.map(|s| s.to_string())),
        contact_person: Set(contact_person.to_string()),
        contact_phone: Set(contact_phone.to_string()),
        remarks: Set(remarks.map(|s| s.to_string())),
        created_at: Set(created_at.to_string()),
    };
    let result = product.insert(conn).await?;
    Ok(result.id)
}

pub async fn update_security_product(
    conn: &DatabaseConnection,
    id: i32,
    name: Option<&str>,
    category: Option<&str>,
    vendor: Option<&str>,
    model: Option<&str>,
    version: Option<&str>,
    serial_number: Option<&str>,
    license_type: Option<&str>,
    license_expiry: Option<&str>,
    management_ip: Option<&str>,
    deployment_mode: Option<&str>,
    cloud_platform_id: Option<i32>,
    machine_room_id: Option<i32>,
    provider_id: Option<i32>,
    status: Option<&str>,
    features: Option<&str>,
    throughput: Option<&str>,
    contact_person: Option<&str>,
    contact_phone: Option<&str>,
    remarks: Option<&str>,
) -> Result<(), DbErr> {
    if let Some(product) = security_product::Entity::find_by_id(id).one(conn).await? {
        let mut product_active: security_product::ActiveModel = product.into();
        if let Some(v) = name { product_active.name = Set(v.to_string()); }
        if let Some(v) = category { product_active.category = Set(v.to_string()); }
        if let Some(v) = vendor { product_active.vendor = Set(v.to_string()); }
        if let Some(v) = model { product_active.model = Set(v.to_string()); }
        if let Some(v) = version { product_active.version = Set(v.to_string()); }
        if let Some(v) = serial_number { product_active.serial_number = Set(Some(v.to_string())); }
        if let Some(v) = license_type { product_active.license_type = Set(v.to_string()); }
        if let Some(v) = license_expiry { product_active.license_expiry = Set(Some(v.to_string())); }
        if let Some(v) = management_ip { product_active.management_ip = Set(Some(v.to_string())); }
        if let Some(v) = deployment_mode { product_active.deployment_mode = Set(v.to_string()); }
        if let Some(v) = cloud_platform_id { product_active.cloud_platform_id = Set(Some(v)); }
        if let Some(v) = machine_room_id { product_active.machine_room_id = Set(Some(v)); }
        if let Some(v) = provider_id { product_active.provider_id = Set(Some(v)); }
        if let Some(v) = status { product_active.status = Set(v.to_string()); }
        if let Some(v) = features { product_active.features = Set(Some(v.to_string())); }
        if let Some(v) = throughput { product_active.throughput = Set(Some(v.to_string())); }
        if let Some(v) = contact_person { product_active.contact_person = Set(v.to_string()); }
        if let Some(v) = contact_phone { product_active.contact_phone = Set(v.to_string()); }
        if let Some(v) = remarks { product_active.remarks = Set(Some(v.to_string())); }
        product_active.update(conn).await?;
    }
    Ok(())
}

pub async fn delete_security_product(conn: &DatabaseConnection, id: i32) -> Result<(), DbErr> {
    security_product::Entity::delete_by_id(id)
        .exec(conn)
        .await?;
    Ok(())
}

