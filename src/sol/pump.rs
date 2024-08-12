use crate::sol;
use anchor_lang::AnchorSerialize;
use anyhow::Result;
use borsh::BorshSerialize;
pub use pump::*;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey,
    pubkey::Pubkey,
};

anchor_lang::declare_program!(pump);

/// Calculated from pumpfun program id
///
/// ```
/// Pubkey::find_program_address(&[b"global"], &ID).0
/// ```
pub static GLOBAL: Pubkey = pubkey!("4wTV1YmiEkRvAtNtsSGPtUrqRYQMe5SKy2uB4Jjaxnjf");

/// Buys tokens from a bonding curve
#[derive(AnchorSerialize)]
pub struct Buy {
    /// Buy token amount
    pub amount: u64,
    /// max sol sot for the transaction
    pub max_sol_cost: u64,
}

/// Buys tokens from a bonding curve
#[derive(AnchorSerialize)]
pub struct Sell {
    /// Buy token amount
    pub amount: u64,
    /// max sol sot for the transaction
    pub max_sol_cost: u64,
}

/// pumpfun trading required accounts
pub struct TradeAccounts {
    pub fee_recipient: Pubkey,
    pub mint: Pubkey,
    pub user: Pubkey,
}

/// Get bonding curve address from mint address
pub fn bonding_curve(mint: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"bonding-curve".as_ref(), &mint.to_bytes()], &pump::ID).0
}

/// Get pumpfun trade accounts
pub fn trade_accounts(mint: Pubkey, user: Pubkey, fee_recipient: Pubkey) -> Vec<AccountMeta> {
    let bc = bonding_curve(&mint);
    vec![
        AccountMeta::new_readonly(GLOBAL, false),
        AccountMeta::new(fee_recipient, false),
        AccountMeta::new_readonly(mint, false),
        AccountMeta::new(bc.clone(), false),
        AccountMeta::new(sol::atk_addr(&mint, &bc), false),
        AccountMeta::new(sol::atk_addr(&mint, &user), false),
        AccountMeta::new(user, true),
        AccountMeta::new_readonly(sol::SYSTEM_PROGRAM, false),
        AccountMeta::new_readonly(sol::TOKEN_PROGRAM, false),
        AccountMeta::new_readonly(sol::RENT, false),
        AccountMeta::new_readonly(sol::EVENT_AUTHORITY, false),
        AccountMeta::new_readonly(ID, false),
    ]
}
