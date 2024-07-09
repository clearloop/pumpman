use anyhow::Result;
use async_trait::async_trait;
use mpl_token_metadata::accounts::Metadata;
use solana_client::{nonblocking::rpc_client::RpcClient, rpc_config::RpcTransactionConfig};
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use solana_transaction_status::{EncodedConfirmedTransactionWithStatusMeta, UiTransactionEncoding};
use std::{str::FromStr, sync::Arc};

/// Solana Rpc sugar
#[async_trait]
pub trait SolRpcApi {
    /// Solana rpc client
    fn rpc(&self) -> &Arc<RpcClient>;

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
