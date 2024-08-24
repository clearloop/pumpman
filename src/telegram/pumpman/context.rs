use crate::{
    api::{PumpApi, SolRpcApi},
    config,
    context::{Cache, Client},
    model::{Pumpman, PumpmanGlobal, PumpmanJob, User},
    schema::{pumpman_global, pumpmen, users},
    sol::{
        self,
        pump::{
            self,
            accounts::{BondingCurve, Global},
        },
        utils::MICRO_LAMPORTS_PER_LAMPORT,
        Lamports,
    },
    utils::{DAY, HOUR},
    Context,
};
use anyhow::Result;
use bigdecimal::Zero;
use borsh::BorshDeserialize;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use redis::Commands;
use redis::Connection;
use solana_client::{client_error::ClientErrorKind, rpc_request::RpcError};
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction,
    instruction::Instruction,
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    system_instruction,
    transaction::{Transaction, TransactionError},
};
use spl_associated_token_account::instruction as aca_ix;
use std::{
    ops::{Add, Deref},
    str::FromStr,
    sync::Arc,
};
use time::OffsetDateTime;

/// Wrapped context
#[derive(Clone)]
pub struct PumpmanContext {
    /// command context
    pub context: Context,
    /// cutomized data in context
    pub global: Arc<config::PumpmanGlobal>,
}

impl PumpmanContext {
    /// Create new wrapped context
    pub fn new(context: Context, global: config::PumpmanGlobal) -> Self {
        Self {
            context,
            global: Arc::new(global),
        }
    }

    pub async fn bump(&self, global: &Global, job: &Pumpman) -> Result<()> {
        let tx = self.bump_tx(global, job).await?;
        if let Err(e) = self.client.helius().send_transaction(&tx).await {
            if let ClientErrorKind::RpcError(RpcError::RpcResponseError { code, .. }) = e.kind() {
                // NOTE:
                //
                // RPC response error -32002: Transaction simulation failed: Blockhash not found
                if *code == -32002 {
                    return Ok(());
                }
            }

            if let Some(err) = e.get_transaction_error() {
                match err {
                    TransactionError::BlockhashNotFound => {}
                    _ => anyhow::bail!("{err:?}"),
                }
            }
            return Err(e.into());
        }

        let postgres = &mut self.context.postgres().await?;
        diesel::update(pumpmen::table)
            .filter(pumpmen::id.eq(job.id()))
            .set((
                pumpmen::bumps.eq(pumpmen::bumps.add(job.batch as i64)),
                pumpmen::charged.eq(pumpmen::charged.add(job.service_fee(&self.global))),
            ))
            .execute(postgres)
            .await?;
        Ok(())
    }

    pub async fn simulate_bump(&self, global: &Global, job: &Pumpman) -> Result<()> {
        let tx = self.bump_tx(global, job).await?;
        let resp = self.client.helius().simulate_transaction(&tx).await?;
        if let Some(err) = resp.value.err {
            match err {
                TransactionError::BlockhashNotFound => return Ok(()),
                _ => anyhow::bail!("{err:?}"),
            }
        }

        let postgres = &mut self.context.postgres().await?;
        diesel::update(pumpmen::table)
            .filter(pumpmen::id.eq(job.id()))
            .set((
                pumpmen::bumps.eq(pumpmen::bumps.add(job.batch as i64)),
                pumpmen::charged.eq(pumpmen::charged.add(job.service_fee(&self.global))),
            ))
            .execute(postgres)
            .await?;

        Ok(())
    }

    /// bump token
    pub async fn bump_tx(&self, global: &Global, job: &Pumpman) -> Result<Transaction> {
        let redis = &mut self.redis()?;
        self.bump_builder(global, job)
            .await?
            .ix_service_fee()?
            .ix_caca(&self.client, redis)
            .await?
            .ixs_bump(&self.client, redis)
            .await?
            .ixs_compute_budget()?
            .build(&self.client)
            .await
    }

    /// Stop job
    pub async fn stop(&self, job: i64) -> Result<()> {
        let postgres = &mut self.context.postgres().await?;
        diesel::update(pumpmen::table)
            .filter(pumpmen::id.eq(job))
            .set(pumpmen::active.eq(false))
            .execute(postgres)
            .await?;
        Ok(())
    }

    /// Get wallet address from telegram user id
    pub async fn wallet(&self, tgid: i64) -> Result<Keypair> {
        let postgres = &mut self.context.postgres().await?;

        let wallet = if let Some(wallet) = users::table
            .select(users::wallet)
            .filter(users::tgid.eq(tgid))
            .first::<String>(postgres)
            .await
            .optional()?
        {
            wallet
        } else {
            let user = User::new(tgid);
            diesel::insert_into(users::table)
                .values(&user)
                .execute(postgres)
                .await?;

            user.wallet
        };

        let bytes = bs58::decode(wallet).into_vec()?;
        Keypair::from_bytes(&bytes).map_err(Into::into)
    }

    /// Create a new user
    pub async fn create_user(&self, tgid: i64) -> Result<User> {
        let con = &mut self.redis()?;
        let wallet = Keypair::new();

        if let Err(e) = self.client.users(&self.global.profile, &wallet, con).await {
            tracing::warn!("Failed to create profile for {}, {e}", wallet.pubkey());
        }

        Ok(User {
            id: None,
            created_at: OffsetDateTime::now_utc().date(),
            tgid,
            wallet: bs58::encode(wallet.to_bytes()).into_string(),
        })
    }

    pub async fn pglobal(&self, tgid: i64) -> Result<PumpmanGlobal> {
        let postgres = &mut self.context.postgres().await?;
        if let Some(global) = pumpman_global::table
            .filter(pumpman_global::owner.eq(tgid))
            .first::<PumpmanGlobal>(postgres)
            .await
            .optional()?
        {
            Ok(global)
        } else {
            let global = PumpmanGlobal::new(&self.global, tgid);
            diesel::insert_into(pumpman_global::table)
                .values(&global)
                .execute(postgres)
                .await?;

            Ok(global)
        }
    }

    /// Get wallet address from telegram user id
    pub async fn job(&self, tgid: i64, mint: &str) -> Result<Pumpman> {
        let postgres = &mut self.context.postgres().await?;
        if let Some(job) = pumpmen::table
            .filter(pumpmen::owner.eq(tgid))
            .filter(pumpmen::mint.eq(mint))
            .first::<Pumpman>(postgres)
            .await
            .optional()?
        {
            Ok(job)
        } else {
            let job = self.pglobal(tgid).await?.generate(mint);
            diesel::insert_into(pumpmen::table)
                .values(&job)
                .execute(postgres)
                .await?;

            Ok(job)
        }
    }

    /// Get job by job id
    pub async fn job_by_id(&self, id: i64) -> Result<Pumpman> {
        let postgres = &mut self.context.postgres().await?;
        pumpmen::table
            .filter(pumpmen::id.eq(id))
            .first::<Pumpman>(postgres)
            .await
            .map_err(Into::into)
    }

    pub async fn jobs(&self, tgid: i64) -> Result<Vec<Pumpman>> {
        let postgres = &mut self.context.postgres().await?;
        pumpmen::table
            .filter(pumpmen::owner.eq(tgid))
            .load(postgres)
            .await
            .map_err(Into::into)
    }

    /// Get pumpfun global
    pub async fn fee_basis_points(&self) -> Result<u64> {
        let redis = &mut self.redis()?;
        let key = Cache::PumpFeeBasisPoints;
        if let Ok(points) = redis.get(&key) {
            return Ok(points);
        }

        let points = self.client.global().await?.fee_basis_points;
        redis.set_ex(&key, points, DAY)?;
        Ok(points)
    }

    /// Get priority fee with cache
    pub async fn priority_fee(&self) -> Result<u64> {
        let redis = &mut self.redis()?;
        let key = Cache::PumpPriorityFee;
        if let Ok(fee) = redis.get(&key) {
            return Ok(fee);
        }

        let fee = self.client.priority_fee().await?;
        redis.set_ex(&key, fee, HOUR)?;
        Ok(fee)
    }

    /// Create a builder for bump transactions
    async fn bump_builder<'i>(
        &'i self,
        global: &'i Global,
        job: &'i Pumpman,
    ) -> Result<BumpBuilder<'i>> {
        Ok(BumpBuilder {
            global,
            job,
            config: &self.global,
            mint: Pubkey::from_str(&job.mint)?,
            wallet: self.wallet(job.owner).await?,
            ixs: Default::default(),
            units: Pumpman::BASIC_UNITS,
        })
    }
}

impl Deref for PumpmanContext {
    type Target = Context;
    fn deref(&self) -> &Self::Target {
        &self.context
    }
}

/// bump transaction builder
pub struct BumpBuilder<'i> {
    config: &'i config::PumpmanGlobal,
    job: &'i Pumpman,
    global: &'i Global,
    mint: Pubkey,
    wallet: Keypair,
    ixs: Vec<Instruction>,
    units: u32,
}

impl<'i> BumpBuilder<'i> {
    pub async fn build(self, client: &Client) -> Result<Transaction> {
        let payer = self.wallet.pubkey();
        let blockhash = client.helius().get_latest_blockhash().await?;
        Ok(Transaction::new_signed_with_payer(
            &self.ixs,
            Some(&payer),
            &[&self.wallet],
            blockhash,
        ))
    }

    pub fn ix_service_fee(mut self) -> Result<Self> {
        let fee = self.job.service_fee(self.config);
        if fee.is_zero() {
            return Ok(self);
        }

        let user = self.wallet.pubkey();
        let treasury = Pubkey::from_str(&self.config.treasury)?;
        self.ixs.push(system_instruction::transfer(
            &user,
            &treasury,
            fee.lamports()?,
        ));

        self.units += Pumpman::TRANSFER_UNITS;
        Ok(self)
    }

    /// Default is 200k CU per ix
    ///
    /// <https://solana.com/docs/core/fees#compute-unit-limit>
    pub fn ixs_compute_budget(mut self) -> Result<Self> {
        let lamports = self.job.priority_fee.lamports()?;

        if lamports.is_zero() {
            return Ok(self);
        }

        let units = self.units();
        self.ixs
            .push(ComputeBudgetInstruction::set_compute_unit_limit(units));
        self.ixs
            .push(ComputeBudgetInstruction::set_compute_unit_price(
                lamports
                    .saturating_mul(MICRO_LAMPORTS_PER_LAMPORT)
                    .saturating_div(units.into()),
            ));

        self.units += Pumpman::BUDGET_UNITS;
        Ok(self)
    }

    pub async fn ix_caca(mut self, client: &Client, redis: &mut Connection) -> Result<Self> {
        let user = self.wallet.pubkey();
        let key = Cache::Aca(&self.mint);
        if redis.exists(&key)? {
            return Ok(self);
        }

        let exists = client.check_auser(self.mint, user).await;
        if exists {
            redis.set(&key, true)?;
            return Ok(self);
        }

        self.ixs.push(aca_ix::create_associated_token_account(
            &user,
            &user,
            &self.mint,
            &sol::TOKEN_PROGRAM,
        ));
        self.units += Pumpman::CACA_UNITS;
        Ok(self)
    }

    pub async fn ixs_bump(mut self, client: &Client, redis: &mut Connection) -> Result<Self> {
        let bc = self.bonding_curve(client, redis).await?;
        let user = self.wallet.pubkey();

        // pumpfun fee
        let sol = self.job.amount.lamports()?;

        // Calculate bump amount
        let amount = self.global.buy(bc.real_sol_reserves, sol)?;
        let slippage = sol * (self.config.slippage as u64) / 100;
        let buy =
            pump::Buy::new(amount, sol.saturating_add(slippage)).ix(self.global, self.mint, user);
        let sell =
            pump::Sell::new(amount, sol.saturating_sub(slippage)).ix(self.global, self.mint, user);

        self.ixs.append(
            &mut vec![vec![buy.clone(), sell.clone()]; self.job.batch as usize]
                .into_iter()
                .flatten()
                .collect(),
        );
        self.units += Pumpman::BUMP_UNITS * (self.job.batch as u32);
        Ok(self)
    }

    /// Get bonding curve with cache
    async fn bonding_curve(&self, client: &Client, redis: &mut Connection) -> Result<BondingCurve> {
        let key = Cache::BondingCurve(&self.mint);
        if redis.exists(&key)? {
            let bytes: Vec<u8> = redis.get(&key)?;
            return BondingCurve::deserialize(&mut bytes.as_ref()).map_err(Into::into);
        }

        let bc = client.bonding_curve(&self.mint).await?;
        redis.set_ex(key, borsh::to_vec(&bc)?, self.config.cache.bonding_curve)?;
        Ok(bc)
    }

    /// Align units to 10_000
    fn units(&self) -> u32 {
        let unit = 10_000;
        self.units + unit - self.units % unit
    }
}
