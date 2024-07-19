//! Apis

use crate::{
    model::{pump::Coin, DexScreenerPair},
    utils::{MIN, THOURS},
};
use anyhow::Result;
use dex::DexScreenerTokensResult;
use redis::{Commands, Connection};
use reqwest::Client;
use serde::de::DeserializeOwned;
use std::sync::Arc;
pub use {
    dex::DexScreenerApi,
    pump::PumpApi,
    sol::{Holders, SolRpcApi},
};

pub mod dex;
mod pump;
mod sol;

/// General http client
#[async_trait::async_trait]
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

    /// get coin of pump fun
    async fn coin(&self, mint: &str, update: bool, con: &mut Connection) -> Result<Coin> {
        self.cget(&PumpApi::coin(mint), update, MIN, con)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get pump coin {mint}: {e}");
                e
            })
    }

    /// dexscreener pairs
    async fn pairs(
        &self,
        mint: &str,
        update: bool,
        con: &mut Connection,
    ) -> Result<Vec<DexScreenerPair>> {
        let tokens: DexScreenerTokensResult = self
            .cget(&DexScreenerApi::tokens(mint), update, THOURS, con)
            .await?;
        Ok(tokens.pairs.unwrap_or_default())
    }

    /// Get dexscreener url
    async fn pair(&self, mint: &str, update: bool, con: &mut Connection) -> Option<String> {
        self.pairs(mint, update, con)
            .await
            .ok()?
            .first()
            .map(|p| p.url.clone())
    }
}
