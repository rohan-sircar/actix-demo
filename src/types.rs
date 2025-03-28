use std::fmt::Display;

use ::r2d2::PooledConnection;
use diesel::r2d2::{self, ConnectionManager};
use diesel_tracing::pg::InstrumentedPgConnection;
use tokio::task::JoinHandle;

use crate::errors::DomainError;
pub type DbPool = r2d2::Pool<ConnectionManager<InstrumentedPgConnection>>;
pub type DbConnection =
    PooledConnection<ConnectionManager<InstrumentedPgConnection>>;
pub type RedisPrefixFn = Box<dyn Fn(&dyn Display) -> String + Send + Sync>;
pub type Task<T> = JoinHandle<Result<T, DomainError>>;
