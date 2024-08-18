use crate::{
    config,
    model::pumpman::{Pumpman, Speed},
    telegram::{
        pumpman::callback::{Callback, JobCommand},
        Result,
    },
};
use bigdecimal::BigDecimal;
use diesel::{pg::Pg, prelude::*};
use serde::{Deserialize, Serialize};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use time::OffsetDateTime;

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
#[diesel(table_name = crate::schema::pumpman_global)]
pub struct PumpmanGlobal {
    /// Sequence id
    #[diesel(deserialize_as = i64)]
    pub id: Option<i64>,
    /// Owner of this bump bot
    pub owner: i64,
    /// Fee for each transaction
    pub tx_fee: BigDecimal,
    /// Amount for each bump
    pub amount: BigDecimal,
    /// How many bump transactions will be included at once
    pub batch: i32,
    /// Duration for each bump in millis
    pub speed: i32,
}

impl PumpmanGlobal {
    /// Create a new pumpman config
    pub fn new(global: &config::PumpmanGlobal, owner: i64) -> Self {
        Self {
            id: None,
            owner,
            tx_fee: global.tx_fee.clone(),
            amount: global.amount.clone(),
            batch: 1,
            speed: global.speed,
        }
    }

    /// Create a new pumpman config from global
    pub fn generate(&self, mint: &str) -> Pumpman {
        Pumpman {
            id: None,
            active: false,
            created_at: OffsetDateTime::now_utc().date(),
            owner: self.owner,
            mint: mint.into(),
            tx_fee: self.tx_fee.clone(),
            amount: self.amount.clone(),
            batch: self.batch,
            speed: self.speed,
            bumps: 0,
            wallet: None,
        }
    }

    /// Total fee per bump
    pub fn total_fee(&self, fee: &BigDecimal) -> BigDecimal {
        let pf_fee = self.amount.clone() / 50u32;
        pf_fee.clone() + &self.tx_fee + fee
    }

    /// Show the markup from the current config
    pub fn markup(&self, global: &config::PumpmanGlobal) -> Result<InlineKeyboardMarkup> {
        Ok(InlineKeyboardMarkup::new(vec![
            self.batch_button(global)?,
            self.tx_fee_button(global)?,
            self.amount_button(global)?,
            self.speed_button()?,
        ]))
    }

    fn speed_button(&self) -> Result<Vec<InlineKeyboardButton>> {
        Ok(vec![InlineKeyboardButton::callback(
            Speed::from(self.speed).format(),
            Callback::Global(JobCommand::Speed).format()?,
        )])
    }

    fn batch_button(&self, global: &config::PumpmanGlobal) -> Result<Vec<InlineKeyboardButton>> {
        let btn = InlineKeyboardButton::callback(
            format!("Batch {}", self.batch),
            Callback::DoNothing.format()?,
        );

        let up =
            InlineKeyboardButton::callback("+", Callback::Global(JobCommand::BatchUp).format()?);

        let down =
            InlineKeyboardButton::callback("-", Callback::Global(JobCommand::BatchDown).format()?);

        Ok(if self.batch == global.batch {
            vec![btn, up]
        } else {
            vec![btn, down, up]
        })
    }

    fn tx_fee_button(&self, global: &config::PumpmanGlobal) -> Result<Vec<InlineKeyboardButton>> {
        let btn = InlineKeyboardButton::callback(
            format!("Tx Fee {}", self.tx_fee.round(6)),
            Callback::DoNothing.format()?,
        );

        let up =
            InlineKeyboardButton::callback("+", Callback::Global(JobCommand::TxFeeUp).format()?);

        let down =
            InlineKeyboardButton::callback("-", Callback::Global(JobCommand::TxFeeDown).format()?);

        Ok(if self.tx_fee.round(6) == global.tx_fee.round(6) {
            vec![btn, up]
        } else {
            vec![btn, down, up]
        })
    }

    fn amount_button(&self, global: &config::PumpmanGlobal) -> Result<Vec<InlineKeyboardButton>> {
        let btn = InlineKeyboardButton::callback(
            format!("Bump Amount {} SOL", self.amount.round(3)),
            Callback::DoNothing.format()?,
        );

        let up =
            InlineKeyboardButton::callback("+", Callback::Global(JobCommand::AmountUp).format()?);

        let down =
            InlineKeyboardButton::callback("-", Callback::Global(JobCommand::AmountDown).format()?);

        Ok(if self.amount.round(3) == global.amount.round(3) {
            vec![btn, up]
        } else {
            vec![btn, down, up]
        })
    }

    pub fn toggle_speed(&mut self) {
        self.speed = match self.speed {
            13 => 7,
            5 => 13,
            _ => 5,
        }
    }
}
