use diesel::pg::Pg;
use diesel::result::QueryResult;
use diesel::{query_builder::*, PgConnection};
use diesel::{Connection, RunQueryDsl};

// below code is taken from diesel_cli

fn change_database_of_url(
    database_url: &str,
    default_database: &str,
) -> (String, String) {
    let base = ::url::Url::parse(database_url).unwrap();
    let database = base.path_segments().unwrap().last().unwrap().to_owned();
    let mut new_url = base.join(default_database).unwrap();
    new_url.set_query(base.query());
    (database, new_url.to_string())
}

#[derive(Debug, Clone)]
pub struct CreateDatabaseStatement {
    db_name: String,
}

impl CreateDatabaseStatement {
    pub fn new(db_name: &str) -> Self {
        CreateDatabaseStatement {
            db_name: db_name.to_owned(),
        }
    }
}

impl QueryFragment<Pg> for CreateDatabaseStatement {
    fn walk_ast(&self, mut out: AstPass<Pg>) -> QueryResult<()> {
        out.push_sql("CREATE DATABASE ");
        out.push_identifier(&self.db_name)?;
        Ok(())
    }
}

impl<Conn> RunQueryDsl<Conn> for CreateDatabaseStatement {}

impl QueryId for CreateDatabaseStatement {
    type QueryId = ();

    const HAS_STATIC_QUERY_ID: bool = false;
}

fn create_database(db_name: &str) -> CreateDatabaseStatement {
    CreateDatabaseStatement::new(db_name)
}

pub fn create_database_if_needed(database_url: &str) -> anyhow::Result<()> {
    if PgConnection::establish(database_url).is_err() {
        let (database, postgres_url) =
            change_database_of_url(database_url, "postgres");
        tracing::info!("Creating database: {database}");
        let conn = PgConnection::establish(&postgres_url)?;
        create_database(&database).execute(&conn)?;
    } else {
        tracing::info!("Detected existing database")
    }
    Ok(())
}
