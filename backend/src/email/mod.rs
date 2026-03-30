pub mod templates;

use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

use crate::config::Config;
use crate::errors::AppError;

pub struct EmailService {
    transport: EmailTransport,
    from: String,
    frontend_url: String,
}

enum EmailTransport {
    Console,
    Smtp(AsyncSmtpTransport<Tokio1Executor>),
}

impl EmailService {
    pub fn from_config(config: &Config) -> Self {
        let transport = match &config.smtp_host {
            Some(host) => {
                let mut builder = AsyncSmtpTransport::<Tokio1Executor>::relay(host)
                    .expect("Invalid SMTP host");

                if let (Some(username), Some(password)) =
                    (&config.smtp_username, &config.smtp_password)
                {
                    builder =
                        builder.credentials(Credentials::new(username.clone(), password.clone()));
                }

                if let Some(port) = config.smtp_port {
                    builder = builder.port(port);
                }

                EmailTransport::Smtp(builder.build())
            }
            None => {
                tracing::info!("No SMTP host configured, emails will be logged to console");
                EmailTransport::Console
            }
        };

        Self {
            transport,
            from: config.smtp_from.clone(),
            frontend_url: config.frontend_url.clone(),
        }
    }

    pub async fn send_password_reset(&self, to: &str, token: &str) -> Result<(), AppError> {
        let reset_url = format!("{}/reset-password?token={}", self.frontend_url, token);
        let body = templates::password_reset_email(&reset_url);

        let message = Message::builder()
            .from(
                self.from
                    .parse()
                    .map_err(|e| AppError::Internal(format!("Invalid from address: {e}")))?,
            )
            .to(to
                .parse()
                .map_err(|e| AppError::Internal(format!("Invalid to address: {e}")))?)
            .subject("Password Reset Request")
            .header(ContentType::TEXT_HTML)
            .body(body)
            .map_err(|e| AppError::Internal(format!("Failed to build email: {e}")))?;

        match &self.transport {
            EmailTransport::Console => {
                tracing::info!(
                    "=== EMAIL (console mode) ===\nTo: {}\nSubject: Password Reset Request\nReset URL: {}\n===========================",
                    to,
                    reset_url
                );
                Ok(())
            }
            EmailTransport::Smtp(transport) => {
                transport
                    .send(message)
                    .await
                    .map_err(|e| AppError::Internal(format!("Failed to send email: {e}")))?;
                Ok(())
            }
        }
    }
}
