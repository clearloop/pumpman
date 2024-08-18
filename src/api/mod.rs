//! Apis

use anyhow::Result;
use redis::{Commands, Connection};
use reqwest::Client;
use serde::de::DeserializeOwned;
use std::sync::Arc;
pub use {
    dex::DexScreenerApi,
    pump::{PumpApi, PUMPFUN_FEE_BASIS},
    sol::{Holders, SolRpcApi},
};

mod dex;
mod pump;
mod sol;

/// General http client
pub trait HttpClient {
    /// Get the http client
    fn client(&self) -> &Arc<Client>;

    /// Get result with redis cache
    async fn cget<T: DeserializeOwned>(
        &self,
        uri: &str,
        force: bool,
        exp: u64,
        redis: &mut Connection,
    ) -> Result<T> {
        let text = if !redis.exists(uri)? || force {
            let text = self.client().get(uri).send().await?.text().await?;
            redis.set_ex(uri, &text, exp)?;
            text
        } else {
            redis.get(uri)?
        };

        serde_json::from_str(&text).map_err(Into::into)
    }
}
