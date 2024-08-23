use bigdecimal::{BigDecimal, ToPrimitive};
use diesel::{pg::Pg, prelude::*};
use serde::{Deserialize, Serialize};
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use time::{Date, Duration};

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
    /// Target coin to be bumped
    pub mint: String,
    /// Owner of this bump bot
    pub owner: i64,
    /// Specific wallet for this job
    pub wallet: Option<String>,
    /// creation time
    pub created_at: Date,
    /// If the job is active
    pub active: bool,
    /// Amount for each bump
    pub amount: BigDecimal,
    /// Fee for each transaction
    pub priority_fee: BigDecimal,
    /// How many bumps will be included at once
    pub batch: i32,
    /// Duration for each bump in millis
    pub speed: i32,
    /// Count of history bumps
    pub bumps: i64,
    /// How much service fee have been charged from this job
    pub charged: BigDecimal,
}

impl Pumpman {
    pub const BASIC_UNITS: u32 = 134;
    pub const CACA_UNITS: u32 = 23203;
    /// 82872 units in empty bonding curve
    pub const BUMP_UNITS: u32 = 100_000;
    pub const TRANSFER_UNITS: u32 = 150;
    pub const BUDGET_UNITS: u32 = 300;

    /// Calculate how much time can it go
    pub fn duration(&self, fee: &BigDecimal, balance: u64) -> String {
        let bumps = BigDecimal::from(balance) / LAMPORTS_PER_SOL / fee;
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

    /// Get the id of this job
    pub fn id(&self) -> i64 {
        self.id.unwrap_or_default()
    }
}

#[derive(Debug)]
pub enum Speed {
    Low,
    Normal,
    Fast,
}

impl Speed {
    /// Get the secs repl
    pub const fn secs(&self) -> i32 {
        match self {
            Self::Low => 13,
            Self::Normal => 7,
            Self::Fast => 5,
        }
    }

    /// format to string
    pub fn format(&self) -> String {
        format!("Speed {self:?} ({}s)", self.secs())
    }
}

impl From<i32> for Speed {
    fn from(s: i32) -> Self {
        match s {
            5 => Self::Fast,
            13 => Self::Low,
            _ => Self::Normal,
        }
    }
}
