//! Community take over
use crate::schema::coins;
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
pub struct Coin {
    /// Sequence id
    #[diesel(deserialize_as = i64)]
    pub id: Option<i64>,
    /// Mint address of this coin
    pub mint: String,
}

impl Coin {
    /// New community takeover
    pub fn new(mint: String) -> Self {
        Self {
            mint,
            ..Default::default()
        }
    }
}
