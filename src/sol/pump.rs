use crate::sol;
use anchor_lang::AnchorSerialize;
use anyhow::Result;
use borsh::BorshSerialize;
pub use pump::*;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

anchor_lang::declare_program!(pump);

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

impl TradeAccounts {
    /// Create new pumpfun trade accounts
    pub fn new(mint: Pubkey, user: Pubkey, fee_recipient: Pubkey) -> Self {
        Self {
            mint,
            user,
            fee_recipient,
        }
    }

    /// Get global account
    pub fn global() -> Pubkey {
        Pubkey::find_program_address(&[b"global"], &ID).0
    }

    /// Get bonding curve account
    pub fn bonding_curve(&self) -> Pubkey {
        Pubkey::find_program_address(
            &[b"bonding-curve".as_ref(), &self.mint.to_bytes()],
            &pump::ID,
        )
        .0
    }
}

impl From<TradeAccounts> for Vec<AccountMeta> {
    fn from(accs: TradeAccounts) -> Vec<AccountMeta> {
        let bc = Pubkey::find_program_address(
            &[b"bonding-curve".as_ref(), &accs.mint.to_bytes()],
            &pump::ID,
        )
        .0;
        vec![
            AccountMeta::new_readonly(Pubkey::find_program_address(&[b"global"], &ID).0, false),
            AccountMeta::new(accs.fee_recipient, false),
            AccountMeta::new_readonly(accs.mint, false),
            AccountMeta::new(bc.clone(), false),
            AccountMeta::new(sol::atk_addr(&accs.mint, &bc), false),
            AccountMeta::new(sol::atk_addr(&accs.mint, &accs.user), false),
            AccountMeta::new(accs.user, true),
            AccountMeta::new_readonly(sol::SYSTEM_PROGRAM, false),
            AccountMeta::new_readonly(sol::TOKEN_PROGRAM, false),
            AccountMeta::new_readonly(sol::RENT, false),
            AccountMeta::new_readonly(sol::EVENT_AUTHORITY, false),
            AccountMeta::new_readonly(ID, false),
        ]
    }
}
