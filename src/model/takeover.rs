//! Community take over
use crate::{
    api::HttpClient,
    context::Context,
    model::{Coin, User},
    schema::{coins, takeovers, users},
};
use anyhow::Result;
use async_graphql::SimpleObject;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// Community take over information
#[derive(
    SimpleObject,
    Insertable,
    Queryable,
    Selectable,
    AsChangeset,
    Clone,
    Default,
    PartialEq,
    Debug,
    Serialize,
    Deserialize,
    Identifiable,
    Eq,
    PartialOrd,
    Ord,
    Hash,
)]
pub struct Takeover {
    /// Sequence id
    #[diesel(deserialize_as = i64)]
    pub id: Option<i64>,
    /// Banner of the cto
    pub banner: Option<String>,
    /// The address of the token to be taken over
    pub mint: String,
    /// Telegram user id of this takeover
    pub admin: String,
    /// Telegram link
    pub telegram: String,
    /// Twitter link
    pub twitter: Option<String>,
    /// Website link
    pub website: Option<String>,
}

impl Takeover {
    /// New community takeover
    pub fn new(mint: String) -> Self {
        Self {
            mint,
            ..Default::default()
        }
    }

    /// Write self into database
    ///
    /// 1. check if coin exists
    /// 2. check if admin exists
    /// 3. insert self
    pub async fn write(&self, context: &Context) -> Result<()> {
        let postgres = &mut context.postgres()?;
        let redis = &mut context.redis()?;
        let coin: Coin = context.client.coin(&self.mint, false, redis).await?.into();
        diesel::insert_into(coins::table)
            .values(coin)
            .on_conflict_do_nothing()
            .execute(postgres)?;

        diesel::insert_into(users::table)
            .values(User::new(self.admin.clone()))
            .on_conflict_do_nothing()
            .execute(postgres)?;

        diesel::insert_into(takeovers::table)
            .values(self)
            .execute(postgres)?;
        Ok(())
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TakeoverWithCoin {
    pub takeover: Takeover,
    pub coin: Coin,
}
