//! Model of dev soldout alert
#![allow(dead_code)]

use crate::{
    api::Holders,
    model::{pump::Coin, DexScreenerPair},
    telegram::Escape,
};
use anyhow::Result;
use std::fmt::Display;
use teloxide::{
    payloads::SendMessageSetters,
    requests::Requester,
    types::{Message, ParseMode, Recipient},
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
    /// If dev is soldout
    pub soldout: bool,
}

impl Alert {
    /// Create new alert
    pub fn new(title: AlertTitle, coin: Coin, soldout: bool) -> Self {
        Self {
            title,
            coin,
            soldout,
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
            &self.to_string(),
        )
        .parse_mode(ParseMode::MarkdownV2)
        // .reply_markup(value)
        // .disable_web_page_preview(true)
        .await
        .map_err(Into::into)
    }
}

impl Display for Alert {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let percent = self
            .holders
            .percent()
            .unwrap_or_default()
            .to_string()
            .escaped();
        let socials = self.coin.socials();
        let dex = !self.pairs.is_empty();
        let (address, title, name, symbol, mint, mc, count, soldout) = (
            &self.coin.mint,
            &self.title.escaped(),
            &self.coin.name.escaped(),
            &self.coin.symbol.escaped(),
            &self.coin.mint.escaped(),
            &self
                .coin
                .usd_market_cap
                .clone()
                .map(|mc| (mc / 1000u32).round(2))
                .unwrap_or_default()
                .escaped(),
            self.holders.len(),
            self.soldout,
        );

        let count = if count == 19 {
            "> 20".escaped()
        } else {
            format!("{count}")
        };

        write!(
            f,
            r#"
*{title}* \- [{name}](https://pump.fun/{mint}) \(${symbol}\)

\- dev wallet sold out: {soldout}
\- market cap: ${mc}k
\- holders count: {count}
\- top 20 holders HODL: {percent}%
\- listed on dex: {dex}
{socials}
```copy
{address}
```
"#,
        )
    }
}

/// Title of message alert
pub enum AlertTitle {
    ClaimCTO,
    DevSoldOut,
    DevOutOfTop20,
    HoldersChanged(u8),
}

impl Display for AlertTitle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::ClaimCTO => "Claim CTO".into(),
                Self::DevSoldOut => "Dev Soldout".into(),
                Self::DevOutOfTop20 => "Dev Out of Top 20".into(),
                Self::HoldersChanged(threshold) => format!("Top 20 HODL under {threshold}%"),
            }
        )
    }
}
