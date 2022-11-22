use diesel::r2d2::{self, ConnectionManager};
pub type DbPool =
    r2d2::Pool<ConnectionManager<diesel_tracing::pg::InstrumentedPgConnection>>;
