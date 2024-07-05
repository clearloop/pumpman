//! Solana programs

use crate::{
    config::Cluster,
    context::{Redis, Telegram},
    sol,
};
use anyhow::Result;
use futures_util::StreamExt;
use mpl_token_metadata::accounts::Metadata;
use redis::Commands;
use solana_client::{
    nonblocking::{pubsub_client::PubsubClient, rpc_client::RpcClient},
    rpc_config::{RpcTransactionConfig, RpcTransactionLogsConfig, RpcTransactionLogsFilter},
};
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Signature};
use solana_transaction_status::UiTransactionEncoding;
use std::str::FromStr;

/// Replika client
pub struct Client {
    /// Solana RPC client
    rpc: RpcClient,
    /// Pubsub client for latest events
    pubsub: PubsubClient,
    /// Redis client
    redis: Redis,
    /// Telegram handler
    telegram: Telegram,
}

impl Client {
    /// Create new solana client
    pub async fn new(cluster: &Cluster, redis: Redis, telegram: Telegram) -> Result<Self> {
        Ok(Self {
            rpc: RpcClient::new(cluster.http.to_string()),
            pubsub: PubsubClient::new(&cluster.ws.to_string()).await?,
            redis,
            telegram,
        })
    }

    /// Subscribe pump events
    pub async fn subscribe(&self) -> Result<()> {
        let mut sub = self
            .pubsub
            .logs_subscribe(
                RpcTransactionLogsFilter::Mentions(vec![sol::pump::ID.to_string()]),
                RpcTransactionLogsConfig {
                    commitment: Some(CommitmentConfig::finalized()),
                },
            )
            .await?;

        let mut redis = self.redis.con().await?;
        tracing::info!("subscribe: {}", sol::pump::ID.to_string());
        while let Some(resp) = sub.0.next().await {
            if resp.value.err.is_some() {
                continue;
            }

            if let Some(event) = sol::parse::<sol::pump::events::TradeEvent>(resp.value.logs) {
                let mint = event.mint.to_string();
                let exists: bool = redis.exists(&mint)?;
                if !exists {
                    let _: String = redis.set(&mint, self.metadata(&mint).await?.symbol)?;
                }

                self.telegram.trade(event).await?;
            }
        }

        Ok(())
    }

    /// Get token metadata from address
    pub async fn metadata(&self, mint: &str) -> Result<Metadata> {
        let acc = Pubkey::find_program_address(
            &[
                b"metadata",
                &mpl_token_metadata::ID.to_bytes(),
                &Pubkey::from_str(mint)?.to_bytes(),
            ],
            &mpl_token_metadata::ID.to_bytes().into(),
        );

        let data = self.rpc.get_account_data(&acc.0).await?;
        Metadata::from_bytes(&data).map_err(Into::into)
    }

    /// Get siganture and print UI transaction
    pub async fn sig(&self, sig: &str) -> Result<()> {
        let sig = Signature::from_str(sig)?;
        let r = self
            .rpc
            .get_transaction_with_config(
                &sig,
                RpcTransactionConfig {
                    encoding: Some(UiTransactionEncoding::Json),
                    commitment: None,
                    max_supported_transaction_version: Some(0),
                },
            )
            .await?;

        println!("{r:#?}");
        Ok(())
    }
}
