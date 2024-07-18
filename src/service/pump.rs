//! Solana subscriber
use crate::{
    api::{HttpClient, SolRpcApi},
    context::{Conn, Context, TaskCache},
    schema::coins,
    service::Event,
    sol::{self, pump::events},
    Config,
};
use anyhow::Result;
use bigdecimal::BigDecimal;
use core::time;
use diesel::prelude::*;
use futures_util::StreamExt;
use redis::{Commands, Connection};
use solana_client::{
    nonblocking::pubsub_client::PubsubClient,
    rpc_config::{RpcTransactionLogsConfig, RpcTransactionLogsFilter},
};
use solana_sdk::commitment_config::CommitmentConfig;
use std::{
    collections::HashSet,
    rc::Rc,
    str::FromStr,
    sync::Arc,
    time::{Duration, SystemTime},
};
use tokio::sync::mpsc::Sender;

/// Solana subscriber
pub struct PumpSub {
    /// Service context
    context: Context,
    /// Solana pubsub client
    pubsub: Rc<PubsubClient>,
    /// Queue for checking soldout
    soldout: HashSet<String>,
    /// Queue for checking holder changes
    holders: HashSet<(String, u8)>,
    /// Pump event sender
    tx: Sender<Event>,
}

impl PumpSub {
    /// Create new pubsub
    pub async fn new(config: &Config, context: Context, tx: Sender<Event>) -> Result<Self> {
        tracing::trace!("Starting pubsub service ...");
        Ok(Self {
            context,
            pubsub: Rc::new(PubsubClient::new(config.cluster.ws.as_ref()).await?),
            soldout: Default::default(),
            holders: Default::default(),
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

        let mut last = SystemTime::now();
        let postgres = &mut self.context.postgres()?;
        let redis = &mut self.context.redis()?;
        while let Some(resp) = sub.0.next().await {
            if resp.value.err.is_some() {
                continue;
            }

            if let Some(event) = sol::parse::<events::TradeEvent>(&resp.value.logs) {
                self.handle_trade(event, postgres, redis).await?;
            }

            // TODO: handle events of new created tokens
            //
            // if let Some(event) = sol::parse::<sol::pump::events::CompleteEvent>(&resp.value.logs) {
            //     tracing::trace!("{event:?}");
            //     self.handle_complete(event, postgres, redis).await?;
            // }

            // Send changes to receiver
            if !self.soldout.is_empty() {
                self.tx
                    .send(PumpEvent::DevSoldout(self.soldout.drain().collect()).into())
                    .await?;
            }

            // NOTE: pause the holders notification
            //
            // if !self.holders.is_empty() {
            //     self.tx
            //         .send(PumpEvent::HoldersChanged(self.holders.drain().collect()).into())
            //         .await?;
            // }
        }

        Ok(())
    }

    /// Handle trade event
    ///
    /// - subscribe dev soldout
    /// - subscribe change of holders
    async fn handle_trade(
        &mut self,
        event: events::TradeEvent,
        postgres: &mut Conn,
        redis: &mut Connection,
    ) -> Result<()> {
        let mint = event.mint.to_string();
        self.ensure_coin(&mint, postgres, redis)?;

        if !redis.exists(TaskCache::DevSoldOut(&mint))? {
            self.soldout.insert(mint.clone());
        }

        if redis.exists(TaskCache::Top10Holder {
            mint: &mint,
            percent: 10,
        })? {
            return Ok(());
        }

        self.holders.insert((mint.clone(), 10));
        Ok(())
    }

    async fn handle_complete(
        &self,
        _event: events::CompleteEvent,
        _postgres: &mut Conn,
        _redis: &mut Connection,
    ) -> Result<()> {
        Ok(())
    }

    /// Ensure coin has been recorded in database
    fn ensure_coin(&self, mint: &str, postgres: &mut Conn, redis: &mut Connection) -> Result<()> {
        if !redis.exists(mint)? {
            diesel::insert_into(coins::table)
                .values(coins::mint.eq(mint.to_string()))
                .on_conflict(coins::mint)
                .do_nothing()
                .execute(postgres)?;
            redis.set_nx(mint, true)?;
        }

        Ok(())
    }
}

/// Pumpfun events
#[derive(Debug)]
pub enum PumpEvent {
    DevSoldout(Vec<String>),
    HoldersChanged(Vec<(String, u8)>),
}

impl From<PumpEvent> for Event {
    fn from(pe: PumpEvent) -> Event {
        Event::Pump(pe)
    }
}
