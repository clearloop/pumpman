//! Redis instance

use anyhow::Result;
use redis::{Client, Connection, ToRedisArgs};
use std::sync::Arc;
use url::Url;

/// Redis instance
#[derive(Clone)]
pub struct Redis(Arc<Client>);

impl Redis {
    /// Redis url
    pub fn new(uri: &Url) -> Result<Self> {
        Ok(Self(Arc::new(Client::open(uri.to_string())?)))
    }

    /// Get redis connection
    pub fn con(&self) -> Result<Connection> {
        self.0.get_connection().map_err(Into::into)
    }
}

/// Redis task cache
pub enum TaskCache<'t> {
    DevSoldOut(&'t str),
    Top10Holder { mint: &'t str, percent: u8 },
}

impl<'t> ToRedisArgs for TaskCache<'t> {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        match self {
            Self::DevSoldOut(mint) => mint.write_redis_args(out),
            Self::Top10Holder { mint, percent } => {
                mint.write_redis_args(out);
                percent.write_redis_args(out);
            }
        }
    }
}
