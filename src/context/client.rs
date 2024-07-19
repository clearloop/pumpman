//! Solana programs

use crate::{
    api::{HttpClient, SolRpcApi},
    config::Cluster,
};
use anyhow::Result;
use redis::Connection;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::{str::FromStr, sync::Arc};

/// Replika client
#[derive(Clone)]
pub struct Client {
    /// Http client
    http: Arc<reqwest::Client>,
    /// Solana rpc client
    rpc: Arc<RpcClient>,
    /// Helius rpc client
    helius: Arc<RpcClient>,
}

impl Client {
    /// Create new client
    pub fn new(cluster: &Cluster) -> Result<Self> {
        Ok(Self {
            http: Arc::new(reqwest::Client::new()),
            rpc: Arc::new(RpcClient::new(cluster.http.to_string())),
            helius: Arc::new(RpcClient::new(cluster.helius.to_string())),
        })
    }

    /// Check if account is soldout
    pub async fn soldout(
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
            .map(|acc| (acc.0.clone(), acc.1.starts_with("0")))
            .unwrap_or((acc.to_string(), false)))
    }
}

impl SolRpcApi for Client {
    fn rpc(&self) -> &Arc<RpcClient> {
        &self.rpc
    }

    fn helius(&self) -> &Arc<RpcClient> {
        &self.helius
    }
}

impl HttpClient for Client {
    fn client(&self) -> &Arc<reqwest::Client> {
        &self.http
    }
}
