//! Telegram takeover bot

use crate::{
    api::HttpClient,
    context::Context,
    model::Takeover,
    schema::takeovers,
    telegram::{self, keyboard},
};
use anyhow::anyhow;
use diesel::RunQueryDsl;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::{str::FromStr, sync::Arc};
use teloxide::{
    dispatching::dialogue::{self, serializer::Json, ErasedStorage, RedisStorage, Storage},
    dptree::case,
    payloads::SendMessageSetters,
    prelude::*,
    types::{
        InlineKeyboardButton, InlineKeyboardButtonKind, InlineKeyboardMarkup, MenuButton,
        ParseMode, ReplyMarkup, WebAppInfo,
    },
    utils::command::BotCommands,
};

type TakeoverDialogue = Dialogue<State, ErasedStorage<State>>;
type TakeoverStorage = Arc<ErasedStorage<State>>;
type Result<T> = core::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

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
    pub async fn start(&self) -> anyhow::Result<()> {
        tracing::info!("Starting the takeover bot");

        let cache: TakeoverStorage = RedisStorage::open(self.redis.clone(), Json).await?.erase();
        let command = teloxide::filter_command::<Command, _>()
            .branch(case![State::Start].branch(case![Command::Takeover].endpoint(takeover)))
            .branch(case![Command::Start].endpoint(start))
            .branch(case![Command::Cancel].endpoint(cancel));

        let message = Update::filter_message()
            .branch(command)
            .branch(case![State::ReceiveCto].endpoint(receive_cto))
            .branch(case![State::ReceiveCtoAddress(takeover)].endpoint(receive_cto_address))
            .branch(dptree::endpoint(invalid_state));

        let callback =
            Update::filter_callback_query().branch(case![State::Start].endpoint(callback_takeover));

        let schema = dialogue::enter::<Update, ErasedStorage<State>, State, _>()
            .branch(callback)
            .branch(message);

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

async fn start(bot: Bot, dialogue: TakeoverDialogue, msg: Message) -> Result<()> {
    dialogue.exit().await?;
    if !msg.chat.id.is_user() {
        bot.send_message(
            msg.chat.id,
            r#"
Building better CTOs, feedbacks and ideas are welcome! @takeoverfyi

Only support private chats atm )) 
"#,
        )
        .await?;
        return Ok(());
    }

    bot.send_message(
        msg.chat.id,
        r#"
Building better CTOs, feedbacks and ideas are welcome! @takeoverfyi 
"#,
    )
    .reply_markup(menu()?)
    .await?;
    Ok(())
}

async fn callback_takeover(bot: Bot, dialogue: TakeoverDialogue, q: CallbackQuery) -> Result<()> {
    let Some(msg) = q.message else {
        return Ok(());
    };

    tracing::trace!("{:#?}", msg);
    takeover(bot, dialogue, msg).await
}

async fn takeover(bot: Bot, dialogue: TakeoverDialogue, msg: Message) -> Result<()> {
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
) -> Result<()> {
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

    let redis = &mut context.redis().await?;
    let Ok(coin) = context
        .client
        .coin(&mint, false, redis)
        .await
        .map_err(|e| tracing::error!("{e}"))
    else {
        bot.send_message(
            msg.chat.id,
            r#"
Failed to get token metadata, re-input the token address to retry.

If you believe this is a bug, please contact our dev @takeoverfyi
"#,
        )
        .await?;
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
"#,
            coin.symbol, coin.name, coin.mint,
        ),
    )
    .parse_mode(ParseMode::MarkdownV2)
    .reply_markup(keyboard::coin(
        &mint,
        context.client.pair(&mint, false, redis).await,
    )?)
    .await?;

    bot.send_message(
        msg.chat.id,
        r#"
Almost done! Please enter the telegram group handle of your community.
    
for example: @takeoverfyi
"#,
    )
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
    mut takeover: Takeover,
    msg: Message,
) -> Result<()> {
    let handle = msg.text().unwrap_or_default().trim().to_string();
    if !handle.starts_with('@') || handle.contains(|w: char| w.is_ascii_whitespace()) {
        bot.send_message(msg.chat.id, "Invalid telegram group handle.")
            .await?;
        return Ok(());
    }

    takeover.telegram = handle;
    takeover.proposer = telegram::uid(&msg)
        .ok_or(anyhow!("Takeover action running in group"))?
        .to_string();

    diesel::insert_into(takeovers::table)
        .values(takeover)
        .execute(&mut context.postgres().await?)?;

    // TODO: forward a message from alerts to user
    bot.send_message(
        msg.chat.id,
        "All set up! Your community page is https://symbol.takeover.fyi".to_string(),
    )
    .reply_markup(ReplyMarkup::InlineKeyboard(InlineKeyboardMarkup {
        inline_keyboard: vec![vec![InlineKeyboardButton {
            text: "Share to my community!".to_string(),
            kind: InlineKeyboardButtonKind::WebApp(WebAppInfo {
                url: "https://symbol.takeover.fyi".parse()?,
            }),
        }]],
    }))
    .await?;
    dialogue.exit().await?;
    Ok(())
}

async fn cancel(bot: Bot, dialogue: TakeoverDialogue, msg: Message) -> Result<()> {
    bot.send_message(
        msg.chat.id,
        "Cancelling the dialogue. Type /start to see the menu.",
    )
    .await?;
    dialogue.exit().await?;
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

fn menu() -> Result<ReplyMarkup> {
    let keyboard = vec![
        vec![InlineKeyboardButton::callback(
            "Claim Community Takeover",
            "/takeover",
        )],
        vec![InlineKeyboardButton::url(
            "Community Takeover Progress",
            "https://t.me/takeoverfyi".parse()?,
        )],
        vec![
            InlineKeyboardButton::url("Alerts", "https://t.me/takeoveralerts".parse()?),
            InlineKeyboardButton::url("Support", "https://t.me/takeoverfyi".parse()?),
        ],
    ];

    Ok(ReplyMarkup::InlineKeyboard(InlineKeyboardMarkup::new(
        keyboard,
    )))
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    /// Start bot use.
    #[command(description = "Start bot use.")]
    Start,
    /// Cancel the current operation.
    #[command(description = "Cancel the current operation.")]
    Cancel,
    #[command(description = "Claim a community take over.")]
    /// Claim a community take over
    Takeover,
}
