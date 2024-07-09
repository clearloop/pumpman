//! Apis

use crate::{
    model::pump::Coin,
    utils::{DAY, THOURS},
};
use anyhow::Result;
use dex::DexScreenerTokensResult;
use redis::{Commands, Connection};
use reqwest::Client;
use serde::de::DeserializeOwned;
use std::sync::Arc;
pub use {dex::DexScreenerApi, pump::PumpApi, sol::SolRpcApi};

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
        con: &mut Connection,
    ) -> Result<T> {
        let mbv = con.get(uri);
        let text = if mbv.is_err() || force {
            let text = self.client().get(uri).send().await?.text().await?;
            con.set_ex(uri, &text, exp)?;
            text
        } else {
            mbv?
        };

        serde_json::from_str(&text).map_err(Into::into)
    }

    /// get coin of pump fun
    async fn coin(&self, mint: &str, update: bool, con: &mut Connection) -> Result<Coin> {
        self.cget(&PumpApi::coin(mint), update, DAY, con).await
    }

    /// dexscreener tokens
    async fn tokens(
        &self,
        mint: &str,
        update: bool,
        con: &mut Connection,
    ) -> Result<DexScreenerTokensResult> {
        self.cget(&DexScreenerApi::tokens(mint), update, THOURS, con)
            .await
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
