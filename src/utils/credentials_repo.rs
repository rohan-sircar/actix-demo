use async_trait::async_trait;

use crate::errors::DomainError;
use crate::models::users::UserId;

#[async_trait(?Send)]
pub trait CredentialsRepo {
    async fn load(
        &self,
        user_id: &UserId,
    ) -> Result<Option<String>, DomainError>;

    async fn save(
        &self,
        user_id: &UserId,
        jwt: &str,
    ) -> Result<(), DomainError>;

    async fn delete(&self, user_id: &UserId) -> Result<(), DomainError>;
}
