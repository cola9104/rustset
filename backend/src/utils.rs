use axum::http::{HeaderMap, header::AUTHORIZATION};
use std::sync::{Arc, RwLock};
use shared::{User, AuditLog, ZoneConfig, NetworkZone};
use crate::middleware::AuthUser;
use chrono::Utc;
use uuid::Uuid;
use std::net::IpAddr;
use ipnetwork::IpNetwork;

pub fn get_current_user(headers: &HeaderMap, users: &Arc<RwLock<Vec<User>>>) -> Option<User> {
    let auth_header = headers.get(AUTHORIZATION)?;
    let auth_value = auth_header.to_str().ok()?.trim();
    let token = auth_value
        .strip_prefix("Bearer ")
        .or_else(|| auth_value.strip_prefix("bearer "))?;

    let claims = crate::auth::verify_token(token).ok()?;
    let users_guard = users.read().ok()?;

    users_guard
        .iter()
        .find(|u| u.id == claims.user_id && u.username == claims.username)
        .cloned()
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
    {
        let mut logs_guard = logs.write().unwrap();
        logs_guard.push(log_entry.clone());
    }

    // Try to persist to database asynchronously (don't block if it fails)
    let log_entry_for_db = log_entry;
    tokio::spawn(async move {
        let _ = crate::database::insert_audit_log_wrapper(&log_entry_for_db).await;
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
    {
        let mut logs_guard = logs.write().unwrap();
        logs_guard.push(log_entry.clone());
    }

    // Try to persist to database asynchronously (don't block if it fails)
    let log_entry_for_db = log_entry;
    tokio::spawn(async move {
        let _ = crate::database::insert_audit_log_wrapper(&log_entry_for_db).await;
    });
}

pub fn determine_zone(ip_str: &str, zones: &[ZoneConfig]) -> NetworkZone {
    if let Ok(ip) = ip_str.parse::<IpAddr>() {
        let mut matching_zones: Vec<&ZoneConfig> = zones.iter()
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
    use std::sync::{Arc, RwLock};
    use shared::Role;
    use axum::http::HeaderMap;

    fn setup_jwt_secret() {
        std::env::set_var("JWT_SECRET", "test-jwt-secret");
    }

    fn create_test_user() -> User {
        User {
            id: "test-id".to_string(),
            username: "testuser".to_string(),
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
            failed_login_attempts: Some(0),
            locked_until: None,
        }
    }

    #[test]
    fn test_get_current_user_with_valid_token() {
        setup_jwt_secret();
        let user = create_test_user();
        let users = Arc::new(RwLock::new(vec![user.clone()]));
        let token = crate::auth::generate_token(&user).unwrap();

        let mut headers = HeaderMap::new();
        headers.insert("Authorization", format!("Bearer {}", token).parse().unwrap());

        let result = get_current_user(&headers, &users);
        assert!(result.is_some());
        assert_eq!(result.unwrap().username, "testuser");
    }

    #[test]
    fn test_get_current_user_with_whitespace_token() {
        setup_jwt_secret();
        let user = create_test_user();
        let users = Arc::new(RwLock::new(vec![user.clone()]));
        let token = crate::auth::generate_token(&user).unwrap();

        let mut headers = HeaderMap::new();
        headers.insert("Authorization", format!("Bearer {}", token).parse().unwrap());

        let result = get_current_user(&headers, &users);
        assert!(result.is_some());
        assert_eq!(result.unwrap().username, "testuser");
    }

    #[test]
    fn test_get_current_user_with_no_header() {
        let users = Arc::new(RwLock::new(vec![create_test_user()]));
        let headers = HeaderMap::new();

        let result = get_current_user(&headers, &users);
        assert!(result.is_none());
    }

    #[test]
    fn test_get_current_user_with_invalid_token() {
        setup_jwt_secret();
        let users = Arc::new(RwLock::new(vec![create_test_user()]));
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", "Bearer invalid.jwt.token".parse().unwrap());

        let result = get_current_user(&headers, &users);
        assert!(result.is_none());
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
        let zones = vec![
            ZoneConfig {
                id: "1".to_string(),
                name: "Intranet".to_string(),
                cidr: "10.0.0.0/8".to_string(),
                priority: 100,
            },
        ];

        let result = determine_zone("8.8.8.8", &zones);
        assert!(matches!(result, NetworkZone::Internet));
    }

    #[test]
    fn test_determine_zone_intranet() {
        let zones = vec![
            ZoneConfig {
                id: "1".to_string(),
                name: "Intranet".to_string(),
                cidr: "10.0.0.0/8".to_string(),
                priority: 100,
            },
        ];

        let result = determine_zone("10.0.1.5", &zones);
        assert!(matches!(result, NetworkZone::Intranet));
    }

    #[test]
    fn test_determine_zone_dmz() {
        let zones = vec![
            ZoneConfig {
                id: "1".to_string(),
                name: "DMZ".to_string(),
                cidr: "192.168.1.0/24".to_string(),
                priority: 100,
            },
        ];

        let result = determine_zone("192.168.1.10", &zones);
        assert!(matches!(result, NetworkZone::DMZ));
    }

    #[test]
    fn test_determine_zone_custom() {
        let zones = vec![
            ZoneConfig {
                id: "1".to_string(),
                name: "CustomZone".to_string(),
                cidr: "172.16.0.0/16".to_string(),
                priority: 100,
            },
        ];

        let result = determine_zone("172.16.5.10", &zones);
        assert!(matches!(result, NetworkZone::Custom(_)));
        if let NetworkZone::Custom(name) = result {
            assert_eq!(name, "CustomZone");
        }
    }

    #[test]
    fn test_determine_zone_invalid_ip() {
        let zones = vec![
            ZoneConfig {
                id: "1".to_string(),
                name: "Intranet".to_string(),
                cidr: "10.0.0.0/8".to_string(),
                priority: 100,
            },
        ];

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
            },
            ZoneConfig {
                id: "2".to_string(),
                name: "HighPriority".to_string(),
                cidr: "10.0.0.0/8".to_string(),
                priority: 100,
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
        let zones = vec![
            ZoneConfig {
                id: "1".to_string(),
                name: "InvalidZone".to_string(),
                cidr: "invalid-cidr".to_string(),
                priority: 100,
            },
        ];

        // Invalid CIDR should be skipped, return Internet default
        let result = determine_zone("10.0.1.5", &zones);
        assert!(matches!(result, NetworkZone::Internet));
    }
}
