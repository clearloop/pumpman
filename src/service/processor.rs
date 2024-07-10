//! Task processor

use std::{str::FromStr, sync::Arc};

use crate::{
    api::{HttpClient, SolRpcApi},
    context::Context,
    model::Alert,
    service::{Event, PumpEvent},
};
use anyhow::Result;
use bigdecimal::BigDecimal;
use teloxide::Bot;
use tokio::sync::mpsc::Receiver;

/// Pump events handler
pub struct Processor {
    reporter: Bot,
    context: Arc<Context>,
    rx: Receiver<Event>,
}

impl Processor {
    pub fn new(reporter: Bot, context: Arc<Context>, rx: Receiver<Event>) -> Self {
        Self {
            reporter,
            context,
            rx,
        }
    }

    /// Start the reporter service
    pub async fn start(&mut self) -> Result<()> {
        while let Some(event) = self.rx.recv().await {
            match event {
                Event::Pump(PumpEvent::DevSoldout(mint)) => self.pump_soldout(mint).await?,
                Event::Pump(PumpEvent::HoldersChanged(holders)) => {
                    self.pump_holders(holders).await?
                }
            }
        }

        Ok(())
    }

    /// Handle pump soldout
    async fn pump_soldout(&self, mints: Vec<String>) -> Result<()> {
        let redis = &mut self.context.redis()?;
        for mint in mints {
            // check if the user is dev
            let coin = self.context.client.coin(&mint, false, redis).await?;

            // check if dev soldout
            let holders = self.context.client.top_holders(&mint, true, redis).await?;
            if !holders.iter().any(|acc| {
                if acc.address != coin.creator {
                    return false;
                }

                // if dev is soldout
                BigDecimal::from_str(&acc.amount.ui_amount_string)
                    .map(|b| b < BigDecimal::from(100))
                    .unwrap_or(true)
            }) {
                return Ok(());
            }

            // subscribe to channel about soldout
            let pairs = self
                .context
                .client
                .tokens(&mint, true, redis)
                .await?
                .pairs
                .unwrap_or_default();

            let _alert = Alert::new(coin).pairs(pairs).holders(holders);
        }

        Ok(())
    }

    /// Handle holders change
    async fn pump_holders(&self, _holders: Vec<(String, u8)>) -> Result<()> {
        Ok(())
    }
}
