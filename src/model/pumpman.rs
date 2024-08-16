use bigdecimal::BigDecimal;
use diesel::{pg::Pg, prelude::*};
use serde::{Deserialize, Serialize};
use time::Date;

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
#[diesel(table_name = crate::schema::pumpmen)]
pub struct PumpMan {
    /// Sequence id
    #[diesel(deserialize_as = i64)]
    pub id: Option<i64>,
    /// creation time
    pub created_at: Date,
    /// Owner of this bump bot
    pub owner: i64,
    /// Address of this bump bot
    pub address: String,
    /// Target coin to be bumped
    pub mint: String,
    /// How many bump transactions will be included at once
    pub batch: i64,
    /// Fee for each transaction
    pub tx_fee: BigDecimal,
    /// Amount for each bump
    pub amount: BigDecimal,
    /// Duration for each bump in millis
    pub speed: i64,
    /// Count of history bumps
    pub bump: i64,
}
