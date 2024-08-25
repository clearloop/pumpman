use crate::{
    api::SolRpcApi,
    model::PumpmanJob,
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
    types::{
        InlineKeyboardButton, MediaKind, MediaText, MessageCommon, MessageKind, ParseMode,
        ReplyMarkup,
    },
    utils::command::BotCommands,
    Bot,
};

use super::{
    callback::{Callback, WithdrawCallback},
    state::State,
    BotDialogue,
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
    /// Wallet info
    #[command(description = "Show the details of your wallet.")]
    Wallet,
    /// List all running jobs
    #[command(description = "List all jobs.")]
    List,
    /// Show support
    #[command(description = "Get support")]
    Support,
    /// Show FAQ
    #[command(description = "Show FAQ")]
    Faq,
}

impl Command {
    /// command start
    pub async fn start(
        bot: Bot,
        dialogue: BotDialogue,
        context: PumpmanContext,
        msg: Message,
    ) -> Result<()> {
        dialogue.update(State::Start).await?;
        let MessageKind::Common(MessageCommon {
            media_kind: MediaKind::Text(MediaText { text, .. }),
            ..
        }) = msg.kind
        else {
            // handle common start
            bot.send_message(msg.chat.id, message::menu(&context, msg.chat.id.0).await?)
                .parse_mode(ParseMode::Html)
                .await?;

            return Ok(());
        };

        let mint = text.trim_start_matches("/start").trim();
        let job = context.job(msg.chat.id.0, mint).await?;
        bot.send_message(msg.chat.id, message::job(&context, &job).await?)
            .parse_mode(ParseMode::Html)
            .reply_markup(job.markup(&context.global)?)
            .await?;
        Ok(())
    }

    /// Show the details of wallet
    pub async fn wallet(bot: Bot, context: PumpmanContext, msg: Message) -> Result<()> {
        let wallet = context.wallet(msg.chat.id.0).await?;
        let pubkey = wallet.pubkey();
        let balance = (BigDecimal::from(context.client.helius().get_balance(&pubkey).await?)
            / SOL_SCALE)
            .round(6);

        bot.send_message(msg.chat.id, message::wallet(&pubkey).await?)
            .parse_mode(ParseMode::Html)
            .reply_markup(ReplyMarkup::inline_kb(vec![vec![
                InlineKeyboardButton::callback(
                    format!("Withdraw ({balance} SOL)"),
                    Callback::Withdraw(WithdrawCallback::Input).format()?,
                ),
            ]]))
            .await?;

        Ok(())
    }

    /// Send config to users
    pub async fn config(bot: Bot, context: PumpmanContext, msg: Message) -> Result<()> {
        let global = context.pglobal(msg.chat.id.0).await?;
        bot.send_message(msg.chat.id, message::config(&context, &global).await?)
            .parse_mode(ParseMode::Html)
            .reply_markup(global.markup(&context.global)?)
            .await?;
        Ok(())
    }

    /// Send service fee details to users
    pub async fn fees(bot: Bot, context: PumpmanContext, msg: Message) -> Result<()> {
        bot.send_message(msg.chat.id, message::fees(&context, msg.chat.id.0).await?)
            .parse_mode(ParseMode::Html)
            .await?;
        Ok(())
    }

    /// List all jobs
    pub async fn list(bot: Bot, context: PumpmanContext, msg: Message) -> Result<()> {
        let jobs = context.jobs(msg.chat.id.0).await?;
        bot.send_message(msg.chat.id, message::list(&jobs))
            .parse_mode(ParseMode::Html)
            .reply_markup(message::list_markup(&context, &jobs).await?)
            .await?;
        Ok(())
    }

    /// Show FAQ
    pub async fn faq(bot: Bot, msg: Message) -> Result<()> {
        bot.send_message(msg.chat.id, message::FAQ)
            .parse_mode(ParseMode::Html)
            .await?;
        Ok(())
    }

    /// Show FAQ
    pub async fn support(bot: Bot, msg: Message) -> Result<()> {
        bot.send_message(msg.chat.id, "@todamup")
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
