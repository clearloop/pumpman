use crate::{
    sol::pump::TOTAL_SUPPLY,
    utils::{sol, FIVE_MINS},
};
use anyhow::Result;
use bigdecimal::BigDecimal;
use mpl_token_metadata::accounts::Metadata;
use redis::{Commands, Connection};
use serde::{Deserialize, Serialize};
use solana_client::{
    nonblocking::rpc_client::RpcClient, rpc_config::RpcTransactionConfig,
    rpc_request::TokenAccountsFilter, rpc_response::RpcTokenAccountBalance,
};
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Signature};
use solana_transaction_status::{EncodedConfirmedTransactionWithStatusMeta, UiTransactionEncoding};
use std::{ops::Deref, str::FromStr, sync::Arc};

/// Solana Rpc sugar
pub trait SolRpcApi {
    /// Solana rpc client
    fn rpc(&self) -> &Arc<RpcClient>;

    /// Helius advanced client
    fn helius(&self) -> &Arc<RpcClient>;

    async fn top_holders(
        &self,
        mint: &str,
        update: bool,
        redis: &mut Connection,
    ) -> Result<Holders> {
        let key = format!("sol::holders::{mint}");
        let holders = if update || !redis.exists(&key)? {
            let holders: Holders = self
                .helius()
                .get_token_largest_accounts_with_commitment(
                    &Pubkey::from_str(mint)?,
                    CommitmentConfig::finalized(),
                )
                .await?
                .value
                .into();

            redis.set_ex(key, bitcode::serialize(&holders)?, FIVE_MINS)?;
            holders
        } else {
            let holders: Vec<u8> = redis.get(&key)?;
            bitcode::deserialize(&holders)?
        };

        Ok(holders)
    }

    /// Get toekn account
    async fn token_account(
        &self,
        mint: Pubkey,
        acc: &Pubkey,
        update: bool,
        redis: &mut Connection,
    ) -> Result<Vec<(String, String)>> {
        let key = format!("sol::tokenacc::{mint}::{acc}");
        let accs = if update || !redis.exists(&key)? {
            let accs = self
                .rpc()
                .get_token_accounts_by_owner(acc, TokenAccountsFilter::Mint(mint))
                .await?;

            let accs = sol::parse_token_accounts(accs)?;
            redis.set_ex(key, bitcode::serialize(&accs)?, FIVE_MINS)?;
            accs
        } else {
            let accs: Vec<u8> = redis.get(&key)?;
            bitcode::deserialize(&accs)?
        };

        Ok(accs)
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

/// Token holders
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Holders(Vec<(String, String)>);

impl Holders {
    pub fn percent(&self) -> Result<BigDecimal> {
        let total = BigDecimal::from(TOTAL_SUPPLY);
        let present = self
            .iter()
            .map(|acc| BigDecimal::from_str(&acc.1).map_err(Into::into))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .reduce(|cur, acc| acc + cur)
            .unwrap_or_default();

        Ok((present / total * 100u32).round(2))
    }

    /// Skip bonding curve
    pub fn skip_bc(self, bc: &str) -> Self {
        Self(self.0.into_iter().filter(|(acc, _)| acc != bc).collect())
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
                .filter(|b| b.amount.ui_amount_string != "0")
                .map(|b| (b.address, b.amount.ui_amount_string))
                .collect(),
        )
    }
}
