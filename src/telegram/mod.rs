pub use takeover::TakeoverBot;
use teloxide::types::Message;

mod button;
mod keyboard;
mod takeover;

/// Get the user id from message
pub fn uid(msg: &Message) -> Option<u64> {
    msg.from().map(|u| u.id.0)
}
