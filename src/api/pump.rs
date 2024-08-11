#![allow(unused)]
use crate::{api::HttpClient, model::pump::Coin};
use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::EncodableKeypair};
use std::{fmt::Display, sync::Arc};

const PUMPFUN: &str = "https://frontend-api.pump.fun";

/// pump.fun api set
pub struct PumpApi;

impl PumpApi {
    /// pumpfun coins api
    pub fn coin(mint: &str) -> String {
        format!("{PUMPFUN}/coins/{mint}")
    }
}

/// Pumpfun account
pub struct PumpAccount {
    /// Auth token of this account
    auth: String,
    /// Keypair of this account
    pair: Keypair,
}
