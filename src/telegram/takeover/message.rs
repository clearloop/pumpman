use crate::{
    api::{HttpClient, SolRpcApi},
    context::Context,
    model::{pump::Coin as PumpCoin, Alert, AlertTitle},
};
use anyhow::Result;

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
    let alert = Alert::new(AlertTitle::ClaimCTO, coin.clone(), soldout)
        .pairs(pairs)
        .holders(holders);

    Ok(alert.to_string())
}
