//! Dexscreener APis
use serde::{Deserialize, Serialize};

const DEX_SCREENER: &str = "https://api.dexscreener.com/latest/dex";

/// Dexscreener API
pub struct DexScreenerApi;

impl DexScreenerApi {
    /// Dexscreener tokens
    pub fn tokens(mint: &str) -> String {
        format!("{DEX_SCREENER}/tokens/{mint}")
    }
}

#[derive(Clone, Deserialize, Debug)]
pub struct DexScreenerTokensResult {
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
