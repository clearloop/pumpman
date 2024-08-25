use crate::{
    api::{PumpApi, SolRpcApi},
    model::{Pumpman, PumpmanGlobal, PumpmanJob},
    telegram::{
        pumpman::{
            callback::{Callback, ListCallback, WithdrawCallback},
            PumpmanContext,
        },
        Result,
    },
};
use bigdecimal::{BigDecimal, Zero};
use solana_sdk::{native_token::LAMPORTS_PER_SOL, pubkey::Pubkey, signer::Signer};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

/// message while entring group
pub const ENTER_GROUP: &str = r#"Only support private chats atm ))"#;

/// Send menu message
pub async fn wallet(pubkey: &Pubkey) -> Result<String> {
    Ok(format!(r#"Your wallet address: <code>{pubkey}</code>"#))
}

/// Send menu message
pub async fn menu(context: &PumpmanContext, user: i64) -> Result<String> {
    let fee_basis_points = context.fee_basis_points().await?;
    let global = context.pglobal(user).await?;
    let fee = 10 * 60 / global.speed * global.fee(&context.global, fee_basis_points);

    Ok(format!(
        r#"
The easiest way to keep your token staying on the first page of PumpFun!

<b>🎁 30% off on service fee till we reach 300 users!</b>

/fees of bumping a token for 10 mins with /config: <code>{} SOL</code>

Please paste a pumpfun link in the chat, for example: <code>https://pump.fun/2GncVSSwhxsJu4B5wGt14jRoG2iGCCFzQk6D9Lmspump</code>
"#,
        &fee.round(6),
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

/fees of <code>{bumps}</code> bumps in 10 mins: <code>{est} SOL</code>

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

* Token Account Fee: <code>~ 0.002 SOL</code>
There will be approximate <code>0.002 SOL</code> charged by solana in the first trade of each new tokens, it's for creating a token account that you can trade tokens on pumpfun, no other bots or platforms including pumpfun can not avoid it.
"#,
        global.bump_fee(&context.global, fee_basis_points).round(6),
        global.pumpfun_fee(fee_basis_points).round(6),
        global.amount.round(4),
        global.tx_fee().round(6),
        global.service_fee(&context.global).round(6),
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
    let balance = context.client.helius().get_balance(&pubkey).await?;
    let sol = BigDecimal::from(&balance) / LAMPORTS_PER_SOL;

    let fee_basis_points = context.fee_basis_points().await?;
    let fee = job.fee(&context.global, fee_basis_points);
    let bumps = 10 * 60 / job.speed;
    let fee10: BigDecimal = &fee * bumps;

    Ok(format!(
        r#"
Job <a href="https://pump.fun/{}">{} (${})</a>

Your Wallet Address: <code>{pubkey}</code>

* <code>{bumps} bumps</code> in <b>10 mins</b> with current config: <code>{} SOL</code>
* <b>Reserved balance</b> for bump amount: <code>{} SOL</code>
* <b>Free balance</b> for /fees: <code>{} SOL</code> which can bump ${} for around <code>{}</code>.
"#,
        coin.mint,
        coin.name,
        coin.symbol,
        sol.round(6).min((job.amount).round(6)),
        (sol - &job.amount).max(BigDecimal::zero()).round(6),
        coin.symbol,
        job.duration(&fee, balance),
        fee10.round(6)
    ))
}

/// List all jobs
pub fn list(jobs: &[Pumpman]) -> String {
    let total = jobs.len();
    let active = jobs.iter().filter(|j| j.active).count();
    format!(
        r#"
You currently have <code>{total}</code> jobs in total, <code>{active}</code> of them are active.

Tap job names to enter their dashboards. You can safely <code>delete</code> inactive jobs. Allocated balance will automatically return to your /wallet upon deletion.

NOTE: Only jobs have processed bumps will be listed here.
"#
    )
}

pub async fn list_markup(
    context: &PumpmanContext,
    jobs: &[Pumpman],
) -> Result<InlineKeyboardMarkup> {
    let redis = &mut context.redis()?;
    let mut kbs = Vec::new();
    for job in jobs.into_iter() {
        let coin = context.client.coin(&job.mint, false, redis).await?;
        let job_id = job.id();
        let mut commands = vec![InlineKeyboardButton::callback(
            format!("{} (${})", coin.name, coin.symbol),
            Callback::list(ListCallback::ShowJob(job_id)).format()?,
        )];

        if job.active {
            commands.push(InlineKeyboardButton::callback(
                "Stop",
                Callback::list(ListCallback::Stop(job_id)).format()?,
            ));
        } else {
            commands.push(InlineKeyboardButton::callback(
                "Start",
                Callback::list(ListCallback::Start(job_id)).format()?,
            ));
        }
        kbs.push(commands);
    }

    kbs.push(vec![
        InlineKeyboardButton::callback("All", Callback::DoNothing.format()?),
        InlineKeyboardButton::callback("Start", Callback::list(ListCallback::StartAll).format()?),
        InlineKeyboardButton::callback("Stop", Callback::list(ListCallback::StopAll).format()?),
    ]);
    Ok(InlineKeyboardMarkup::new(kbs))
}

pub fn iwithdraw(balance: u64) -> String {
    format!(
        r#"Enter an address to refund <code>{} (approx)</code>"#,
        (BigDecimal::from(balance) / LAMPORTS_PER_SOL).round(6)
    )
}

pub fn cwithdraw(balance: u64, recipient: &Pubkey) -> String {
    format!(
        r#"Sending <code>{} (approx)</code> to <code>{recipient}</code> ?"#,
        (BigDecimal::from(balance) / LAMPORTS_PER_SOL).round(6)
    )
}

pub fn cwithdraw_markup(recipient: &Pubkey) -> Result<InlineKeyboardMarkup> {
    Ok(InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::url(
            "solscan",
            format!("https://solscan.io/account/{recipient}").parse()?,
        )],
        vec![
            InlineKeyboardButton::callback(
                "Cancel",
                Callback::Withdraw(WithdrawCallback::Cancel).format()?,
            ),
            InlineKeyboardButton::callback(
                "Confirm",
                Callback::Withdraw(WithdrawCallback::Confirm).format()?,
            ),
        ],
    ]))
}

pub fn withdraw(sig: impl std::fmt::Display) -> String {
    format!(
        r#"
Thanks for trying out pumpman ^ ^

https://solscan.io/tx/{sig}
"#,
    )
}

pub const FAQ: &str = r#"
1. <b>How this bot works?</b>
Make your tokens stay on the first page of PumpFun via batching buy & sell small amount of your tokens.

2. <b>Why the first bump is more expensive than the followings?</b>
The first bump includes a token account creation instruction which takes around 0.02 SOL charged by Solana. No other bots or platforms, including Pumpfun itself, cannot avoid it, check /fee for more details.

3. <b>Why can't I bump my new created token?</b>
pumpfun recently updated their logic that the token creators can free launch tokens, and the creation fee will be paid by the first buyer which is the real creator of the token on-chain, we only support bumping tokens on-chain, which at least has one previous trade in the history.

4. <b>Why my job suddenly stopped</b>
To safeguard your transaction fees, the job will be halted if a transaction fails. Please ensure your wallet has sufficient funds and consider adjusting your bump configurations by increasing transaction fees and bump amounts.

5. <b>Why PumpFun only shows buy or sell instead of the both in some bumps</b>
They failed to index our bumps... You can click the transaction signature on the trading history and you will see each transaction contains both buy & sell instructions.

6. <b>How to delete inactive jobs?</b>
This feature is under development ))

7. <b>Are you reliable?</b>
This service is part of @takeoveralerts, we are currently in heavy development of our trading bot as well in the meanwhile, we want to be one of top projects in this industry, we are always doing our best!
"#;
