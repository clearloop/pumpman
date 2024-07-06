//! Solana programs

use crate::{
    config::Cluster,
    context::{Redis, Telegram},
    model::Coin,
};
use anyhow::Result;
use async_lock::Mutex;
use futures_util::StreamExt;
use mpl_token_metadata::accounts::Metadata;
use redis::Commands;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use solana_client::{
    nonblocking::{pubsub_client::PubsubClient, rpc_client::RpcClient},
    rpc_config::{RpcTransactionConfig, RpcTransactionLogsConfig, RpcTransactionLogsFilter},
};
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Signature};
use solana_transaction_status::UiTransactionEncoding;
use std::{str::FromStr, sync::Arc};

use self::dex_screender::DexScreenerResult;

const DEX_SCREENER: &str = "https://api.dexscreener.com/latest/dex/tokens/";

/// Replika client
#[derive(Clone)]
pub struct Client(Arc<Mutex<RpcClient>>);

impl Client {
    /// Create new solana client
    pub fn new(cluster: &Cluster) -> Result<Self> {
        Ok(Self(Arc::new(Mutex::new(RpcClient::new(
            cluster.http.to_string(),
        )))))
    }

    /// Get token metadata from address
    pub async fn coin(&self, mint: &str) -> Result<Coin> {
        let acc = Pubkey::find_program_address(
            &[
                b"metadata",
                &mpl_token_metadata::ID.to_bytes(),
                &Pubkey::from_str(mint)?.to_bytes(),
            ],
            &mpl_token_metadata::ID.to_bytes().into(),
        );

        let data = self.0.lock().await.get_account_data(&acc.0).await?;
        let mplmeta = Metadata::from_bytes(&data)?;

        let meta: Value = reqwest::get(&mplmeta.uri).await?.json().await?;
        let mut coin: Coin = mplmeta.into();
        coin.append(meta);

        // get url if exist
        if let Ok(dex) = reqwest::get(format!("{DEX_SCREENER}/{}", coin.mint))
            .await?
            .json::<DexScreenerResult>()
            .await
        {
            if let Some(p) = dex.pairs.get(0) {
                coin.dex = Some(p.url.to_string());
            }
        }

        Ok(coin)
    }

    /// Get siganture and print UI transaction
    pub async fn sig(&self, sig: &str) -> Result<()> {
        let sig = Signature::from_str(sig)?;
        let r = self
            .0
            .lock()
            .await
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

mod dex_screender {
    use serde::Deserialize;

    #[derive(Clone, Deserialize)]
    pub struct DexScreenerResult {
        pub pairs: Vec<DexScreenerPair>,
    }

    #[derive(Clone, Deserialize)]
    pub struct DexScreenerPair {
        pub url: String,
    }
}
