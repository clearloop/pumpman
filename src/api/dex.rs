//! Dexscreener APis
use crate::api::HttpClient;
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

const DEX_SCREENER: &str = "https://api.dexscreener.com/latest/dex/tokens/";

/// Dexscreener APi
#[async_trait]
pub trait DexScreenerApi: HttpClient {
    /// Get one or multiple pairs by token address
    async fn tokens(&self, mint: &str) -> Result<DexScreenerResult> {
        self.get(&format!("{DEX_SCREENER}/{}", mint))
            .await
            .map_err(Into::into)
    }
}

#[derive(Clone, Deserialize, Debug)]
pub struct DexScreenerResult {
    pub pairs: Option<Vec<DexScreenerPair>>,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct DexScreenerPair {
    /// dex name of this pair
    #[serde(rename = "dexId")]
    pub dex_id: String,
    /// dexscreener url of this pair
    pub url: String,
}
