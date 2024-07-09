//! Community take over
use crate::schema::takeovers;
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
}
