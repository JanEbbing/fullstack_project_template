use axum::Json;
use axum::extract::State;
use rusqlite::params;
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;
use validator::Validate;

use crate::auth::middleware::AuthUser;
use crate::errors::AppError;
use crate::models::user::{UserData, UserResponse};
use crate::routes::AppState;

pub async fn get_me(
    auth_user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Value>, AppError> {
    let db = state.db.clone();
    let user_id = auth_user.user_id;

    let user = tokio::task::spawn_blocking(move || {
        let conn = db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        conn.query_row(
            "SELECT id, email, created_at FROM users WHERE id = ?1",
            params![user_id],
            |row| {
                Ok(UserResponse {
                    id: row.get(0)?,
                    email: row.get(1)?,
                    created_at: row.get(2)?,
                })
            },
        )
        .map_err(|_| AppError::NotFound("User not found".to_string()))
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    Ok(Json(json!({ "user": user })))
}

pub async fn list_data(
    auth_user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Value>, AppError> {
    let db = state.db.clone();
    let user_id = auth_user.user_id;

    let data = tokio::task::spawn_blocking(move || {
        let conn = db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        let mut stmt = conn
            .prepare(
                "SELECT id, user_id, title, content, created_at, updated_at
                 FROM user_data WHERE user_id = ?1 ORDER BY created_at DESC",
            )
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let items: Vec<UserData> = stmt
            .query_map(params![user_id], |row| {
                Ok(UserData {
                    id: row.get(0)?,
                    user_id: row.get(1)?,
                    title: row.get(2)?,
                    content: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            })
            .map_err(|e| AppError::Internal(e.to_string()))?
            .collect::<Result<_, _>>()
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok::<_, AppError>(items)
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    Ok(Json(json!({ "data": data })))
}

#[derive(Deserialize, Validate)]
pub struct CreateUserDataRequest {
    #[validate(length(min = 1, max = 255))]
    pub title: String,
    #[validate(length(max = 65536))]
    pub content: String,
}

pub async fn create_data(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<CreateUserDataRequest>,
) -> Result<(axum::http::StatusCode, Json<Value>), AppError> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let db = state.db.clone();
    let user_id = auth_user.user_id;
    let data_id = Uuid::new_v4().to_string();

    let item = tokio::task::spawn_blocking(move || {
        let conn = db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        conn.execute(
            "INSERT INTO user_data (id, user_id, title, content) VALUES (?1, ?2, ?3, ?4)",
            params![data_id, user_id, body.title, body.content],
        )
        .map_err(|e| AppError::Internal(e.to_string()))?;

        conn.query_row(
            "SELECT id, user_id, title, content, created_at, updated_at
             FROM user_data WHERE id = ?1",
            params![data_id],
            |row| {
                Ok(UserData {
                    id: row.get(0)?,
                    user_id: row.get(1)?,
                    title: row.get(2)?,
                    content: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            },
        )
        .map_err(|e| AppError::Internal(e.to_string()))
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    Ok((
        axum::http::StatusCode::CREATED,
        Json(json!({ "data": item })),
    ))
}
