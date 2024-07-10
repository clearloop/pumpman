use std::fmt::Display;
pub use takeover::TakeoverBot;
use teloxide::{types::Message, utils::markdown};

mod button;
mod keyboard;
mod takeover;

/// Get the user id from message
pub fn uid(msg: &Message) -> Option<u64> {
    msg.from().map(|u| u.id.0)
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
