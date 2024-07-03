use serde::{Deserialize, Serialize};
use url::Url;

/// Replika service config
#[derive(Serialize, Deserialize)]
pub struct Config {
    /// RPC endpoint of solana
    cluster: Url,
    /// Postgres URL
    postgres: Url,
    /// Redis url
    redis: Url,
    /// Sync from slot
    from: u64,
}
