use crate::database::{db_custom_role_to_shared, get_custom_roles};
use crate::middleware::{ApiError, AuthUser};
use crate::state::AppState;
use chrono::Utc;
use ipnetwork::IpNetwork;
use shared::{AuditLog, CustomRole, NetworkZone, Permissions, Role, User, ZoneConfig};
use std::net::IpAddr;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

pub async fn get_current_user_from_auth(
    auth_user: &AuthUser,
    users: &Arc<RwLock<Vec<User>>>,
) -> Option<User> {
    if crate::database::get_db().is_some() {
        return match crate::database::get_user_by_id(&auth_user.user_id).await {
            Ok(Some(user)) if user.username == auth_user.username => Some(user),
            _ => None,
        };
    }

    users
        .read()
        .ok()?
        .iter()
        .find(|u| u.id == auth_user.user_id && u.username == auth_user.username)
        .cloned()
}

pub async fn require_current_user_from_auth(
    auth_user: &AuthUser,
    state: &AppState,
) -> Result<User, ApiError> {
    get_current_user_from_auth(auth_user, &state.users)
        .await
        .ok_or_else(|| ApiError::unauthorized("Unauthorized"))
}

pub fn user_has_any_role(user: &User, allowed_roles: &[Role]) -> bool {
    allowed_roles.contains(&user.role)
}

pub fn role_has_any_role(role: &Role, allowed_roles: &[Role]) -> bool {
    allowed_roles.iter().any(|allowed| role == allowed)
}

pub fn ensure_user_has_any_role(
    user: &User,
    allowed_roles: &[Role],
    message: &str,
) -> Result<(), ApiError> {
    if user_has_any_role(user, allowed_roles) {
        Ok(())
    } else {
        Err(ApiError::forbidden(message))
    }
}

pub fn ensure_role_has_any_role(
    role: &Role,
    allowed_roles: &[Role],
    message: &str,
) -> Result<(), ApiError> {
    if role_has_any_role(role, allowed_roles) {
        Ok(())
    } else {
        Err(ApiError::forbidden(message))
    }
}

pub fn sync_cached_user(users: &Arc<RwLock<Vec<User>>>, user: &User) {
    if let Ok(mut users_guard) = users.write() {
        if let Some(existing) = users_guard
            .iter_mut()
            .find(|existing| existing.id == user.id)
        {
            *existing = user.clone();
        } else {
            users_guard.push(user.clone());
        }
    }
}

pub fn sync_cached_users(users: &Arc<RwLock<Vec<User>>>, next_users: Vec<User>) {
    if let Ok(mut users_guard) = users.write() {
        *users_guard = next_users;
    }
}

pub fn is_builtin_system_account(username: &str) -> bool {
    matches!(username, "admin" | "sec" | "audit")
}

pub fn remove_cached_user(users: &Arc<RwLock<Vec<User>>>, user_id: &str) {
    if let Ok(mut users_guard) = users.write() {
        users_guard.retain(|user| user.id != user_id);
    }
}

pub fn builtin_permissions_for_role(role: &Role) -> Option<Permissions> {
    match role {
        Role::SysAdmin => Some(Permissions::sys_admin()),
        Role::SecAdmin => Some(Permissions::sec_admin()),
        Role::Auditor => Some(Permissions::auditor()),
        Role::Custom(_) => None,
    }
}

pub async fn find_custom_role_by_name(state: &AppState, role_name: &str) -> Option<CustomRole> {
    if let Ok(db_roles) = get_custom_roles().await {
        let roles: Vec<CustomRole> = db_roles.into_iter().map(db_custom_role_to_shared).collect();
        if let Ok(mut cache) = state.custom_roles.write() {
            *cache = roles.clone();
        }
        return roles.into_iter().find(|role| role.name == role_name);
    }

    state
        .custom_roles
        .read()
        .ok()?
        .iter()
        .find(|role| role.name == role_name)
        .cloned()
}

pub async fn permissions_for_role_assignment(
    state: &AppState,
    role: &Role,
) -> Result<Option<Permissions>, String> {
    if let Some(permissions) = builtin_permissions_for_role(role) {
        return Ok(Some(permissions));
    }

    match role {
        Role::Custom(role_name) => find_custom_role_by_name(state, role_name)
            .await
            .map(|role| Some(role.permissions))
            .ok_or_else(|| format!("自定义角色“{}”不存在", role_name)),
        _ => Ok(None),
    }
}

pub async fn effective_permissions(state: &AppState, user: &User) -> Permissions {
    if let Some(permissions) = builtin_permissions_for_role(&user.role) {
        return permissions;
    }

    if let Some(permissions) = user.permissions.clone() {
        return permissions;
    }

    if let Role::Custom(role_name) = &user.role {
        if let Some(role) = find_custom_role_by_name(state, role_name).await {
            return role.permissions;
        }
    }

    Permissions::default()
}

pub fn log_action(
    logs: &Arc<RwLock<Vec<AuditLog>>>,
    user: &User,
    action: &str,
    target: &str,
    details: &str,
) {
    let log_entry = AuditLog {
        id: Uuid::new_v4().to_string(),
        user_id: user.id.clone(),
        username: user.username.clone(),
        action: action.to_string(),
        target: target.to_string(),
        details: details.to_string(),
        timestamp: Utc::now(),
    };

    // Add to in-memory storage
    if let Ok(mut logs_guard) = logs.write() {
        logs_guard.push(log_entry.clone());
    } else {
        tracing::error!("Failed to acquire audit log write lock");
    }

    // Try to persist to database asynchronously (don't block if it fails)
    let log_entry_for_db = log_entry;
    let log_entry_for_stream = log_entry_for_db.clone();
    tokio::spawn(async move {
        let _ = crate::database::insert_audit_log_wrapper(&log_entry_for_db).await;
    });
    crate::redis::publish_audit_event(log_entry_for_stream);
    tokio::spawn(async {
        let _ = crate::redis::cache_delete("rustset:cache:audit_logs").await;
    });
}

/// Log action with AuthUser (JWT authentication)
pub fn log_action_auth(
    logs: &Arc<RwLock<Vec<AuditLog>>>,
    user: &AuthUser,
    action: &str,
    target: &str,
    details: &str,
) {
    let log_entry = AuditLog {
        id: Uuid::new_v4().to_string(),
        user_id: user.user_id.clone(),
        username: user.username.clone(),
        action: action.to_string(),
        target: target.to_string(),
        details: details.to_string(),
        timestamp: Utc::now(),
    };

    // Add to in-memory storage
    if let Ok(mut logs_guard) = logs.write() {
        logs_guard.push(log_entry.clone());
    } else {
        tracing::error!("Failed to acquire audit log write lock");
    }

    // Try to persist to database asynchronously (don't block if it fails)
    let log_entry_for_db = log_entry;
    let log_entry_for_stream = log_entry_for_db.clone();
    tokio::spawn(async move {
        let _ = crate::database::insert_audit_log_wrapper(&log_entry_for_db).await;
    });
    crate::redis::publish_audit_event(log_entry_for_stream);
    tokio::spawn(async {
        let _ = crate::redis::cache_delete("rustset:cache:audit_logs").await;
    });
}

pub fn determine_zone(ip_str: &str, zones: &[ZoneConfig]) -> NetworkZone {
    if let Ok(ip) = ip_str.parse::<IpAddr>() {
        let mut matching_zones: Vec<&ZoneConfig> = zones
            .iter()
            .filter(|z| {
                if let Ok(net) = z.cidr.parse::<IpNetwork>() {
                    net.contains(ip)
                } else {
                    false
                }
            })
            .collect();

        matching_zones.sort_by(|a, b| b.priority.cmp(&a.priority));

        if let Some(best_match) = matching_zones.first() {
            return match best_match.name.as_str() {
                "Internet" => NetworkZone::Internet,
                "DMZ" => NetworkZone::DMZ,
                "Intranet" => NetworkZone::Intranet,
                other => NetworkZone::Custom(other.to_string()),
            };
        }
    }
    NetworkZone::Internet
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::port_details::PortDetail;
    use crate::handlers::scanners::ScanResult;
    use crate::state::AppState;
    use shared::{PasswordPolicy, Role};
    use std::sync::{Arc, RwLock};
    use tokio::sync::RwLock as TokioRwLock;

    fn create_test_app_state() -> AppState {
        AppState {
            assets: Arc::new(RwLock::new(Vec::new())),
            tasks: Arc::new(RwLock::new(Vec::new())),
            risks: Arc::new(RwLock::new(Vec::new())),
            zones: Arc::new(RwLock::new(Vec::new())),
            users: Arc::new(RwLock::new(Vec::new())),
            audit_logs: Arc::new(RwLock::new(Vec::new())),
            advanced_tasks: Arc::new(RwLock::new(Vec::new())),
            custom_roles: Arc::new(RwLock::new(Vec::new())),
            scan_manager: Arc::new(TokioRwLock::new(None)),
            password_policy: Arc::new(RwLock::new(PasswordPolicy::default())),
            password_history: Arc::new(RwLock::new(Vec::new())),
            port_details: Arc::new(RwLock::new(Vec::<PortDetail>::new())),
            scanners: Arc::new(RwLock::new(Vec::new())),
            scan_results: Arc::new(RwLock::new(Vec::<ScanResult>::new())),
        }
    }

    fn create_test_user() -> User {
        User {
            id: "test-id".to_string(),
            username: "testuser".to_string(),
            real_name: None,
            password: "hashed_password".to_string(),
            role: Role::Auditor,
            permissions: None,
            created_at: Utc::now(),
            password_changed_at: Some(Utc::now()),
            password_strength: Some("medium".to_string()),
            force_password_change: Some(false),
            last_login_at: None,
            email: None,
            phone: None,
            status: Some("active".to_string()),
            organization_id: None,
            department_id: None,
            failed_login_attempts: Some(0),
            locked_until: None,
        }
    }

    #[tokio::test]
    async fn effective_permissions_prefers_builtin_permissions_for_system_roles() {
        let state = create_test_app_state();
        let mut user = create_test_user();
        user.role = Role::SysAdmin;
        user.permissions = Some(Permissions {
            can_view_resource_tickets: false,
            can_create_resource_tickets: false,
            can_approve_resource_tickets: false,
            can_provision_resource_tickets: false,
            can_deliver_resource_tickets: false,
            can_delete_resource_tickets: false,
            resource_ticket_scope: shared::DataScope::SelfOnly,
            ..Permissions::default()
        });

        let permissions = effective_permissions(&state, &user).await;

        assert!(permissions.can_view_resource_tickets);
        assert!(permissions.can_create_resource_tickets);
        assert!(permissions.can_approve_resource_tickets);
        assert!(permissions.can_provision_resource_tickets);
        assert!(permissions.can_deliver_resource_tickets);
        assert!(permissions.can_delete_resource_tickets);
        assert_eq!(permissions.resource_ticket_scope, shared::DataScope::All);
    }

    #[tokio::test]
    async fn test_log_action_creates_entry() {
        let logs = Arc::new(RwLock::new(Vec::new()));
        let user = create_test_user();

        log_action(&logs, &user, "TEST_ACTION", "test_target", "test details");

        // Give time for the async operation to complete
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let logs_guard = logs.read().unwrap();
        assert_eq!(logs_guard.len(), 1);

        let log_entry = &logs_guard[0];
        assert_eq!(log_entry.user_id, "test-id");
        assert_eq!(log_entry.username, "testuser");
        assert_eq!(log_entry.action, "TEST_ACTION");
        assert_eq!(log_entry.target, "test_target");
        assert_eq!(log_entry.details, "test details");
    }

    #[test]
    fn test_determine_zone_internet() {
        let zones = vec![ZoneConfig {
            id: "1".to_string(),
            name: "Intranet".to_string(),
            cidr: "10.0.0.0/8".to_string(),
            priority: 100,
            cloud_platform_id: None,
            cloud_platform_name: None,
            machine_room_id: None,
            machine_room_name: None,
        }];

        let result = determine_zone("8.8.8.8", &zones);
        assert!(matches!(result, NetworkZone::Internet));
    }

    #[test]
    fn test_determine_zone_intranet() {
        let zones = vec![ZoneConfig {
            id: "1".to_string(),
            name: "Intranet".to_string(),
            cidr: "10.0.0.0/8".to_string(),
            priority: 100,
            cloud_platform_id: None,
            cloud_platform_name: None,
            machine_room_id: None,
            machine_room_name: None,
        }];

        let result = determine_zone("10.0.1.5", &zones);
        assert!(matches!(result, NetworkZone::Intranet));
    }

    #[test]
    fn test_determine_zone_dmz() {
        let zones = vec![ZoneConfig {
            id: "1".to_string(),
            name: "DMZ".to_string(),
            cidr: "192.168.1.0/24".to_string(),
            priority: 100,
            cloud_platform_id: None,
            cloud_platform_name: None,
            machine_room_id: None,
            machine_room_name: None,
        }];

        let result = determine_zone("192.168.1.10", &zones);
        assert!(matches!(result, NetworkZone::DMZ));
    }

    #[test]
    fn test_determine_zone_custom() {
        let zones = vec![ZoneConfig {
            id: "1".to_string(),
            name: "CustomZone".to_string(),
            cidr: "172.16.0.0/16".to_string(),
            priority: 100,
            cloud_platform_id: None,
            cloud_platform_name: None,
            machine_room_id: None,
            machine_room_name: None,
        }];

        let result = determine_zone("172.16.5.10", &zones);
        assert!(matches!(result, NetworkZone::Custom(_)));
        if let NetworkZone::Custom(name) = result {
            assert_eq!(name, "CustomZone");
        }
    }

    #[test]
    fn test_determine_zone_invalid_ip() {
        let zones = vec![ZoneConfig {
            id: "1".to_string(),
            name: "Intranet".to_string(),
            cidr: "10.0.0.0/8".to_string(),
            priority: 100,
            cloud_platform_id: None,
            cloud_platform_name: None,
            machine_room_id: None,
            machine_room_name: None,
        }];

        let result = determine_zone("invalid-ip", &zones);
        assert!(matches!(result, NetworkZone::Internet));
    }

    #[test]
    fn test_determine_zone_priority() {
        let zones = vec![
            ZoneConfig {
                id: "1".to_string(),
                name: "LowPriority".to_string(),
                cidr: "10.0.0.0/8".to_string(),
                priority: 50,
                cloud_platform_id: None,
                cloud_platform_name: None,
                machine_room_id: None,
                machine_room_name: None,
            },
            ZoneConfig {
                id: "2".to_string(),
                name: "HighPriority".to_string(),
                cidr: "10.0.0.0/8".to_string(),
                priority: 100,
                cloud_platform_id: None,
                cloud_platform_name: None,
                machine_room_id: None,
                machine_room_name: None,
            },
        ];

        let result = determine_zone("10.0.1.5", &zones);
        assert!(matches!(result, NetworkZone::Custom(_)));
        if let NetworkZone::Custom(name) = result {
            assert_eq!(name, "HighPriority");
        }
    }

    #[test]
    fn test_determine_zone_invalid_cidr() {
        let zones = vec![ZoneConfig {
            id: "1".to_string(),
            name: "InvalidZone".to_string(),
            cidr: "invalid-cidr".to_string(),
            priority: 100,
            cloud_platform_id: None,
            cloud_platform_name: None,
            machine_room_id: None,
            machine_room_name: None,
        }];

        // Invalid CIDR should be skipped, return Internet default
        let result = determine_zone("10.0.1.5", &zones);
        assert!(matches!(result, NetworkZone::Internet));
    }
}
