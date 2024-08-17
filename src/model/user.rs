use crate::schema::users;
use async_graphql::SimpleObject;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use solana_sdk::signature::Keypair;

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
    pub tgid: i64,
    /// User keypair
    pub wallet: String,
}

impl User {
    /// Create a new user
    pub fn new(tgid: i64) -> Self {
        Self {
            id: None,
            tgid,
            wallet: bs58::encode(Keypair::new().to_bytes()).into_string(),
        }
    }
}
