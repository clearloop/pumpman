use crate::{
    api::HttpClient,
    context::Context,
    model::{Takeover, TakeoverWithCoin},
    telegram::{
        keyboard,
        takeover::{markup, message, Result, TakeoverDialogue},
    },
};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use teloxide::{
    payloads::SendMessageSetters,
    prelude::*,
    types::{ParseMode, ReplyMarkup},
};

#[derive(Clone, Default, Serialize, Deserialize)]
pub enum State {
    #[default]
    Start,
    ReceiveCto,
    ReceiveCtoAddress(Takeover),
    ReceiveCtoTelegramGroup(Takeover),
    Info(Vec<TakeoverWithCoin>),
}

pub async fn cto(
    bot: Bot,
    context: Context,
    dialogue: TakeoverDialogue,
    msg: Message,
) -> Result<()> {
    let mint = msg.text().unwrap_or_default().to_string();
    if Pubkey::from_str(&mint).is_err() {
        bot.send_message(msg.chat.id, message::INVALID_ADDRESS)
            .reply_markup(markup::website("See exists CTOs")?)
            .await?;

        return Ok(());
    }

    let redis = &mut context.redis()?;
    let Ok(coin) = context
        .client
        .coin(&mint, false, redis)
        .await
        .map_err(|e| tracing::error!("{e}"))
    else {
        bot.send_message(msg.chat.id, message::NO_METADATA).await?;
        return Ok(());
    };

    bot.send_message(msg.chat.id, message::coin(&coin, &context).await?)
        .parse_mode(ParseMode::MarkdownV2)
        .reply_markup(keyboard::coin(
            &mint,
            context.client.pair(&mint, false, redis).await,
        )?)
        .await?;

    bot.send_message(msg.chat.id, message::INPUT_HANDLE).await?;
    dialogue
        .update(State::ReceiveCtoAddress(Takeover::new(mint)))
        .await?;
    Ok(())
}

pub async fn token(
    bot: Bot,
    context: Context,
    dialogue: TakeoverDialogue,
    mut takeover: Takeover,
    msg: Message,
) -> Result<()> {
    let handle = msg.text().unwrap_or_default().trim().to_string();
    if !handle.starts_with('@') || handle.contains(|w: char| w.is_ascii_whitespace()) {
        bot.send_message(msg.chat.id, "Invalid telegram group handle.")
            .await?;
        return Ok(());
    }

    takeover.telegram = handle;
    takeover.admin = msg.chat.id.0.to_string();
    takeover.write(&context).await?;

    bot.send_message(msg.chat.id, "All set up!".to_string())
        .await?;
    dialogue.exit().await?;
    Ok(())
}

pub async fn invalid(bot: Bot, dialogue: TakeoverDialogue, msg: Message) -> Result<()> {
    bot.send_message(msg.chat.id, message::INVALID)
        .reply_markup(ReplyMarkup::kb_remove())
        .await?;
    dialogue.exit().await?;
    Ok(())
}
