use actix_extensible_rate_limit::backend;
use actix_extensible_rate_limit::backend::memory::InMemoryBackend;
use actix_extensible_rate_limit::backend::redis::RedisBackend;
use actix_extensible_rate_limit::backend::{
    Backend, Decision, SimpleInput, SimpleOutput,
};
use thiserror::Error;
use tokio::time::Instant;

#[derive(Debug, Error)]
pub enum RateLimitBackendError {
    #[error("Redis error: {0}")]
    Redis(#[from] backend::redis::Error),
    #[error("InMemory error: {0}")]
    InMemory(#[from] std::convert::Infallible),
}

impl actix_web::ResponseError for RateLimitBackendError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            RateLimitBackendError::Redis(_)
            | RateLimitBackendError::InMemory(_) => {
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }
}

#[derive(Clone)]
pub enum RateLimitBackend {
    Noop,
    Redis(RedisBackend),
    InMemory(InMemoryBackend),
}

impl Backend<SimpleInput> for RateLimitBackend {
    type Output = SimpleOutput;
    type RollbackToken = String;
    type Error = RateLimitBackendError;

    async fn request(
        &self,
        input: SimpleInput,
    ) -> Result<(Decision, Self::Output, Self::RollbackToken), Self::Error>
    {
        match self {
            RateLimitBackend::Noop => {
                let output = SimpleOutput {
                    limit: 0,
                    remaining: 0,
                    reset: Instant::now(),
                };
                Ok((Decision::Allowed, output, "".to_string()))
            }
            RateLimitBackend::Redis(redis) => redis
                .request(input)
                .await
                .map_err(RateLimitBackendError::Redis),
            RateLimitBackend::InMemory(memory) => memory
                .request(input)
                .await
                .map_err(RateLimitBackendError::InMemory),
        }
    }

    async fn rollback(
        &self,
        token: Self::RollbackToken,
    ) -> Result<(), Self::Error> {
        match self {
            RateLimitBackend::Noop => Ok(()),
            RateLimitBackend::Redis(redis) => redis
                .rollback(token)
                .await
                .map_err(RateLimitBackendError::Redis),
            RateLimitBackend::InMemory(memory) => memory
                .rollback(token)
                .await
                .map_err(RateLimitBackendError::InMemory),
        }
    }
}
