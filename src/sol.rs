#![allow(unused)]

use crate::utils::base64;
use anchor_lang::AnchorDeserialize;

anchor_lang::declare_program!(pump);
anchor_lang::declare_program!(raydium);

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
