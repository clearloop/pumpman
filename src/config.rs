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

/// Pumpman config
#[derive(Serialize, Deserialize, Clone)]
pub struct Pumpman {
    /// pumpman bot token
    pub bot: Option<String>,
    /// pumpman global config
    pub global: PumpmanGlobal,
}

/// Pumpman config context
#[derive(Serialize, Deserialize, Clone)]
pub struct PumpmanGlobal {
    /// Bump amount in sol
    pub amount: BigDecimal,
    /// Bump amount step in sol
    pub amount_step: BigDecimal,
    /// Priority fee in sol
    pub priority_fee: BigDecimal,
    /// Priority fee in sol
    pub priority_fee_step: BigDecimal,
    /// pumpman service fee basis points
    pub service_fee: BigDecimal,
    /// bump fee threshold per token
    pub threshold: BigDecimal,
    /// transaction slippage percent
    pub slippage: i32,
    /// The max limit of bumps to be batched
    pub batch: i32,
    /// bumping duration in seconds
    pub speed: i32,
    /// Pumpman cache config
    pub cache: PumpmanCache,
    /// Profile config of pumpman
    pub profile: PumpmanProfile,
    /// treasury account
    pub treasury: String,
}

/// Pumpman config context
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PumpmanProfile {
    pub username: String,
    pub bio: String,
    pub profile_image: String,
}

/// Cache config of pumpman
#[derive(Serialize, Deserialize, Clone)]
pub struct PumpmanCache {
    pub bonding_curve: u64,
}
