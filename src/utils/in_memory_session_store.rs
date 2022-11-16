use actix_session::storage::{
    CookieSessionStore, LoadError, SaveError, SessionKey, SessionStore,
    UpdateError,
};
use actix_web::cookie::time::Duration;
use anyhow::Error;
use async_trait;
use std::convert::TryInto;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

pub struct InMemorySessionStore {
    sessions: RwLock<Arc<HashMap<String, String>>>,
}

impl InMemorySessionStore {
    fn new() -> Self {
        InMemorySessionStore {
            sessions: RwLock::new(Arc::new(HashMap::new())),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl SessionStore for InMemorySessionStore {
    async fn load(
        &self,
        session_key: &SessionKey,
    ) -> Result<Option<HashMap<String, String>>, LoadError> {
        let x = serde_json::from_str(session_key.as_ref())
            .map(Some)
            .map_err(anyhow::Error::new)
            .map_err(LoadError::Deserialization);
        x
    }

    async fn save(
        &self,
        session_state: HashMap<String, String>,
        _ttl: &Duration,
    ) -> Result<SessionKey, SaveError> {
        let session_key = serde_json::to_string(&session_state)
            .map_err(anyhow::Error::new)
            .map_err(SaveError::Serialization)?;

        Ok(session_key
            .try_into()
            .map_err(Into::into)
            .map_err(SaveError::Other)?)
    }

    async fn update(
        &self,
        _session_key: SessionKey,
        session_state: HashMap<String, String>,
        ttl: &Duration,
    ) -> Result<SessionKey, UpdateError> {
        self.save(session_state, ttl)
            .await
            .map_err(|err| match err {
                SaveError::Serialization(err) => {
                    UpdateError::Serialization(err)
                }
                SaveError::Other(err) => UpdateError::Other(err),
            })
    }

    async fn update_ttl(
        &self,
        _session_key: &SessionKey,
        _ttl: &Duration,
    ) -> Result<(), Error> {
        Ok(())
    }

    async fn delete(
        &self,
        _session_key: &SessionKey,
    ) -> Result<(), anyhow::Error> {
        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::{
//         storage::utils::generate_session_key,
//         test_helpers::acceptance_test_suite,
//     };

//     #[actix_web::test]
//     async fn test_session_workflow() {
//         acceptance_test_suite(CookieSessionStore::default, false).await;
//     }

//     #[actix_web::test]
//     async fn loading_a_random_session_key_returns_deserialization_error() {
//         let store = CookieSessionStore::default();
//         let session_key = generate_session_key();
//         assert!(matches!(
//             store.load(&session_key).await.unwrap_err(),
//             LoadError::Deserialization(_),
//         ));
//     }
// }
