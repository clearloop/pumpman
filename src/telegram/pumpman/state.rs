use crate::{
    model::PumpmanJob,
    telegram::{
        pumpman::{message, message::INVALID_PUMPFUN_LINK, BotDialogue, PumpmanContext},
        Result,
    },
};
use serde::{Deserialize, Serialize};
use teloxide::{
    payloads::SendMessageSetters, prelude::Message, requests::Requester, types::ParseMode, Bot,
};

const PUMPFUN_BASE: &str = "https://pump.fun/";

#[derive(Clone, Default, Serialize, Deserialize, PartialEq)]
pub enum State {
    #[default]
    Start,
    BackToList,
    NoUpdateMarkup,
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
