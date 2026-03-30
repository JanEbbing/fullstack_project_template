use std::sync::{Arc, Mutex};

use fullstack_app::config;
use fullstack_app::db;
use fullstack_app::email;
use fullstack_app::routes::AppState;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let config = config::Config::from_env();
    let addr = format!("{}:{}", config.host, config.port);

    let conn = db::init(&config.database_url).expect("Failed to initialize database");
    let email_service = email::EmailService::from_config(&config);

    let state = AppState {
        db: Arc::new(Mutex::new(conn)),
        config: Arc::new(config),
        email: Arc::new(email_service),
    };

    let app = fullstack_app::routes::create_router(state);
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind");
    tracing::info!("Listening on {}", addr);
    axum::serve(listener, app).await.expect("Server error");
}
