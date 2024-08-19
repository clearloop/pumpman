use crate::{
    api::{PumpApi, SolRpcApi},
    model::{Pumpman, PumpmanGlobal, PumpmanJob},
    telegram::{
        pumpman::{callback::Callback, PumpmanContext},
        Result,
    },
};
use bigdecimal::{BigDecimal, Zero};
use solana_sdk::{native_token::LAMPORTS_PER_SOL, pubkey::Pubkey, signer::Signer};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

/// message while entring group
pub const ENTER_GROUP: &str = r#"
Only support private chats atm ))
"#;

/// Send menu message
pub async fn menu(context: &PumpmanContext, user: i64, wallet: &Pubkey) -> Result<String> {
    let fee_basis_points = context.fee_basis_points().await?;
    let global = context.pglobal(user).await?;
    let fee = 60 / global.speed * global.fee(&context.global, fee_basis_points);

    Ok(format!(
        r#"
The easist way to keep your token staying on the first page of PumpFun!

Your Wallet Address: <code>{}</code>
/fees of bumping a token for 10 mins with /config: <code>{} SOL</code>

Please paste a pumpfun link in the chat, for example: <code>https://pump.fun/8CTjSbj6h3pAMx1UJcQXLwA4KXAwRF6nQ1JVMkBjpump</code>
"#,
        wallet,
        10 * &fee.round(6),
    ))
}

/// message the default config of the pumpman bot
pub async fn config(context: &PumpmanContext, global: &PumpmanGlobal) -> Result<String> {
    let fee_basis_points = context.fee_basis_points().await?;
    let fee = global.fee(&context.global, fee_basis_points);
    let bumps = 10 * 60 / global.speed;
    let est = (bumps * fee).round(6);

    Ok(format!(
        r#"
Your new created jobs will inherit this config on initialization.

/fees for 10 mins with <code>{bumps}</code> bumps: <code>{est} SOL</code>

* <b>Batch Bumps</b>: How many bumps will be included per transaction.
* <b>Transaction Fee</b>: Tips for the validators to make your bumps confirm faster.
* <b>Bump Amount</b>: Buy and sell amounts used in your bump transactions.
* <b>Speed</b>: Duration between each bump transaction.
"#
    ))
}

/// Message the details of the fees
pub async fn fees(context: &PumpmanContext, tgid: i64) -> Result<String> {
    let global = context.pglobal(tgid).await?;
    let fee_basis_points = context.fee_basis_points().await?;

    Ok(format!(
        r#"
<b>Fees</b> per bump based on /config - <code>{} SOL</code>

* PumpFun Fee: <code>{} SOL</code>
2% from a <b>bump amount</b> ({} SOL) charged by PumpFun

* Transaction Fee: <code>{}</code>
Incentive for validators to put your transaction in a block as fast as possible.

* Service Fee: <code>{} SOL</code>
Once you have spent over <code>{} SOL</code> of service fee on a specific token, there will be
no service fees applied on that token anymore!
"#,
        global.bump_fee(&context.global, fee_basis_points).round(6),
        global.pumpfun_fee(fee_basis_points).round(6),
        global.amount.round(4),
        global.tx_fee().round(6),
        global.service_fee(&context.global).round(4),
        context.global.threshold.round(2),
    ))
}

pub const INVALID_PUMPFUN_LINK: &str = r#"
Invalid PumpFun link, for example:

https://pump.fun/8CTjSbj6h3pAMx1UJcQXLwA4KXAwRF6nQ1JVMkBjpump
"#;

pub async fn job(context: &PumpmanContext, job: &Pumpman) -> Result<String> {
    let redis = &mut context.redis()?;
    let coin = context.client.coin(&job.mint, true, redis).await?;
    let wallet = context.wallet(job.owner).await?;
    let pubkey = wallet.pubkey();
    let global = context.client.global().await?;
    let balance = context.client.rpc().get_balance(&pubkey).await?;
    let sol = BigDecimal::from(&balance) / LAMPORTS_PER_SOL;

    Ok(format!(
        r#"
Job <a href="https://pump.fun/{}">{} (${})</a>

Your Wallet Address: <code>{pubkey}</code> (<code>{} SOL</code>)

Reserved balance for bump amount: <code>{} SOL</code>
Free balance for bump fees <code>{} SOL</code> which can bump ${} for around {}.
"#,
        coin.mint,
        coin.name,
        coin.symbol,
        (&job.amount / LAMPORTS_PER_SOL).round(6),
        sol.round(6),
        coin.symbol,
        (sol - &job.amount).min(BigDecimal::zero()),
        job.duration(&context.global, &global, balance)
    ))
}

/// List all jobs
pub fn list(jobs: &[Pumpman]) -> String {
    format!(r#"You currently have {} jobs running"#, jobs.len())
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
