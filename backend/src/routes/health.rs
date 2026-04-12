use axum::Json;
use axum::extract::State;
use serde_json::{Value, json};
use std::sync::Arc;

use crate::errors::AppError;
use crate::routes::AppState;

pub async fn health_check(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    let db = Arc::clone(&state.db);
    tokio::task::spawn_blocking(move || {
        db.lock()
            .map_err(|_| AppError::Internal("DB mutex poisoned".into()))?
            .query_row("SELECT 1", [], |_| Ok(()))
            .map_err(|e| AppError::Internal(e.to_string()))
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    Ok(Json(json!({ "status": "ok" })))
}
