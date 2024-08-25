//! Telegram takeover bot

use crate::{
    api::{DexScreenerApi, PumpApi, SolRpcApi},
    config,
    context::{Cache, Context},
    model::{Alert, AlertTitle},
};
use bigdecimal::BigDecimal;
use redis::Commands;
use std::sync::Arc;
use teloxide::Bot;
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
mod message;
mod state;

type TakeoverDialogue = Dialogue<State, ErasedStorage<State>>;
type TakeoverStorage = Arc<ErasedStorage<State>>;
type Result<T> = core::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Start the takeover bot
pub async fn start(bot: &str, context: Context, redis: String) -> anyhow::Result<()> {
    tracing::info!("Starting the takeover bot ...");

    let bot = Bot::new(bot);
    let command = teloxide::filter_command::<Command, _>()
        .branch(case![Command::Start].endpoint(command::start))
        .branch(case![Command::Cancel].endpoint(command::cancel))
        .branch(case![Command::Info].endpoint(command::info))
        .branch(case![Command::Inspect].endpoint(command::inspect))
        .branch(case![Command::SetTwitter].endpoint(command::set_twitter))
        .branch(case![Command::SetWebsite].endpoint(command::set_website))
        .branch(case![Command::SetBanner].endpoint(command::set_banner))
        .branch(case![Command::SetTelegram].endpoint(command::set_telegram))
        .branch(case![Command::Takeover].endpoint(command::takeover))
        .branch(dptree::endpoint(state::invalid));

    let message = Update::filter_message()
        .branch(command)
        .branch(case![State::ReceiveCto].endpoint(state::cto))
        .branch(case![State::ReceiveCtoAddress(takeover)].endpoint(state::token))
        .branch(dptree::endpoint(state::invalid));

    let group = Update::filter_message()
        .filter(|msg: Message| msg.chat.is_group())
        .branch(dptree::endpoint(command::unsupport));

    let callback =
        Update::filter_callback_query().branch(case![State::Start].endpoint(callback::takeover));

    let schema = dialogue::enter::<Update, ErasedStorage<State>, State, _>()
        .branch(group)
        .branch(message)
        .branch(callback)
        .branch(dptree::endpoint(state::invalid));

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

    Ok(())
}

/// pumpfun soldout alert
pub async fn alert(
    config: &config::Takeover,
    context: &Context,
    bot: &Bot,
    mint: String,
) -> anyhow::Result<()> {
    let redis = &mut context.redis()?;
    let client = context.client.clone();

    let key = Cache::DevSoldOut(&mint);
    if redis.exists(&key)? {
        return Ok(());
    }

    // 1. filter out mc less than $8k
    let Ok(coin) = client.coin(&mint, false, redis).await else {
        redis.set(key, true)?;
        return Ok(());
    };
    if let Some(mc) = &coin.usd_market_cap {
        if *mc < BigDecimal::from(config.marketcap) {
            return Ok(());
        }
    }

    // 2. check if dev is soldout
    let (_, soldout) = client
        .soldout(&coin.mint, &coin.creator, false, redis)
        .await?;

    if !soldout {
        return Ok(());
    }

    // check holders amount
    let holders = client
        .top_holders(&mint, false, redis)
        .await?
        .skip_bc(&coin.associated_bonding_curve);

    if holders.len() < config.holders {
        return Ok(());
    }

    let pairs = client.pairs(&mint, false, redis).await?;
    context.update_coin(coin.clone()).await?;
    Alert::new(AlertTitle::DevSoldOut, coin, soldout)
        .pairs(pairs)
        .holders(holders)
        .alert(bot, &config.subscription)
        .await?;
    redis.set(key, true)?;
    Ok(())
}
