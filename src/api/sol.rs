use anyhow::Result;
use async_trait::async_trait;
use bigdecimal::BigDecimal;
use mpl_token_metadata::accounts::Metadata;
use redis::{Commands, Connection};
use serde::{Deserialize, Serialize};
use solana_client::{
    nonblocking::rpc_client::RpcClient, rpc_config::RpcTransactionConfig,
    rpc_response::RpcTokenAccountBalance,
};
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Signature};
use solana_transaction_status::{EncodedConfirmedTransactionWithStatusMeta, UiTransactionEncoding};
use std::{ops::Deref, str::FromStr, sync::Arc};

/// Solana Rpc sugar
#[async_trait]
pub trait SolRpcApi {
    /// Solana rpc client
    fn rpc(&self) -> &Arc<RpcClient>;

    async fn top_holders(
        &self,
        mint: &str,
        update: bool,
        redis: &mut Connection,
    ) -> Result<Holders> {
        let key = format!("holders::{mint}");
        let holders = if update || !redis.exists(&key)? {
            let holders: Holders = self
                .rpc()
                .get_token_largest_accounts_with_commitment(
                    &Pubkey::from_str(&mint)?,
                    CommitmentConfig::finalized(),
                )
                .await?
                .value
                .into();

            redis.set(key, bitcode::serialize(&holders)?)?;
            holders
        } else {
            let holders: Vec<u8> = redis.get(&key)?;
            bitcode::deserialize(&holders)?
        };

        Ok(holders.into())
    }

    #[allow(unused)]
    /// get mpl token metadata
    async fn mpl_token_metadata(&self, mint: &str) -> Result<Metadata> {
        let acc = Pubkey::find_program_address(
            &[
                b"metadata",
                &mpl_token_metadata::ID.to_bytes(),
                &Pubkey::from_str(mint)?.to_bytes(),
            ],
            &mpl_token_metadata::ID.to_bytes().into(),
        );

        let data = self.rpc().get_account_data(&acc.0).await?;
        Metadata::from_bytes(&data).map_err(Into::into)
    }

    /// Get encoded solana transaction with meta
    async fn tx(&self, sig: &str) -> Result<EncodedConfirmedTransactionWithStatusMeta> {
        let sig = Signature::from_str(sig)?;
        self.rpc()
            .get_transaction_with_config(
                &sig,
                RpcTransactionConfig {
                    encoding: Some(UiTransactionEncoding::Json),
                    commitment: None,
                    max_supported_transaction_version: Some(0),
                },
            )
            .await
            .map_err(Into::into)
    }
}

const TOTAL_SUPPLY: u64 = 1_000_000_000;

/// Token holders
#[derive(Default, Serialize, Deserialize)]
pub struct Holders(Vec<(String, String)>);

impl Holders {
    pub fn top10percent(&self) -> Result<BigDecimal> {
        let total = BigDecimal::from(TOTAL_SUPPLY);
        let present = if self.len() > 10 {
            &self.0[..10]
        } else {
            &self.0
        }
        .iter()
        .map(|acc| BigDecimal::from_str(&acc.1).map_err(Into::into))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .reduce(|cur, acc| acc + cur)
        .unwrap_or_default();

        Ok((present / total * 100u32).round(2))
    }
}

impl Deref for Holders {
    type Target = Vec<(String, String)>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Vec<RpcTokenAccountBalance>> for Holders {
    fn from(v: Vec<RpcTokenAccountBalance>) -> Self {
        Self(
            v.into_iter()
                .map(|b| (b.address, b.amount.ui_amount_string))
                .collect(),
        )
    }
}
