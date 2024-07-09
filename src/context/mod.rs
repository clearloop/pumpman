//! Global context

use self::{client::Client, postgres::Conn};
use crate::Config;
use ::redis::Connection;
use anyhow::Result;
pub use {postgres::Postgres, redis::Redis};

mod client;
mod postgres;
pub mod redis;
// mod telegram;

/// Database interface
#[derive(Clone)]
pub struct Context {
    /// pg interface
    pub postgres: Postgres,
    /// redis interface
    pub redis: Redis,
    /// Solana client
    pub client: Client,
}

impl Context {
    /// Create new database interface
    pub fn new(config: &Config) -> Result<Self> {
        Ok(Self {
            postgres: Postgres::new(&config.postgres)?,
            redis: Redis::new(&config.redis)?,
            client: Client::new(&config.cluster)?,
        })
    }

    /// Init context
    pub async fn init(&self) -> Result<()> {
        self.postgres.init().await
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
