//! Dexscreener pairs

use std::{fmt::Display, ops::Div};

use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct DexScreenerPair {
    /// dex name of this pair
    #[serde(rename = "dexId")]
    pub dex_id: String,
    /// dexscreener url of this pair
    pub url: String,
    pub txns: Transactions,

    #[serde(rename = "priceChange")]
    pub price_change: PriceChange,

    pub liquidity: Liquidity,

    pub fdv: BigDecimal,

    pub volume: Volume,
}

/// ds txns
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Transactions {
    pub m5: Transaction,
    pub h1: Transaction,
    pub h6: Transaction,
    pub h24: Transaction,
}

/// transaction
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Transaction {
    pub buys: u64,
    pub sells: u64,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct PriceChange {
    pub m5: f32,
    pub h1: f32,
    pub h6: f32,
    pub h24: f32,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Liquidity {
    pub usd: BigDecimal,
    pub base: BigDecimal,
    pub quote: BigDecimal,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Volume {
    pub h24: BigDecimal,
    pub h6: BigDecimal,
    pub h1: BigDecimal,
    pub m5: BigDecimal,
}

impl Display for DexScreenerPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r#"
market cap: ${}k
"#,
            self.fdv.clone().div(1000)
        )
    }
}
