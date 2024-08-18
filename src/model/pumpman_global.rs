use crate::{
    api::PUMPFUN_FEE_BASIS,
    config,
    model::{Pumpman, PumpmanJob},
    sol::pump::{accounts::Global, SLIPPAGE_BASIS},
    telegram::Result,
};
use bigdecimal::BigDecimal;
use diesel::{pg::Pg, prelude::*};
use serde::{Deserialize, Serialize};
use teloxide::types::InlineKeyboardMarkup;
use time::OffsetDateTime;

/// Instance of a bump bot
#[derive(
    Insertable,
    Queryable,
    Selectable,
    AsChangeset,
    Clone,
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
#[diesel(check_for_backend(Pg))]
#[diesel(table_name = crate::schema::pumpman_global)]
pub struct PumpmanGlobal {
    /// Sequence id
    #[diesel(deserialize_as = i64)]
    pub id: Option<i64>,
    /// Owner of this bump bot
    pub owner: i64,
    /// Amount for each bump
    pub amount: BigDecimal,
    /// Fee for each transaction
    pub priority_fee: BigDecimal,
    /// global slippage settings
    pub slippage: i32,
    /// How many bump transactions will be included at once
    pub batch: i32,
    /// Duration for each bump in millis
    pub speed: i32,
}

impl PumpmanGlobal {
    /// Create a new pumpman config
    pub fn new(global: &config::PumpmanGlobal, owner: i64) -> Self {
        Self {
            id: None,
            owner,
            priority_fee: global.priority_fee.clone(),
            amount: global.amount.clone(),
            slippage: global.slippage,
            batch: 1,
            speed: global.speed,
        }
    }

    /// Create a new pumpman config from global
    pub fn generate(&self, mint: &str) -> Pumpman {
        Pumpman {
            id: None,
            active: false,
            created_at: OffsetDateTime::now_utc().date(),
            owner: self.owner,
            mint: mint.into(),
            priority_fee: self.priority_fee.clone(),
            amount: self.amount.clone(),
            slippage: self.slippage,
            batch: self.batch,
            speed: self.speed,
            bumps: 0,
            wallet: None,
        }
    }

    /// * pumpfun fee
    /// * slippage
    /// * transaction fee
    /// * service fee
    pub fn avg_fee(&self, config: &config::PumpmanGlobal, global: &Global) -> BigDecimal {
        let pfee = &self.amount * global.fee_basis_points / PUMPFUN_FEE_BASIS;
        let sfee = &self.amount * self.slippage / SLIPPAGE_BASIS;
        let fee = &pfee * 2 + &config.service_fee + &sfee;

        fee * self.batch + self.tx_fee()
    }

    /// Show the markup from the current config
    pub fn markup(&self, global: &config::PumpmanGlobal) -> Result<InlineKeyboardMarkup> {
        Ok(InlineKeyboardMarkup::new(vec![
            self.batch_button(global)?,
            self.slippage_button()?,
            self.tx_fee_button()?,
            self.amount_button(global)?,
            self.speed_button()?,
        ]))
    }
}
