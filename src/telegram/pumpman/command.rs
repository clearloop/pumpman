use crate::{
    config::PumpmanGlobal,
    context::TgContext,
    telegram::{pumpman::message, Escape, Result},
};
use teloxide::{
    payloads::SendMessageSetters, prelude::Message, requests::Requester, types::ParseMode,
    utils::command::BotCommands, Bot,
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
    /// Show the pumpman configuration
    #[command(description = "Show the static pumpman configuration.")]
    Config,
    /// Show the details of service fee
    #[command(description = "Show the details of service fee.")]
    Fee,
}

impl Command {
    /// command start
    pub async fn start(bot: Bot, context: TgContext<PumpmanGlobal>, msg: Message) -> Result<()> {
        bot.send_message(
            msg.chat.id,
            message::menu(&context.data, Default::default()),
        )
        .parse_mode(ParseMode::Html)
        .await?;
        Ok(())
    }

    /// Send config to users
    pub async fn config(bot: Bot, context: TgContext<PumpmanGlobal>, msg: Message) -> Result<()> {
        bot.send_message(msg.chat.id, message::config(&context.data))
            .parse_mode(ParseMode::Html)
            .await?;
        Ok(())
    }

    /// Send service fee details to users
    pub async fn fee(bot: Bot, context: TgContext<PumpmanGlobal>, msg: Message) -> Result<()> {
        bot.send_message(msg.chat.id, message::fee(&context.data))
            .parse_mode(ParseMode::Html)
            .await?;
        Ok(())
    }
}

/// Group response
pub async fn group(bot: Bot, msg: Message) -> Result<()> {
    bot.send_message(msg.chat.id, message::ENTER_GROUP.escaped())
        .parse_mode(ParseMode::MarkdownV2)
        .await?;
    Ok(())
}
