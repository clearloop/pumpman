//! Takeover commands

use crate::{
    context::Context,
    telegram::takeover::{markup, message, state::State, Result, TakeoverDialogue},
};
use teloxide::{
    payloads::SendMessageSetters, prelude::*, types::ReplyMarkup, utils::command::BotCommands,
};

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum Command {
    /// Start bot use.
    #[command(description = "Start bot use.")]
    Start,
    /// Cancel the current operation.
    #[command(description = "Cancel the current operation.")]
    Cancel,
    #[command(description = "Claim a community take over.")]
    /// Claim a community take over
    Takeover,
    #[command(description = "Show the info of your ctos. (in development)")]
    Info,
    #[command(description = "Inspect a cto. (in development)")]
    Inspect,
    #[command(description = "Set twitter of your cto. (in development)")]
    SetTwitter,
    #[command(description = "Set website of your cto. (in development)")]
    SetWebsite,
    #[command(description = "Set banner of your cto. (in development)")]
    SetBanner,
    #[command(description = "Set telegram of your cto. (in development)")]
    SetTelegram,
}

/// Command start
pub async fn start(bot: Bot, dialogue: TakeoverDialogue, msg: Message) -> Result<()> {
    dialogue.exit().await?;
    if !msg.chat.id.is_user() {
        bot.send_message(msg.chat.id, message::ENTER_GROUP).await?;
        return Ok(());
    }

    bot.send_message(msg.chat.id, message::BRANDING)
        .reply_markup(markup::menu()?)
        .await?;
    Ok(())
}

/// Command cancel
pub async fn cancel(bot: Bot, dialogue: TakeoverDialogue, msg: Message) -> Result<()> {
    bot.send_message(msg.chat.id, message::CANCEL)
        .reply_markup(ReplyMarkup::kb_remove())
        .await?;
    dialogue.exit().await?;
    Ok(())
}

/// Command takeover
pub async fn takeover(
    bot: Bot,
    context: Context,
    dialogue: TakeoverDialogue,
    msg: Message,
) -> Result<()> {
    if !context.eligible(&msg.chat.id.0.to_string()).await? {
        bot.send_message(msg.chat.id, message::INSUFFICIENT_CREDITS)
            .await?;
        return Ok(());
    }

    bot.send_message(msg.chat.id, message::TAKEOVER).await?;
    dialogue.update(State::ReceiveCto).await?;
    Ok(())
}

pub async fn info(
    bot: Bot,
    context: Context,
    dialogue: TakeoverDialogue,
    msg: Message,
) -> Result<()> {
    let takeovers = context.takeovers(msg.chat.id.0).await?;
    if takeovers.is_empty() {
        bot.send_message(msg.chat.id, message::NO_CTOS).await?;
        return Ok(());
    }

    bot.send_message(msg.chat.id, message::CHOOSE_INFO)
        .reply_markup(markup::list(&takeovers)?)
        .await?;
    dialogue.update(State::Info(takeovers)).await?;
    Ok(())
}

pub async fn inspect(
    _bot: Bot,
    _context: Context,
    _dialogue: TakeoverDialogue,
    _msg: Message,
) -> Result<()> {
    Ok(())
}

pub async fn set_twitter(
    _bot: Bot,
    _context: Context,
    _dialogue: TakeoverDialogue,
    _msg: Message,
) -> Result<()> {
    Ok(())
}

pub async fn set_website(
    _bot: Bot,
    _context: Context,
    _dialogue: TakeoverDialogue,
    _msg: Message,
) -> Result<()> {
    Ok(())
}

pub async fn set_banner(
    _bot: Bot,
    _context: Context,
    _dialogue: TakeoverDialogue,
    _msg: Message,
) -> Result<()> {
    Ok(())
}

pub async fn set_telegram(
    _bot: Bot,
    _context: Context,
    _dialogue: TakeoverDialogue,
    _msg: Message,
) -> Result<()> {
    Ok(())
}
