use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::sync::{Arc, RwLock as StdRwLock};
use tokio::sync::RwLock as TokioRwLock;
use tower_cookies::CookieManagerLayer;
use tower_http::trace::TraceLayer;
// use utoipa_swagger_ui::SwaggerUi;  // 暂时禁用
use chrono::Utc;
use sea_orm::ConnectionTrait;
use shared::{PasswordPolicy, Role, User};
use uuid::Uuid;

const DEFAULT_BOOTSTRAP_ORG_NAME: &str = "RustSet 默认组织";
const DEFAULT_BOOTSTRAP_ORG_CODE: &str = "RUSTSET_DEFAULT";
const DEFAULT_BOOTSTRAP_DEPT_NAME: &str = "默认运维部";
const DEFAULT_BOOTSTRAP_DEPT_CODE: &str = "DEFAULT_OPS";

// 加载 .env 文件
fn load_env() {
    let _ = dotenvy::from_filename(".env");
    let _ = dotenvy::from_filename("backend/.env");
}

fn bool_env(name: &str, default: bool) -> bool {
    match std::env::var(name) {
        Ok(value) => match value.trim().to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => true,
            "0" | "false" | "no" | "off" => false,
            _ => default,
        },
        Err(_) => default,
    }
}

fn should_seed_default_users() -> bool {
    bool_env("BOOTSTRAP_DEFAULT_USERS", cfg!(debug_assertions))
}

fn build_default_users() -> Vec<User> {
    vec![
        {
            let admin_password = "admin";
            let password_hash = password::hash_password(admin_password).unwrap_or_else(|e| {
                tracing::error!("Failed to hash admin password: {}", e);
                admin_password.to_string()
            });
            User {
                id: Uuid::new_v4().to_string(),
                username: "admin".to_string(),
                password: password_hash,
                role: Role::SysAdmin,
                permissions: Some(shared::Permissions::sys_admin()),
                created_at: Utc::now(),
                password_changed_at: Some(Utc::now()),
                password_strength: Some(
                    password::get_strength_label(password::check_password_strength(admin_password))
                        .to_string(),
                ),
                force_password_change: Some(true),
                last_login_at: None,
                email: Some("admin@rustset.local".to_string()),
                phone: Some("".to_string()),
                status: Some("active".to_string()),
                real_name: Some("系统管理员".to_string()),
                organization_id: None,
                department_id: None,
                failed_login_attempts: Some(0),
                locked_until: None,
            }
        },
        {
            let sec_password = "sec";
            let password_hash = password::hash_password(sec_password).unwrap_or_else(|e| {
                tracing::error!("Failed to hash sec password: {}", e);
                sec_password.to_string()
            });
            User {
                id: Uuid::new_v4().to_string(),
                username: "sec".to_string(),
                password: password_hash,
                role: Role::SecAdmin,
                permissions: Some(shared::Permissions::sec_admin()),
                created_at: Utc::now(),
                password_changed_at: Some(Utc::now()),
                password_strength: Some(
                    password::get_strength_label(password::check_password_strength(sec_password))
                        .to_string(),
                ),
                force_password_change: Some(true),
                last_login_at: None,
                email: Some("sec@rustset.local".to_string()),
                phone: Some("".to_string()),
                status: Some("active".to_string()),
                real_name: Some("安全管理员".to_string()),
                organization_id: None,
                department_id: None,
                failed_login_attempts: Some(0),
                locked_until: None,
            }
        },
        {
            let audit_password = "audit";
            let password_hash = password::hash_password(audit_password).unwrap_or_else(|e| {
                tracing::error!("Failed to hash audit password: {}", e);
                audit_password.to_string()
            });
            User {
                id: Uuid::new_v4().to_string(),
                username: "audit".to_string(),
                password: password_hash,
                role: Role::Auditor,
                permissions: Some(shared::Permissions::auditor()),
                created_at: Utc::now(),
                password_changed_at: Some(Utc::now()),
                password_strength: Some(
                    password::get_strength_label(password::check_password_strength(audit_password))
                        .to_string(),
                ),
                force_password_change: Some(true),
                last_login_at: None,
                email: Some("audit@rustset.local".to_string()),
                phone: Some("".to_string()),
                status: Some("active".to_string()),
                real_name: Some("审计员".to_string()),
                organization_id: None,
                department_id: None,
                failed_login_attempts: Some(0),
                locked_until: None,
            }
        },
    ]
}

#[derive(Clone, Copy)]
struct DefaultUserBinding {
    organization_id: i32,
    department_id: i32,
}

async fn ensure_default_org_department(
    conn: &sea_orm::DatabaseConnection,
) -> Result<DefaultUserBinding, sea_orm::DbErr> {
    let organization_id = match database::get_all_organizations(conn)
        .await?
        .into_iter()
        .find(|item| item.code == DEFAULT_BOOTSTRAP_ORG_CODE)
    {
        Some(item) => item.id,
        None => {
            let now = Utc::now().to_rfc3339();
            database::insert_organization(
                conn,
                DEFAULT_BOOTSTRAP_ORG_NAME,
                DEFAULT_BOOTSTRAP_ORG_CODE,
                "active",
                Some("开发环境默认组织"),
                &now,
            )
            .await?
        }
    };

    let department_id = match database::get_all_departments(conn)
        .await?
        .into_iter()
        .find(|item| {
            item.organization_id == organization_id && item.code == DEFAULT_BOOTSTRAP_DEPT_CODE
        }) {
        Some(item) => item.id,
        None => {
            let now = Utc::now().to_rfc3339();
            database::insert_department(
                conn,
                organization_id,
                DEFAULT_BOOTSTRAP_DEPT_NAME,
                DEFAULT_BOOTSTRAP_DEPT_CODE,
                None,
                1,
                "active",
                Some("开发环境默认部门"),
                &now,
            )
            .await?
        }
    };

    Ok(DefaultUserBinding {
        organization_id,
        department_id,
    })
}

async fn ensure_builtin_user_bindings(
    conn: &sea_orm::DatabaseConnection,
    users: Vec<User>,
) -> Result<Vec<User>, sea_orm::DbErr> {
    if !users
        .iter()
        .any(|user| crate::utils::is_builtin_system_account(&user.username))
    {
        return Ok(users);
    }

    let default_binding = ensure_default_org_department(conn).await?;
    let mut next_users = Vec::with_capacity(users.len());

    for mut user in users {
        if !crate::utils::is_builtin_system_account(&user.username) {
            next_users.push(user);
            continue;
        }

        match (user.organization_id, user.department_id) {
            (None, None) => {
                user.organization_id = Some(default_binding.organization_id);
                user.department_id = Some(default_binding.department_id);
                database::update_user_by_id(conn, &user).await?;
                tracing::info!(
                    "Bound builtin account {} to default organization/department",
                    user.username
                );
            }
            (Some(org_id), None) if org_id == default_binding.organization_id => {
                user.department_id = Some(default_binding.department_id);
                database::update_user_by_id(conn, &user).await?;
                tracing::info!(
                    "Bound builtin account {} to default department",
                    user.username
                );
            }
            (None, Some(_)) => {
                tracing::warn!(
                    "Builtin account {} is missing organization but already has a department; leaving unchanged",
                    user.username
                );
            }
            (Some(org_id), None) => {
                tracing::warn!(
                    "Builtin account {} is missing department under non-default organization {}; leaving unchanged",
                    user.username,
                    org_id
                );
            }
            (Some(_), Some(_)) => {}
        }

        next_users.push(user);
    }

    Ok(next_users)
}

async fn load_users_from_db(conn: &sea_orm::DatabaseConnection) -> Vec<shared::User> {
    match database::get_users_with_conn(conn).await {
        Ok(users) => users,
        Err(e) => {
            tracing::warn!("Error loading users from database: {}", e);
            vec![]
        }
    }
}

async fn load_audit_logs_from_db(conn: &sea_orm::DatabaseConnection) -> Vec<shared::AuditLog> {
    use crate::database::get_audit_logs_with_conn as get_audit_logs_db;

    match get_audit_logs_db(conn, Some(1000)).await {
        Ok(logs) => logs
            .into_iter()
            .map(|db_log| shared::AuditLog {
                id: db_log.id,
                user_id: db_log.user_id,
                username: db_log.username,
                action: db_log.action,
                target: db_log.target,
                details: db_log.details,
                timestamp: chrono::DateTime::parse_from_rfc3339(&db_log.timestamp)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
            .collect(),
        Err(e) => {
            tracing::warn!("Error loading audit logs from database: {}", e);
            vec![]
        }
    }
}

mod auth;
mod config;
mod database;
mod entities;
mod handlers;
mod middleware;
mod migration;
mod openapi;
mod password;
mod scanners;
mod state;
mod utils;

use handlers::{
    advanced_scan::{
        cancel_advanced_scan, delete_advanced_scan, execute_advanced_scan, export_scan_results,
        get_advanced_task, get_advanced_tasks, get_scan_engines_status, scan_progress_stream,
    },
    assets::{
        add_asset, add_asset_port, bind_port, delete_asset, delete_asset_port, get_assets,
        update_asset, update_asset_port,
    },
    auth::{login, logout, refresh_token},
    business_applications::{
        create_application_endpoint, create_business_application, delete_application_endpoint,
        delete_business_application, get_business_applications, update_application_endpoint,
        update_business_application,
    },
    business_resources::{
        create_business_resource, delete_business_resource, get_business_resources,
        update_business_resource,
    },
    cloud_platform_configs::{
        create_cloud_platform_config, delete_cloud_platform_config, get_cloud_platform_config,
        get_cloud_platform_configs, update_cloud_platform_config,
    },
    cloud_platforms::{
        create_cloud_platform, delete_cloud_platform, get_cloud_platform, get_cloud_platforms,
        get_platforms_by_zone, update_cloud_platform,
    },
    cloud_providers::{
        create_cloud_provider_config, delete_cloud_provider_config,
        get_active_cloud_provider_configs, get_cloud_provider_config, get_cloud_provider_configs,
        get_cloud_provider_options, test_cloud_provider_connection, update_cloud_provider_config,
    },
    cloud_service_assets::{get_cloud_service_assets, get_cloud_service_stats},
    cloud_zones::{
        create_cloud_zone, delete_cloud_zone, get_cloud_zone, get_cloud_zones, update_cloud_zone,
    },
    dashboard::get_dashboard_summary,
    departments::{
        create_department, delete_department_handler, get_departments, update_department_handler,
    },
    health::{health_check, liveness_check, metrics, readiness_check},
    ip_zones::{create_ip_zone, delete_ip_zone, find_zone_by_ip, get_ip_zones, update_ip_zone},
    logs::get_audit_logs,
    machine_rooms::{
        create_machine_room, delete_machine_room, get_machine_room, get_machine_rooms,
        update_machine_room,
    },
    organizations::{
        create_organization, delete_organization_handler, get_organizations,
        update_organization_handler,
    },
    port_details::{
        batch_bind_ports, create_port_detail, delete_port_detail, get_port_detail,
        get_port_details, update_port_detail,
    },
    resource_tickets::{
        approve_ticket, create_resource_ticket, delete_resource_ticket, deliver_ticket,
        get_resource_ticket, get_resource_tickets, provision_ticket, update_resource_ticket,
    },
    risks::{get_risks, resolve_risk, update_risk_status},
    scanners::{
        batch_scan_ips, create_scanner, delete_scanner, get_scan_result, get_scan_results,
        get_scanners, scan_ip, update_scanner,
    },
    security_products::{
        create_security_product, delete_security_product, get_security_product,
        get_security_products, update_security_product,
    },
    service_providers::{
        create_service_provider, delete_service_provider, get_service_provider,
        get_service_providers, update_service_provider,
    },
    tasks::{create_task, delete_task, get_tasks, trigger_scan, update_task},
    users::{
        change_password, create_user, delete_user, get_current_user_info, get_password_policy,
        get_users, update_current_user_profile, update_password_policy, update_user,
        update_user_permissions,
    },
    zones::{create_zone, delete_zone, get_zones, update_zone},
};
use middleware::auth_middleware::auth_middleware;
use middleware::cors::create_cors_layer;
use middleware::performance::performance_monitoring;
use middleware::rate_limit::{init_rate_limiter, rate_limit_middleware, RateLimitConfig};
use middleware::session::create_session_layer_sync;
use state::AppState;

#[tokio::main]
async fn main() {
    // 加载 .env 文件（优先级最低，不会覆盖现有环境变量）
    load_env();

    tracing_subscriber::fmt::init();

    let default_users = build_default_users();

    // 初始化数据库 (使用 SeaORM)
    let db_config = config::DatabaseConfig::from_env();
    if let Err(error) = database::init_db(&db_config.connection_string).await {
        tracing::error!("Failed to initialize database: {}", error);
        return;
    }
    tracing::info!("Database initialized: type={}", db_config.db_type);

    // 获取数据库连接
    let Some(db_conn) = database::get_db() else {
        tracing::error!("Database initialization completed without a shared connection");
        return;
    };

    // 添加新的申请与交付状态字段（如果不存在）
    // 注意：这些 SQL 语句会在表已存在时执行，用于升级现有数据库
    let alter_sqls = vec![
        "ALTER TABLE business_resources ADD COLUMN application_status TEXT DEFAULT '待审核'",
        "ALTER TABLE business_resources ADD COLUMN delivery_status TEXT DEFAULT '待交付'",
        "ALTER TABLE business_resources ADD COLUMN delivery_confirmed_at TEXT",
        "ALTER TABLE business_resources ADD COLUMN delivery_confirmed_by TEXT",
    ];
    // 尝试执行 ALTER TABLE，如果字段已存在会忽略错误
    for sql in alter_sqls {
        // 使用 execute_unprepared 执行原生 SQL
        match db_conn.execute_unprepared(sql).await {
            Ok(result) => {
                tracing::info!("Added new column(s), result: {:?}", result);
            }
            Err(e) => {
                // 字段可能已存在，忽略错误
                tracing::debug!("Column migration note: {}", e);
            }
        }
    }

    let loaded_users = load_users_from_db(&db_conn).await;
    tracing::info!("Loaded {} users from database", loaded_users.len());

    // 仅在显式启用或 debug 构建时，向空库注入默认账号。
    let initial_users = if loaded_users.is_empty() && should_seed_default_users() {
        tracing::info!("Database empty, seeding default users");
        for user in &default_users {
            let user_with_defaults = shared::User {
                last_login_at: Some(Utc::now()),
                ..user.clone()
            };
            if let Err(e) = database::insert_user_with_conn(&db_conn, &user_with_defaults).await {
                tracing::error!("Failed to insert initial user {}: {}", user.username, e);
            }
        }
        load_users_from_db(&db_conn).await
    } else if loaded_users.is_empty() {
        tracing::warn!(
            "Database is empty and BOOTSTRAP_DEFAULT_USERS is disabled; no default users were created."
        );
        loaded_users
    } else {
        loaded_users
    };

    let initial_users = if should_seed_default_users() {
        match ensure_builtin_user_bindings(&db_conn, initial_users).await {
            Ok(users) => users,
            Err(error) => {
                tracing::error!("Failed to ensure builtin user bindings: {}", error);
                load_users_from_db(&db_conn).await
            }
        }
    } else {
        initial_users
    };

    let loaded_audit_logs = load_audit_logs_from_db(&db_conn).await;
    tracing::info!(
        "Loaded {} audit logs from database",
        loaded_audit_logs.len()
    );

    // 初始化扫描管理器
    let scan_manager = scanners::engine::ScanManager::new().await.ok();

    // 初始化 Rate Limiter
    let rate_limit_config = RateLimitConfig {
        requests_per_minute: 60,    // 每分钟60次请求
        block_duration_seconds: 60, // 超限后阻塞60秒
    };
    init_rate_limiter(rate_limit_config.clone());
    tracing::info!(
        "Rate limiter initialized: {} requests/minute, {}s block duration",
        rate_limit_config.requests_per_minute,
        rate_limit_config.block_duration_seconds
    );

    let state = AppState {
        assets: Arc::new(StdRwLock::new(vec![])),
        tasks: Arc::new(StdRwLock::new(vec![])),
        risks: Arc::new(StdRwLock::new(vec![])),
        zones: Arc::new(StdRwLock::new(vec![])),
        users: Arc::new(StdRwLock::new(initial_users)),
        audit_logs: Arc::new(StdRwLock::new(loaded_audit_logs)),
        advanced_tasks: Arc::new(StdRwLock::new(vec![])),
        custom_roles: Arc::new(StdRwLock::new(vec![])),
        scan_manager: Arc::new(TokioRwLock::new(scan_manager)),
        password_policy: Arc::new(StdRwLock::new(PasswordPolicy::default())),
        password_history: Arc::new(StdRwLock::new(vec![])),
        port_details: Arc::new(StdRwLock::new(vec![])),
        scanners: Arc::new(StdRwLock::new(vec![])),
        scan_results: Arc::new(StdRwLock::new(vec![])),
    };

    // 不需要创建 session layer，使用中间件方式
    // session 会通过 session_middleware 中间件注入

    let auth_state = state.clone();

    let app = Router::new()
        // Health & Metrics
        .route("/api/health", get(health_check))
        .route("/api/ready", get(readiness_check))
        .route("/api/live", get(liveness_check))
        .route("/api/metrics", get(metrics))
        // Auth
        .route("/api/login", post(login))
        .route("/api/logout", post(logout))
        .route("/api/refresh-token", post(refresh_token))
        .route(
            "/api/users/me",
            get(get_current_user_info).put(update_current_user_profile),
        ) // 必须在 {id} 之前
        .route("/api/users", get(get_users).post(create_user))
        .route("/api/users/{id}", put(update_user).delete(delete_user))
        .route("/api/users/{id}/permissions", put(update_user_permissions))
        .route(
            "/api/organizations",
            get(get_organizations).post(create_organization),
        )
        .route(
            "/api/organizations/{id}",
            put(update_organization_handler).delete(delete_organization_handler),
        )
        .route(
            "/api/departments",
            get(get_departments).post(create_department),
        )
        .route(
            "/api/departments/{id}",
            put(update_department_handler).delete(delete_department_handler),
        )
        .route("/api/users/change-password", post(change_password))
        .route("/api/dashboard-summary", get(get_dashboard_summary))
        .route(
            "/api/password-policy",
            get(get_password_policy).put(update_password_policy),
        )
        .route("/api/logs", get(get_audit_logs))
        // Roles
        .route(
            "/api/roles",
            get(handlers::roles::get_roles).post(handlers::roles::create_role),
        )
        .route(
            "/api/roles/{id}",
            get(handlers::roles::get_role)
                .put(handlers::roles::update_role)
                .delete(handlers::roles::delete_role),
        )
        // Assets
        .route("/api/assets", get(get_assets).post(add_asset))
        .route("/api/assets/{id}", delete(delete_asset).put(update_asset))
        .route("/api/assets/{id}/ports", post(add_asset_port))
        .route(
            "/api/assets/{id}/ports/{port}",
            delete(delete_asset_port).put(update_asset_port),
        )
        .route("/api/assets/{ip}/ports/{port}/bind", post(bind_port))
        // Tasks & Risks
        .route("/api/tasks", get(get_tasks).post(create_task))
        .route("/api/tasks/{id}", delete(delete_task).put(update_task))
        .route("/api/risks", get(get_risks))
        .route("/api/risks/{id}/resolve", post(resolve_risk))
        .route("/api/risks/{id}/status/{status}", post(update_risk_status))
        // Zones
        .route("/api/zones", get(get_zones).post(create_zone))
        .route("/api/zones/{id}", delete(delete_zone).put(update_zone))
        // Business Resources (业务申请)
        .route(
            "/api/business-resources",
            get(get_business_resources).post(create_business_resource),
        )
        .route(
            "/api/business-resources/{id}",
            put(update_business_resource).delete(delete_business_resource),
        )
        .route(
            "/api/business-applications",
            get(get_business_applications).post(create_business_application),
        )
        .route(
            "/api/business-applications/{id}",
            put(update_business_application).delete(delete_business_application),
        )
        .route(
            "/api/business-applications/endpoints",
            post(create_application_endpoint),
        )
        .route(
            "/api/business-applications/endpoints/{id}",
            put(update_application_endpoint).delete(delete_application_endpoint),
        )
        // Advanced Scanning (新增高级扫描 API)
        .route("/api/scan/advanced", post(execute_advanced_scan))
        .route("/api/scan/advanced/tasks", get(get_advanced_tasks))
        .route(
            "/api/scan/advanced/tasks/{id}",
            get(get_advanced_task).delete(delete_advanced_scan),
        )
        .route(
            "/api/scan/advanced/tasks/{id}/cancel",
            post(cancel_advanced_scan),
        )
        .route(
            "/api/scan/advanced/tasks/{id}/export",
            get(export_scan_results),
        )
        .route(
            "/api/scan/advanced/engines/status",
            get(get_scan_engines_status),
        )
        .route(
            "/api/scan/advanced/tasks/{id}/progress",
            get(scan_progress_stream),
        )
        // Cloud Provider Configuration (云厂商对接)
        .route(
            "/api/cloud-provider-configs",
            get(get_cloud_provider_configs).post(create_cloud_provider_config),
        )
        .route(
            "/api/cloud-provider-configs/options",
            get(get_cloud_provider_options),
        )
        .route(
            "/api/cloud-provider-configs/active",
            get(get_active_cloud_provider_configs),
        )
        .route(
            "/api/cloud-provider-configs/{id}",
            get(get_cloud_provider_config)
                .put(update_cloud_provider_config)
                .delete(delete_cloud_provider_config),
        )
        .route(
            "/api/cloud-provider-configs/{id}/test",
            post(test_cloud_provider_connection),
        )
        // Providers/Vendors (运营商/厂家管理)
        .route(
            "/api/providers",
            get(get_cloud_zones).post(create_cloud_zone),
        )
        .route(
            "/api/providers/{id}",
            get(get_cloud_zone)
                .put(update_cloud_zone)
                .delete(delete_cloud_zone),
        )
        // Cloud Services (云服务管理)
        .route(
            "/api/cloud-services",
            get(get_cloud_platforms).post(create_cloud_platform),
        )
        .route(
            "/api/cloud-services/{id}",
            get(get_cloud_platform)
                .put(update_cloud_platform)
                .delete(delete_cloud_platform),
        )
        .route(
            "/api/cloud-services/provider/{provider_id}",
            get(get_platforms_by_zone),
        )
        // Cloud Service Assets (云服务资产 - 统一视图)
        .route("/api/cloud-service-assets", get(get_cloud_service_assets))
        .route(
            "/api/cloud-service-assets/stats",
            get(get_cloud_service_stats),
        )
        // IP Zones (IP 区域管理)
        .route("/api/ip-zones", get(get_ip_zones).post(create_ip_zone))
        .route(
            "/api/ip-zones/{id}",
            delete(delete_ip_zone).put(update_ip_zone),
        )
        .route("/api/ip-zones/find", get(find_zone_by_ip))
        // Port Details (端口详细信息管理)
        .route(
            "/api/port-details",
            get(get_port_details).post(create_port_detail),
        )
        .route(
            "/api/port-details/{id}",
            get(get_port_detail)
                .put(update_port_detail)
                .delete(delete_port_detail),
        )
        .route("/api/port-details/batch-bind", post(batch_bind_ports))
        // Scanner Configuration (扫描器配置管理)
        .route("/api/scanners", get(get_scanners).post(create_scanner))
        .route(
            "/api/scanners/{id}",
            put(update_scanner).delete(delete_scanner),
        )
        // Scanners (扫描器接口)
        .route("/api/scan-ip", post(scan_ip))
        .route("/api/batch-scan-ips", post(batch_scan_ips))
        .route("/api/scan-results", get(get_scan_results))
        .route("/api/scan-results/{id}", get(get_scan_result))
        // Cloud Assets (Multi-Cloud Management) - 暂时禁用
        // .route("/api/cloud-assets", get(get_cloud_assets).post(create_cloud_asset))
        // .route("/api/cloud-assets/stats", get(get_cloud_asset_stats))
        // .route("/api/cloud-assets/sync", post(sync_cloud_assets))
        // .route("/api/cloud-assets/{id}", get(get_cloud_asset).put(update_cloud_asset).delete(delete_cloud_asset))
        // Scan
        .route("/api/scan", post(trigger_scan))
        // Resource Tickets (资源工单)
        .route(
            "/api/resource-tickets",
            get(get_resource_tickets).post(create_resource_ticket),
        )
        .route(
            "/api/resource-tickets/{id}",
            get(get_resource_ticket)
                .put(update_resource_ticket)
                .delete(delete_resource_ticket),
        )
        .route("/api/resource-tickets/{id}/approve", post(approve_ticket))
        .route(
            "/api/resource-tickets/{id}/provision",
            post(provision_ticket),
        )
        .route("/api/resource-tickets/{id}/deliver", post(deliver_ticket))
        // Service Providers (服务商管理)
        .route(
            "/api/service-providers",
            get(get_service_providers).post(create_service_provider),
        )
        .route(
            "/api/service-providers/{id}",
            get(get_service_provider)
                .put(update_service_provider)
                .delete(delete_service_provider),
        )
        // Machine Rooms (机房管理)
        .route(
            "/api/machine-rooms",
            get(get_machine_rooms).post(create_machine_room),
        )
        .route(
            "/api/machine-rooms/{id}",
            get(get_machine_room)
                .put(update_machine_room)
                .delete(delete_machine_room),
        )
        // Security Products (安全产品管理)
        .route(
            "/api/security-products",
            get(get_security_products).post(create_security_product),
        )
        .route(
            "/api/security-products/{id}",
            get(get_security_product)
                .put(update_security_product)
                .delete(delete_security_product),
        )
        // Cloud Platform Configs (云平台配置管理)
        .route(
            "/api/cloud-platform-configs",
            get(get_cloud_platform_configs).post(create_cloud_platform_config),
        )
        .route(
            "/api/cloud-platform-configs/{id}",
            get(get_cloud_platform_config)
                .put(update_cloud_platform_config)
                .delete(delete_cloud_platform_config),
        )
        .with_state(state)
        // Swagger UI - 暂时禁用，网络问题导致下载失败
        // .merge(
        //     SwaggerUi::new("/api-docs")
        //         .url("/api-docs/openapi.json", openapi::ApiDoc::openapi())
        // )
        // Layer order is important! With Axum .layer(), each call wraps the previous service.
        // For incoming requests, layers run from last-added to first-added (outside to inside).
        // For outgoing responses, layers run from first-added to last-added (inside to outside).
        //
        // The correct order for sessions:
        // 1. CookieManager (outermost) - must run first to parse cookies
        // 2. SessionManager - loads session from cookie
        // 3. Auth middleware - validates user from session
        // 4. Rate limiting, performance, trace, CORS
        //
        // Current layer chain (last to run = first to execute for incoming):
        // - CORS (outermost, runs last)
        // - Trace
        // - Performance monitoring
        // - Rate limiting
        // - CookieManager
        // - SessionManager
        // - Auth middleware (innermost, runs first)
        .layer(axum::middleware::from_fn_with_state(
            auth_state,
            auth_middleware,
        ))
        .layer(create_session_layer_sync())
        .layer(CookieManagerLayer::new())
        .layer(axum::middleware::from_fn(rate_limit_middleware))
        .layer(axum::middleware::from_fn(performance_monitoring))
        .layer(TraceLayer::new_for_http())
        .layer(create_cors_layer());

    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(3003);
    let bind_addr = format!("{}:{}", host, port);

    tracing::info!("Backend listening on {}", bind_addr);
    let listener = match tokio::net::TcpListener::bind(&bind_addr).await {
        Ok(listener) => listener,
        Err(error) => {
            tracing::error!(
                "Failed to bind backend listener on {}: {}",
                bind_addr,
                error
            );
            return;
        }
    };

    if let Err(error) = axum::serve(listener, app).await {
        tracing::error!("Backend server exited with error: {}", error);
    }
}
