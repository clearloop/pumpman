use std::str::FromStr;

use crate::{
    api::SolRpcApi,
    model::PumpmanJob,
    telegram::{
        pumpman::{
            message::{self, INVALID_PUMPFUN_LINK},
            BotDialogue, PumpmanContext,
        },
        Result,
    },
};
use serde::{Deserialize, Serialize};
use solana_sdk::{pubkey::Pubkey, signer::Signer};
use teloxide::{
    payloads::{EditMessageTextSetters, SendMessageSetters},
    prelude::Message,
    requests::Requester,
    types::{MessageId, ParseMode},
    Bot,
};

const PUMPFUN_BASE: &str = "https://pump.fun/";

#[derive(Clone, Default, Serialize, Deserialize, PartialEq)]
pub enum State {
    #[default]
    Start,
    BackToList,
    NoUpdateMarkup,
    Withdraw(WithdrawState),
}

/// Handle any message
pub async fn info_job(
    bot: Bot,
    dialogue: BotDialogue,
    context: PumpmanContext,
    msg: Message,
) -> Result<()> {
    let Some(text) = &msg.text() else {
        return Ok(());
    };

    if !text.starts_with(PUMPFUN_BASE) {
        return Ok(());
    };

    if text.len() != 61 {
        bot.send_message(msg.chat.id, INVALID_PUMPFUN_LINK).await?;
        return Ok(());
    }

    let tgid = msg.chat.id.0;
    let mint = text.trim_start_matches(PUMPFUN_BASE);
    let job = context.job(tgid, mint).await?;
    dialogue.update(State::Start).await?;
    bot.send_message(msg.chat.id, message::job(&context, &job).await?)
        .parse_mode(ParseMode::Html)
        .reply_markup(job.markup(&context.global)?)
        .await?;

    Ok(())
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub enum WithdrawState {
    Input(MessageId),
    Check(Pubkey),
}

impl WithdrawState {
    pub async fn handle(
        bot: Bot,
        dialogue: BotDialogue,
        state: Self,
        context: PumpmanContext,
        msg: Message,
    ) -> Result<()> {
        let Self::Input(orgmid) = state else {
            return Ok(());
        };

        Self::collect_input(bot, dialogue, orgmid, context, msg).await
    }

    pub async fn collect_input(
        bot: Bot,
        dialogue: BotDialogue,
        orgmid: MessageId,
        context: PumpmanContext,
        msg: Message,
    ) -> Result<()> {
        let Some(text) = msg.text() else {
            return Ok(());
        };

        let recipient = Pubkey::from_str(text)?;
        dialogue
            .update(State::Withdraw(WithdrawState::Check(recipient.clone())))
            .await?;

        let tgid = msg.chat.id.0;
        let wallet = context.wallet(tgid).await?;
        let pubkey = wallet.pubkey();
        let balance = context.client.helius().get_balance(&pubkey).await?;

        bot.edit_message_text(msg.chat.id, orgmid, message::cwithdraw(balance, &recipient))
            .parse_mode(ParseMode::Html)
            .reply_markup(message::cwithdraw_markup()?)
            .await?;

        bot.delete_message(msg.chat.id, msg.id).await?;
        Ok(())
    }
}
