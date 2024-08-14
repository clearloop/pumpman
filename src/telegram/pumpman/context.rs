use crate::{config::PumpmanGlobal, model::User, schema::users, Context};
use anyhow::Result;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use solana_sdk::signature::Keypair;
use std::sync::Arc;

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
}
