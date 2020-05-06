use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
pub type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;
