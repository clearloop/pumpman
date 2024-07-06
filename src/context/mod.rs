//! Global context

use crate::{model::Coin, schema::coins, Config};

use self::{client::Client, postgres::Conn};
use ::redis::{Commands, Connection};
use anyhow::Result;
use diesel::QueryDsl;
use diesel::*;
use mpl_token_metadata::accounts::Metadata;
use url::Url;
pub use {postgres::Postgres, redis::Redis, telegram::Telegram};

mod client;
mod postgres;
mod redis;
mod telegram;

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

    /// Postgres connection
    pub async fn postgres(&self) -> Result<Conn> {
        self.postgres.conn().await
    }

    /// Redis connection
    pub async fn redis(&self) -> Result<Connection> {
        self.redis.con().await.map_err(Into::into)
    }

    /// Get token metadata
    pub async fn coin(&self, mint: &str) -> Result<Coin> {
        let postgres = &mut self.postgres().await?;

        let coin = coins::table
            .filter(coins::mint.eq(mint))
            .first::<Coin>(postgres)
            .optional();

        // let mbmeta = if let Some(meta) = redis
        //     .get(mint)
        //     .ok()
        //     .and_then(|s: String| serde_json::from_str(&s).ok())
        // {
        //     return Ok(meta);
        // };
        //
        // self.client.metadata(mint).await?;
        //
        todo!()
    }
}
