#![allow(unused)]
use crate::{
    api::{HttpClient, SolRpcApi},
    model::pump::Coin,
    sol::pump::{
        self,
        accounts::{BondingCurve, Global},
        GLOBAL,
    },
    utils::FIVE_MINS,
};
use anchor_lang::AccountDeserialize;
use anyhow::Result;
use redis::Connection;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

const PUMPFUN: &str = "https://frontend-api.pump.fun";

/// pump.fun api set
pub trait PumpApi: HttpClient + SolRpcApi {
    /// get coin of pump fun
    async fn coin(&self, mint: &str, update: bool, con: &mut Connection) -> Result<Coin> {
        self.cget(&format!("{PUMPFUN}/coins/{mint}"), update, FIVE_MINS, con)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get pump coin {mint}: {e}");
                e
            })
    }

    /// Check if account is soldout
    async fn soldout(
        &self,
        mint: &str,
        acc: &str,
        update: bool,
        redis: &mut Connection,
    ) -> Result<(String, bool)> {
        let mint = Pubkey::from_str(mint)?;
        let pk = Pubkey::from_str(acc)?;
        let accs = self.token_account(mint, &pk, update, redis).await?;

        // The dev has never bought the token
        if accs.is_empty() {
            return Ok((acc.to_string(), true));
        }

        Ok(accs
            .first()
            .map(|acc| (acc.0.clone(), acc.1.starts_with('0')))
            .unwrap_or((acc.to_string(), false)))
    }

    /// Get global account info
    async fn global(&self) -> Result<Global> {
        self.data(&GLOBAL).await
    }

    /// Get global account info
    async fn bonding_curve(&self, mint: &Pubkey) -> Result<BondingCurve> {
        let bc = pump::bonding_curve(mint);
        self.data(&bc).await
    }

    /// Buy a pumpfun token
    async fn buy(&self) -> Result<()> {
        Ok(())
    }

    /// Buy a pumpfun token
    async fn sell(&self) -> Result<()> {
        Ok(())
    }

    /// Bump a pumpfun token
    async fn bump(&self) -> Result<()> {
        Ok(())
    }
}
