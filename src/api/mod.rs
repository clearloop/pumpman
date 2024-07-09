//! Apis

use anyhow::Result;
use redis::{Commands, Connection};
use reqwest::Client;
use serde::de::DeserializeOwned;
use std::sync::Arc;

pub mod dex;
mod pump;
mod sol;

use self::dex::DexScreenerTokensResult;
use crate::model::pump::Coin;
pub use {dex::DexScreenerApi, pump::PumpApi, sol::SolRpcApi};

/// General http client
#[async_trait::async_trait]
pub trait HttpClient {
    /// Get the http client
    fn client(&self) -> &Arc<Client>;

    /// Cache expiration in seconds
    const CACHE_MAX_AGE: u64;

    /// Get result with redis cache
    async fn cget<T: DeserializeOwned>(
        &self,
        uri: &str,
        force: bool,
        con: &mut Connection,
    ) -> Result<T> {
        let mbv = con.get(uri);
        let text = if mbv.is_err() || force {
            let text = self.client().get(uri).send().await?.text().await?;
            con.set_ex(uri, &text, Self::CACHE_MAX_AGE)?;
            text
        } else {
            mbv?
        };

        serde_json::from_str(&text).map_err(Into::into)
    }

    /// get coin of pump fun
    async fn coin(&self, mint: &str, update: bool, con: &mut Connection) -> Result<Coin> {
        self.cget(&PumpApi::coin(mint), update, con).await
    }

    /// dexscreener tokens
    async fn tokens(
        &self,
        mint: &str,
        update: bool,
        con: &mut Connection,
    ) -> Result<DexScreenerTokensResult> {
        self.cget(&DexScreenerApi::tokens(mint), update, con).await
    }

    /// Get dexscreener url
    async fn pair(&self, mint: &str, update: bool, con: &mut Connection) -> Option<String> {
        self.tokens(mint, update, con)
            .await
            .ok()?
            .pairs?
            .first()
            .map(|p| p.url.clone())
    }
}
