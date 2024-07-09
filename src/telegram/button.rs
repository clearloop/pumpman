//! Telegram buttons

use anyhow::Result;
use teloxide::types::InlineKeyboardButton;

/// Inline keyboard button with url
pub fn inline_url(text: &str, url: &str) -> Result<InlineKeyboardButton> {
    Ok(InlineKeyboardButton::url(text, url.parse()?))
}
