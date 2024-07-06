use crate::utils::base64;
use anchor_lang::AnchorDeserialize;
use mpl_token_metadata::accounts::Metadata as MplMetadata;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use solana_sdk::pubkey::Pubkey;

// pub mod raydium;
pub mod pump;

const LOG_PREFIX: &str = "Program data: ";
const DISCRIMINATOR_SIZE: usize = 8;

/// Parse logs to event
pub fn parse<T: AnchorDeserialize>(logs: Vec<String>) -> Option<T> {
    let event = logs
        .iter()
        .find(|l| l.starts_with(LOG_PREFIX))?
        .replace(LOG_PREFIX, "");

    T::deserialize(&mut &base64::decode(&event).ok()?[DISCRIMINATOR_SIZE..]).ok()
}
