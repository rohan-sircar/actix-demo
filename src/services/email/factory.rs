use std::sync::Arc;

use crate::config::EnvConfig;
use crate::errors::DomainError;
use crate::services::email::smtp::SmtpSender;
use crate::services::email::Mailer;

pub fn create_mailer(
    config: &EnvConfig,
) -> Result<Arc<dyn Mailer>, DomainError> {
    let sender = SmtpSender::new(
        &config.smtp_host,
        config.smtp_port,
        config.smtp_tls_mode.clone(),
        &config.smtp_username,
        &config.smtp_password,
        &config.smtp_from_email,
        &config.verification_link_template,
        &config.password_reset_link_template,
    )?;

    Ok(Arc::new(sender))
}
