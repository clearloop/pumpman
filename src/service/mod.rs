//! Replika services
#![allow(unused)]

use crate::{context::Context, telegram::takeover, Config};
use anyhow::Result;
use processor::Processor;
use pump::{PumpEvent, PumpSub};
use std::sync::Arc;
use teloxide::Bot;
use tokio::{signal, sync::mpsc};

mod processor;
mod pump;

/// Replika events
#[derive(Debug)]
pub enum Event {
    Pump(PumpEvent),
}

/// Start all service
pub async fn start(config: &Config, context: Context) -> Result<()> {
    loop {
        let (tx, rx) = mpsc::channel::<Event>(50);
        let mut pumpsub = PumpSub::new(config, context.clone(), tx).await?;
        let mut processor = Processor::new(
            config.telegram.takeover_alerts.clone(),
            Bot::new(config.telegram.takeover_alerts_bot.clone()),
            context.clone(),
            rx,
        );

        let takeover_future = takeover::start(
            &config.telegram.takeover_bot,
            context.clone(),
            format!("{}/15", config.redis),
        );

        let r = tokio::select! {
            r = signal::ctrl_c() => break,
            r = takeover_future => r,
            r = pumpsub.start() => r,
            r = processor.start() => r
        };

        if let Err(e) = r {
            tracing::error!("{e}");
        }
    }

    Ok(())
}
