pub mod auth;
pub mod health;
pub mod user;

use std::sync::{Arc, Mutex};

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::http::HeaderValue;
use axum::http::Method;
use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
use axum::routing::{get, post};
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::set_header::SetResponseHeaderLayer;
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

    let origin: HeaderValue = state
        .config
        .frontend_url
        .parse()
        .expect("FRONTEND_URL is not a valid HTTP origin");
    let cors = CorsLayer::new()
        .allow_origin(origin)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([AUTHORIZATION, CONTENT_TYPE]);

    let static_dir = state.config.static_dir.clone();
    let fallback_file = std::path::Path::new(&static_dir).join("200.html");

    Router::new()
        .nest("/api/v1", api_routes)
        .fallback_service(
            ServeDir::new(&static_dir).not_found_service(ServeFile::new(fallback_file)),
        )
        .layer(DefaultBodyLimit::max(1024 * 1024))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::HeaderName::from_static("x-frame-options"),
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::HeaderName::from_static("x-content-type-options"),
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::HeaderName::from_static("referrer-policy"),
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        ))
        .with_state(state)
}
