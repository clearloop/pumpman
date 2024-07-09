//! Telegram keyboards

use crate::telegram::button;
use anyhow::Result;
use teloxide::types::ReplyMarkup;

/// Coin keyboard
pub fn coin(mint: &str, pair: Option<String>) -> Result<ReplyMarkup> {
    let mut keyboards = vec![];

    keyboards.push(button::inline_url(
        "View on pump.fun",
        &format!("https://pump.fun/{}", mint),
    )?);

    if let Some(pair) = pair {
        keyboards.push(button::inline_url("View on dexscreener", &pair)?);
    }

    Ok(ReplyMarkup::inline_kb(vec![keyboards]))
}
