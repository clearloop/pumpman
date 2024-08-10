//! Solana subscriber
use crate::{
    context::{Context, TaskCache},
    service::Event,
    sol::{self, pump::events},
    Config,
};
use anyhow::Result;
use futures_util::StreamExt;
use redis::{Commands, Connection};
use solana_client::{
    nonblocking::pubsub_client::PubsubClient,
    rpc_config::{RpcTransactionLogsConfig, RpcTransactionLogsFilter},
};
use solana_sdk::commitment_config::CommitmentConfig;
use std::{collections::HashSet, rc::Rc};
use tokio::sync::mpsc::Sender;

/// Solana subscriber
pub struct PumpSub {
    /// batch coins for soldout
    coins: usize,
    /// Service context
    context: Context,
    /// Solana pubsub client
    pubsub: Rc<PubsubClient>,
    /// Queue for checking soldout
    soldout: HashSet<String>,
    /// Pump event sender
    tx: Sender<Event>,
}

impl PumpSub {
    /// Create new pubsub
    pub async fn new(config: &Config, context: Context, tx: Sender<Event>) -> Result<Self> {
        tracing::trace!("Starting pubsub service ...");
        Ok(Self {
            coins: config.takeover.coins,
            context,
            pubsub: Rc::new(PubsubClient::new(config.cluster.ws.as_ref()).await?),
            soldout: Default::default(),
            tx,
        })
    }

    /// pumpfun subscriber
    pub async fn start(&mut self) -> Result<()> {
        let pubsub = self.pubsub.clone();
        let mut sub = pubsub
            .logs_subscribe(
                RpcTransactionLogsFilter::Mentions(vec![sol::pump::ID.to_string()]),
                RpcTransactionLogsConfig {
                    commitment: Some(CommitmentConfig::finalized()),
                },
            )
            .await?;

        let redis = &mut self.context.redis()?;
        while let Some(resp) = sub.0.next().await {
            if resp.value.err.is_some() {
                continue;
            }

            if let Some(event) = sol::parse::<events::TradeEvent>(&resp.value.logs) {
                self.takeover(event, redis).await?;
            }
        }

        Ok(())
    }

    /// Handle trade event for takeover alerts
    async fn takeover(&mut self, event: events::TradeEvent, redis: &mut Connection) -> Result<()> {
        let mint = event.mint.to_string();

        if !redis.exists(TaskCache::DevSoldOut(&mint))? {
            self.soldout.insert(mint.clone());
        }

        if self.soldout.len() > self.coins {
            self.tx
                .send(PumpEvent::DevSoldout(self.soldout.drain().collect()).into())
                .await?;
        }

        Ok(())
    }
}

/// Pumpfun events
#[derive(Debug)]
pub enum PumpEvent {
    DevSoldout(Vec<String>),
}

impl From<PumpEvent> for Event {
    fn from(pe: PumpEvent) -> Event {
        Event::Pump(pe)
    }
}
