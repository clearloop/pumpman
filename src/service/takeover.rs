//! Task processor

use crate::{
    api::{HttpClient, SolRpcApi},
    config,
    context::{Context, TaskCache},
    model::{Alert, AlertTitle},
    service::{Event, PumpEvent},
    Config,
};
use anyhow::Result;
use bigdecimal::BigDecimal;
use redis::Commands;
use std::time::Duration;
use teloxide::Bot;
use tokio::sync::mpsc::Receiver;

/// Pump events handler
pub struct Takeover {
    config: config::Takeover,
    context: Context,
    rx: Receiver<Event>,
}

impl Takeover {
    /// Create new takeover service
    pub fn new(config: &Config, context: Context, rx: Receiver<Event>) -> Self {
        Self {
            config: config.takeover.clone(),
            context,
            rx,
        }
    }

    /// Start the reporter service
    pub async fn start(&mut self) -> Result<()> {
        let Some(reporter) = self.config.bot.take() else {
            tracing::warn!("Takeover alerts is disabled since the bot token is not set.");
            loop {
                tokio::time::sleep(Duration::from_secs(86400)).await
            }
        };

        tracing::trace!("Starting takeover alert ...");
        let bot = Bot::new(reporter);
        while let Some(event) = self.rx.recv().await {
            tracing::trace!("Received event: {event:?}");
            if let Err(e) = match event {
                Event::Pump(PumpEvent::DevSoldout(mints)) => self.soldout(&bot, mints).await,
            } {
                tracing::warn!("Failed to process event, error: {e}");
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }

        Ok(())
    }

    /// split mints into windows
    async fn soldout(&self, bot: &Bot, mints: Vec<String>) -> Result<()> {
        let len = mints.len();
        let mut ptr = 0;

        while ptr < len {
            let to = (ptr + self.config.batch).min(len);
            futures::future::join_all(
                mints[ptr..to]
                    .iter()
                    .map(|mint| self.alert(bot, mint.to_string()))
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
    async fn alert(&self, bot: &Bot, mint: String) -> Result<()> {
        let redis = &mut self.context.redis()?;
        let client = self.context.client.clone();

        let key = TaskCache::DevSoldOut(&mint);
        if redis.exists(&key)? {
            return Ok(());
        }

        // filter out mc less than $8k
        let coin = client.coin(&mint, false, redis).await?;
        if let Some(mc) = &coin.usd_market_cap {
            if *mc < BigDecimal::from(self.config.marketcap) {
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

        if holders.len() < self.config.holders {
            return Ok(());
        }

        let pairs = client.pairs(&mint, false, redis).await?;
        self.context.update_coin(coin.clone()).await?;
        Alert::new(AlertTitle::DevSoldOut, coin, soldout)
            .pairs(pairs)
            .holders(holders)
            .alert(bot, &self.config.subscription)
            .await?;
        redis.set(key, true)?;

        Ok(())
    }
}
