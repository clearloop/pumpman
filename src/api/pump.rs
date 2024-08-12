// use solana_client::nonblocking::rpc_client::RpcClient;
// use std::sync::Arc;

use std::str::FromStr;

use crate::{
    api::{HttpClient, SolRpcApi},
    model::pump::Coin,
    utils::FIVE_MINS,
};
use anyhow::Result;
use redis::Connection;
use solana_sdk::pubkey::Pubkey;

const PUMPFUN: &str = "https://frontend-api.pump.fun";

// format!("{PUMPFUN}/coins/{mint}")

/// pump.fun api set
pub trait PumpApi: HttpClient + SolRpcApi {
    /// get coin of pump fun
    async fn coin(&self, mint: &str, update: bool, con: &mut Connection) -> Result<Coin> {
        self.cget(&format!("{PUMPFUN}/coins/{mint}"), update, FIVE_MINS, con)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get pump coin {mint}: {e}");
                e
            })
    }

    /// Check if account is soldout
    async fn soldout(
        &self,
        mint: &str,
        acc: &str,
        update: bool,
        redis: &mut Connection,
    ) -> Result<(String, bool)> {
        let mint = Pubkey::from_str(mint)?;
        let pk = Pubkey::from_str(acc)?;
        let accs = self.token_account(mint, &pk, update, redis).await?;

        // The dev has never bought the token
        if accs.is_empty() {
            return Ok((acc.to_string(), true));
        }

        Ok(accs
            .first()
            .map(|acc| (acc.0.clone(), acc.1.starts_with('0')))
            .unwrap_or((acc.to_string(), false)))
    }
}
