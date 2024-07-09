use crate::schema::users;
use async_graphql::SimpleObject;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// Replika users
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
pub struct User {
    /// Sequence id
    #[diesel(deserialize_as = i64)]
    pub id: Option<i64>,
    /// Telegram user id
    pub tgid: String,
    /// Credits this user has
    pub credits: i64,
}
