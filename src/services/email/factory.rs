use std::sync::Arc;

use crate::config::EnvConfig;
use crate::errors::DomainError;
use crate::services::email::smtp::{SmtpSender, SmtpSenderConfig};
use crate::services::email::Mailer;

pub fn create_mailer(
    config: &EnvConfig,
) -> Result<Arc<dyn Mailer>, DomainError> {
    let sender_config = SmtpSenderConfig {
        smtp_host: config.smtp_host.clone(),
        smtp_port: config.smtp_port,
        tls_mode: config.smtp_tls_mode,
        username: config.smtp_username.clone(),
        password: config.smtp_password.clone(),
        from_email: config.smtp_from_email.clone(),
        app_base_url: config.app_base_url.clone(),
        verification_link_template: config.verification_link_template.clone(),
        password_reset_link_template: config
            .password_reset_link_template
            .clone(),
        mobile_verification_link_template: config
            .mobile_verification_link_template
            .clone(),
    };

    let sender = SmtpSender::new(&sender_config)?;

    Ok(Arc::new(sender))
}
