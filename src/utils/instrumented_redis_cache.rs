use crate::metrics::CacheMetrics;
use cached::IOCached;
use cached::RedisCache;
use cached::RedisCacheError;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::sync::Arc;
use std::time::Instant;
use tracing::debug;

#[derive(Clone)]
pub struct InstrumentedRedisCache<K, V> {
    inner: Arc<RedisCache<K, V>>,
    metrics: CacheMetrics,
}

impl<K, V> InstrumentedRedisCache<K, V>
where
    K: std::fmt::Display + Clone + Send + Sync + 'static,
    V: Clone + Serialize + DeserializeOwned + Send + Sync + 'static,
{
    pub fn new(inner: RedisCache<K, V>, metrics: CacheMetrics) -> Self {
        Self {
            inner: Arc::new(inner),
            metrics,
        }
    }

    pub fn get(&self, key: &K) -> Result<Option<V>, RedisCacheError> {
        let start = Instant::now();
        let result = self.inner.cache_get(key);
        let duration = start.elapsed();

        self.metrics
            .latency
            .with_label_values(&["user_ids", "get"])
            .observe(duration.as_secs_f64());

        match &result {
            Ok(Some(_)) => {
                let _ = debug!("Cache hit for key: {}", key);
                self.metrics.hits.with_label_values(&["user_ids"]).inc();
            }
            Ok(None) => {
                let _ = debug!("Cache miss for key: {}", key);
                self.metrics.misses.with_label_values(&["user_ids"]).inc();
            }
            Err(err) => {
                let _ = tracing::error!(
                    "Error retrieving cache value for key: {key} err: {err:?}"
                );
                self.metrics.errors.with_label_values(&["user_ids"]).inc();
            }
        }
        result
    }

    pub fn set(&self, key: K, value: V) -> Result<Option<V>, RedisCacheError> {
        let start = Instant::now();
        let _ = debug!("Setting cache value for key: {}", key);
        let result = self.inner.cache_set(key, value);
        let duration = start.elapsed();

        self.metrics
            .latency
            .with_label_values(&["user_ids", "set"])
            .observe(duration.as_secs_f64());

        if result.is_err() {
            self.metrics.errors.with_label_values(&["user_ids"]).inc();
        }

        result
    }

    pub fn remove(&self, key: &K) -> Result<Option<V>, RedisCacheError> {
        let start = Instant::now();
        let _ = debug!("Deleting cache value for key: {}", key);
        let result = self.inner.cache_remove(key);
        let duration = start.elapsed();

        self.metrics
            .latency
            .with_label_values(&["user_ids", "remove"])
            .observe(duration.as_secs_f64());

        if result.is_err() {
            self.metrics.errors.with_label_values(&["user_ids"]).inc();
        }

        result
    }
}
