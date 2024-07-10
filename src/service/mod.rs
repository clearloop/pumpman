//! Replika services
#![allow(unused)]

use crate::{context::Context, telegram::TakeoverBot, Config};
use anyhow::Result;
use processor::Processor;
use pump::{PumpEvent, PumpSub};
use std::sync::Arc;
use teloxide::Bot;
use tokio::sync::mpsc;

mod processor;
mod pump;

/// Replika events
#[derive(Debug)]
pub enum Event {
    Pump(PumpEvent),
}

/// Start all service
pub async fn start(config: &Config, context: Arc<Context>) -> Result<()> {
    // let bot = TakeoverBot::new(
    //     &config.telegram.takeover_bot,
    //     context.clone(),
    //     format!("{}/15", config.redis),
    // );

    let (tx, rx) = mpsc::channel::<Event>(50);
    let mut pumpsub = PumpSub::new(&config, context.clone(), tx).await?;
    let mut processor = Processor::new(
        config.telegram.takeover_alerts.clone(),
        Bot::new(config.telegram.takeover_alerts_bot.clone()),
        context.clone(),
        rx,
    );

    tokio::select! {
        // r = bot.start() => r,
        r = pumpsub.start() => r,
        r = processor.start() => r
    }
}
