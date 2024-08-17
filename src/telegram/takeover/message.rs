use crate::{
    api::{DexScreenerApi, PumpApi, SolRpcApi},
    context::Context,
    model::{pump::Coin as PumpCoin, Alert, AlertTitle},
};
use crate::{model::TakeoverWithCoin, telegram::takeover::Result};
use teloxide::types::{
    InlineKeyboardButton, InlineKeyboardMarkup, KeyboardButton, ReplyMarkup, WebAppInfo,
};

pub const INVALID_ADDRESS: &str = "Invalid solana token address.";

pub const NO_METADATA: &str = r#"
Failed to ge token metadata, re-input the token address to retry.

If you believe this is a bug, please contact our dev @takeoverfyi
"#;

pub const INPUT_HANDLE: &str = r#"
Almost done! Please enter the telegram group handle of your community.
    
(for example: @takeoverfyi)
"#;

pub const ENTER_GROUP: &str = r#"
Building better CTOs, feedbacks and ideas are welcome! @takeoverfyi

Only support private chats atm ))
"#;

pub const BRANDING: &str = r#"
Building better CTOs, feedbacks and ideas are welcome! @takeoverfyi
"#;

pub const CANCEL: &str = r#"
Cancelling the dialogue. Type /start to see the menu.
"#;

pub const TAKEOVER: &str = r#"
Let's start! Which token your community are about to take over?
"#;

pub const INVALID: &str = r#"
Unable to handle the message. Type /start to see the usage.
"#;

pub const INSUFFICIENT_CREDITS: &str = r#"
Does not have enough credits, ask for more @takeoverfyi
"#;

pub const CHOOSE_INFO: &str = r#"Choose a CTO you want to inspect info."#;

pub const NO_CTOS: &str = r#"You currently have no CTOs, type /start to claim yours!"#;

pub async fn coin(coin: &PumpCoin, context: &Context) -> Result<String> {
    let mint = &coin.mint;
    let redis = &mut context.redis()?;
    let pairs = context.client.pairs(mint, false, redis).await?;
    let holders = context
        .client
        .top_holders(mint, false, redis)
        .await?
        .skip_bc(&coin.associated_bonding_curve);
    let soldout = context
        .client
        .soldout(mint, &coin.creator, false, redis)
        .await?;
    let alert = Alert::new(AlertTitle::ClaimCTO, coin.clone(), soldout.1)
        .pairs(pairs)
        .holders(holders);

    Ok(alert.to_string())
}

pub fn website(text: impl AsRef<str>) -> Result<ReplyMarkup> {
    Ok(ReplyMarkup::InlineKeyboard(InlineKeyboardMarkup {
        inline_keyboard: vec![vec![InlineKeyboardButton::web_app(
            text.as_ref().to_string(),
            WebAppInfo {
                url: "https://takeover.fyi".parse()?,
            },
        )]],
    }))
}

pub fn menu() -> Result<ReplyMarkup> {
    let keyboard = vec![
        vec![InlineKeyboardButton::callback(
            "Claim Community Takeover",
            "/takeover",
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

pub fn list(takeovers: &[TakeoverWithCoin]) -> Result<ReplyMarkup> {
    let mut takeovers = takeovers
        .iter()
        .map(|t| KeyboardButton::new(t.takeover.telegram.to_string()))
        .collect::<Vec<_>>();

    let mut last: Vec<KeyboardButton> = vec![];
    if takeovers.len() % 2 != 0 {
        if let Some(kb) = takeovers.pop() {
            last.push(kb);
        }
    }

    let mut windows: Vec<Vec<KeyboardButton>> = takeovers.windows(2).map(|v| v.to_vec()).collect();
    if !last.is_empty() {
        windows.push(last);
    }

    Ok(ReplyMarkup::keyboard(windows))
}
