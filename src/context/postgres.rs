use std::sync::Arc;

use anyhow::Result;
use async_lock::Mutex;
use diesel::{
    pg::PgConnection,
    prelude::*,
    r2d2::{ConnectionManager, Pool, PooledConnection},
};
use url::Url;

/// Pooled connection
pub type Conn = PooledConnection<ConnectionManager<PgConnection>>;

/// Postgres instance
#[derive(Clone)]
pub struct Postgres {
    pool: Arc<Mutex<Pool<ConnectionManager<PgConnection>>>>,
}

impl Postgres {
    /// Initialize database
    pub fn new(uri: &Url) -> Result<Self> {
        tracing::debug!("initializing database ...");

        Ok(Self {
            pool: Arc::new(Mutex::new(Pool::builder().build(ConnectionManager::<
                PgConnection,
            >::new(
                &uri.to_string()
            ))?)),
        })
    }

    /// Get pooled connection
    pub async fn conn(&self) -> Result<Conn> {
        self.pool.lock().await.get().map_err(Into::into)
    }
}
