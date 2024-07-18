use crate::{
    api::HttpClient,
    context::Context,
    model::Takeover,
    schema::takeovers,
    telegram::{
        self, keyboard,
        takeover::{markup, Result, TakeoverDialogue},
    },
};
use anyhow::anyhow;
use diesel::RunQueryDsl;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use teloxide::{payloads::SendMessageSetters, prelude::*, types::ParseMode};

#[derive(Clone, Default, Serialize, Deserialize)]
pub enum State {
    #[default]
    Start,
    ReceiveCto,
    ReceiveCtoAddress(Takeover),
    ReceiveCtoTelegramGroup(Takeover),
}

pub async fn receive_cto(
    bot: Bot,
    context: Context,
    dialogue: TakeoverDialogue,
    msg: Message,
) -> Result<()> {
    tracing::trace!("Received CTO requested");
    let mint = msg.text().unwrap_or_default().to_string();
    if Pubkey::from_str(&mint).is_err() {
        bot.send_message(msg.chat.id, "Invalid solana token address.")
            .reply_markup(markup::website("See exists CTOs")?)
            .await?;

        return Ok(());
    }

    let redis = &mut context.redis()?;
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

pub async fn receive_cto_address(
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
    takeover.admin = telegram::uid(&msg)
        .ok_or(anyhow!("Takeover action running in group"))?
        .to_string();

    diesel::insert_into(takeovers::table)
        .values(takeover)
        .execute(&mut context.postgres()?)?;

    // TODO: forward a message from alerts to user
    bot.send_message(
        msg.chat.id,
        "All set up! Your community page is https://symbol.takeover.fyi".to_string(),
    )
    // .reply_markup(ReplyMarkup::InlineKeyboard(InlineKeyboardMarkup {
    //     inline_keyboard: vec![vec![InlineKeyboardButton {
    //         text: "Share to my community!".to_string(),
    //         kind: InlineKeyboardButtonKind::WebApp(WebAppInfo {
    //             url: "https://symbol.takeover.fyi".parse()?,
    //         }),
    //     }]],
    // }))
    .await?;
    dialogue.exit().await?;
    Ok(())
}
