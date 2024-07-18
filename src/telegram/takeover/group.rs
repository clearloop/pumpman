//! Group handler

use crate::telegram::takeover::{message, Result};
use teloxide::{requests::Requester, types::Message, Bot};

pub async fn unsupport(bot: Bot, msg: Message) -> Result<()> {
    bot.send_message(msg.chat.id, message::ENTER_GROUP).await?;
    Ok(())
}
