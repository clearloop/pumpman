//! Telegram takeover bot

use crate::{
    context::{Context, Postgres, Redis},
    model::Takeover,
    schema::takeovers,
};
use anyhow::{anyhow, Result};
use diesel::RunQueryDsl;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::{str::FromStr, sync::Arc};
use teloxide::{
    dispatching::{
        dialogue::{self, serializer::Json, ErasedStorage, InMemStorage, RedisStorage, Storage},
        UpdateHandler,
    },
    dptree::case,
    payloads::SendMessageSetters,
    prelude::*,
    types::{
        BotCommand, InlineKeyboardButton, InlineKeyboardButtonKind, InlineKeyboardMarkup,
        MenuButton, ParseMode, ReplyMarkup, WebAppInfo,
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
    ReceiveCtoAddress(Takeover),
    ReceiveCtoTelegramGroup(Takeover),
}

/// Telegram takeover bot
#[derive(Clone)]
pub struct TakeoverBot {
    /// Takeover bot instance
    bot: Bot,
    /// Databse interface
    context: Context,
    /// Redis db number
    redis: String,
}

impl TakeoverBot {
    /// Create a new takeover bot
    pub fn new(takeover: &str, context: Context, redis: String) -> Self {
        Self {
            bot: Bot::new(takeover),
            context,
            redis,
        }
    }

    /// Start the take over bot
    pub async fn start(&self) -> Result<()> {
        tracing::info!("Starting the takeover bot");

        let cache: TakeoverStorage = RedisStorage::open(self.redis.clone(), Json).await?.erase();
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
            .branch(case![State::ReceiveCtoAddress(takeover)].endpoint(receive_cto_address))
            .branch(dptree::endpoint(invalid_state));

        let schema = dialogue::enter::<Update, ErasedStorage<State>, State, _>().branch(message);
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
            .dependencies(dptree::deps![self.context.clone(), cache])
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
        "Let's start! Which token your community are about to take over?",
    )
    .await?;

    dialogue.update(State::ReceiveCto).await?;
    Ok(())
}

async fn receive_cto(
    bot: Bot,
    context: Context,
    dialogue: TakeoverDialogue,
    msg: Message,
) -> HandlerResult {
    let mint = msg.text().unwrap_or_default().to_string();
    if Pubkey::from_str(&mint).is_err() {
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

        return Ok(());
    }

    let Ok(coin) = context
        .client
        .coin(&mint)
        .await
        .map_err(|e| tracing::error!("{e}"))
    else {
        bot.send_message(
            msg.chat.id,
            r#"
Failed to get token metadata, re-input the token address to retry.

/cancel - cancel the current operation
/help - See the usage

If you believe this is a bug, please contact our dev @takeoverfyi
"#,
        );
        return Ok(());
    };

    bot.send_message(
        msg.chat.id,
        format!(
            r#"
${} \- {} 

```copy
{}
```

Almost done\! Please enter the telegram group handle of your community\.

for example @takeoverfyi
"#,
            coin.symbol, coin.name, coin.mint,
        ),
    )
    .parse_mode(ParseMode::MarkdownV2)
    .reply_markup(coin.keyboards()?)
    .await?;

    dialogue
        .update(State::ReceiveCtoAddress(Takeover::new(mint)))
        .await?;
    Ok(())
}

async fn receive_cto_address(
    bot: Bot,
    context: Context,
    dialogue: TakeoverDialogue,
    takeover: Takeover,
    msg: Message,
) -> HandlerResult {
    let mut link = msg.text().unwrap_or_default().trim().to_string();
    link.retain(|c| !c.is_whitespace());

    if !link.starts_with("@") {
        bot.send_message(msg.chat.id, "Invalid telegram group link.")
            .reply_markup(ReplyMarkup::InlineKeyboard(InlineKeyboardMarkup {
                inline_keyboard: vec![vec![InlineKeyboardButton {
                    text: "see exist ctos".to_string(),
                    kind: InlineKeyboardButtonKind::WebApp(WebAppInfo {
                        url: "https://takeover.fyi".parse()?,
                    }),
                }]],
            }))
            .await?;
        return Ok(());
    }

    diesel::insert_into(takeovers::table)
        .values(takeover)
        .execute(&mut context.postgres().await?)?;

    bot.send_message(
        msg.chat.id,
        format!("All set up! Your community page is https://symbol.takeover.fyi"),
    )
    .reply_markup(ReplyMarkup::InlineKeyboard(InlineKeyboardMarkup {
        inline_keyboard: vec![vec![InlineKeyboardButton {
            text: "Enter my community".to_string(),
            kind: InlineKeyboardButtonKind::WebApp(WebAppInfo {
                url: "https://symbol.takeover.fyi".parse()?,
            }),
        }]],
    }))
    .await?;
    dialogue.exit().await?;
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
