use async_trait::async_trait;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

use crate::errors::DomainError;
use crate::models::users::UserId;

use super::CredentialsRepo;

#[derive(Clone)]
pub struct InMemoryCredentialsRepo {
    credentials: Arc<RwLock<HashMap<UserId, String>>>,
}

impl InMemoryCredentialsRepo {
    pub fn new(creds: HashMap<UserId, String>) -> Self {
        InMemoryCredentialsRepo {
            credentials: Arc::new(RwLock::new(creds)),
        }
    }
}

impl Default for InMemoryCredentialsRepo {
    fn default() -> Self {
        InMemoryCredentialsRepo {
            credentials: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait(?Send)]
impl CredentialsRepo for InMemoryCredentialsRepo {
    async fn load(
        &self,
        user_id: &UserId,
    ) -> Result<Option<String>, DomainError> {
        let jwt = self.credentials.read().await.get(user_id).cloned();
        Ok(jwt)
    }

    async fn save(
        &self,
        user_id: &UserId,
        jwt: &str,
    ) -> Result<(), DomainError> {
        {
            let mut sessions = self.credentials.write().await;
            let _ = sessions.insert(*user_id, jwt.to_owned());
        }
        Ok(())
    }

    // async fn update(
    //     &self,
    //     _session_key: SessionKey,
    //     session_state: HashMap<String, String>,
    //     ttl: &Duration,
    // ) -> Result<SessionKey, UpdateError> {
    //     self.save(session_state, ttl)
    //         .await
    //         .map_err(|err| match err {
    //             SaveError::Serialization(err) => {
    //                 UpdateError::Serialization(err)
    //             }
    //             SaveError::Other(err) => UpdateError::Other(err),
    //         })
    // }

    // async fn update_ttl(
    //     &self,
    //     _session_key: &SessionKey,
    //     _ttl: &Duration,
    // ) -> Result<(), DomainError> {
    //     Ok(())
    // }

    async fn delete(&self, user_id: &UserId) -> Result<(), DomainError> {
        {
            let mut sessions = self.credentials.write().await;
            let _ = sessions.remove(user_id);
        }
        Ok(())
    }
}
