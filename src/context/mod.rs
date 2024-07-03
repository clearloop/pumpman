//! Global context

pub use {redis::Redis, telegram::Telegram};

mod postgres;
mod redis;
mod telegram;
