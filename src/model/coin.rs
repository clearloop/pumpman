//! Basic coin information

use crate::schema::coins;
use async_graphql::SimpleObject;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// Original coin information
#[derive(
    SimpleObject,
    Insertable,
    Queryable,
    Selectable,
    AsChangeset,
    Clone,
    PartialEq,
    Debug,
    Serialize,
    Default,
    Eq,
    PartialOrd,
    Ord,
    Hash,
)]
pub struct Coin {
    /// Address of this token
    pub address: String,
    /// Name of this token
    pub name: String,
    /// Symbol of this token
    pub symbol: String,
    /// Image of this token
    pub image: String,
}
