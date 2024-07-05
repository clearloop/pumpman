//! Redis instance

use std::sync::Arc;

use anyhow::Result;
use async_lock::Mutex;
use redis::{Client, Connection};
use url::Url;

/// Redis instance
#[derive(Clone)]
pub struct Redis(Arc<Mutex<Client>>);

impl Redis {
    /// Redis url
    pub fn new(uri: &Url) -> Result<Self> {
        Ok(Self(Arc::new(Mutex::new(Client::open(uri.to_string())?))))
    }

    /// Get redis connection
    pub async fn con(&self) -> Result<Connection> {
        self.0.lock().await.get_connection().map_err(Into::into)
    }
}
