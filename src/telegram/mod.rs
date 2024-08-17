use std::fmt::Display;
use teloxide::utils::markdown;

mod button;
mod keyboard;
pub mod pumpman;
pub mod takeover;

/// Escape to pattern markdown style
pub trait Escape {
    fn escaped(&self) -> String;
}

impl<T: Display> Escape for T {
    fn escaped(&self) -> String {
        markdown::escape(&self.to_string())
    }
}

pub type Result<T> = core::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
