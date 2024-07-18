//! Takeover commands

use crate::telegram::takeover::{markup, state::State, Result, TakeoverDialogue};
use teloxide::{payloads::SendMessageSetters, prelude::*, utils::command::BotCommands};

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
        bot.send_message(
            msg.chat.id,
            r#"
Building better CTOs, feedbacks and ideas are welcome! @takeoverfyi

Only support private chats atm )) 
"#,
        )
        .await?;
        return Ok(());
    }

    bot.send_message(
        msg.chat.id,
        "Building better CTOs, feedbacks and ideas are welcome! @takeoverfyi",
    )
    .reply_markup(markup::menu()?)
    .await?;
    Ok(())
}

/// Command cancel
pub async fn cancel(bot: Bot, dialogue: TakeoverDialogue, msg: Message) -> Result<()> {
    bot.send_message(
        msg.chat.id,
        "Cancelling the dialogue. Type /start to see the menu.",
    )
    .await?;
    dialogue.exit().await?;
    Ok(())
}

/// Command takeover
pub async fn takeover(bot: Bot, dialogue: TakeoverDialogue, msg: Message) -> Result<()> {
    bot.send_message(
        msg.chat.id,
        "Let's start! Which token your community are about to take over?",
    )
    .await?;

    dialogue.update(State::ReceiveCto).await?;
    Ok(())
}
