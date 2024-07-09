//! Redis instance

use crate::utils::DAY;
use anyhow::Result;
use async_lock::Mutex;
use redis::{Client, Commands, Connection};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;
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

/// Set value to redis
pub fn set<T: Serialize>(
    key: &str,
    value: &T,
    exp: impl Into<Option<u64>>,
    con: &mut Connection,
) -> Result<()> {
    con.set_ex(
        key,
        serde_json::to_string(value)?,
        exp.into().unwrap_or(DAY),
    )?;
    Ok(())
}

/// Get parsed value from redis
pub fn get<T: DeserializeOwned>(key: &str, con: &mut Connection) -> Result<T> {
    let s: String = con.get(key)?;
    serde_json::from_str(&s).map_err(Into::into)
}
