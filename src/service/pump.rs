//! Solana subscriber
use crate::{
    api::{HttpClient, SolRpcApi},
    context::{Conn, Context},
    schema::coins,
    sol::{self, pump::events},
};
use anyhow::Result;
use bigdecimal::BigDecimal;
use diesel::prelude::*;
use futures_util::StreamExt;
use redis::{Commands, Connection};
use solana_client::{
    nonblocking::pubsub_client::PubsubClient,
    rpc_config::{RpcTransactionLogsConfig, RpcTransactionLogsFilter},
};
use solana_sdk::commitment_config::CommitmentConfig;
use std::{str::FromStr, sync::Arc};

/// Solana subscriber
pub struct PumpSub {
    context: Context,
    pubsub: Arc<PubsubClient>,
}

impl PumpSub {
    /// pumpfun subscriber
    pub async fn start(&self) -> Result<()> {
        let mut sub = self
            .pubsub
            .logs_subscribe(
                RpcTransactionLogsFilter::Mentions(vec![sol::pump::ID.to_string()]),
                RpcTransactionLogsConfig {
                    commitment: Some(CommitmentConfig::finalized()),
                },
            )
            .await?;

        // subscribe pumpfun events
        let postgres = &mut self.context.postgres()?;
        let redis = &mut self.context.redis()?;
        while let Some(resp) = sub.0.next().await {
            if resp.value.err.is_some() {
                continue;
            }

            if let Some(event) = sol::parse::<events::TradeEvent>(&resp.value.logs) {
                self.handle_trade(event, postgres, redis).await?;
                continue;
            }

            if let Some(event) = sol::parse::<sol::pump::events::CompleteEvent>(&resp.value.logs) {
                self.handle_complete(event, postgres, redis).await?;
            }
        }

        Ok(())
    }

    /// Handle trade event
    async fn handle_trade(
        &self,
        event: events::TradeEvent,
        postgres: &mut Conn,
        redis: &mut Connection,
    ) -> Result<()> {
        let mint = event.mint.to_string();
        self.ensure_coin(&mint, postgres, redis)?;

        // check if the user is dev
        let coin = self.context.client.coin(&mint, false, redis).await?;
        if event.user.to_string() != coin.creator {
            return Ok(());
        }

        // check if dev soldout
        let holders = self.context.client.top_holders(&mint, true, redis).await?;
        if !holders.iter().any(|acc| {
            if acc.address != coin.creator {
                return false;
            }

            BigDecimal::from_str(&acc.amount.ui_amount_string)
                .map(|b| b < BigDecimal::from(100))
                .unwrap_or(false)
        }) {
            return Ok(());
        }

        // subscribe to channel on soldout
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
                .execute(postgres)?;
            redis.set_nx(mint, true)?;
        }

        Ok(())
    }
}
