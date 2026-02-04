use axum::http::HeaderMap;
use std::sync::{Arc, Mutex};
use shared::{User, AuditLog, ZoneConfig, NetworkZone};
use chrono::Utc;
use uuid::Uuid;
use std::net::IpAddr;
use ipnetwork::IpNetwork;

pub fn get_current_user(headers: &HeaderMap, users: &Arc<Mutex<Vec<User>>>) -> Option<User> {
    if let Some(auth_header) = headers.get("Authorization") {
        if let Ok(token) = auth_header.to_str() {
            // Simple Mock: Token is just the username (trimmed to handle whitespace)
            let token = token.trim();
            let users_guard = users.lock().unwrap();
            return users_guard.iter().find(|u| u.username == token).cloned();
        }
    }
    None
}

pub fn log_action(
    logs: &Arc<Mutex<Vec<AuditLog>>>,
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
        let mut logs_guard = logs.lock().unwrap();
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
