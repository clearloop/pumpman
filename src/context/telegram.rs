//! Telegram instance

use crate::{config::Telegram as TelegramConfig, context::Redis, sol::pump::events::TradeEvent};
use anyhow::Result;
use redis::Commands;
use teloxide::{
    payloads::SendMessageSetters,
    requests::Requester,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode, Recipient, ReplyMarkup},
    Bot, RequestError,
};
use tokio::time;

/// Telegram instance
#[derive(Clone)]
pub struct Telegram {
    bot: Bot,
    subscription: String,
    redis: Redis,
}

impl Telegram {
    /// Create a new telegram instance
    pub fn new(config: &TelegramConfig, redis: Redis) -> Self {
        Self {
            bot: Bot::new(&config.token),
            subscription: config.subscription.clone(),
            redis,
        }
    }

    /// Subscribe trade event
    pub async fn trade(&self, trade: TradeEvent) -> Result<()> {
        let mut redis = self.redis.con().await?;
        let symbol: String = redis.get(trade.mint.to_string())?;
        let message = trade.to_string().replace("{SYMBOL}", &symbol);

        if let Err(e) = self
            .bot
            .send_message(
                Recipient::ChannelUsername(self.subscription.clone()),
                message,
            )
            .parse_mode(ParseMode::MarkdownV2)
            .disable_web_page_preview(true)
            .await
        {
            if let RequestError::RetryAfter(dur) = e {
                time::sleep(dur).await
            } else {
                return Err(e.into());
            }
        }

        Ok(())
    }

    /// Subscribe trade event
    pub async fn subscribe(&self, message: String) -> Result<()> {
        self.bot
            .send_message(
                Recipient::ChannelUsername(self.subscription.clone()),
                message,
            )
            .parse_mode(ParseMode::MarkdownV2)
            .reply_markup(markup()?)
            .disable_web_page_preview(true)
            .await?;
        Ok(())
    }
}

/// Subscription markups
fn markup() -> Result<ReplyMarkup> {
    Ok(ReplyMarkup::InlineKeyboard(InlineKeyboardMarkup {
        inline_keyboard: vec![vec![InlineKeyboardButton {
            text: "pumpman.io".into(),
            kind: teloxide::types::InlineKeyboardButtonKind::Url("https://pumpman.io".parse()?),
        }]],
    }))
}
