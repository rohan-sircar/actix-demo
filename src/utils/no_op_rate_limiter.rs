use std::convert::Infallible;

use actix_extensible_rate_limit::backend::{
    Backend, Decision, SimpleInput, SimpleOutput,
};
use tokio::time::Instant;

#[derive(Clone, Default)]
pub struct NoopBackend;

impl Backend<SimpleInput> for NoopBackend {
    type Output = SimpleOutput;
    type RollbackToken = ();
    type Error = Infallible;

    async fn request(
        &self,
        _input: SimpleInput,
    ) -> Result<(Decision, Self::Output, Self::RollbackToken), Self::Error>
    {
        let output = SimpleOutput {
            limit: 0,
            remaining: 0,
            reset: Instant::now(),
        };
        Ok((Decision::Allowed, output, ()))
    }

    async fn rollback(
        &self,
        _token: Self::RollbackToken,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}
