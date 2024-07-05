//! Telegram takeover bot

use crate::context::{Db, Postgres, Redis};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::{str::FromStr, sync::Arc};
use teloxide::{
    dispatching::{
        dialogue::{self, serializer::Bincode, ErasedStorage, InMemStorage, RedisStorage, Storage},
        UpdateHandler,
    },
    dptree::case,
    payloads::SendMessageSetters,
    prelude::*,
    types::{
        BotCommand, InlineKeyboardButton, InlineKeyboardButtonKind, InlineKeyboardMarkup,
        MenuButton, ReplyMarkup, WebAppInfo,
    },
    utils::command::{BotCommands, CommandDescriptions},
};

type TakeoverDialogue = Dialogue<State, ErasedStorage<State>>;
type TakeoverStorage = Arc<ErasedStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(Clone, Default, Serialize, Deserialize)]
pub enum State {
    #[default]
    Start,
    ReceiveCto,
    ReceiveCtoAddress {
        address: String,
    },
    ReceiveCtoTelegramGroup {
        group: String,
    },
}

/// Telegram takeover bot
#[derive(Clone)]
pub struct TakeoverBot {
    /// Takeover bot instance
    bot: Bot,
    /// Databse interface
    db: Db,
    /// Redis db number
    redis: String,
}

impl TakeoverBot {
    /// Create a new takeover bot
    pub fn new(takeover: &str, db: Db, redis: String) -> Self {
        Self {
            bot: Bot::new(takeover),
            db,
            redis,
        }
    }

    /// Start the take over bot
    pub async fn start(&self) -> Result<()> {
        tracing::info!("Starting the takeover bot");

        let cache: TakeoverStorage = RedisStorage::open(self.redis.clone(), Bincode)
            .await?
            .erase();
        let command = teloxide::filter_command::<Command, _>()
            .branch(
                case![State::Start]
                    .branch(case![Command::Help].endpoint(help))
                    .branch(case![Command::Takeover].endpoint(cto))
                    .branch(case![Command::Ctos].endpoint(cto)),
            )
            .branch(case![Command::Start].endpoint(start))
            .branch(case![Command::Cancel].endpoint(cancel));

        let message = Update::filter_message()
            .branch(command)
            .branch(case![State::ReceiveCto].endpoint(receive_cto))
            // .branch(case![State::ReceiveCtoAddress { address }].endpoint(cto))
            // .branch(case![State::ReceiveFullName].endpoint(receive_full_name))
            .branch(dptree::endpoint(invalid_state));

        let schema = dialogue::enter::<Update, InMemStorage<State>, State, _>().branch(message);
        self.bot
            .set_chat_menu_button()
            .menu_button(MenuButton::WebApp {
                text: "takeover".into(),
                web_app: WebAppInfo {
                    url: "https://takeover.fyi".parse()?,
                },
            })
            .await?;
        self.bot
            .set_my_commands(Command::bot_commands().into_iter().collect::<Vec<_>>())
            .await?;
        self.bot.get_updates().timeout(60).await?;

        // dispatching
        Dispatcher::builder(self.bot.clone(), schema)
            .dependencies(dptree::deps![self.db.clone(), cache])
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;

        Ok(())
    }
}

async fn start(bot: Bot, dialogue: TakeoverDialogue, msg: Message) -> HandlerResult {
    dialogue.exit().await?;
    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .await?;
    Ok(())
}

async fn cto(bot: Bot, dialogue: TakeoverDialogue, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "Let's start! Which token your community want to take over?",
    )
    .await?;

    dialogue.update(State::ReceiveCto).await?;
    Ok(())
}

async fn receive_cto(bot: Bot, db: Db, dialogue: TakeoverDialogue, msg: Message) -> HandlerResult {
    let address = msg.text().unwrap_or_default();
    if Pubkey::from_str(&address).is_err() {
        bot.send_message(msg.chat.id, "Invalid solana token address.")
            .reply_markup(ReplyMarkup::InlineKeyboard(InlineKeyboardMarkup {
                inline_keyboard: vec![vec![InlineKeyboardButton {
                    text: "see exist ctos".to_string(),
                    kind: InlineKeyboardButtonKind::WebApp(WebAppInfo {
                        url: "https://takeover.fyi".parse()?,
                    }),
                }]],
            }))
            .await?;
    }

    // hset the address

    Ok(())
}

async fn receive_cto_address(
    bot: Bot,
    dialogue: TakeoverDialogue,
    address: String,
    msg: Message,
) -> HandlerResult {
    if Pubkey::from_str(&address).is_err() {
        bot.send_message(msg.chat.id, "Invalid solana token address.")
            .reply_markup(ReplyMarkup::InlineKeyboard(InlineKeyboardMarkup {
                inline_keyboard: vec![vec![InlineKeyboardButton {
                    text: "see exist ctos".to_string(),
                    kind: InlineKeyboardButtonKind::WebApp(WebAppInfo {
                        url: "https://takeover.fyi".parse()?,
                    }),
                }]],
            }))
            .await?;
    }

    Ok(())
}

async fn cancel(bot: Bot, dialogue: TakeoverDialogue, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "Cancelling the dialogue. Type /help to see the usage.",
    )
    .await?;
    dialogue.exit().await?;
    Ok(())
}

async fn help(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .await?;
    Ok(())
}

async fn invalid_state(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "Unable to handle the message. Type /help to see the usage.",
    )
    .await?;
    Ok(())
}

async fn to_takeover_webapp(bot: Bot, msg: &str, id: ChatId) -> HandlerResult {
    bot.send_message(id, msg)
        .reply_markup(ReplyMarkup::InlineKeyboard(InlineKeyboardMarkup {
            inline_keyboard: vec![vec![InlineKeyboardButton {
                text: "takeover".to_string(),
                kind: InlineKeyboardButtonKind::WebApp(WebAppInfo {
                    url: "https://takeover.fyi".parse()?,
                }),
            }]],
        }))
        .await?;

    Ok(())
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    /// Display this text.
    #[command(description = "Display the help message.")]
    Help,
    /// Start bot use.
    #[command(description = "Start bot use.")]
    Start,
    /// Cancel the current operation.
    #[command(description = "Cancel the current operation.")]
    Cancel,
    #[command(description = "Register a community take over.")]
    /// Register a community take over
    Takeover,
    /// Get ctos of a token
    #[command(description = "Get ctos of a solana token.")]
    Ctos,
}
