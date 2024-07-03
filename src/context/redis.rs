//! Redis instance

use anyhow::Result;
use redis::{Client, Connection};
use url::Url;

/// Redis instance
pub struct Redis(Client);

impl Redis {
    /// Redis url
    pub fn new(uri: Url) -> Result<Self> {
        Ok(Self(Client::open(uri)?))
    }

    /// Get redis connection
    pub fn con(&self) -> Result<Connection> {
        self.0.get_connection().map_err(Into::into)
    }
}
