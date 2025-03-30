use cached::IOCached;
use cached::RedisCache;
use cached::RedisCacheError;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::sync::Arc;
use tracing::debug;

#[derive(Clone)]
pub struct InstrumentedRedisCache<K, V> {
    inner: Arc<RedisCache<K, V>>,
}

impl<K, V> InstrumentedRedisCache<K, V>
where
    K: std::fmt::Display + Clone + Send + Sync + 'static,
    V: Clone + Serialize + DeserializeOwned + Send + Sync + 'static,
{
    pub fn new(inner: RedisCache<K, V>) -> Self {
        Self {
            inner: Arc::new(inner),
        }
    }
    pub fn get(&self, key: &K) -> Result<Option<V>, RedisCacheError> {
        let result = self.inner.cache_get(key);
        match &result {
            Ok(Some(_)) => {
                let _ = debug!("Cache hit for key: {}", key);
            }
            Ok(None) => {
                let _ = debug!("Cache miss for key: {}", key);
            }
            Err(err) => {
                let _ = tracing::error!(
                    "Error retrieving cache value for key: {key} err: {err:?}"
                );
            }
        }
        result
    }

    pub fn set(&self, key: K, value: V) -> Result<Option<V>, RedisCacheError> {
        let _ = debug!("Setting cache value for key: {}", key);
        self.inner.cache_set(key, value)
    }

    pub fn remove(&self, key: &K) -> Result<Option<V>, RedisCacheError> {
        let _ = debug!("Deleting cache value for key: {}", key);
        self.inner.cache_remove(key)
    }
}
