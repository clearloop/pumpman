//! Community take over
use crate::{model::Coin, schema::takeovers};
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
#[diesel(belongs_to(Coin, foreign_key = address))]
#[diesel(primary_key(id))]
pub struct Takeover {
    /// Sequence id
    #[diesel(deserialize_as = i64)]
    pub id: Option<i64>,
    /// The address of the coin to be taken over
    pub address: String,
    /// Banner of the cto
    pub banner: Option<String>,
    /// Telegram link
    pub telegram: String,
    /// Twitter link
    pub twitter: Option<String>,
    /// Website link
    pub website: Option<String>,
}

impl Takeover {
    pub fn new(address: String) -> Self {
        Self {
            address,
            ..Default::default()
        }
    }
}
