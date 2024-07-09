//! Redis instance

use anyhow::Result;
use redis::{Client, Connection};
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
