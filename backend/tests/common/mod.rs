use std::sync::{Arc, Mutex};

use fullstack_app::config::Config;
use fullstack_app::db;
use fullstack_app::email::EmailService;
use fullstack_app::routes::{AppState, create_router};
use reqwest::Client;
use tokio::net::TcpListener;

pub struct TestApp {
    pub addr: String,
    pub client: Client,
}

impl TestApp {
    pub async fn spawn() -> Self {
        let config = Config {
            host: "127.0.0.1".to_string(),
            port: 0,
            database_url: ":memory:".to_string(),
            jwt_secret: "test-secret-for-integration-tests".to_string(),
            jwt_access_expiry_secs: 900,
            jwt_refresh_expiry_secs: 604800,
            smtp_host: None,
            smtp_port: None,
            smtp_username: None,
            smtp_password: None,
            smtp_from: "test@example.com".to_string(),
            frontend_url: "http://localhost:3000".to_string(),
            static_dir: "/tmp/nonexistent".to_string(),
        };

        let conn = db::init(&config.database_url).expect("Failed to init test DB");
        let email_service = EmailService::from_config(&config);

        let state = AppState {
            db: Arc::new(Mutex::new(conn)),
            config: Arc::new(config),
            email: Arc::new(email_service),
        };

        let app = create_router(state);
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let addr = format!("http://127.0.0.1:{}", port);

        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        TestApp {
            addr,
            client: Client::new(),
        }
    }

    pub fn url(&self, path: &str) -> String {
        format!("{}/api/v1{}", self.addr, path)
    }
}
