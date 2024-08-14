//! Replika services

use crate::{
    context::{Context, TgContext},
    telegram, Config,
};
use anyhow::Result;
use pump::{PumpEvent, PumpSub};
use std::time::Duration;
use takeover::Takeover;
use teloxide::Bot;
use tokio::{signal, sync::mpsc};

mod pump;
mod takeover;

/// Replika events
#[derive(Debug)]
pub enum Event {
    Pump(PumpEvent),
}

/// Start takeover service
pub async fn takeover(config: &Config, context: Context) -> Result<()> {
    let Some(takeover) = &config.takeover else {
        tracing::debug!("No config found for takeover service");
        return Ok(());
    };

    loop {
        let (tx, rx) = mpsc::channel::<Event>(50);
        let mut pumpsub = PumpSub::new(config, context.clone(), tx).await?;
        let mut takeover = Takeover::new(&takeover, context.clone(), rx);

        let r = tokio::select! {
            _ = signal::ctrl_c() => break,
            r = pumpsub.start() => r,
            r = takeover.start(format!("{}/15", config.database.redis)) => r
        };

        if let Err(e) = r {
            tracing::error!("{e}");
            tokio::time::sleep(Duration::from_secs(20)).await;
        }
    }

    Ok(())
}

/// Start pumpman service
pub async fn pumpman(config: &Config, context: Context) -> Result<()> {
    let Some(pumpman) = &config.pumpman else {
        tracing::error!("pumpman config not found");
        return Ok(());
    };

    let bot = Bot::new(&pumpman.bot);
    let context = TgContext::new(context, pumpman.global.clone());

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

/// Disabled the current thread
pub async fn disable() -> Result<()> {
    loop {
        tokio::time::sleep(Duration::from_secs(86400)).await
    }
}
