use anyhow::{anyhow, Result};
use bigdecimal::BigDecimal;
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
    pub takeover: Option<Takeover>,
    /// Replika pumpman service
    pub pumpman: Option<Pumpman>,
}

impl Config {
    /// Get pumpsub config
    pub fn pumpsub(&self) -> PumpSub {
        let takeover_coins = if let Some(takeover) = &self.takeover {
            takeover.coins
        } else {
            0
        };
        PumpSub { takeover_coins }
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
}

/// Pumpman config
#[derive(Serialize, Deserialize, Clone)]
pub struct Pumpman {
    /// pumpman bot token
    pub bot: String,
    /// pumpman global config
    pub global: PumpmanGlobal,
}

/// Pumpman config context
#[derive(Serialize, Deserialize, Clone)]
pub struct PumpmanGlobal {
    /// Transaction fee in sol
    pub tx_fee: BigDecimal,
    /// Bump amount in sol
    pub amount: BigDecimal,
    /// bumping duration in seconds
    pub speed: i64,
    /// pumpman service fee per bump
    pub fee: BigDecimal,
    /// bump fee threshold per token
    pub threshold: BigDecimal,
}
