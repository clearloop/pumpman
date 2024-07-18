//! Telegram takeover bot

use crate::context::Context;
use std::sync::Arc;
use teloxide::{
    dispatching::dialogue::{self, serializer::Json, ErasedStorage, RedisStorage, Storage},
    dptree::case,
    payloads::SetChatMenuButtonSetters,
    prelude::*,
    types::MenuButton,
    utils::command::BotCommands,
};
use {command::Command, state::State};

mod callback;
mod command;
mod context;
mod group;
mod markup;
mod message;
mod result;
mod state;

type TakeoverDialogue = Dialogue<State, ErasedStorage<State>>;
type TakeoverStorage = Arc<ErasedStorage<State>>;
type Result<T> = core::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Start the takeover bot
pub async fn start(takeover: &str, context: Context, redis: String) -> anyhow::Result<()> {
    tracing::info!("Starting the takeover bot ...");

    let bot = Bot::new(takeover);
    let command = teloxide::filter_command::<Command, _>()
        .branch(case![State::Start].branch(case![Command::Takeover].endpoint(command::takeover)))
        .branch(case![Command::Start].endpoint(command::start))
        .branch(case![Command::Cancel].endpoint(command::cancel));

    let message = Update::filter_message()
        .branch(command)
        .branch(case![State::ReceiveCto].endpoint(state::cto))
        .branch(case![State::ReceiveCtoAddress(takeover)].endpoint(state::token))
        .branch(dptree::endpoint(state::invalid));

    let group = Update::filter_message()
        .filter(|msg: Message| msg.chat.is_group())
        .filter_command::<Command>()
        .branch(dptree::endpoint(group::unsupport));

    let callback =
        Update::filter_callback_query().branch(case![State::Start].endpoint(callback::takeover));

    let schema = dialogue::enter::<Update, ErasedStorage<State>, State, _>()
        .branch(group)
        .branch(message)
        .branch(callback);

    settings(&bot).await?;

    let cache: TakeoverStorage = RedisStorage::open(redis, Json).await?.erase();
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
    bot.get_updates().timeout(60).await?;

    Ok(())
}
