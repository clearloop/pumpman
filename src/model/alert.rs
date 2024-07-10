//! Model of dev soldout alert
#![allow(dead_code)]

use crate::model::{pump::Coin, DexScreenerPair};
use solana_client::rpc_response::RpcTokenAccountBalance;

/// Basic alert
pub struct Alert {
    /// Pumpfun coins
    pub coin: Coin,
    /// Dexscreener apirs
    pub pairs: Vec<DexScreenerPair>,
    /// Top holders
    pub holders: Vec<RpcTokenAccountBalance>,
    // /// Alert event
    // pub event: Event,
    //
    // takeover details
}

impl Alert {
    /// Create new alert
    pub fn new(coin: Coin) -> Self {
        Self {
            coin,
            pairs: Default::default(),
            holders: Default::default(),
        }
    }

    /// Set up pairs
    pub fn pairs(mut self, pairs: Vec<DexScreenerPair>) -> Self {
        self.pairs = pairs;
        self
    }

    /// Set up holders
    pub fn holders(mut self, holders: Vec<RpcTokenAccountBalance>) -> Self {
        self.holders = holders;
        self
    }
}
