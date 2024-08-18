use crate::sol;
use accounts::Global;
use anchor_lang::{Discriminator, InstructionData};
use anyhow::{anyhow, Result};
use bigdecimal::{BigDecimal, ToPrimitive};
use borsh::BorshSerialize;
pub use pump::*;
use serde::Serialize;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    native_token::LAMPORTS_PER_SOL,
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

/// Slippage basis
pub const SLIPPAGE_BASIS: u64 = 100;

/// SOL amount scale
pub const SOL_SCALE: u64 = LAMPORTS_PER_SOL;

/// Token amount decimals
pub const TOKEN_DECIMALS: u32 = 6;

/// Token amount scale
pub const TOKEN_SCALE: u64 = 1_000_000;

/// Buys tokens from a bonding curve
#[derive(BorshSerialize)]
pub struct Buy {
    /// Buy token amount
    pub amount: u64,
    /// max sol sot for the transaction
    pub max_sol_cost: u64,
}

impl Discriminator for Buy {
    const DISCRIMINATOR: [u8; 8] = [102, 6, 61, 18, 1, 218, 235, 234];
}

impl InstructionData for Buy {}

impl Buy {
    /// Create new buy instruction
    pub fn new(amount: u64, max_sol_cost: u64) -> Self {
        Self {
            amount,
            max_sol_cost,
        }
    }

    /// Convert self to instruction
    pub fn ix(&self, global: &Global, mint: Pubkey, user: Pubkey) -> Instruction {
        let bc = bonding_curve(&mint);
        Instruction::new_with_bytes(
            pump::ID,
            &self.data(),
            vec![
                AccountMeta::new_readonly(GLOBAL, false),
                AccountMeta::new(global.fee_recipient, false),
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
            ],
        )
    }
}

/// Buys tokens from a bonding curve
#[derive(BorshSerialize)]
pub struct Sell {
    /// Buy token amount
    pub amount: u64,
    /// max sol sot for the transaction
    pub min_sol_ouput: u64,
}

impl Discriminator for Sell {
    const DISCRIMINATOR: [u8; 8] = [51, 230, 133, 164, 1, 127, 131, 173];
}

impl Sell {
    /// Create new buy instruction
    pub fn new(amount: u64, min_sol_ouput: u64) -> Self {
        Self {
            amount,
            min_sol_ouput,
        }
    }

    /// Convert self to instruction
    pub fn ix(&self, global: &Global, mint: Pubkey, user: Pubkey) -> Instruction {
        let bc = bonding_curve(&mint);
        Instruction::new_with_bytes(
            pump::ID,
            &self.data(),
            vec![
                AccountMeta::new_readonly(GLOBAL, false),
                AccountMeta::new(global.fee_recipient, false),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new(bc, false),
                AccountMeta::new(sol::atk_addr(&mint, &bc), false),
                AccountMeta::new(sol::atk_addr(&mint, &user), false),
                AccountMeta::new(user, true),
                AccountMeta::new_readonly(sol::SYSTEM_PROGRAM, false),
                AccountMeta::new_readonly(sol::ASSOCIATED_TOKEN_PROGRAM, false),
                AccountMeta::new_readonly(sol::TOKEN_PROGRAM, false),
                AccountMeta::new_readonly(sol::EVENT_AUTHORITY, false),
                AccountMeta::new_readonly(ID, false),
            ],
        )
    }
}

impl InstructionData for Sell {}

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

impl accounts::Global {
    /// Default implemention of global
    pub fn cached() -> Self {
        Self {
            initialized: true,
            authority: pubkey!("DCpJReAfonSrgohiQbTmKKbjbqVofspFRHz9yQikzooP"),
            fee_recipient: pubkey!("CebN5WGQ4jvEPvsVU4EoHEpgzq1VV7AbicfhtW4xC9iM"),
            initial_virtual_token_reserves: 1073000000000000,
            initial_virtual_sol_reserves: 30000000000,
            initial_real_token_reserves: 793100000000000,
            token_total_supply: 1000000000000000,
            fee_basis_points: 100,
        }
    }

    /// Get token supply from total sol reserves
    fn supply(&self, sol: u64) -> Result<u64> {
        let vtoken = BigDecimal::from(self.initial_virtual_token_reserves);
        let s = vtoken.clone() * self.initial_virtual_sol_reserves
            / (self.initial_virtual_sol_reserves + sol);

        (vtoken - s).to_u64().ok_or(anyhow!(
            "Failed to calculate total supply from reserve: {sol}"
        ))
    }

    /// Calculate token amount to buy with max sol spent
    pub fn buy(&self, reserves: u64, sol: u64) -> Result<u64> {
        Ok(self.supply(sol + reserves)? - self.supply(reserves)?)
    }

    /// Calculate token amount to sell with min sol receive
    pub fn sell(&self, reserves: u64, sol: u64) -> Result<u64> {
        Ok(self.supply(reserves)? - self.supply(reserves - sol)?)
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
