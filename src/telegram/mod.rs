use anyhow::{anyhow, Result};
use std::fmt::Display;
use teloxide::{types::Message, utils::markdown};

mod button;
mod keyboard;
pub mod takeover;

/// Get the user id from message
pub fn uid(msg: &Message) -> Result<u64> {
    msg.from()
        .map(|u| u.id.0)
        .ok_or_else(|| anyhow!("Running bot in group"))
}

/// Escape to pattern markdown style
pub trait Escape {
    fn escaped(&self) -> String;
}

impl<T: Display> Escape for T {
    fn escaped(&self) -> String {
        markdown::escape(&self.to_string())
    }
}
