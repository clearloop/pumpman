use crate::{sol, utils::base64};
use anyhow::Result;
use borsh::BorshDeserialize;
use solana_sdk::pubkey::Pubkey;

const LOG_PREFIX: &str = "Program data: ";
const DISCRIMINATOR_SIZE: usize = 8;

/// Parse logs to event
pub fn parse<T: BorshDeserialize>(logs: &[String]) -> Option<T> {
    let event = logs
        .iter()
        .find(|l| l.starts_with(LOG_PREFIX))?
        .replace(LOG_PREFIX, "");

    T::deserialize(&mut &base64::decode(&event).ok()?[DISCRIMINATOR_SIZE..]).ok()
}

/// Parse logs to event
pub fn parse2<T: BorshDeserialize>(logs: &[String]) -> Result<Vec<T>> {
    let mut events = vec![];
    for log in logs.iter().filter(|l| l.starts_with(LOG_PREFIX)) {
        if let Ok(event) = T::deserialize(
            &mut &base64::decode(&log.replace(LOG_PREFIX, ""))?[DISCRIMINATOR_SIZE..],
        ) {
            events.push(event);
        }
    }

    Ok(events)
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
