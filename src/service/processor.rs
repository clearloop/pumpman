//! Task processor

use crate::{
    api::{HttpClient, SolRpcApi},
    context::{Context, TaskCache},
    model::{Alert, AlertTitle, Coin},
    schema::coins,
    service::{Event, PumpEvent},
    utils::TTMINS,
};
use anyhow::Result;
use bigdecimal::BigDecimal;
use redis::Commands;
use std::{str::FromStr, sync::Arc, time::Duration};
use teloxide::{requests::Requester, types::Recipient, Bot};
use tokio::{sync::mpsc::Receiver, time::sleep};

/// Pump events handler
pub struct Processor {
    channel: String,
    reporter: Bot,
    context: Context,
    rx: Receiver<Event>,
}

impl Processor {
    pub fn new(channel: String, reporter: Bot, context: Context, rx: Receiver<Event>) -> Self {
        Self {
            channel,
            reporter,
            context,
            rx,
        }
    }

    /// Start the reporter service
    pub async fn start(&mut self) -> Result<()> {
        tracing::trace!("Starting processor ...");
        while let Some(event) = self.rx.recv().await {
            tracing::trace!("Received event: {event:?}");
            if let Err(e) = match event {
                Event::Pump(PumpEvent::DevSoldout(mint)) => self.pump_soldout(mint).await,
            } {
                tracing::warn!("{e}");
                sleep(Duration::from_secs(10)).await;
            }

            sleep(Duration::from_secs(1)).await;
        }

        Ok(())
    }

    /// Handle pump soldout
    async fn pump_soldout(&self, mints: Vec<String>) -> Result<()> {
        let redis = &mut self.context.redis()?;
        let client = self.context.client.clone();
        for mint in mints {
            let key = TaskCache::DevSoldOut(&mint);
            if redis.exists(&key)? {
                continue;
            }

            // check if dev is soldout
            let coin = client.coin(&mint, false, redis).await?;
            if !client
                .soldout(&coin.mint, &coin.creator, false, redis)
                .await?
            {
                return Ok(());
            }

            // check holders amount
            let holders = client
                .top_holders(&mint, true, redis)
                .await?
                .skip_bc(&coin.associated_bonding_curve);

            if holders.len() < 15 {
                continue;
            }

            let coin = client.coin(&mint, true, redis).await?;
            let pairs = client.pairs(&mint, false, redis).await?;

            self.context.update_coin(coin.clone())?;
            Alert::new(AlertTitle::DevSoldOut, coin, true)
                .pairs(pairs)
                .holders(holders)
                .alert(&self.reporter, &self.channel)
                .await?;
            redis.set(key, true)?;
        }

        Ok(())
    }
}
