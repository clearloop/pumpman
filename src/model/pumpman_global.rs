use crate::{config, model::Pumpman};
use bigdecimal::{BigDecimal, Zero};
use diesel::{pg::Pg, prelude::*};
use serde::{Deserialize, Serialize};
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
            wallet: None,
            mint: mint.into(),
            priority_fee: self.priority_fee.clone(),
            amount: self.amount.clone(),
            batch: self.batch,
            speed: self.speed,
            bumps: 0,
            charged: BigDecimal::zero(),
        }
    }
}
