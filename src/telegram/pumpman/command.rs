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
}

impl Command {
    /// command start
    pub async fn start(bot: Bot, context: PumpmanContext, msg: Message) -> Result<()> {
        let wallet = context.wallet(msg.chat.id.0).await?;
        let pubkey = wallet.pubkey();
        let balance = (BigDecimal::from(context.client.rpc().get_balance(&pubkey).await?)
            / SOL_SCALE)
            .round(6);
        bot.send_message(msg.chat.id, message::menu(&context.global, pubkey))
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
