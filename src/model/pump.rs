//! Pumpfun models

use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

/// Pump fun coin model
///
/// https://frontend-api.pump.fun/coins/:coin
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Coin {
    /// Mint address of this coin
    pub mint: String,
    /// Name of this coin
    pub name: String,
    /// Symbol of this coin
    pub symbol: String,
    /// Description of this coin
    pub description: Option<String>,
    /// Image uri of this coin
    pub image_uri: String,
    /// Twitter of this coin
    pub twitter: Option<String>,
    /// Telegram of this coin
    pub telegram: Option<String>,
    /// Bonding curve address
    pub bonding_curve: String,
    /// Associated bonding curve
    pub associated_bonding_curve: String,
    /// Creator of this coin
    pub creator: String,
    /// Website of this coin
    pub website: Option<String>,
    /// King of hill
    pub king_of_hill_timestamp: Option<u64>,
    /// current market cap
    pub market_cap: Option<BigDecimal>,
    /// Reply count
    pub reply_count: u32,
    /// last reply
    pub last_reply: u64,
    /// username of the dev
    pub username: String,
    /// Profile image of the dev
    pub profile_image: String,
    /// market cap in USD
    pub usd_market_cap: Option<BigDecimal>,
}
