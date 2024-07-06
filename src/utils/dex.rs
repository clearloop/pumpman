//! Dex utils

use std::ops::Deref;

use crate::{redis, utils::DAY};
use ::redis::{Commands, Connection};
use anyhow::Result;
use dexscreener::{DexScreenerPair, DexScreenerResult};
use serde::{Deserialize, Serialize};

/// Dex score of the token
#[derive(Deserialize, Serialize)]
pub struct Dex(Vec<DexScreenerPair>);

impl Dex {
    /// Fetch token
    pub async fn fetch(mint: &str, con: &mut Connection) -> Result<Self> {
        let key = Self::key(&mint);

        if let Ok(cache) = redis::get(&key, con) {
            return Ok(cache);
        }

        if let Ok(fresh) = dexscreener::fetch(mint).await {
            let this = Self(fresh.pairs);
            redis::set(&key, &this, DAY, con)?;
            return Ok(this);
        }

        Ok(Self(vec![]))
    }

    /// Returns dexscreener url if exists
    pub async fn dexscreener(mint: &str, con: &mut Connection) -> Option<String> {
        tracing::trace!("Fetching pairs for {mint}");
        let key = Self::key(&mint);
        let this: Self = Self::fetch(mint, con)
            .await
            .map_err(|e| tracing::error!("{e}"))
            .ok()?;
        this.first().and_then(|p| Some(p.url.clone()))
    }

    /// Get the redis key
    fn key(mint: &str) -> String {
        format!("dex::{mint}")
    }
}

impl Deref for Dex {
    type Target = Vec<DexScreenerPair>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

mod dexscreener {
    use anyhow::Result;
    use serde::{Deserialize, Serialize};

    const DEX_SCREENER: &str = "https://api.dexscreener.com/latest/dex/tokens/";

    #[derive(Clone, Deserialize)]
    pub struct DexScreenerResult {
        pub pairs: Vec<DexScreenerPair>,
    }

    #[derive(Clone, Deserialize, Serialize)]
    pub struct DexScreenerPair {
        /// dex name of this pair
        #[serde(rename = "dexId")]
        pub dex_id: String,
        /// dexscreener url of this pair
        pub url: String,
    }

    /// Fetch pairs from dex screener
    pub async fn fetch(mint: &str) -> Result<DexScreenerResult> {
        reqwest::get(format!("{DEX_SCREENER}/{}", mint))
            .await?
            .json::<DexScreenerResult>()
            .await
            .map_err(Into::into)
    }
}
