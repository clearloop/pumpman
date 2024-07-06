//! Basic coin information

use crate::{context::Context, redis, schema::coins, utils::Dex};
use ::redis::Connection;
use anyhow::Result;
use async_graphql::SimpleObject;
use diesel::prelude::*;
use mpl_token_metadata::accounts::Metadata;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use teloxide::types::{
    InlineKeyboardButton, InlineKeyboardButtonKind, InlineKeyboardMarkup, ReplyMarkup,
};

/// Original coin information
#[derive(
    SimpleObject,
    Insertable,
    Queryable,
    Selectable,
    AsChangeset,
    Clone,
    PartialEq,
    Debug,
    Serialize,
    Default,
    Eq,
    PartialOrd,
    Ord,
    Hash,
)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Coin {
    /// Description of this coin
    pub description: Option<String>,
    /// Image of this coin
    pub image: Option<String>,
    /// mint address of this coin
    pub mint: String,
    /// Name of this coin
    pub name: String,
    /// Symbol of this coin
    pub symbol: String,
    /// Telegram of this coin
    pub telegram: Option<String>,
    /// Twitter of this coin
    pub twitter: Option<String>,
    /// Website of this coin
    pub website: Option<String>,
    /// Where this coin created on
    pub created_on: Option<String>,
}

impl Coin {
    /// Get self from context
    pub async fn get(mint: &str, context: &Context) -> Result<Self> {
        let conn = &mut context.postgres().await?;
        if let Ok(coin) = coins::table
            .filter(coins::mint.eq(mint))
            .first::<Self>(conn)
        {
            return Ok(coin);
        }

        let coin = context.client.coin(mint).await?;
        diesel::insert_into(coins::table)
            .values(&coin)
            .execute(conn)?;
        Ok(coin)
    }

    /// Append json to metadata
    pub fn append(&mut self, json: Value) {
        let mbstr = |field: &str| {
            json.get(field)
                .and_then(|v| v.as_str())
                .map(|s| s.trim().to_string())
        };

        self.description = mbstr("description");
        self.image = mbstr("image");
        self.twitter = mbstr("twitter");
        self.telegram = mbstr("telegram");
        self.website = mbstr("website");
        self.created_on = mbstr("createdOn");
    }

    /// Get telegram keyboards for this coin
    pub async fn keyboards(&self, con: &mut Connection) -> Result<ReplyMarkup> {
        let mut keyboards = vec![];
        if let Some(true) = self.created_on.as_ref().map(|p| p.contains("pump.fun")) {
            keyboards.push(InlineKeyboardButton::url(
                "View on pump.fun",
                format!("https://pump.fun/{}", self.mint).parse()?,
            ));
        }

        if let Some(dex) = Dex::dexscreener(&self.mint, con).await {
            keyboards.push(InlineKeyboardButton::url(
                "View on dexscreener",
                dex.parse()?,
            ));
        }

        Ok(ReplyMarkup::InlineKeyboard(InlineKeyboardMarkup {
            inline_keyboard: vec![keyboards],
        }))
    }
}

impl From<Metadata> for Coin {
    fn from(m: Metadata) -> Self {
        Coin {
            name: m.name.trim().to_string().replace("\0", ""),
            symbol: m.symbol.to_string().replace("\0", ""),
            mint: m.mint.to_string(),
            ..Default::default()
        }
    }
}
