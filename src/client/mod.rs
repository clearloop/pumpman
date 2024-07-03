//! Solana programs

use anyhow::Result;
use futures_util::StreamExt;
use mpl_token_metadata::accounts::Metadata;
use solana_client::{
    nonblocking::{pubsub_client::PubsubClient, rpc_client::RpcClient},
    rpc_config::{RpcTransactionConfig, RpcTransactionLogsConfig, RpcTransactionLogsFilter},
};
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Signature};
use solana_transaction_status::UiTransactionEncoding;
use std::str::FromStr;

mod sol;

/// Replika client
pub struct Client {
    /// Solana RPC client
    rpc: RpcClient,
    /// Pubsub client for latest events
    pubsub: PubsubClient,
}

impl Client {
    pub async fn new(cluster: &str) -> Result<Self> {
        Ok(Self {
            rpc: RpcClient::new(cluster.replace("ws", "https").into()),
            pubsub: PubsubClient::new(cluster).await?,
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

        tracing::info!("subscribe: {}", sol::pump::ID.to_string());
        while let Some(resp) = sub.0.next().await {
            if resp.value.err.is_some() {
                continue;
            }

            if let Some(event) = sol::parse::<sol::pump::events::TradeEvent>(resp.value.logs) {
                println!("{:#?}", event);
            }
        }

        Ok(())
    }

    /// Get token metadata from address
    pub async fn metadata(&self, addr: Pubkey) -> Result<Metadata> {
        let acc = Pubkey::find_program_address(
            &[
                b"metadata",
                &mpl_token_metadata::ID.to_bytes(),
                &addr.to_bytes(),
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
