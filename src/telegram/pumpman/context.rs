use crate::{
    config,
    model::{Pumpman, PumpmanGlobal, User},
    schema::{pumpman_global, pumpmen, users},
    Context,
};
use anyhow::Result;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use solana_sdk::signature::Keypair;
use std::{ops::Deref, sync::Arc};

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

    /// Get job by job id
    pub async fn global_by_id(&self, id: i64) -> Result<PumpmanGlobal> {
        let postgres = &mut self.context.postgres().await?;
        pumpman_global::table
            .filter(pumpman_global::id.eq(id))
            .first::<PumpmanGlobal>(postgres)
            .await
            .map_err(Into::into)
    }
}

impl Deref for PumpmanContext {
    type Target = Context;
    fn deref(&self) -> &Self::Target {
        &self.context
    }
}
