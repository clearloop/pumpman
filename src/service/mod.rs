//! Replika services

use crate::{context::Context, Config};
use anyhow::Result;
use pump::{PumpEvent, PumpSub};
use std::time::Duration;
use takeover::Takeover;
use tokio::{signal, sync::mpsc};

mod pump;
mod takeover;

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
        let mut processor = Takeover::new(&config, context.clone(), rx);

        let r = tokio::select! {
            _ = signal::ctrl_c() => break,
            r = pumpsub.start() => r,
            r = processor.start() => r
        };

        if let Err(e) = r {
            tracing::error!("{e}");
            tokio::time::sleep(Duration::from_secs(20)).await;
        }
    }

    Ok(())
}
