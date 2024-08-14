use std::sync::Arc;

use crate::{config::PumpmanGlobal, context::TgContext};
use command::Command;
use state::State;
use teloxide::{
    dispatching::dialogue::{
        self, serializer::Json, Dialogue, ErasedStorage, RedisStorage, Storage,
    },
    dptree::case,
    payloads::SetChatMenuButtonSetters,
    prelude::*,
    types::MenuButton,
    utils::command::BotCommands,
};

mod command;
mod message;
mod state;

type BotStorage = Arc<ErasedStorage<State>>;
type BotDialogue = Dialogue<State, ErasedStorage<State>>;

/// Start the pumpman bot
pub async fn start(
    bot: &Bot,
    context: TgContext<PumpmanGlobal>,
    redis: String,
) -> anyhow::Result<()> {
    tracing::info!("Starting the pumpman bot ...");

    let command = teloxide::filter_command::<Command, _>()
        .branch(case![Command::Start].endpoint(Command::start))
        .branch(case![Command::Config].endpoint(Command::config))
        .branch(case![Command::Fee].endpoint(Command::fee));

    let group = Update::filter_message()
        .filter(|msg: Message| msg.chat.is_group())
        .branch(dptree::endpoint(command::group));

    let message = Update::filter_message().branch(command);
    let schema = dialogue::enter::<Update, ErasedStorage<State>, State, _>()
        .branch(group)
        .branch(message);

    settings(bot).await?;

    let cache: BotStorage = RedisStorage::open(redis, Json).await?.erase();
    Dispatcher::builder(bot.clone(), schema)
        .dependencies(dptree::deps![context, cache])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}

async fn settings(bot: &Bot) -> anyhow::Result<()> {
    bot.set_chat_menu_button()
        .menu_button(MenuButton::Commands)
        .await?;
    bot.set_my_commands(Command::bot_commands().into_iter().collect::<Vec<_>>())
        .await?;

    Ok(())
}
