use crate::telegram::takeover::Result;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, ReplyMarkup, WebAppInfo};

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
        // vec![InlineKeyboardButton::url(
        //     "Community Takeover Progress",
        //     "https://t.me/takeoverfyi".parse()?,
        // )],
        vec![
            InlineKeyboardButton::url("Alerts", "https://t.me/takeoveralerts".parse()?),
            InlineKeyboardButton::url("Support", "https://t.me/takeoverfyi".parse()?),
        ],
    ];

    Ok(ReplyMarkup::InlineKeyboard(InlineKeyboardMarkup::new(
        keyboard,
    )))
}
