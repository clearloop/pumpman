use crate::{
    config::PumpmanGlobal,
    model::{Pumpman, User},
    schema::{pumpmen, users},
    Context,
};
use anyhow::Result;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use solana_sdk::{pubkey::Pubkey, signature::Keypair};
use std::{ops::Deref, sync::Arc};

/// Wrapped context
#[derive(Clone)]
pub struct PumpmanContext {
    /// command context
    pub context: Context,
    /// cutomized data in context
    pub global: Arc<PumpmanGlobal>,
}

impl PumpmanContext {
    /// Create new wrapped context
    pub fn new(context: Context, global: PumpmanGlobal) -> Self {
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

    /// Get wallet address from telegram user id
    pub async fn job(&self, tgid: i64, mint: Pubkey) -> Result<Pumpman> {
        let postgres = &mut self.context.postgres().await?;
        let mint_str = mint.to_string();
        if let Some(job) = pumpmen::table
            .filter(pumpmen::owner.eq(tgid))
            .filter(pumpmen::mint.eq(&mint_str))
            .first::<Pumpman>(postgres)
            .await
            .optional()?
        {
            Ok(job)
        } else {
            let job = Pumpman::new(&self.global, tgid, mint_str);
            diesel::insert_into(pumpmen::table)
                .values(&job)
                .execute(postgres)
                .await?;

            Ok(job)
        }
    }
}

impl Deref for PumpmanContext {
    type Target = Context;
    fn deref(&self) -> &Self::Target {
        &self.context
    }
}
