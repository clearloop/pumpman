//! Dexscreener APis
use crate::model::DexScreenerPair;
use serde::Deserialize;

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
