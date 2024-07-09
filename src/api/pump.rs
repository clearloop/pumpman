#![allow(unused)]
use crate::{api::HttpClient, model::pump::Coin};
use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde::de::DeserializeOwned;
use std::sync::Arc;

/// Api of pumpfun
#[async_trait]
pub trait PumpApi: HttpClient {
    /// Get pumpfun coin
    ///
    /// https://frontend-api.pump.fun/coins/:coin
    async fn coin(&self, mint: &str) -> Result<Coin> {
        self.get(&format!("https://frontend-api.pump.fun/coins/{mint}"))
            .await
    }
}
