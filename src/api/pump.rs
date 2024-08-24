#![allow(unused)]
use crate::{
    api::{HttpClient, SolRpcApi},
    config::PumpmanProfile,
    context::Cache,
    model::pump::Coin,
    sol::{
        self,
        pump::{
            self,
            accounts::{BondingCurve, Global},
            Buy, Sell, GLOBAL, SOL_SCALE,
        },
        utils::MICRO_LAMPORTS_PER_LAMPORT,
        Lamports,
    },
    utils::FIVE_MINS,
};
use anchor_lang::AccountDeserialize;
use anyhow::Result;
use bigdecimal::{BigDecimal, ToPrimitive, Zero};
use borsh::BorshSerialize;
use redis::{Commands, Connection};
use reqwest::StatusCode;
use serde_json::json;
use solana_client::rpc_response::RpcPrioritizationFee;
use solana_sdk::{
    account::Account,
    compute_budget::ComputeBudgetInstruction,
    instruction::Instruction,
    message::Message,
    pubkey::Pubkey,
    signer::{keypair::Keypair, signers::Signers, Signer},
    system_instruction,
    transaction::Transaction,
};
use spl_associated_token_account::instruction;
use std::{
    collections::{vec_deque, VecDeque},
    str::FromStr,
};

const PUMPFUN: &str = "https://frontend-api.pump.fun";
const SIGN_IN_MESSAGE: &str = "Sign in to pump.fun:";
const AUTH_TOKEN: &str = "auth_token";

/// pump.fun api set
pub trait PumpApi: HttpClient + SolRpcApi {
    /// Sign in to pump fun
    async fn auth(&self, pair: &Keypair, con: &mut Connection) -> Result<String> {
        let pubkey = pair.pubkey();
        let key = Cache::PumpFunAuthToken(&pubkey);
        if let Ok(token) = con.get(&key) {
            return Ok(token);
        }

        let timestamp = time::OffsetDateTime::now_utc().unix_timestamp() * 1000;
        let message = format!("{SIGN_IN_MESSAGE} {timestamp}");
        let signature = bs58::encode(pair.sign_message(message.as_bytes())).into_string();
        let body = json!({
            "address": pair.pubkey().to_string(),
            "timestamp": timestamp.to_string(),
            "signature": signature
        });

        let res = self
            .client()
            .post(format!("{PUMPFUN}/auth/login"))
            .header("Origin", "https://pump.fun")
            .json(&body)
            .send()
            .await?;

        for cookie in res.cookies() {
            if cookie.name() != AUTH_TOKEN {
                continue;
            }

            let value = cookie.value();
            let max_age = cookie.max_age().unwrap_or_default().as_secs();
            con.set_ex(&key, value, max_age)?;
            return Ok(cookie.value().into());
        }

        anyhow::bail!(
            "Failed to sign in to pump.fun with {}, no auth_token was found, code {}",
            pair.pubkey(),
            res.status()
        );
    }

    async fn users(
        &self,
        profile: &PumpmanProfile,
        pair: &Keypair,
        con: &mut Connection,
    ) -> Result<Pubkey> {
        let token = self.auth(pair, con).await?;
        let res = self
            .client()
            .post(format!("{PUMPFUN}/users"))
            .header("Origin", "https://pump.fun")
            .header("Cookie", format!("auth_token={token}"))
            .json(profile)
            .send()
            .await?;

        let pubkey = pair.pubkey();
        if res.status() != StatusCode::CREATED {
            anyhow::bail!("Failed to create profile for {pubkey}");
        }

        Ok(pubkey)
    }

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

    /// Bump a pumpfun token
    async fn bump(
        &self,
        mint: &Pubkey,
        sol: u64,
        payer: &Keypair,
        exists: bool,
    ) -> Result<Transaction> {
        let bc = self.bonding_curve(mint).await?;
        let global = self.global().await.unwrap_or(Global::cached());

        // create instructions
        let slippage = sol / 100 * 15;
        let user = payer.pubkey();
        let amount = global.buy(bc.real_sol_reserves, sol)?;
        let buy = Buy::new(amount, sol + slippage).ix(&global, *mint, user);
        let sell = Sell::new(amount, sol - slippage).ix(&global, *mint, user);

        // create transaction
        let mut ixs = vec![];
        ixs.push(system_instruction::transfer(
            &user,
            &user,
            BigDecimal::from_str("0.0001")?.lamports()?,
        ));

        if !exists {
            let ix = instruction::create_associated_token_account(
                &user,
                &user,
                mint,
                &sol::TOKEN_PROGRAM,
            );
            ixs.push(ix);
        }

        ixs.append(&mut vec![buy.clone(), sell.clone()]);
        ixs.push(ComputeBudgetInstruction::set_compute_unit_limit(640_000));
        ixs.push(ComputeBudgetInstruction::set_compute_unit_price(
            (BigDecimal::from_str("0.000040")? * MICRO_LAMPORTS_PER_LAMPORT / 640_000u64)
                .lamports()?,
        ));
        let blockhash = self.helius().get_latest_blockhash().await?;
        Ok(Transaction::new_signed_with_payer(
            &ixs,
            Some(&user),
            &[payer],
            blockhash,
        ))
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

    /// get associated account
    async fn check_auser(&self, mint: Pubkey, user: Pubkey) -> bool {
        let auser = sol::atk_addr(&mint, &user);

        // TODO: check the error details
        if self.helius().get_account(&auser).await.is_ok() {
            return true;
        }
        false
    }

    async fn priority_fee(&self) -> Result<u64> {
        let fees: Vec<RpcPrioritizationFee> = self
            .helius()
            .get_recent_prioritization_fees(&[pump::ID])
            .await?
            .into_iter()
            .filter(|f| !f.prioritization_fee.is_zero())
            .collect();

        let avg_fee =
            fees.iter().fold(0, |acc, e| acc + e.prioritization_fee) / (fees.len() as u64);

        Ok(avg_fee)
    }
}
