use crate::schema::users;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use solana_sdk::signature::Keypair;
use time::{Date, OffsetDateTime};

/// Pumpman users
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
pub struct User {
    /// Sequence id
    #[diesel(deserialize_as = i64)]
    pub id: Option<i64>,
    /// creation time
    pub created_at: Date,
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
            created_at: OffsetDateTime::now_utc().date(),
            tgid,
            wallet: bs58::encode(Keypair::new().to_bytes()).into_string(),
        }
    }
}
