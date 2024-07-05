//! Telegram takeover bot

use crate::context::{Db, Postgres, Redis};
use anyhow::Result;
use teloxide::{
    dispatching::{
        dialogue::{self, InMemStorage},
        UpdateHandler,
    },
    dptree::case,
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, MenuButton, WebAppInfo},
    utils::command::BotCommands,
};

type MyDialogue = Dialogue<State, InMemStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
    ReceiveFullName,
    ReceiveProductChoice {
        full_name: String,
    },
}

/// Telegram takeover bot
#[derive(Clone)]
pub struct TakeoverBot {
    /// Takeover bot instance
    bot: Bot,
    /// Databse interface
    db: Db,
}

impl TakeoverBot {
    /// Create a new takeover bot
    pub fn new(takeover: &str, db: Db) -> Self {
        Self {
            bot: Bot::new(takeover),
            db,
        }
    }

    /// Start the take over bot
    pub async fn start(&self) -> Result<()> {
        tracing::info!("Starting the takeover bot");

        let command = teloxide::filter_command::<Command, _>().branch(
            case![State::Start]
                .branch(case![Command::Help].endpoint(help))
                .branch(case![Command::Start].endpoint(start))
                .branch(case![Command::Register].endpoint(help))
                .branch(case![Command::Cancel].endpoint(cancel))
                .branch(case![Command::Ctos].endpoint(start)),
        );
        // .branch(case![Command::Cancel].endpoint(cancel));

        let message = Update::filter_message().branch(command);
        // .branch(case![State::ReceiveFullName].endpoint(receive_full_name))
        // .branch(dptree::endpoint(invalid_state));

        let schema = dialogue::enter::<Update, InMemStorage<State>, State, _>().branch(message);
        self.bot
            .set_chat_menu_button()
            .menu_button(MenuButton::WebApp {
                text: "Takeover".into(),
                web_app: WebAppInfo {
                    url: "https://pumpman.io".parse()?,
                },
            })
            .await?;
        self.bot
            .set_my_commands(
                Command::bot_commands()
                    .into_iter()
                    .filter(|c| c.command != "/start".to_string())
                    .collect::<Vec<_>>(),
            )
            .await?;
        self.bot.get_updates().timeout(60).await?;

        // dispatching
        Dispatcher::builder(self.bot.clone(), schema)
            .dependencies(dptree::deps![InMemStorage::<State>::new()])
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;

        Ok(())
    }
}

async fn start(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .await?;
    Ok(())
}

async fn cancel(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Cancelling the dialogue.")
        .await?;
    dialogue.exit().await?;
    Ok(())
}

async fn help(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, Command::descriptions().to_string())
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
    Register,
    /// Get ctos of a token
    #[command(description = "Get ctos of a solana token.")]
    Ctos,
}
