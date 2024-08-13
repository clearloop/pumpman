use crate::sol;
use anchor_lang::AnchorSerialize;
use anyhow::Result;
use bigdecimal::{BigDecimal, RoundingMode, ToPrimitive};
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

/// Total supply of pumpfun coins
pub const TOTAL_SUPPLY: u64 = 1_000_000_000;

/// SOL amount scale
pub const SOL_SCALE: u64 = 1_000_000_000;

/// Token amount decimals
pub const TOKEN_DECIMALS: u32 = 6;

/// Token amount scale
pub const TOKEN_SCALE: u64 = 1_000_000;

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
        AccountMeta::new(bc, false),
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

impl accounts::Global {
    /// Get token supply from total sol reserves
    fn supply(&self, sol: u64) -> u64 {
        (self.initial_virtual_token_reserves
            - (BigDecimal::from(self.initial_virtual_token_reserves)
                * self.initial_virtual_sol_reserves
                / (self.initial_virtual_sol_reserves + sol)))
            .with_scale_round(0, RoundingMode::Floor)
            .to_u64()
            .expect("only the mutiple operation causes overflow")
    }

    /// Calculate token amount to buy with max sol spent
    pub fn buy(&self, bc: &accounts::BondingCurve, sol: u64) -> u64 {
        self.supply(sol + bc.real_sol_reserves) - self.supply(bc.real_sol_reserves)
    }

    /// Calculate token amount to sell with min sol receive
    pub fn sell(&self, bc: &accounts::BondingCurve, sol: u64) -> u64 {
        self.supply(bc.real_sol_reserves) - self.supply(bc.real_sol_reserves - sol)
    }

    /// Mock the new initialized bonding curve account
    pub fn init(&self) -> accounts::BondingCurve {
        accounts::BondingCurve {
            virtual_sol_reserves: self.initial_virtual_sol_reserves,
            virtual_token_reserves: self.initial_virtual_token_reserves,
            ..Default::default()
        }
    }
}
