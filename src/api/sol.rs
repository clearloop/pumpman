use anyhow::Result;
use async_trait::async_trait;
use mpl_token_metadata::accounts::Metadata;
use redis::Connection;
use solana_client::{
    nonblocking::rpc_client::RpcClient, rpc_config::RpcTransactionConfig,
    rpc_response::RpcTokenAccountBalance,
};
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Signature};
use solana_transaction_status::{EncodedConfirmedTransactionWithStatusMeta, UiTransactionEncoding};
use std::{str::FromStr, sync::Arc};

/// Solana Rpc sugar
#[async_trait]
pub trait SolRpcApi {
    /// Solana rpc client
    fn rpc(&self) -> &Arc<RpcClient>;

    async fn top_holders(
        &self,
        mint: &str,
        _update: bool,
        _con: &mut Connection,
    ) -> Result<Vec<RpcTokenAccountBalance>> {
        let holders = self
            .rpc()
            .get_token_largest_accounts_with_commitment(
                &Pubkey::from_str(&mint)?,
                CommitmentConfig::finalized(),
            )
            .await?
            .value;

        Ok(holders)
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
