use crate::{
    api::SolRpcApi,
    sol::pump::SOL_SCALE,
    telegram::{
        pumpman::{message, PumpmanContext},
        Escape, Result,
    },
};
use bigdecimal::BigDecimal;
use solana_sdk::signer::Signer;
use teloxide::{
    payloads::SendMessageSetters,
    prelude::Message,
    requests::Requester,
    types::{InlineKeyboardButton, ParseMode, ReplyMarkup},
    utils::command::BotCommands,
    Bot,
};

use super::callback::Callback;

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
    /// List all running jobs
    #[command(description = "List all running jobs")]
    List,
}

impl Command {
    /// command start
    pub async fn start(bot: Bot, context: PumpmanContext, msg: Message) -> Result<()> {
        let wallet = context.wallet(msg.chat.id.0).await?;
        let pubkey = wallet.pubkey();
        let balance = (BigDecimal::from(context.client.rpc().get_balance(&pubkey).await?)
            / SOL_SCALE)
            .round(6);
        let global = context.global(msg.chat.id.0).await?;
        bot.send_message(msg.chat.id, message::menu(&context.global, &global, pubkey))
            .parse_mode(ParseMode::Html)
            .reply_markup(ReplyMarkup::inline_kb(vec![vec![
                InlineKeyboardButton::callback(
                    format!("Withdraw (balance: {balance} SOL)"),
                    Callback::Withdraw(msg.chat.id.0).format()?,
                ),
            ]]))
            .await?;
        Ok(())
    }

    /// Send config to users
    pub async fn config(bot: Bot, context: PumpmanContext, msg: Message) -> Result<()> {
        let global = context.global(msg.chat.id.0).await?;
        bot.send_message(msg.chat.id, message::config(&global))
            .parse_mode(ParseMode::Html)
            .reply_markup(global.markup()?)
            .await?;
        Ok(())
    }

    /// Send service fee details to users
    pub async fn fees(bot: Bot, context: PumpmanContext, msg: Message) -> Result<()> {
        let global = context.global(msg.chat.id.0).await?;
        bot.send_message(msg.chat.id, message::fees(&context.global, &global))
            .parse_mode(ParseMode::Html)
            .await?;
        Ok(())
    }

    /// List all jobs
    pub async fn list(bot: Bot, context: PumpmanContext, msg: Message) -> Result<()> {
        let jobs = context.jobs(msg.chat.id.0).await?;

        bot.send_message(msg.chat.id, message::list(&jobs))
            .reply_markup(message::list_markup(&context, &jobs).await?)
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
