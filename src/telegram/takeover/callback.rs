use crate::{
    context::Context,
    telegram::takeover::{command, Result, TakeoverDialogue},
};
use teloxide::{types::CallbackQuery, Bot};

pub async fn takeover(
    bot: Bot,
    context: Context,
    dialogue: TakeoverDialogue,
    q: CallbackQuery,
) -> Result<()> {
    let Some(msg) = q.message else {
        return Ok(());
    };

    command::takeover(bot, context, dialogue, msg).await
}
