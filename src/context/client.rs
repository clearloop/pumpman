//! Solana programs

use crate::{
    api::{HttpClient, SolRpcApi},
    config::Cluster,
};
use anyhow::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use std::sync::Arc;

/// Replika client
#[derive(Clone)]
pub struct Client {
    /// Http client
    http: Arc<reqwest::Client>,

    /// Solana rpc client
    rpc: Arc<RpcClient>,
}

impl Client {
    /// Create new client
    pub fn new(cluster: &Cluster) -> Result<Self> {
        Ok(Self {
            http: Arc::new(reqwest::Client::new()),
            rpc: Arc::new(RpcClient::new(cluster.http.to_string())),
        })
    }
}

impl SolRpcApi for Client {
    fn rpc(&self) -> &Arc<RpcClient> {
        &self.rpc
    }
}

impl HttpClient for Client {
    fn client(&self) -> &Arc<reqwest::Client> {
        &self.http
    }
}
