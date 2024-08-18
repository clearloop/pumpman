use crate::{
    api::{PumpApi, SolRpcApi, PUMPFUN_FEE_BASIS},
    config,
    context::{Cache, Client},
    model::{Pumpman, PumpmanGlobal, User},
    schema::{pumpman_global, pumpmen, users},
    sol::{
        self,
        pump::{
            self,
            accounts::{BondingCurve, Global},
        },
        utils::{LAMPORTS_PER_SIGNATURE, MICRO_LAMPORTS_PER_LAMPORT},
        Lamports,
    },
    Context,
};
use anyhow::Result;
use bigdecimal::Zero;
use borsh::BorshDeserialize;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use redis::Commands;
use redis::Connection;
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction, instruction::Instruction, pubkey::Pubkey,
    signature::Keypair, signer::Signer, system_instruction, transaction::Transaction,
};
use spl_associated_token_account::instruction as aca_ix;
use std::{ops::Deref, str::FromStr, sync::Arc};

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

    /// bump token
    pub async fn bump(&self, global: &Global, job: &Pumpman) -> Result<Transaction> {
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

    pub async fn global(&self, tgid: i64) -> Result<PumpmanGlobal> {
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

    /// Get job by job id
    pub async fn global_by_id(&self, id: i64) -> Result<PumpmanGlobal> {
        let postgres = &mut self.context.postgres().await?;
        pumpman_global::table
            .filter(pumpman_global::id.eq(id))
            .first::<PumpmanGlobal>(postgres)
            .await
            .map_err(Into::into)
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
            let job = self.global(tgid).await?.generate(mint);
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
            units: BumpBuilder::BASIC_UNITS,
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
    const SIGNATURES: u64 = 1;
    const BASIC_UNITS: u32 = 134;
    const CACA_UNITS: u32 = 23203;
    const BUMP_UNITS: u32 = 82872;
    const TRANSFER_UNITS: u32 = 150;
    const BUDGET_UNITS: u32 = 300;

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

    /// TODO: check if no charging
    pub fn ix_service_fee(mut self) -> Result<Self> {
        let user = self.wallet.pubkey();
        let treasury = Pubkey::from_str(&self.config.treasury)?;
        self.ixs.push(system_instruction::transfer(
            &user,
            &treasury,
            self.config.fee.lamports()?,
        ));

        self.units += Self::TRANSFER_UNITS;
        Ok(self)
    }

    pub fn ixs_compute_budget(mut self) -> Result<Self> {
        let lamports =
            (self.job.tx_fee.lamports()?).saturating_sub(LAMPORTS_PER_SIGNATURE * Self::SIGNATURES);

        if lamports.is_zero() {
            return Ok(self);
        }

        self.ixs
            .push(ComputeBudgetInstruction::set_compute_unit_limit(self.units));
        self.ixs
            .push(ComputeBudgetInstruction::set_compute_unit_price(
                lamports
                    .saturating_mul(MICRO_LAMPORTS_PER_LAMPORT)
                    .saturating_div(self.units.into()),
            ));

        self.units += Self::BUDGET_UNITS;
        Ok(self)
    }

    pub async fn ix_caca(mut self, client: &Client, redis: &mut Connection) -> Result<Self> {
        let user = self.wallet.pubkey();
        let key = Cache::ACA(&self.mint);
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
        self.units += Self::CACA_UNITS;
        Ok(self)
    }

    pub async fn ixs_bump(mut self, client: &Client, redis: &mut Connection) -> Result<Self> {
        let bc = self.bonding_curve(client, redis).await?;
        let user = self.wallet.pubkey();

        // pumpfun fee
        let sol = self.job.amount.lamports()?;

        // Calculate bump amount
        let amount = self.global.buy(bc.real_sol_reserves, sol)?;
        let pfee = sol * self.global.fee_basis_points / PUMPFUN_FEE_BASIS;
        let buy = pump::Buy::new(amount, sol.saturating_add(pfee)).ix(self.global, self.mint, user);
        let sell =
            pump::Sell::new(amount, sol.saturating_sub(pfee)).ix(self.global, self.mint, user);

        self.ixs.append(&mut vec![buy, sell]);
        self.units += Self::BUMP_UNITS;
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
}
