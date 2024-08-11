use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};
use url::Url;

/// Replika service config
#[derive(Serialize, Deserialize)]
pub struct Config {
    /// RPC endpoint of solana
    pub cluster: Cluster,
    /// Database uri
    pub database: Database,
    /// Replika takeover service
    pub takeover: Takeover,
}

impl Config {
    /// Get pumpsub config
    pub fn pumpsub(&self) -> PumpSub {
        PumpSub {
            takeover_coins: self.takeover.coins,
            takeover_disabled: self.takeover.disabled,
        }
    }

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

/// Database config
#[derive(Serialize, Deserialize, Clone)]
pub struct Database {
    /// Postgres url
    pub postgres: Url,
    /// Redis url
    pub redis: Url,
}

/// Takeover service config
#[derive(Serialize, Deserialize, Clone)]
pub struct Takeover {
    /// If disabled takeover
    #[serde(default)]
    pub disabled: bool,
    /// Takeover alert bot
    pub bot: Option<String>,
    /// If start takeover registry
    #[serde(default)]
    pub registry: bool,
    /// Batch coins in events
    pub coins: usize,
    /// Batch requests of coins
    pub batch: usize,
    /// Holders should greater than
    pub holders: usize,
    /// Market cap should greater than
    pub marketcap: u64,
    /// Takeover subscription channel
    pub subscription: String,
}

/// Pumpsub config
pub struct PumpSub {
    pub takeover_coins: usize,
    pub takeover_disabled: bool,
}
