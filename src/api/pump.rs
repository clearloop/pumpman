// use solana_client::nonblocking::rpc_client::RpcClient;
// use std::sync::Arc;

use crate::{api::HttpClient, model::pump::Coin, utils::FIVE_MINS};
use anyhow::Result;
use redis::Connection;

const PUMPFUN: &str = "https://frontend-api.pump.fun";

// format!("{PUMPFUN}/coins/{mint}")

/// pump.fun api set
pub trait PumpApi: HttpClient {
    /// get coin of pump fun
    async fn coin(&self, mint: &str, update: bool, con: &mut Connection) -> Result<Coin> {
        self.cget(&format!("{PUMPFUN}/coins/{mint}"), update, FIVE_MINS, con)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get pump coin {mint}: {e}");
                e
            })
    }
}
