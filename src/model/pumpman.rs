use crate::{config::PumpmanGlobal, sol::pump::SOL_SCALE, telegram::Result};
use bigdecimal::{BigDecimal, ToPrimitive};
use diesel::{pg::Pg, prelude::*};
use serde::{Deserialize, Serialize};
use teloxide::types::{InlineKeyboardButton, ReplyMarkup};
use time::{Date, Duration, OffsetDateTime};

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
pub struct Pumpman {
    /// Sequence id
    #[diesel(deserialize_as = i64)]
    pub id: Option<i64>,
    /// If the job is active
    pub active: bool,
    /// creation time
    pub created_at: Date,
    /// Owner of this bump bot
    pub owner: i64,
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

impl Pumpman {
    /// Create a new pumpman config
    pub fn new(global: &PumpmanGlobal, owner: i64, mint: String) -> Self {
        Self {
            id: None,
            active: false,
            created_at: OffsetDateTime::now_utc().date(),
            owner,
            mint,
            batch: 1,
            tx_fee: global.tx_fee.clone(),
            amount: global.amount.clone(),
            speed: global.speed,
            bump: 0,
        }
    }

    /// Calculate how much time can it go
    pub fn duration(&self, global: &PumpmanGlobal, balance: u64) -> String {
        let fee = self.amount.clone() / 50 + &global.fee + &self.tx_fee;
        let bumps = BigDecimal::from(balance) / SOL_SCALE / fee;
        let secs: BigDecimal = bumps * self.speed;
        let left = Duration::new(secs.to_i64().unwrap_or_default(), 0);
        let hours = left.whole_hours();
        let mins = left.whole_minutes();
        let mut duration = String::new();
        if hours > 0 {
            duration = format!("{hours} hours ");
        }

        if mins > 0 {
            duration = format!("{duration}{mins} mins");
        }

        if duration.is_empty() {
            return "0 mins".into();
        }

        duration
    }

    /// Show the markup from the current config
    pub fn markup(&self) -> Result<ReplyMarkup> {
        Ok(ReplyMarkup::inline_kb(vec![
            vec![InlineKeyboardButton::url(
                "Start",
                "http://pumpman.io".parse()?,
            )],
            vec![InlineKeyboardButton::url(
                "Batch 1 +",
                "https://pumpman.io".parse()?,
            )],
            vec![InlineKeyboardButton::url(
                "TX FEE +",
                "https://pumpman.io".parse()?,
            )],
            vec![InlineKeyboardButton::url(
                "SPEED +",
                "https://pumpman.io".parse()?,
            )],
            vec![InlineKeyboardButton::url(
                "BUMP AMOUNT +",
                "https://pumpman.io".parse()?,
            )],
        ]))
    }
}
