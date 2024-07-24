use crate::{model::TakeoverWithCoin, telegram::takeover::Result};
use teloxide::types::{
    InlineKeyboardButton, InlineKeyboardMarkup, KeyboardButton, ReplyMarkup, WebAppInfo,
};

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
