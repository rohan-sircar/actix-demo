use actix_extensible_rate_limit::backend;
use actix_extensible_rate_limit::backend::redis::RedisBackend;
use actix_extensible_rate_limit::backend::{
    Backend, Decision, SimpleInput, SimpleOutput,
};
use tokio::time::Instant;

#[derive(Clone)]
pub enum RateLimitBackend {
    Noop,
    Redis(RedisBackend),
}

impl Backend<SimpleInput> for RateLimitBackend {
    type Output = SimpleOutput;
    type RollbackToken = String;
    type Error = backend::redis::Error;

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
            RateLimitBackend::Redis(redis) => redis.request(input).await,
        }
    }

    async fn rollback(
        &self,
        token: Self::RollbackToken,
    ) -> Result<(), Self::Error> {
        match self {
            RateLimitBackend::Noop => Ok(()),
            RateLimitBackend::Redis(redis) => redis.rollback(token).await,
        }
    }
}
