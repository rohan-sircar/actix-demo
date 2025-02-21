use redis::aio::ConnectionManager;
use redis::AsyncCommands;

use crate::errors::DomainError;
use crate::models::users::UserId;

#[derive(new)]
pub struct RedisCredentialsRepo {
    base_key: String,
    redis: ConnectionManager,
}

impl RedisCredentialsRepo {
    pub fn get_key(&self, user_id: &UserId) -> String {
        format!("{}.{user_id}", self.base_key)
    }

    pub async fn load(
        &self,
        user_id: &UserId,
    ) -> Result<Option<String>, DomainError> {
        let jwt = self
            .redis
            .clone()
            .get::<String, String>(self.get_key(user_id))
            .await?;
        Ok(Some(jwt))
    }

    pub async fn save(
        &self,
        user_id: &UserId,
        jwt: &str,
        ttl_seconds: u64,
    ) -> Result<(), DomainError> {
        {
            let _ = self
                .redis
                .clone()
                .set_ex::<String, &str, ()>(
                    self.get_key(user_id),
                    jwt,
                    ttl_seconds,
                )
                .await?;
        }
        Ok(())
    }

    pub async fn delete(&self, user_id: &UserId) -> Result<(), DomainError> {
        {
            let _ = self
                .redis
                .clone()
                .del::<String, ()>(self.get_key(user_id))
                .await?;
        }
        Ok(())
    }
}
