//! Pumpman services

mod comment;
pub mod pumpman;
mod takeover;

use crate::{Config, Context};
use anyhow::Result;
use std::time::Duration;
use tokio::signal;
pub use {comment::Commenter, takeover::Takeover};

/// Starts both pumpman and takeover together
pub async fn start(config: Config, context: Context) -> Result<()> {
    loop {
        let r = tokio::select! {
            _ = signal::ctrl_c() => break,
            r = pumpman::start(config.clone(), context.clone(), false), if config.pumpman.is_some() => r,
            r = Takeover::start(config.clone(), context.clone(), false), if config.takeover.is_some() => r,
        };

        if let Err(e) = r {
            tracing::error!("{e}");
            tokio::time::sleep(Duration::from_secs(10)).await;
        }
    }

    Ok(())
}
