//! Context related methods

use crate::{
    context::Context,
    model::{pump, Coin, Takeover, TakeoverWithCoin},
    schema::{coins, takeovers, users},
};
use anyhow::Result;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

impl Context {
    /// Check if the user has enough credits
    pub async fn eligible(&self, uid: &str) -> Result<bool> {
        let postgres = &mut self.postgres().await?;

        let count = takeovers::table
            .filter(takeovers::admin.eq(&uid))
            .count()
            .get_result(postgres)
            .await?;

        let credits = users::table
            .select(users::credits)
            .filter(users::tgid.eq(&uid))
            .first(postgres)
            .await
            .optional()?
            .unwrap_or(1);

        return Ok(credits > count);
    }

    /// List all takeovers from user id
    pub async fn takeovers(&self, uid: &str) -> Result<Vec<TakeoverWithCoin>> {
        let postgres = &mut self.postgres().await?;

        takeovers::table
            .inner_join(coins::table.on(coins::mint.eq(takeovers::mint)))
            .filter(takeovers::admin.eq(uid))
            .select((Coin::as_select(), Takeover::as_select()))
            .load::<(Coin, Takeover)>(postgres)
            .await
            .map(|i| {
                i.into_iter()
                    .map(|(coin, takeover)| TakeoverWithCoin { coin, takeover })
                    .collect()
            })
            .map_err(Into::into)
    }

    pub async fn update_coin(&self, coin: pump::Coin) -> Result<()> {
        let coin = Coin::from(coin);
        let postgres = &mut self.postgres().await?;

        diesel::insert_into(coins::table)
            .values(coin.clone())
            .on_conflict(coins::mint)
            .do_update()
            .set(coin)
            .execute(postgres)
            .await?;

        Ok(())
    }
}
