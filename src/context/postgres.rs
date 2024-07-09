use std::sync::Arc;

use anyhow::Result;
use diesel::{
    pg::PgConnection,
    prelude::*,
    r2d2::{ConnectionManager, Pool, PooledConnection},
};
use url::Url;

/// A list of SQL statements to create the tables
///
/// NOTE: do not change the orders since there
/// are references inside.
const CREATE_TABLES: [&str; 3] = [
    include_str!("../../sql/coins.sql"),
    include_str!("../../sql/users.sql"),
    include_str!("../../sql/takeovers.sql"),
];

/// Pooled connection
pub type Conn = PooledConnection<ConnectionManager<PgConnection>>;

/// Postgres instance
#[derive(Clone)]
pub struct Postgres {
    pool: Arc<Pool<ConnectionManager<PgConnection>>>,
}

impl Postgres {
    /// Initialize database
    pub fn new(uri: &Url) -> Result<Self> {
        tracing::debug!("initializing database ...");

        let this = Self {
            pool: Arc::new(
                Pool::builder().build(ConnectionManager::<PgConnection>::new(uri.to_string()))?,
            ),
        };

        Ok(this)
    }

    /// Get pooled connection
    pub fn conn(&self) -> Result<Conn> {
        self.pool.get().map_err(Into::into)
    }

    /// Init database
    pub fn init(&self) -> Result<()> {
        self.create_tables()
    }

    /// Create tables if not exists
    fn create_tables(&self) -> Result<()> {
        let conn = &mut self.conn()?;
        for create_table in CREATE_TABLES {
            if let Err(e) = diesel::sql_query(create_table).execute(conn) {
                tracing::debug!("Failed to table: {e}\n{}", create_table);
            }
        }

        Ok(())
    }
}
