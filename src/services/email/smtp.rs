use crate::config::TlsMode;
use crate::errors::DomainError;
use crate::services::email::Mailer;
use lettre::{
    message::{header::ContentType, MultiPart, SinglePart},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use lettre::transport::smtp::client::Tls;
use lettre::transport::smtp::client::TlsParameters;

pub struct SmtpSender {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    from_email: String,
    verification_link_template: String,
    password_reset_link_template: String,
}

impl SmtpSender {
    pub fn new(
        smtp_host: &str,
        smtp_port: u16,
        tls_mode: TlsMode,
        username: &str,
        password: &str,
        from_email: &str,
        verification_link_template: &str,
        password_reset_link_template: &str,
    ) -> Result<Self, DomainError> {
        let tls_config = match tls_mode {
            TlsMode::None => Tls::None,
            TlsMode::StartTls => {
                let tls_params = TlsParameters::new(smtp_host.to_owned())
                    .map_err(|e| DomainError::new_internal_error(format!("TLS config error: {e}")))?;
                Tls::Required(tls_params)
            }
            TlsMode::Tls => {
                let tls_params = TlsParameters::builder(smtp_host.to_owned())
                    .dangerous_accept_invalid_certs(true)
                    .dangerous_accept_invalid_hostnames(true)
                    .build()
                    .map_err(|e| DomainError::new_internal_error(format!("TLS config error: {e}")))?;
                Tls::Wrapper(tls_params)
            }
        };

        let mut builder = AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(smtp_host)
            .port(smtp_port)
            .tls(tls_config);

        if !username.is_empty() {
            builder = builder.credentials(Credentials::new(
                username.to_owned(),
                password.to_owned(),
            ));
        }

        let transport = builder.build();

        Ok(Self {
            transport,
            from_email: from_email.to_owned(),
            verification_link_template: verification_link_template.to_owned(),
            password_reset_link_template: password_reset_link_template.to_owned(),
        })
    }

    fn build_email(
        from: &str,
        to: &str,
        subject: &str,
        body_html: &str,
        body_text: &str,
    ) -> Result<Message, DomainError> {
        Message::builder()
            .from(from.parse().map_err(|e| DomainError::InternalError {
                message: format!("Invalid from email: {e}"),
            })?)
            .to(to.parse().map_err(|e| DomainError::InternalError {
                message: format!("Invalid to email: {e}"),
            })?)
            .subject(subject)
            .multipart(
                MultiPart::alternative()
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_PLAIN)
                            .body(body_text.to_owned()),
                    )
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_HTML)
                            .body(body_html.to_owned()),
                    ),
            )
            .map_err(|e| DomainError::InternalError {
                message: format!("Failed to build email message: {e}"),
            })
    }

    fn render_verification_link(&self, token: &str, user_name: &str) -> String {
        self.verification_link_template
            .replace("{token}", token)
            .replace("{user_name}", user_name)
    }

    fn render_reset_link(&self, token: &str, user_name: &str) -> String {
        self.password_reset_link_template
            .replace("{token}", token)
            .replace("{user_name}", user_name)
    }
}

#[async_trait::async_trait]
impl Mailer for SmtpSender {
    async fn send_verification_email(
        &self,
        to_email: &str,
        user_name: &str,
        token: &str,
    ) -> Result<(), DomainError> {
        let link = self.render_verification_link(token, user_name);
        let body_html = format!(
            "<html><body><p>Hello {},</p><p>Please verify your email address by clicking the button below:</p><p><a href=\"{}\" style=\"background-color: #4CAF50; color: white; padding: 14px 20px; text-decoration: none; border-radius: 4px;\">Verify Email</a></p><p>This link expires in 24 hours.</p></body></html>",
            user_name, link
        );
        let body_text = format!(
            "Hello {},\n\nPlease verify your email address by clicking the link below:\n{}\n\nThis link expires in 24 hours.",
            user_name, link
        );

        let email = Self::build_email(
            &self.from_email,
            to_email,
            "Verify your email address",
            &body_html,
            &body_text,
        )?;

        tracing::info!(to_email = to_email, "Sending verification email");

        self.transport
            .send(email)
            .await
            .map_err(|e| DomainError::InternalError {
                message: format!("Failed to send verification email: {e}"),
            })?;

        Ok(())
    }

    async fn send_reset_email(
        &self,
        to_email: &str,
        user_name: &str,
        token: &str,
    ) -> Result<(), DomainError> {
        let link = self.render_reset_link(token, user_name);
        let body_html = format!(
            "<html><body><p>Hello {},</p><p>We received a request to reset your password.</p><p><a href=\"{}\" style=\"background-color: #2196F3; color: white; padding: 14px 20px; text-decoration: none; border-radius: 4px;\">Reset Password</a></p><p>If you didn't request this, please ignore this email.</p><p>This link expires in 15 minutes.</p></body></html>",
            user_name, link
        );
        let body_text = format!(
            "Hello {},\n\nWe received a request to reset your password. Click the link below to reset it:\n{}\n\nIf you didn't request this, please ignore this email.\n\nThis link expires in 15 minutes.",
            user_name, link
        );

        let email = Self::build_email(
            &self.from_email,
            to_email,
            "Password Reset Request",
            &body_html,
            &body_text,
        )?;

        tracing::info!(to_email = to_email, "Sending password reset email");

        self.transport
            .send(email)
            .await
            .map_err(|e| DomainError::InternalError {
                message: format!("Failed to send password reset email: {e}"),
            })?;

        Ok(())
    }
}
