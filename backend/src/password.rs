use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PasswordError {
    #[error("Hashing failed: {0}")]
    HashError(String),
    #[allow(dead_code)]
    #[error("Verification failed")]
    VerifyError,
}

/// Hash a plain-text password using Argon2id (recommended for password hashing)
pub fn hash_password(password: &str) -> Result<String, PasswordError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| PasswordError::HashError(e.to_string()))?
        .to_string();

    Ok(password_hash)
}

/// Verify a plain-text password against a hashed password
pub fn verify_password(password: &str, hashed_password: &str) -> Result<bool, PasswordError> {
    let parsed_hash = PasswordHash::new(hashed_password)
        .map_err(|e| PasswordError::HashError(e.to_string()))?;

    let argon2 = Argon2::default();

    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Check password strength (simple implementation)
/// Returns score: 0-4 (weak to strong)
pub fn check_password_strength(password: &str) -> usize {
    let mut score = 0;

    // Length check
    if password.len() >= 8 {
        score += 1;
    }
    if password.len() >= 12 {
        score += 1;
    }

    // Complexity checks
    if password.chars().any(|c| c.is_ascii_uppercase()) {
        score += 1;
    }
    if password.chars().any(|c| !c.is_alphanumeric()) {
        score += 1;
    }

    // Only count digits if there are other character types (avoid "12345678" scoring)
    if password.chars().any(|c| c.is_ascii_digit()) &&
       (password.chars().any(|c| c.is_ascii_uppercase()) ||
        password.chars().any(|c| !c.is_alphanumeric())) {
        score += 1;
    }

    // Cap at 4
    score.min(4)
}

/// Get password strength description
pub fn get_strength_label(score: usize) -> &'static str {
    match score {
        0 => "very weak",
        1 => "weak",
        2 => "moderate",
        3 => "strong",
        4 => "very strong",
        _ => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify() {
        let password = "test_password_123";
        let hash = hash_password(password).unwrap();
        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_password_strength() {
        assert_eq!(check_password_strength("123"), 0); // very weak
        assert_eq!(check_password_strength("12345678"), 1); // weak
        assert_eq!(check_password_strength("12345678Abc!"), 4); // very strong
    }
}
