//! Telegram instance

use teloxide::Bot;

/// Telegram instance
pub struct Telegram {
    bot: Bot,
}

impl Telegram {
    /// Create a new telegram instance
    pub fn new(token: &str) -> Self {
        Self {
            bot: Bot::new(token),
        }
    }
}
