//! Model of dev soldout alert
#![allow(dead_code)]

use crate::{
    api::Holders,
    model::{pump::Coin, DexScreenerPair},
};
use anyhow::Result;
use std::fmt::Display;
use teloxide::{
    requests::Requester,
    types::{Message, Recipient},
    Bot,
};

/// Basic alert
///
/// TODO: takeover details
pub struct Alert {
    pub title: AlertTitle,
    /// Pumpfun coins
    pub coin: Coin,
    /// Dexscreener apirs
    pub pairs: Vec<DexScreenerPair>,
    /// Top holders
    pub holders: Holders,
}

impl Alert {
    /// Create new alert
    pub fn new(title: AlertTitle, coin: Coin) -> Self {
        Self {
            title,
            coin,
            pairs: Default::default(),
            holders: Default::default(),
        }
    }

    /// Set up pairs
    pub fn pairs(mut self, pairs: Vec<DexScreenerPair>) -> Self {
        self.pairs = pairs;
        self
    }

    /// Set up holders
    pub fn holders(mut self, holders: Holders) -> Self {
        self.holders = holders;
        self
    }

    pub async fn alert(&self, bot: &Bot, channel: &str) -> Result<Message> {
        bot.send_message(
            Recipient::ChannelUsername(channel.to_string()),
            self.to_string(),
        )
        .await
        .map_err(Into::into)
    }
}

impl Display for Alert {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let soldout = self.coin.soldout(&self.holders);
        let percent = self.holders.top10percent().unwrap_or_default();
        let socials = self.coin.socials();
        let dex = !self.pairs.is_empty();
        let (title, name, symbol, mint) = (
            &self.title,
            &self.coin.name,
            &self.coin.symbol,
            &self.coin.mint,
        );

        write!(
            f,
            r#"
{title} - [{name}](https://pump.fun/{mint}) (${symbol})

1. dev wallet sold out: {soldout}
2. top10 holders HODL: {percent}%
3. listed on dex: {dex}
{socials}
Need more factors? join @takeoverfyi to share your ideas!
"#,
        )
    }
}

/// Title of message alert
pub enum AlertTitle {
    DevSoldOut,
}

impl Display for AlertTitle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::DevSoldOut => "Dev Soldout",
            }
        )
    }
}
