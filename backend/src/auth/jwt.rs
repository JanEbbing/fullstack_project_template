use chrono::Utc;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use crate::errors::AppError;

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessClaims {
    pub sub: String,
    pub email: String,
    pub token_type: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshClaims {
    pub sub: String,
    pub jti: String,
    pub token_type: String,
    pub exp: usize,
    pub iat: usize,
}

pub fn create_access_token(
    user_id: &str,
    email: &str,
    secret: &str,
    expiry_secs: i64,
) -> Result<String, AppError> {
    let now = Utc::now().timestamp() as usize;
    let claims = AccessClaims {
        sub: user_id.to_string(),
        email: email.to_string(),
        token_type: "access".to_string(),
        exp: now + expiry_secs as usize,
        iat: now,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("Failed to create access token: {e}")))
}

pub fn create_refresh_token(
    user_id: &str,
    token_id: &str,
    secret: &str,
    expiry_secs: i64,
) -> Result<String, AppError> {
    let now = Utc::now().timestamp() as usize;
    let claims = RefreshClaims {
        sub: user_id.to_string(),
        jti: token_id.to_string(),
        token_type: "refresh".to_string(),
        exp: now + expiry_secs as usize,
        iat: now,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("Failed to create refresh token: {e}")))
}

pub fn verify_access_token(token: &str, secret: &str) -> Result<AccessClaims, AppError> {
    let token_data = decode::<AccessClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| AppError::Unauthorized(format!("Invalid access token: {e}")))?;

    if token_data.claims.token_type != "access" {
        return Err(AppError::Unauthorized("Invalid token type".to_string()));
    }

    Ok(token_data.claims)
}

pub fn verify_refresh_token(token: &str, secret: &str) -> Result<RefreshClaims, AppError> {
    let token_data = decode::<RefreshClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| AppError::Unauthorized(format!("Invalid refresh token: {e}")))?;

    if token_data.claims.token_type != "refresh" {
        return Err(AppError::Unauthorized("Invalid token type".to_string()));
    }

    Ok(token_data.claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SECRET: &str = "test-secret-key-for-unit-tests";

    #[test]
    fn create_and_verify_access_token() {
        let token = create_access_token("user-123", "test@example.com", TEST_SECRET, 300).unwrap();
        let claims = verify_access_token(&token, TEST_SECRET).unwrap();
        assert_eq!(claims.sub, "user-123");
        assert_eq!(claims.email, "test@example.com");
        assert_eq!(claims.token_type, "access");
    }

    #[test]
    fn access_token_expires() {
        // Create a token that expired well past the default 60s leeway
        let now = Utc::now().timestamp() as usize;
        let claims = AccessClaims {
            sub: "user-123".to_string(),
            email: "test@example.com".to_string(),
            token_type: "access".to_string(),
            exp: now - 120,
            iat: now - 130,
        };
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(TEST_SECRET.as_bytes()),
        )
        .unwrap();
        assert!(verify_access_token(&token, TEST_SECRET).is_err());
    }

    #[test]
    fn refresh_token_type_rejected_as_access() {
        let token = create_refresh_token("user-123", "token-id", TEST_SECRET, 300).unwrap();
        assert!(verify_access_token(&token, TEST_SECRET).is_err());
    }

    #[test]
    fn invalid_secret_rejected() {
        let token = create_access_token("user-123", "test@example.com", TEST_SECRET, 300).unwrap();
        assert!(verify_access_token(&token, "wrong-secret").is_err());
    }

    #[test]
    fn create_and_verify_refresh_token() {
        let token = create_refresh_token("user-123", "token-456", TEST_SECRET, 600).unwrap();
        let claims = verify_refresh_token(&token, TEST_SECRET).unwrap();
        assert_eq!(claims.sub, "user-123");
        assert_eq!(claims.jti, "token-456");
        assert_eq!(claims.token_type, "refresh");
    }
}
