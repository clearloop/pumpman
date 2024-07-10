use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};
use url::Url;

/// Replika service config
#[derive(Serialize, Deserialize)]
pub struct Config {
    /// RPC endpoint of solana
    pub cluster: Cluster,
    /// Postgres url
    pub postgres: Url,
    /// Redis url
    pub redis: Url,
    /// Telegram bot token
    pub telegram: Telegram,
}

impl Config {
    /// Load config from path
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let config = fs::read_to_string(path.as_ref())
            .map_err(|e| anyhow!("Could not find config.toml: {e}"))?;

        toml::from_str(&config).map_err(Into::into)
    }
}

/// Solana cluster
#[derive(Serialize, Deserialize)]
pub struct Cluster {
    /// Helius API for advanced usages
    pub helius: Url,
    /// http rpc url
    pub http: Url,
    /// websocket rpc url
    pub ws: Url,
}

/// Telegram config
#[derive(Serialize, Deserialize)]
pub struct Telegram {
    /// Telegram token of the takeover bot
    pub takeover_bot: String,
    /// Subscription chat id
    pub takeover_alerts: String,
    /// Bot for takeover alerts
    pub takeover_alerts_bot: String,
}
