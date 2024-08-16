use crate::{
    api::{PumpApi, SolRpcApi},
    telegram::{
        pumpman::{message::INVALID_PUMPFUN_LINK, BotDialogue, PumpmanContext},
        Result,
    },
};
use serde::{Deserialize, Serialize};
use solana_sdk::{pubkey::Pubkey, signer::Signer};
use teloxide::{
    payloads::SendMessageSetters, prelude::Message, requests::Requester, types::ParseMode, Bot,
};

use super::message;

const PUMPFUN_BASE: &str = "https://pump.fun/";

#[derive(Clone, Default, Serialize, Deserialize)]
pub enum State {
    #[default]
    Start,
    /// Received pubkey
    Coin(Pubkey),
}

/// Handle any message
pub async fn any(
    bot: Bot,
    _dialogue: BotDialogue,
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

    let redis = &mut context.redis()?;
    let mint = text.trim_start_matches(PUMPFUN_BASE);
    let coin = context.client.coin(mint, true, redis).await?;
    let tgid = msg.chat.id.0;
    let wallet = context.wallet(msg.chat.id.0).await?;
    let pubkey = wallet.pubkey();
    let balance = context.client.rpc().get_balance(&pubkey).await?;
    let job = context.job(tgid, &pubkey.to_string()).await?;

    bot.send_message(
        msg.chat.id,
        message::job(&context.global, &job, coin, pubkey, balance),
    )
    .parse_mode(ParseMode::Html)
    .reply_markup(job.markup()?)
    .await?;

    Ok(())
}
