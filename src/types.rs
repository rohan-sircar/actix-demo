use std::fmt::Display;

use diesel::r2d2::{self, ConnectionManager};
use diesel_tracing::pg::InstrumentedPgConnection;
pub type DbPool = r2d2::Pool<ConnectionManager<InstrumentedPgConnection>>;
pub type RedisPrefixFn = Box<dyn Fn(&dyn Display) -> String + Send + Sync>;
