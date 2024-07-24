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
use diesel::sql_types::Decimal;
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
                Event::Pump(PumpEvent::DevSoldout(mints)) => self.pump_soldout_handle(mints).await,
            } {
                tracing::warn!("Failed to process event, error: {e}");
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }

        Ok(())
    }

    /// split mints into windows
    async fn pump_soldout_handle(&self, mints: Vec<String>) -> Result<()> {
        let len = mints.len();
        let mut ptr = 0;

        while ptr < len {
            let to = (ptr + 5).min(len);
            futures::future::join_all(
                mints[ptr..to]
                    .iter()
                    .map(|mint| self.pump_soldout(mint.to_string()))
                    .collect::<Vec<_>>(),
            )
            .await
            .into_iter()
            .collect::<Result<Vec<_>>>()?;

            tokio::time::sleep(Duration::from_millis(500)).await;
            ptr = to;
        }

        Ok(())
    }

    /// Handle pump soldout
    async fn pump_soldout(&self, mint: String) -> Result<()> {
        let redis = &mut self.context.redis()?;
        let client = self.context.client.clone();

        let key = TaskCache::DevSoldOut(&mint);
        if redis.exists(&key)? {
            return Ok(());
        }

        // filter out mc less than $8k
        let coin = client.coin(&mint, false, redis).await?;
        if let Some(mc) = &coin.usd_market_cap {
            if *mc < BigDecimal::from(10000) {
                return Ok(());
            }
        }

        // check if dev is soldout
        let (_, soldout) = client
            .soldout(&coin.mint, &coin.creator, false, redis)
            .await?;

        if !soldout {
            return Ok(());
        }

        // check holders amount
        let holders = client
            .top_holders(&mint, false, redis)
            .await?
            .skip_bc(&coin.associated_bonding_curve);

        if holders.len() < 15 {
            return Ok(());
        }

        let pairs = client.pairs(&mint, false, redis).await?;
        self.context.update_coin(coin.clone()).await?;
        Alert::new(AlertTitle::DevSoldOut, coin, soldout)
            .pairs(pairs)
            .holders(holders)
            .alert(&self.reporter, &self.channel)
            .await?;
        redis.set(key, true)?;

        Ok(())
    }
}
