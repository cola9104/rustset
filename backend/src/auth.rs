use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use shared::{Role, User};

use crate::middleware::ApiError;

const TOKEN_EXPIRATION_HOURS: i64 = 24;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: String,
    pub username: String,
    pub role: Role,
    pub exp: usize,
}

fn jwt_secret() -> Result<String, ApiError> {
    let secret = std::env::var("JWT_SECRET")
        .map_err(|_| ApiError::internal("JWT_SECRET environment variable is not set"))?;

    if secret.trim().is_empty() {
        return Err(ApiError::internal("JWT_SECRET environment variable is empty"));
    }

    Ok(secret)
}

pub fn generate_token(user: &User) -> Result<String, ApiError> {
    let exp = (Utc::now() + Duration::hours(TOKEN_EXPIRATION_HOURS)).timestamp() as usize;
    let claims = Claims {
        user_id: user.id.clone(),
        username: user.username.clone(),
        role: user.role.clone(),
        exp,
    };

    let secret = jwt_secret()?;

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| ApiError::internal(format!("Failed to generate JWT token: {}", e)))
}

pub fn verify_token(token: &str) -> Result<Claims, ApiError> {
    let secret = jwt_secret()?;
    let validation = Validation::new(Algorithm::HS256);

    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|e| ApiError::unauthorized(format!("Invalid or expired token: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use shared::Permissions;

    #[test]
    fn test_generate_and_verify_token() {
        std::env::set_var("JWT_SECRET", "test-secret");

        let user = User {
            id: "123".to_string(),
            username: "testuser".to_string(),
            password: "hashed".to_string(),
            role: Role::SysAdmin,
            permissions: Some(Permissions::sys_admin()),
            created_at: Utc::now(),
            password_changed_at: Some(Utc::now()),
            password_strength: Some("strong".to_string()),
            force_password_change: Some(false),
            last_login_at: None,
            email: None,
            phone: None,
            status: Some("active".to_string()),
            failed_login_attempts: Some(0),
            locked_until: None,
        };

        let token = generate_token(&user).unwrap();
        let claims = verify_token(&token).unwrap();

        assert_eq!(claims.user_id, "123");
        assert_eq!(claims.username, "testuser");
        assert_eq!(claims.role, Role::SysAdmin);
    }

    #[test]
    fn test_verify_invalid_token() {
        std::env::set_var("JWT_SECRET", "test-secret");

        let result = verify_token("invalid.jwt.token");
        assert!(result.is_err());
    }

    #[test]
    fn test_token_expiration() {
        std::env::set_var("JWT_SECRET", "test-secret");

        let user = User {
            id: "123".to_string(),
            username: "testuser".to_string(),
            password: "hashed".to_string(),
            role: Role::Auditor,
            permissions: Some(Permissions::auditor()),
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
        };

        let token = generate_token(&user).unwrap();
        let claims = verify_token(&token).unwrap();

        // 验证过期时间大约是 24 小时后
        let now = Utc::now().timestamp() as usize;
        assert!(claims.exp > now);
        assert!(claims.exp < now + (25 * 3600)); // 25 hours in seconds
    }
}
