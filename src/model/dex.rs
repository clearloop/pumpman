//! Dexscreener pairs

use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct DexScreenerPair {
    /// dex name of this pair
    #[serde(rename = "dexId")]
    pub dex_id: String,
    /// dexscreener url of this pair
    pub url: String,
}
