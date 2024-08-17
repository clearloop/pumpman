//! Replika services

use crate::{
    context::Context,
    telegram::{self, pumpman::PumpmanContext},
    Config,
};
use anyhow::Result;
use std::time::Duration;
pub use takeover::Takeover;
use teloxide::Bot;
use tokio::signal;

// mod pump;
mod takeover;

/// Start pumpman service
pub async fn pumpman(config: &Config, context: Context) -> Result<()> {
    let Some(pumpman) = &config.pumpman else {
        tracing::error!("pumpman config not found");
        return Ok(());
    };

    let bot = Bot::new(&pumpman.bot);
    let context = PumpmanContext::new(context, pumpman.global.clone());

    loop {
        let r = tokio::select! {
            _ = signal::ctrl_c() => break,
            r = telegram::pumpman::start(&bot, context.clone(), config.database.redis.to_string()) => r,
        };

        if let Err(e) = r {
            tracing::error!("{e}");
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }

    Ok(())
}
