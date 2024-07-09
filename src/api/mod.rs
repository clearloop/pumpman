//! Apis

use anyhow::Result;
use reqwest::Client;
use serde::de::DeserializeOwned;
use std::sync::Arc;

mod dex;
mod pump;
mod sol;

pub use {dex::DexScreenerApi, pump::PumpApi, sol::SolRpcApi};

/// General http client
#[async_trait::async_trait]
pub trait HttpClient {
    /// Get the http client
    fn client(&self) -> &Arc<Client>;

    /// Shortcut for get
    async fn get<T: DeserializeOwned>(&self, uri: &str) -> Result<T> {
        self.client()
            .get(uri)
            .send()
            .await?
            .json()
            .await
            .map_err(Into::into)
    }
}
