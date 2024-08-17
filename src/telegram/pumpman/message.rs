use crate::{
    api::{PumpApi, SolRpcApi},
    config,
    model::{Pumpman, PumpmanGlobal},
    sol::pump::SOL_SCALE,
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
pub fn menu(global: &config::PumpmanGlobal, pglobal: &PumpmanGlobal, wallet: Pubkey) -> String {
    let efee = 10 * 60 / pglobal.speed * pglobal.total_fee(&global.fee) * pglobal.batch;
    format!(
        r#"
The easist way to keep your token staying on the first page of PumpFun!

Total /fees of bumping a token for 10 mins with /config - <code>{} SOL</code>

Your Wallet Address: <code>{}</code>

Please paste a pumpfun link in the chat, for example: <code>https://pump.fun/8CTjSbj6h3pAMx1UJcQXLwA4KXAwRF6nQ1JVMkBjpump</code>
"#,
        efee.round(4),
        wallet
    )
}

/// message the default config of the pumpman bot
pub fn config(global: &PumpmanGlobal) -> String {
    format!(
        r#"
You new created jobs will inherit this config by default.

* SOL Amount per Bump: <code>{} SOL</code>
A <b>bump transaction</b> is the combination of buy and sell instructions of your token, the
SOL amount above is used in the buy and sell operations.

* Transaction Fee: <code>{} SOL</code>
Tips for the validators that make sure your bumps will be processed to solana successfully.

* Bump Speed: <code>{}s</code>
Duration between each bump transaction.
"#,
        global.amount.round(3),
        global.tx_fee.round(6),
        global.speed
    )
}

/// Message the details of the fees
pub fn fees(global: &config::PumpmanGlobal, pglobal: &PumpmanGlobal) -> String {
    let pf_fee = pglobal.amount.clone() / 50u32;
    let fee = pf_fee.clone() + &pglobal.tx_fee + &global.fee;

    format!(
        r#"
<b>Total Fee</b> on each bump transaction based on /config - <code>{} SOL</code>

* PumpFun Fee: <code>{} SOL</code>
2% from a <b>bump amount</b> ({} SOL) charged by PumpFun

* Transaction Fee: <code>{}</code>
Incentive for validators to put your transaction in a block as fast as possible.

* Service Fee: <code>{} SOL</code>
Once you have spent over <code>{} SOL</code> of service fee on a specific token, there will be
no service fees applied on that token anymore!
"#,
        fee.round(6),
        pf_fee.round(6),
        pglobal.amount.round(4),
        pglobal.tx_fee.round(6),
        global.fee.round(4),
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

    Ok(format!(
        r#"
Job <a href="https://pump.fun/{}">{} (${})</a>

Your Wallet Address: <code>{}</code>

The current balance <code>{} SOL</code> can bump ${} for {}.
"#,
        coin.mint,
        coin.name,
        coin.symbol,
        pubkey,
        BigDecimal::from(balance) / SOL_SCALE,
        coin.symbol,
        job.duration(&context.global, balance)
    ))
}
