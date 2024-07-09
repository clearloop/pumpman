//! Solana programs

use crate::{
    api::{DexScreenerApi, HttpClient, PumpApi, SolRpcApi},
    config::Cluster,
    model::Coin,
    redis,
    utils::DAY,
};
use ::redis::Connection;
use anyhow::Result;
use serde_json::Value;
use solana_client::nonblocking::rpc_client::RpcClient;
use std::sync::Arc;

const CACHE_DEX_TOKENS: &str = "dexscreener::tokens::";

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

    /// Get token from address
    pub async fn coin(&self, mint: &str) -> Result<Coin> {
        let mplmeta = self.mpl_token_metadata(mint).await?;
        let meta: Value = reqwest::get(&mplmeta.uri).await?.json().await?;
        let mut coin: Coin = mplmeta.into();
        coin.append(meta);
        Ok(coin)
    }

    /// Get dexscreener url
    pub async fn dex_tokens(&self, mint: &str, con: &mut Connection) -> Option<String> {
        let key = format!("{CACHE_DEX_TOKENS}:{mint}");
        let pairs = if let Ok(cache) = redis::get(&key, con) {
            cache
        } else {
            let r = self.tokens(mint).await.ok()?.pairs?;
            redis::set(&key, &r, DAY, con).ok()?;
            r
        };

        pairs.first().and_then(|p| Some(p.url.clone()))
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

impl DexScreenerApi for Client {}
impl PumpApi for Client {}
