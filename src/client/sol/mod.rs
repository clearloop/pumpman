use crate::utils::base64;
use anchor_lang::AnchorDeserialize;

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

#[test]
fn decode() {
    let r =  parse::<pump::events::TradeEvent>(vec!["Program data: vdt/007mYe66y1JXTQoKfWyG6mzuJGZAfqL9HXqV8Fr9AWNJXwt7X65EsHoAAAAA/4B9NP48AAAArPC89fcKqc8R7CBTT1nPPFRlogbDLAYKR+A+jMbLD3ODwYVmAAAAAE/RMhUHAAAAKoUjkGbCAwBPJQ8ZAAAAACrtEETVwwIA".to_string()]);

    println!("{:#?}", r);
}
