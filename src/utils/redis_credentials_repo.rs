use async_trait::async_trait;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;

use crate::errors::DomainError;
use crate::models::users::UserId;

use super::CredentialsRepo;

#[derive(new)]
pub struct RedisCredentialsRepo {
    base_key: String,
    redis: ConnectionManager,
}

impl RedisCredentialsRepo {
    pub fn get_key(&self, user_id: &UserId) -> String {
        format!("actix-demo.{}.{}", self.base_key, user_id)
    }
}

#[async_trait(?Send)]
impl CredentialsRepo for RedisCredentialsRepo {
    async fn load(
        &self,
        user_id: &UserId,
    ) -> Result<Option<String>, DomainError> {
        let jwt = self
            .redis
            .clone()
            .get::<String, String>(self.get_key(user_id))
            .await
            .unwrap();
        Ok(Some(jwt))
    }

    async fn save(
        &self,
        user_id: &UserId,
        jwt: &str,
    ) -> Result<(), DomainError> {
        {
            let _ = self
                .redis
                .clone()
                .set::<String, &str, ()>(self.get_key(user_id), jwt)
                .await
                .unwrap();
        }
        Ok(())
    }

    async fn delete(&self, user_id: &UserId) -> Result<(), DomainError> {
        {
            let _ = self
                .redis
                .clone()
                .del::<String, ()>(self.get_key(user_id))
                .await
                .unwrap();
        }
        Ok(())
    }
}
