//! Solana programs

use std::str::FromStr;

use crate::context::Db;
use anyhow::Result;
use futures_util::StreamExt;
use solana_client::{
    nonblocking::{pubsub_client::PubsubClient, rpc_client::RpcClient},
    rpc_config::{RpcTransactionConfig, RpcTransactionLogsConfig, RpcTransactionLogsFilter},
};
use solana_sdk::{commitment_config::CommitmentConfig, signature::Signature};
use solana_transaction_status::UiTransactionEncoding;

mod sol;

pub struct Client {
    // /// Solana RPC client
    // ///
    /// 1. fetch and sync previous block
    rpc: RpcClient,
    /// Pubsub client for latest events
    pubsub: PubsubClient,
    /// Database instance
    db: Db,
}

impl Client {
    pub async fn new(cluster: &str) -> Result<Self> {
        Ok(Self {
            rpc: RpcClient::new(cluster.replace("ws", "https").into()),
            pubsub: PubsubClient::new(cluster).await?,
            db: Db,
        })
    }

    /// Subscribe pump
    pub async fn subscribe(&self) -> Result<()> {
        // let mut sub = self
        //     .pubsub
        //     .logs_subscribe(
        //         RpcTransactionLogsFilter::Mentions(vec![sol::pump::ID.to_string()]),
        //         RpcTransactionLogsConfig {
        //             commitment: Some(CommitmentConfig::finalized()),
        //         },
        //     )
        //     .await?;
        //
        // while let Some(resp) = sub.0.next().await {
        //     if resp.value.err.is_none() {
        //         continue;
        //     }
        //
        //     println!("{:#?}", resp.value.logs);
        // }
        // let acc = Pubkey::find_program_address(
        //     &[
        //         b"metadata",
        //         &mpl_token_metadata::ID.to_bytes(),
        //         &Pubkey::from_str("3xYsZSKrKwYM2mh4JSXjhqvyqDS3U5jQnLv93QKKpump")?.to_bytes(),
        //     ],
        //     &mpl_token_metadata::ID.to_bytes().into(),
        // );
        //
        // // const accInfo = await provider.connection.getAccountInfo(metadataPDA);
        // // const metadata = Metadata.deserialize(accInfo.data, 0);
        //
        // let data = self.rpc.get_account_data(&acc.0).await?;
        // let meta = Metadata::from_bytes(&data);
        // println!("{meta:#?}");

        let sig = Signature::from_str("3XQY14TCuN8PNNKWZJguQPfTciGQc9pYqEr1jdCfP4omEh1utPGhz1uxHQvxLfTmVbNCJkukyt3pzPYcd7oBEGX")?;
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

    /// Get siganture and print UI transaction
    pub async fn get_sig(&self, sig: &str) -> Result<()> {
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
