pub mod factory;
pub mod smtp;
pub mod tokens;

use crate::errors::DomainError;

#[async_trait::async_trait]
pub trait Mailer: Send + Sync {
    async fn send_verification_email(
        &self,
        to_email: &str,
        user_name: &str,
        token: &str,
    ) -> Result<(), DomainError>;

    async fn send_reset_email(
        &self,
        to_email: &str,
        user_name: &str,
        token: &str,
    ) -> Result<(), DomainError>;
}
