use crate::telegram::takeover::{command, Result, TakeoverDialogue};
use teloxide::{types::CallbackQuery, Bot};

pub async fn takeover(bot: Bot, dialogue: TakeoverDialogue, q: CallbackQuery) -> Result<()> {
    tracing::trace!("Message enter callback");
    let Some(msg) = q.message else {
        return Ok(());
    };

    command::takeover(bot, dialogue, msg).await
}
