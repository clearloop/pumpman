use crate::{sol, utils::base64};
use anchor_lang::AnchorDeserialize;
use solana_sdk::pubkey::Pubkey;

const LOG_PREFIX: &str = "Program data: ";
const DISCRIMINATOR_SIZE: usize = 8;

/// Parse logs to event
pub fn parse<T: AnchorDeserialize>(logs: &[String]) -> Option<T> {
    let event = logs
        .iter()
        .find(|l| l.starts_with(LOG_PREFIX))?
        .replace(LOG_PREFIX, "");

    T::deserialize(&mut &base64::decode(&event).ok()?[DISCRIMINATOR_SIZE..]).ok()
}

/// get associated token address
pub fn atk_addr(mint: &Pubkey, owner: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[
            &owner.to_bytes(),
            &sol::TOKEN_PROGRAM.to_bytes(),
            &mint.to_bytes(),
        ],
        &sol::ASSOCIATED_TOKEN_PROGRAM,
    )
    .0
}
