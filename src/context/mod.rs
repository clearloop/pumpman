//! Global context

use self::postgres::Conn;
use ::redis::Connection;
use anyhow::Result;
use url::Url;
pub use {postgres::Postgres, redis::Redis, telegram::Telegram};

mod postgres;
mod redis;
mod telegram;

/// Database interface
#[derive(Clone)]
pub struct Db {
    /// pg interface
    pub postgres: Postgres,
    /// redis interface
    pub redis: Redis,
}

impl Db {
    /// Create new database interface
    pub fn new(postgres: &Url, redis: &Url) -> Result<Self> {
        Ok(Self {
            postgres: Postgres::new(postgres)?,
            redis: Redis::new(redis)?,
        })
    }

    /// Postgres connection
    pub async fn postgres(&self) -> Result<Conn> {
        self.postgres.conn().await
    }

    /// Redis connection
    pub async fn redis(&self) -> Result<Connection> {
        self.redis.con().await.map_err(Into::into)
    }
}
