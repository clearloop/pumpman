//! Task processor

use crate::{
    config,
    context::{Cache, Context},
    sol::{
        self,
        pump::{self, events},
    },
    telegram::takeover,
    Config,
};
use anyhow::Result;
use futures_util::StreamExt;
use redis::Commands;
use solana_client::{
    nonblocking::pubsub_client::PubsubClient,
    rpc_config::{RpcTransactionLogsConfig, RpcTransactionLogsFilter},
};
use solana_sdk::commitment_config::CommitmentConfig;
use std::{collections::HashSet, time::Duration};
use teloxide::Bot;
use tokio::signal;
use tokio::sync::{
    mpsc,
    mpsc::{Receiver, Sender},
};

/// Pump events handler
pub struct Takeover {
    config: config::Takeover,
    context: Context,
}

impl Takeover {
    /// Create new takeover service
    pub fn new(config: &config::Takeover, context: Context) -> Self {
        Self {
            config: config.clone(),
            context,
        }
    }

    /// Start the service
    pub async fn start(mut config: Config, context: Context) -> Result<()> {
        let Some(mut takeover) = config.takeover.take() else {
            tracing::debug!("No config found for takeover service");
            return Ok(());
        };

        let Some(bot) = takeover.bot.take() else {
            tracing::debug!("No bot token found for takeover service");
            return Ok(());
        };

        let service = Self::new(&takeover, context.clone());
        loop {
            let (tx, rx) = mpsc::channel::<Vec<String>>(50);

            let r = tokio::select! {
                _ = signal::ctrl_c() => break Ok(()),
                r = service.sub_pump(config.cluster.ws.as_ref(), tx) => r,
                r = service.sub_alerts(&bot, rx) => r,
                r = takeover::start(
                      &bot,
                      context.clone(),
                      format!("{}/15", config.database.redis)
                ), if takeover.registry => r,
            };

            if let Err(e) = r {
                tracing::error!("{e}");
                tokio::time::sleep(Duration::from_secs(20)).await;
            }
        }
    }

    /// Start the reporter service
    pub async fn sub_pump(&self, ws: &str, tx: Sender<Vec<String>>) -> Result<()> {
        let pubsub = PubsubClient::new(ws).await?;
        let mut sub = pubsub
            .logs_subscribe(
                RpcTransactionLogsFilter::Mentions(vec![pump::ID.to_string()]),
                RpcTransactionLogsConfig {
                    commitment: Some(CommitmentConfig::finalized()),
                },
            )
            .await?;

        let redis = &mut self.context.redis()?;
        let mut soldout = HashSet::<String>::new();
        while let Some(resp) = sub.0.next().await {
            if resp.value.err.is_some() {
                continue;
            }

            if let Some(event) = sol::parse::<events::TradeEvent>(&resp.value.logs) {
                let mint = event.mint.to_string();

                if !redis.exists(Cache::DevSoldOut(&mint))? {
                    soldout.insert(mint.clone());
                }

                if soldout.len() > self.config.coins {
                    tx.send(soldout.drain().collect()).await?;
                }
            }
        }

        Ok(())
    }

    /// Start the reporter service
    pub async fn sub_alerts(&self, bot: &str, mut rx: Receiver<Vec<String>>) -> Result<()> {
        let bot = Bot::new(bot);
        tracing::trace!("Starting takeover service ...");
        while let Some(mints) = rx.recv().await {
            tracing::trace!("Received mints: {mints:?}");
            if let Err(e) = self.soldout(&bot, mints).await {
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
                    .map(|mint| takeover::alert(&self.config, &self.context, bot, mint.to_string()))
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
}
