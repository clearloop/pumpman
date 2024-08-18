//! Redis instance

use anyhow::Result;
use redis::{Client, Connection, ToRedisArgs};
use solana_sdk::pubkey::Pubkey;
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
pub enum Cache<'t> {
    DevSoldOut(&'t str),
    BondingCurve(&'t Pubkey),
    ACA(&'t Pubkey),
}

impl<'t> ToRedisArgs for Cache<'t> {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        match self {
            Self::DevSoldOut(mint) => {
                format!("task::takeover::soldout::{mint}").write_redis_args(out)
            }
            Self::BondingCurve(mint) => {
                format!("task::pumpman::bonding_curve::{mint}").write_redis_args(out)
            }
            Self::ACA(acc) => format!("task::pumpman::aca::{acc}").write_redis_args(out),
        }
    }
}
