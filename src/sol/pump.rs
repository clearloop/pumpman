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

fn ix(instr: impl AnchorSerialize) -> Result<Instruction> {
    Ok(Instruction {
        program_id: ID,
        accounts: vec![AccountMeta::new_readonly(
            Pubkey::find_program_address(&[b"global"], &ID).0,
            false,
        )],
        data: instr.try_to_vec()?,
    })
}

/// pumpfun trading required accounts
pub struct TradeAccounts {
    pub fee_recipient: Pubkey,
    pub mint: Pubkey,
    pub associated_bonding_curve: Pubkey,
    pub associated_user: Pubkey,
    pub user: Pubkey,
}

impl From<TradeAccounts> for Vec<AccountMeta> {
    fn from(accs: TradeAccounts) -> Vec<AccountMeta> {
        vec![
            TradeAccount::Global,
            TradeAccount::FeeRecipient(accs.fee_recipient),
            TradeAccount::Mint(accs.mint),
            TradeAccount::BondingCurve,
            TradeAccount::AssociatedBondingCurve(accs.associated_bonding_curve),
            TradeAccount::AssociatedUser(accs.associated_user),
            TradeAccount::User(accs.user),
            TradeAccount::SystemProgram,
            TradeAccount::TokenProgram,
            TradeAccount::Rent,
            TradeAccount::EventAuthority,
            TradeAccount::Program,
        ]
        .into_iter()
        .map(Into::into)
        .collect()
    }
}

/// pumpfun accounts
pub enum TradeAccount {
    Global,
    FeeRecipient(Pubkey),
    Mint(Pubkey),
    BondingCurve,
    AssociatedBondingCurve(Pubkey),
    AssociatedUser(Pubkey),
    User(Pubkey),
    SystemProgram,
    TokenProgram,
    Rent,
    EventAuthority,
    Program,
}

impl From<TradeAccount> for AccountMeta {
    fn from(acc: TradeAccount) -> AccountMeta {
        let mut is_signer = false;
        let mut is_writable = false;

        let pubkey = match acc {
            TradeAccount::Global => Pubkey::find_program_address(&[b"global"], &ID).0,
            TradeAccount::FeeRecipient(pk) => {
                is_writable = true;
                pk
            }
            TradeAccount::Mint(pk) => pk,
            TradeAccount::BondingCurve => {
                is_writable = true;
                Pubkey::find_program_address(&[b"bonding-curve"], &ID).0
            }
            TradeAccount::AssociatedBondingCurve(pk) => {
                is_writable = true;
                pk
            }
            TradeAccount::User(pk) => {
                is_signer = true;
                is_writable = true;
                pk
            }
            TradeAccount::SystemProgram => sol::SYSTEM_PROGRAM,
            TradeAccount::TokenProgram => sol::TOKEN_PROGRAM,
            TradeAccount::Rent => sol::RENT,
            TradeAccount::EventAuthority => sol::EVENT_AUTHORITY,
            TradeAccount::Program => ID,
            TradeAccount::AssociatedUser(pk) => pk,
        };

        AccountMeta {
            pubkey,
            is_signer,
            is_writable,
        }
    }
}

#[test]
fn test_keys() {
    assert_eq!(b"global", &[103, 108, 111, 98, 97, 108]);
    assert_eq!(
        b"bonding-curve",
        &[98, 111, 110, 100, 105, 110, 103, 45, 99, 117, 114, 118, 101]
    );
}
