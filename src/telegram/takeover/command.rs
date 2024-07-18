//! Takeover commands

use crate::telegram::takeover::{markup, state::State, Result, TakeoverDialogue};
use teloxide::{payloads::SendMessageSetters, prelude::*, utils::command::BotCommands};

use super::message;

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
    bot.send_message(msg.chat.id, message::CANCEL).await?;
    dialogue.exit().await?;
    Ok(())
}

/// Command takeover
pub async fn takeover(bot: Bot, dialogue: TakeoverDialogue, msg: Message) -> Result<()> {
    bot.send_message(msg.chat.id, message::TAKEOVER).await?;
    dialogue.update(State::ReceiveCto).await?;
    Ok(())
}
