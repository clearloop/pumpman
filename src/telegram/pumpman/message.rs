use crate::{
    config::PumpmanGlobal,
    model::{pump::Coin, Pumpman},
    sol::pump::SOL_SCALE,
};
use bigdecimal::BigDecimal;
use solana_sdk::pubkey::Pubkey;

/// message while entring group
pub const ENTER_GROUP: &str = r#"
Only support private chats atm ))
"#;

/// Send menu message
pub fn menu(global: &PumpmanGlobal, wallet: Pubkey, balance: u64) -> String {
    let efee = 10 * 60 / global.speed * global.total_fee();
    format!(
        r#"
The easist way to keep your token staying on the first page of PumpFun!

Total /fees of bumping a token for 10 mins - <code>{} SOL</code>

Your Bot Address: <code>{}</code>
Balance: <code>{} SOL</code>

Please paste a pumpfun link in the chat, for example: <code>https://pump.fun/8CTjSbj6h3pAMx1UJcQXLwA4KXAwRF6nQ1JVMkBjpump</code>
"#,
        efee.round(4),
        wallet.to_string(),
        BigDecimal::from(balance) / SOL_SCALE,
    )
}

/// message the default config of the pumpman bot
pub fn config(global: &PumpmanGlobal) -> String {
    format!(
        r#"
Pumpman global config:

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
pub fn fees(global: &PumpmanGlobal) -> String {
    let pf_fee = global.amount.clone() / 50u32;
    let fee = pf_fee.clone() + &global.tx_fee + &global.fee;

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
        global.amount.round(4),
        global.tx_fee.round(6),
        global.fee.round(4),
        global.threshold.round(2),
    )
}

pub const INVALID_PUMPFUN_LINK: &str = r#"
Invalid PumpFun link, for example:

https://pump.fun/8CTjSbj6h3pAMx1UJcQXLwA4KXAwRF6nQ1JVMkBjpump
"#;

pub fn job(
    global: &PumpmanGlobal,
    job: &Pumpman,
    coin: Coin,
    wallet: Pubkey,
    balance: u64,
) -> String {
    format!(
        r#"
Job <a href="https://pump.fun/{}">{} (${})</a>

Your Bot Address: <code>{}</code>

The current balance <code>{} SOL</code> can bump ${} for {}.
"#,
        coin.mint,
        coin.name,
        coin.symbol,
        wallet.to_string(),
        BigDecimal::from(balance) / SOL_SCALE,
        coin.symbol,
        job.duration(global, balance)
    )
}
