use anyhow::Result;
use diesel_async::{
    pooled_connection::{
        deadpool::{Object, Pool},
        AsyncDieselConnectionManager,
    },
    AsyncPgConnection, RunQueryDsl,
};
use std::sync::Arc;
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
pub type Conn = Object<AsyncPgConnection>;

/// Postgres instance
#[derive(Clone)]
pub struct Postgres {
    pool: Arc<Pool<AsyncPgConnection>>,
}

impl Postgres {
    /// Initialize database
    pub fn new(uri: &Url) -> Result<Self> {
        tracing::debug!("initializing database ...");

        let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(uri.to_string());
        let pool = Pool::builder(config).max_size(10).build()?;

        let this = Self {
            pool: Arc::new(pool),
        };

        Ok(this)
    }

    /// Get pooled connection
    pub async fn conn(&self) -> Result<Conn> {
        self.pool.get().await.map_err(Into::into)
    }

    /// Init database
    pub async fn init(&self) -> Result<()> {
        self.create_tables().await
    }

    /// Create tables if not exists
    async fn create_tables(&self) -> Result<()> {
        let conn = &mut self.conn().await?;
        for create_table in CREATE_TABLES {
            if let Err(e) = diesel::sql_query(create_table).execute(conn).await {
                tracing::debug!("Failed to table: {e}\n{}", create_table);
            }
        }

        Ok(())
    }
}
