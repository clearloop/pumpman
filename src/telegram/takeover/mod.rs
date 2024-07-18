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
mod markup;
mod state;

type TakeoverDialogue = Dialogue<State, ErasedStorage<State>>;
type TakeoverStorage = Arc<ErasedStorage<State>>;
type Result<T> = core::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Start the takeover bot
pub async fn start(takeover: &str, context: Arc<Context>, redis: String) -> anyhow::Result<()> {
    tracing::info!("Starting the takeover bot");

    let bot = Bot::new(takeover);
    let cache: TakeoverStorage = RedisStorage::open(redis, Json).await?.erase();
    let command = teloxide::filter_command::<Command, _>()
        .branch(case![State::Start].branch(case![Command::Takeover].endpoint(command::takeover)))
        .branch(case![Command::Start].endpoint(command::start))
        .branch(case![Command::Cancel].endpoint(command::cancel));

    let message = Update::filter_message()
        .branch(command)
        .branch(case![State::ReceiveCto].endpoint(state::receive_cto))
        .branch(case![State::ReceiveCtoAddress(takeover)].endpoint(state::receive_cto_address))
        .branch(dptree::endpoint(invalid_state));

    let callback =
        Update::filter_callback_query().branch(case![State::Start].endpoint(callback::takeover));

    let schema = dialogue::enter::<Update, ErasedStorage<State>, State, _>()
        .branch(message)
        .branch(callback);

    bot.set_chat_menu_button()
        .menu_button(MenuButton::Commands)
        // .menu_button(MenuButton::WebApp {
        //     text: "takeover".into(),
        //     web_app: WebAppInfo {
        //         url: "https://takeover.fyi".parse()?,
        //     },
        // })
        .await?;
    bot.set_my_commands(Command::bot_commands().into_iter().collect::<Vec<_>>())
        .await?;
    bot.get_updates().timeout(60).await?;

    // dispatching
    Dispatcher::builder(bot.clone(), schema)
        .dependencies(dptree::deps![context.clone(), cache])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}

async fn invalid_state(bot: Bot, msg: Message) -> Result<()> {
    tracing::warn!("{msg:#?}");
    bot.send_message(
        msg.chat.id,
        "Unable to handle the message. Type /start to see the usage.",
    )
    .await?;
    Ok(())
}
