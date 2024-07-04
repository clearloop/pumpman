use serde::{Deserialize, Serialize};
use url::Url;

/// Replika service config
#[derive(Serialize, Deserialize)]
pub struct Config {
    /// RPC endpoint of solana
    pub cluster: Cluster,
    /// Redis url
    pub redis: Url,
    /// Telegram bot token
    pub telegram: Telegram,
}

/// Solana cluster
#[derive(Serialize, Deserialize)]
pub struct Cluster {
    /// http rpc url
    pub http: Url,
    /// websocket rpc url
    pub ws: Url,
}

/// Telegram config
#[derive(Serialize, Deserialize)]
pub struct Telegram {
    /// Telegram token
    pub token: String,
    /// Subscription chat id
    pub subscription: String,
}
