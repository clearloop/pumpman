use crate::telegram::{pumpman::message, Result};
use teloxide::{prelude::Message, requests::Requester, utils::command::BotCommands, Bot};

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum Command {
    /// Start bot use.
    #[command(description = "Start bot use.")]
    Start,
}

impl Command {
    /// command start
    pub async fn start(_bot: Bot, _msg: Message) -> Result<()> {
        Ok(())
    }
}

/// Group response
pub async fn group(bot: Bot, msg: Message) -> Result<()> {
    bot.send_message(msg.chat.id, message::ENTER_GROUP).await?;
    Ok(())
}
