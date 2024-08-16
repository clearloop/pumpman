use crate::{
    api::SolRpcApi,
    telegram::{
        pumpman::{message, PumpmanContext},
        Escape, Result,
    },
};
use solana_sdk::signer::Signer;
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
    #[command(description = "Show the global config.")]
    Config,
    /// Show the details of service fee
    #[command(description = "Show the details of service fees.")]
    Fees,
}

impl Command {
    /// command start
    pub async fn start(bot: Bot, context: PumpmanContext, msg: Message) -> Result<()> {
        let wallet = context.wallet(msg.chat.id.0).await?;
        let pubkey = wallet.pubkey();
        let balance = context.context.client.rpc().get_balance(&pubkey).await?;
        bot.send_message(msg.chat.id, message::menu(&context.global, pubkey, balance))
            .parse_mode(ParseMode::Html)
            .await?;
        Ok(())
    }

    /// Send config to users
    pub async fn config(bot: Bot, context: PumpmanContext, msg: Message) -> Result<()> {
        bot.send_message(msg.chat.id, message::config(&context.global))
            .parse_mode(ParseMode::Html)
            .await?;
        Ok(())
    }

    /// Send service fee details to users
    pub async fn fees(bot: Bot, context: PumpmanContext, msg: Message) -> Result<()> {
        bot.send_message(msg.chat.id, message::fees(&context.global))
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
