use axum::Json;
use axum::extract::State;
use rusqlite::params;
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use uuid::Uuid;
use validator::Validate;

use crate::auth::jwt;
use crate::auth::middleware::AuthUser;
use crate::auth::password;
use crate::errors::AppError;
use crate::models::user::UserResponse;
use crate::routes::AppState;

fn sha256_hex(input: &str) -> String {
    hex::encode(Sha256::digest(input.as_bytes()))
}

#[derive(Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8, max = 128))]
    pub password: String,
}

pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> Result<(axum::http::StatusCode, Json<Value>), AppError> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let password_hash = password::hash_password(&body.password)?;
    let user_id = Uuid::new_v4().to_string();
    let email = body.email.clone();

    let db = state.db.clone();
    let uid = user_id.clone();
    let em = email.clone();
    let ph = password_hash.clone();

    let user = tokio::task::spawn_blocking(move || {
        let conn = db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        conn.execute(
            "INSERT INTO users (id, email, password_hash) VALUES (?1, ?2, ?3)",
            params![uid, em, ph],
        )
        .map_err(|e| match e {
            rusqlite::Error::SqliteFailure(err, _)
                if err.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                AppError::Conflict("Email already registered".to_string())
            }
            _ => AppError::Internal(e.to_string()),
        })?;

        let user = conn.query_row(
            "SELECT id, email, created_at FROM users WHERE id = ?1",
            params![uid],
            |row| {
                Ok(UserResponse {
                    id: row.get(0)?,
                    email: row.get(1)?,
                    created_at: row.get(2)?,
                })
            },
        ).map_err(|e| AppError::Internal(e.to_string()))?;

        Ok::<_, AppError>(user)
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    let (access_token, refresh_token) =
        create_token_pair(&state, &user_id, &email)?;

    Ok((
        axum::http::StatusCode::CREATED,
        Json(json!({
            "user": user,
            "access_token": access_token,
            "refresh_token": refresh_token,
        })),
    ))
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<Value>, AppError> {
    let db = state.db.clone();
    let email = body.email.clone();

    let user = tokio::task::spawn_blocking(move || {
        let conn = db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        conn.query_row(
            "SELECT id, email, password_hash, created_at FROM users WHERE email = ?1",
            params![email],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            },
        )
        .map_err(|_| AppError::Unauthorized("Invalid email or password".to_string()))
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    let (id, email, password_hash, created_at) = user;

    if !password::verify_password(&body.password, &password_hash)? {
        return Err(AppError::Unauthorized(
            "Invalid email or password".to_string(),
        ));
    }

    let (access_token, refresh_token) = create_token_pair(&state, &id, &email)?;

    Ok(Json(json!({
        "user": UserResponse { id, email, created_at },
        "access_token": access_token,
        "refresh_token": refresh_token,
    })))
}

#[derive(Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

pub async fn refresh(
    State(state): State<AppState>,
    Json(body): Json<RefreshRequest>,
) -> Result<Json<Value>, AppError> {
    let claims = jwt::verify_refresh_token(&body.refresh_token, &state.config.jwt_secret)?;
    let token_hash = sha256_hex(&body.refresh_token);

    let db = state.db.clone();
    let jti = claims.jti.clone();
    let th = token_hash.clone();

    let user_id = claims.sub.clone();

    let email = tokio::task::spawn_blocking(move || {
        let conn = db.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        // Verify the refresh token exists, is not revoked, and hash matches
        let (stored_user_id, stored_hash, revoked): (String, String, bool) = conn
            .query_row(
                "SELECT user_id, token_hash, revoked FROM refresh_tokens WHERE id = ?1",
                params![jti],
                |row| Ok((row.get(0)?, row.get(1)?, row.get::<_, bool>(2)?)),
            )
            .map_err(|_| AppError::Unauthorized("Invalid refresh token".to_string()))?;

        if revoked {
            return Err(AppError::Unauthorized(
                "Refresh token has been revoked".to_string(),
            ));
        }

        if stored_hash != th {
            return Err(AppError::Unauthorized(
                "Invalid refresh token".to_string(),
            ));
        }

        if stored_user_id != claims.sub {
            return Err(AppError::Unauthorized(
                "Token user mismatch".to_string(),
            ));
        }

        // Revoke the old refresh token
        conn.execute(
            "UPDATE refresh_tokens SET revoked = 1 WHERE id = ?1",
            params![jti],
        )
        .map_err(|e| AppError::Internal(e.to_string()))?;

        // Get user email for new access token
        let email: String = conn
            .query_row(
                "SELECT email FROM users WHERE id = ?1",
                params![claims.sub],
                |row| row.get(0),
            )
            .map_err(|_| AppError::Unauthorized("User not found".to_string()))?;

        Ok::<_, AppError>(email)
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    let (access_token, refresh_token) =
        create_token_pair(&state, &user_id, &email)?;

    Ok(Json(json!({
        "access_token": access_token,
        "refresh_token": refresh_token,
    })))
}

#[derive(Deserialize, Validate)]
pub struct ForgotPasswordRequest {
    #[validate(email)]
    pub email: String,
}

pub async fn forgot_password(
    State(state): State<AppState>,
    Json(body): Json<ForgotPasswordRequest>,
) -> Result<Json<Value>, AppError> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let raw_token = Uuid::new_v4().to_string();
    let token_hash = sha256_hex(&raw_token);
    let token_id = Uuid::new_v4().to_string();
    let expires_at =
        chrono::Utc::now() + chrono::Duration::hours(1);
    let expires_at_str = expires_at.format("%Y-%m-%d %H:%M:%S").to_string();

    let db = state.db.clone();
    let email = body.email.clone();
    let raw = raw_token.clone();

    let found = tokio::task::spawn_blocking(move || {
        let conn = db.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        let user_id: Result<String, _> = conn.query_row(
            "SELECT id FROM users WHERE email = ?1",
            params![email],
            |row| row.get(0),
        );

        if let Ok(user_id) = user_id {
            conn.execute(
                "INSERT INTO password_reset_tokens (id, user_id, token_hash, expires_at) VALUES (?1, ?2, ?3, ?4)",
                params![token_id, user_id, token_hash, expires_at_str],
            )
            .map_err(|e| AppError::Internal(e.to_string()))?;
            Ok::<_, AppError>(true)
        } else {
            Ok(false)
        }
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    if found {
        // Send email in background — don't block the response
        let email_service = state.email.clone();
        let to = body.email.clone();
        tokio::spawn(async move {
            if let Err(e) = email_service.send_password_reset(&to, &raw).await {
                tracing::error!("Failed to send password reset email: {e}");
            }
        });
    }

    // Always return success to prevent email enumeration
    Ok(Json(json!({
        "message": "If an account with that email exists, a reset link has been sent."
    })))
}

#[derive(Deserialize, Validate)]
pub struct ResetPasswordRequest {
    pub token: String,
    #[validate(length(min = 8, max = 128))]
    pub new_password: String,
}

pub async fn reset_password(
    State(state): State<AppState>,
    Json(body): Json<ResetPasswordRequest>,
) -> Result<Json<Value>, AppError> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let token_hash = sha256_hex(&body.token);
    let new_password_hash = password::hash_password(&body.new_password)?;

    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let conn = db.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        let (reset_id, user_id): (String, String) = conn
            .query_row(
                "SELECT id, user_id FROM password_reset_tokens
                 WHERE token_hash = ?1 AND used = 0 AND expires_at > datetime('now')",
                params![token_hash],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|_| {
                AppError::BadRequest("Invalid or expired reset token".to_string())
            })?;

        conn.execute(
            "UPDATE users SET password_hash = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![new_password_hash, user_id],
        )
        .map_err(|e| AppError::Internal(e.to_string()))?;

        conn.execute(
            "UPDATE password_reset_tokens SET used = 1 WHERE id = ?1",
            params![reset_id],
        )
        .map_err(|e| AppError::Internal(e.to_string()))?;

        // Revoke all refresh tokens for this user (force re-login)
        conn.execute(
            "UPDATE refresh_tokens SET revoked = 1 WHERE user_id = ?1",
            params![user_id],
        )
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok::<_, AppError>(())
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    Ok(Json(json!({
        "message": "Password has been reset successfully."
    })))
}

#[derive(Deserialize)]
pub struct LogoutRequest {
    pub refresh_token: String,
}

pub async fn logout(
    _auth_user: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<LogoutRequest>,
) -> Result<Json<Value>, AppError> {
    let token_hash = sha256_hex(&body.refresh_token);

    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let conn = db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        conn.execute(
            "UPDATE refresh_tokens SET revoked = 1 WHERE token_hash = ?1",
            params![token_hash],
        )
        .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok::<_, AppError>(())
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    Ok(Json(json!({
        "message": "Logged out successfully."
    })))
}

/// Create a new access/refresh token pair and store the refresh token in the DB.
fn create_token_pair(
    state: &AppState,
    user_id: &str,
    email: &str,
) -> Result<(String, String), AppError> {
    let token_id = Uuid::new_v4().to_string();

    let access_token = jwt::create_access_token(
        user_id,
        email,
        &state.config.jwt_secret,
        state.config.jwt_access_expiry_secs,
    )?;

    let refresh_token = jwt::create_refresh_token(
        user_id,
        &token_id,
        &state.config.jwt_secret,
        state.config.jwt_refresh_expiry_secs,
    )?;

    let token_hash = sha256_hex(&refresh_token);
    let expires_at = chrono::Utc::now()
        + chrono::Duration::seconds(state.config.jwt_refresh_expiry_secs);
    let expires_at_str = expires_at.format("%Y-%m-%d %H:%M:%S").to_string();

    let db = state.db.clone();
    let tid = token_id.clone();
    let uid = user_id.to_string();
    // Store synchronously since we're not in an async context that requires spawn_blocking
    // and this is called within handlers that will handle the error
    let conn = db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    conn.execute(
        "INSERT INTO refresh_tokens (id, user_id, token_hash, expires_at) VALUES (?1, ?2, ?3, ?4)",
        params![tid, uid, token_hash, expires_at_str],
    )
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok((access_token, refresh_token))
}
