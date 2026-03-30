pub mod auth;
pub mod health;
pub mod user;

use std::sync::{Arc, Mutex};

use axum::Router;
use axum::routing::{get, post};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;

use crate::config::Config;
use crate::email::EmailService;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Mutex<rusqlite::Connection>>,
    pub config: Arc<Config>,
    pub email: Arc<EmailService>,
}

pub fn create_router(state: AppState) -> Router {
    let auth_routes = Router::new()
        .route("/register", post(auth::register))
        .route("/login", post(auth::login))
        .route("/refresh", post(auth::refresh))
        .route("/forgot-password", post(auth::forgot_password))
        .route("/reset-password", post(auth::reset_password))
        .route("/logout", post(auth::logout));

    let user_routes = Router::new()
        .route("/me", get(user::get_me))
        .route("/data", get(user::list_data).post(user::create_data));

    let api_routes = Router::new()
        .route("/health", get(health::health_check))
        .nest("/auth", auth_routes)
        .nest("/user", user_routes);

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let static_dir = state.config.static_dir.clone();
    let fallback_file = format!("{}/200.html", static_dir);

    Router::new()
        .nest("/api", api_routes)
        .fallback_service(
            ServeDir::new(&static_dir)
                .not_found_service(ServeFile::new(fallback_file)),
        )
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}
