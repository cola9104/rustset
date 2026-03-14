//! Security Configuration
//!
//! 安全相关配置和工具函数

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};

/// 密码哈希
pub fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| format!("Failed to hash password: {}", e))
}

/// 验证密码
pub fn verify_password(password: &str, hash: &str) -> Result<bool, String> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| format!("Invalid hash format: {}", e))?;
    
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

/// 生成随机令牌
pub fn generate_token() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// 验证 IP 地址格式
pub fn validate_ip(ip: &str) -> bool {
    ip.parse::<std::net::IpAddr>().is_ok()
}

/// 验证端口号
pub fn validate_port(port: u16) -> bool {
    port > 0
}

/// 验证 CIDR 格式
pub fn validate_cidr(cidr: &str) -> bool {
    cidr.parse::<ipnetwork::IpNetwork>().is_ok()
}

/// 检查密码强度
pub fn check_password_strength(password: &str) -> PasswordStrength {
    let mut score = 0;
    let mut has_upper = false;
    let mut has_lower = false;
    let mut has_digit = false;
    let mut has_special = false;
    
    for c in password.chars() {
        if c.is_uppercase() { has_upper = true; }
        else if c.is_lowercase() { has_lower = true; }
        else if c.is_numeric() { has_digit = true; }
        else { has_special = true; }
    }
    
    if has_upper { score += 1; }
    if has_lower { score += 1; }
    if has_digit { score += 1; }
    if has_special { score += 1; }
    if password.len() >= 8 { score += 1; }
    if password.len() >= 12 { score += 1; }
    if password.len() >= 16 { score += 1; }
    
    match score {
        0..=2 => PasswordStrength::Weak,
        3..=4 => PasswordStrength::Medium,
        5..=6 => PasswordStrength::Strong,
        _ => PasswordStrength::VeryStrong,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PasswordStrength {
    Weak,
    Medium,
    Strong,
    VeryStrong,
}

impl std::fmt::Display for PasswordStrength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PasswordStrength::Weak => write!(f, "weak"),
            PasswordStrength::Medium => write!(f, "medium"),
            PasswordStrength::Strong => write!(f, "strong"),
            PasswordStrength::VeryStrong => write!(f, "very_strong"),
        }
    }
}
