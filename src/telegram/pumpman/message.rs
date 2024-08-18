use crate::{
    api::{PumpApi, SolRpcApi},
    config,
    model::{Pumpman, PumpmanGlobal, PumpmanJob},
    sol::pump::{SLIPPAGE_BASIS, SOL_SCALE},
    telegram::{
        pumpman::{callback::Callback, PumpmanContext},
        Result,
    },
};
use bigdecimal::BigDecimal;
use solana_sdk::{pubkey::Pubkey, signer::Signer};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

/// message while entring group
pub const ENTER_GROUP: &str = r#"
Only support private chats atm ))
"#;

/// Send menu message
pub async fn menu(context: &PumpmanContext, user: i64, wallet: &Pubkey) -> Result<String> {
    let global = context.client.global().await?;
    let pglobal = context.global(user).await?;
    let fee = 60 / pglobal.speed * pglobal.avg_fee(&context.global, &global);

    Ok(format!(
        r#"
The easist way to keep your token staying on the first page of PumpFun!

Average /fees of bumping a token for 10 mins with /config: <code>{} SOL</code>

Your Wallet Address: <code>{}</code>

Please paste a pumpfun link in the chat, for example: <code>https://pump.fun/8CTjSbj6h3pAMx1UJcQXLwA4KXAwRF6nQ1JVMkBjpump</code>
"#,
        10 * &fee.round(4),
        wallet
    ))
}

/// message the default config of the pumpman bot
pub const CONFIG: &str = r#"
Your new created jobs will inherit this config on initialization.

* <b>Batch Bumps</b>: How many bumps will be included per transaction.
* <b>Slippage</b>: Maximum slippage of bump operations.
* <b>Transaction Fee</b>: Tips for the validators to make your bumps confirm faster.
* <b>Bump Amount</b>: Buy and sell amounts used in your bump transactions.
* <b>Speed</b>: Duration between each bump transaction.
"#;

/// Message the details of the fees
pub fn fees(global: &config::PumpmanGlobal, pglobal: &PumpmanGlobal) -> String {
    let pfee = &pglobal.amount / 50u32;
    let sfee = &pglobal.amount * pglobal.slippage / SLIPPAGE_BASIS * 2u32;
    let fee = &pfee + &sfee + &pglobal.tx_fee() + &global.service_fee;

    format!(
        r#"
<b>Fees</b> per bump transaction based on /config - <code>{} SOL ~ {} SOL</code>

* PumpFun Fee: <code>{} SOL</code>
2% from a <b>bump amount</b> ({} SOL) charged by PumpFun

* PumpFun Maximum Slippage Amount: <code>{} SOL</code>
This is the Maximum amount of slippage you are willing to accept when bumping.

* Transaction Fee: <code>{}</code>
Incentive for validators to put your transaction in a block as fast as possible.

* Service Fee: <code>{} SOL</code>
Once you have spent over <code>{} SOL</code> of service fee on a specific token, there will be
no service fees applied on that token anymore!
"#,
        (&fee - &sfee).round(6),
        fee.round(6),
        pfee.round(6),
        pglobal.amount.round(4),
        sfee.round(6),
        pglobal.tx_fee().round(6),
        global.service_fee.round(4),
        global.threshold.round(2),
    )
}

pub const INVALID_PUMPFUN_LINK: &str = r#"
Invalid PumpFun link, for example:

https://pump.fun/8CTjSbj6h3pAMx1UJcQXLwA4KXAwRF6nQ1JVMkBjpump
"#;

/// List all jobs
pub fn list(jobs: &[Pumpman]) -> String {
    format!("you current have {} jobs running", jobs.len())
}

pub async fn list_markup(
    context: &PumpmanContext,
    jobs: &[Pumpman],
) -> Result<InlineKeyboardMarkup> {
    let redis = &mut context.redis()?;
    let mut kbs = Vec::new();
    for job in jobs {
        let coin = context.client.coin(&job.mint, false, redis).await?;
        kbs.push(vec![InlineKeyboardButton::callback(
            format!("{} (${})", coin.name, coin.symbol),
            Callback::ShowJob(job.id()).format()?,
        )]);
    }

    Ok(InlineKeyboardMarkup::new(kbs))
}

pub async fn job(context: &PumpmanContext, job: &Pumpman) -> Result<String> {
    let redis = &mut context.redis()?;
    let coin = context.client.coin(&job.mint, true, redis).await?;
    let wallet = context.wallet(job.owner).await?;
    let pubkey = wallet.pubkey();
    let balance = context.client.rpc().get_balance(&pubkey).await?;
    let global = context.client.global().await?;

    Ok(format!(
        r#"
Job <a href="https://pump.fun/{}">{} (${})</a>

Your Wallet Address: <code>{}</code>

The current balance <code>{} SOL</code> can bump ${} for around {}.
"#,
        coin.mint,
        coin.name,
        coin.symbol,
        pubkey,
        BigDecimal::from(balance) / SOL_SCALE,
        coin.symbol,
        job.duration(&context.global, &global, balance)
    ))
}
