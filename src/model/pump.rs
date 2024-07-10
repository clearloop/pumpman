//! Pumpfun models

use crate::api::Holders;
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

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

impl Coin {
    /// If dev is soldout
    pub fn soldout(&self, holders: &Holders) -> bool {
        !holders.iter().any(|acc| {
            if acc.0 != self.creator {
                return false;
            }

            // if dev is soldout
            BigDecimal::from_str(&acc.1)
                .map(|b| b < BigDecimal::from(100))
                .unwrap_or(true)
        })
    }

    /// generate social links in markdown
    pub fn socials(&self) -> String {
        let mut socials = vec![];

        if let Some(telegram) = &self.telegram {
            socials.push(format!("[telegram]({telegram})"));
        }

        if let Some(twitter) = &self.twitter {
            socials.push(format!("[twitter]({twitter})"));
        }

        if let Some(website) = &self.website {
            socials.push(format!("[website]({website})"));
        }

        format!("\n{}\n", socials.join(" | "))
    }
}
