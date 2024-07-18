//! Community take over
use crate::{model::pump, schema::coins};
use async_graphql::SimpleObject;
use diesel::{pg::Pg, prelude::*};
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
#[diesel(check_for_backend(Pg))]
pub struct Coin {
    /// Sequence id
    #[diesel(deserialize_as = i64)]
    pub id: Option<i64>,
    /// Mint address of this coin
    pub mint: String,
    /// Name of the coin
    pub name: String,
    /// Symbol of the coin
    pub symbol: String,
    pub telegram: Option<String>,
    pub twitter: Option<String>,
    pub website: Option<String>,
}

impl From<pump::Coin> for Coin {
    fn from(pump: pump::Coin) -> Self {
        Self {
            id: None,
            mint: pump.mint,
            name: pump.name,
            symbol: pump.symbol,
            telegram: pump.telegram,
            twitter: pump.twitter,
            website: pump.website,
        }
    }
}
