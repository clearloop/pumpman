#![allow(missing_docs)]
#![allow(unused)]

use solana_sdk::{pubkey, pubkey::Pubkey};
pub use utils::{atk_addr, parse, parse2};

pub mod pump;
mod utils;

pub static SYSTEM_PROGRAM: Pubkey = pubkey!("11111111111111111111111111111111");
pub static TOKEN_PROGRAM: Pubkey = pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
pub static ASSOCIATED_TOKEN_PROGRAM: Pubkey =
    pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
pub static RENT: Pubkey = pubkey!("SysvarRent111111111111111111111111111111111");
pub static EVENT_AUTHORITY: Pubkey = pubkey!("Ce6TQqeHC9p8KetsN6JsjHK7UTZk7nasjjnr7XxXp9F1");
