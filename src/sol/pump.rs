//! Pump fun handlers

use bigdecimal::BigDecimal;
pub use pump::*;
use std::{
    fmt::{self, Display},
    ops::{Div, Sub},
};

anchor_lang::declare_program!(pump);

pub const SOL_PRECISON: u32 = 9;
// pub const TOKEN_PRECISON: u8 = 6;

impl Display for events::TradeEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let sol = BigDecimal::from(self.sol_amount)
            .div(10_u64.pow(SOL_PRECISON))
            .round(5)
            .to_string()
            .replace(".", "\\.");
        let user = &self.user.to_string();
        let order = if self.is_buy { "bought" } else { "sold" };
        let url = format!("https://pump.fun/{}", self.mint);
        let dex = BigDecimal::from(86)
            .sub(BigDecimal::from(self.real_sol_reserves).div(10_u64.pow(SOL_PRECISON)))
            .round(5)
            .to_string()
            .replace(".", "\\.");

        write!(
            f,
            r#"
[{}](https://solscan.io/account/{user}) {order} {sol} SOL of [${{SYMBOL}}]({url})

{} SOL to be deposited into Raydium
"#,
            &user[..6],
            dex
        )
    }
}
