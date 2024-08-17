use std::str::FromStr;

use crate::{
    config,
    model::PumpmanGlobal,
    sol::pump::SOL_SCALE,
    telegram::{
        pumpman::callback::{Callback, JobCommand},
        Result,
    },
};
use bigdecimal::{BigDecimal, ToPrimitive};
use diesel::{pg::Pg, prelude::*};
use serde::{Deserialize, Serialize};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
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
    pub fn duration(&self, global: &config::PumpmanGlobal, balance: u64) -> String {
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
    pub fn markup(&self) -> Result<InlineKeyboardMarkup> {
        Ok(InlineKeyboardMarkup::new(vec![
            self.start_button()?,
            self.batch_button()?,
            self.tx_fee_button()?,
            self.amount_button()?,
            self.speed_button()?,
        ]))
    }

    /// Get the id of this job
    pub fn id(&self) -> i64 {
        self.id.unwrap_or_default()
    }

    fn speed_button(&self) -> Result<Vec<InlineKeyboardButton>> {
        Ok(vec![InlineKeyboardButton::callback(
            Speed::from(self.speed).format(),
            Callback::job(self.id(), JobCommand::Speed).format()?,
        )])
    }

    fn start_button(&self) -> Result<Vec<InlineKeyboardButton>> {
        let id = self.id();
        Ok(if self.active {
            vec![InlineKeyboardButton::callback(
                "Stop",
                Callback::job(id, JobCommand::Stop).format()?,
            )]
        } else {
            vec![InlineKeyboardButton::callback(
                "Start",
                Callback::job(id, JobCommand::Start).format()?,
            )]
        })
    }

    fn batch_button(&self) -> Result<Vec<InlineKeyboardButton>> {
        let id = self.id();
        let btn = InlineKeyboardButton::callback(
            format!("Batch {}", self.batch),
            Callback::DoNothing.format()?,
        );

        let up =
            InlineKeyboardButton::callback("+", Callback::job(id, JobCommand::BatchUp).format()?);

        let down =
            InlineKeyboardButton::callback("-", Callback::job(id, JobCommand::BatchDown).format()?);

        Ok(if self.batch == 1 {
            vec![btn, up]
        } else {
            vec![btn, down, up]
        })
    }

    fn tx_fee_button(&self) -> Result<Vec<InlineKeyboardButton>> {
        let id = self.id();
        let btn = InlineKeyboardButton::callback(
            format!("Tx Fee {}", self.tx_fee.round(6)),
            Callback::DoNothing.format()?,
        );

        let up =
            InlineKeyboardButton::callback("+", Callback::job(id, JobCommand::TxFeeUp).format()?);

        let down =
            InlineKeyboardButton::callback("-", Callback::job(id, JobCommand::TxFeeDown).format()?);

        Ok(
            if self.tx_fee.round(6) == BigDecimal::from_str("0.000045")?.round(6) {
                vec![btn, up]
            } else {
                vec![btn, down, up]
            },
        )
    }

    fn amount_button(&self) -> Result<Vec<InlineKeyboardButton>> {
        let id = self.id();
        let btn = InlineKeyboardButton::callback(
            format!("Bump Amount {} SOL", self.amount.round(3)),
            Callback::DoNothing.format()?,
        );

        let up =
            InlineKeyboardButton::callback("+", Callback::job(id, JobCommand::AmountUp).format()?);

        let down = InlineKeyboardButton::callback(
            "-",
            Callback::job(id, JobCommand::AmountDown).format()?,
        );

        Ok(
            if self.amount.round(3) == BigDecimal::from_str("0.01")?.round(3) {
                vec![btn, up]
            } else {
                vec![btn, down, up]
            },
        )
    }

    pub fn toggle_speed(&mut self) {
        self.speed = match self.speed {
            13 => 7,
            5 => 13,
            _ => 5,
        }
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
    pub const fn secs(&self) -> i64 {
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

impl From<i64> for Speed {
    fn from(s: i64) -> Self {
        match s {
            5 => Self::Fast,
            13 => Self::Low,
            _ => Self::Normal,
        }
    }
}
