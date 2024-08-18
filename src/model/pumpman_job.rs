use crate::{
    config,
    model::{pumpman::Speed, Pumpman, PumpmanGlobal},
    sol::utils::LAMPORTS_PER_SIGNATURE,
    telegram::{
        pumpman::callback::{Callback, JobCommand},
        Result,
    },
};
use bigdecimal::{BigDecimal, Zero};
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use teloxide::types::InlineKeyboardButton;

/// Shared interfaces for pumpman jobs
pub trait PumpmanJob {
    fn amount(&self) -> &BigDecimal;

    fn amount_mut(&mut self) -> &mut BigDecimal;

    fn priority_fee(&self) -> &BigDecimal;

    fn priority_fee_mut(&mut self) -> &mut BigDecimal;

    fn slippage(&self) -> i32;

    fn slippage_mut(&mut self) -> &mut i32;

    fn batch(&self) -> i32;

    fn batch_mut(&mut self) -> &mut i32;

    fn speed(&self) -> i32;

    fn speed_mut(&mut self) -> &mut i32;

    fn active(&self) -> bool {
        false
    }

    fn set_active(&mut self, _active: bool) {}

    fn job_id(&self) -> Option<i64> {
        None
    }

    fn tx_fee(&self) -> BigDecimal {
        self.priority_fee() + BigDecimal::from(LAMPORTS_PER_SIGNATURE) / LAMPORTS_PER_SOL
    }

    fn speed_button(&self) -> Result<Vec<InlineKeyboardButton>> {
        Ok(vec![InlineKeyboardButton::callback(
            Speed::from(self.speed()).format(),
            Callback::job(JobCommand::Speed, self.job_id()).format()?,
        )])
    }

    fn start_button(&self) -> Result<Vec<InlineKeyboardButton>> {
        let id = self.job_id();
        Ok(if self.active() {
            vec![InlineKeyboardButton::callback(
                "Stop",
                Callback::job(JobCommand::Stop, id).format()?,
            )]
        } else {
            vec![InlineKeyboardButton::callback(
                "Start",
                Callback::job(JobCommand::Start, id).format()?,
            )]
        })
    }

    fn batch_button(&self, global: &config::PumpmanGlobal) -> Result<Vec<InlineKeyboardButton>> {
        let id = self.job_id();
        let batch = self.batch();
        let btn = InlineKeyboardButton::callback(
            format!("Batch Bumps {}", batch),
            Callback::DoNothing.format()?,
        );

        let up =
            InlineKeyboardButton::callback("+", Callback::job(JobCommand::BatchUp, id).format()?);

        let down =
            InlineKeyboardButton::callback("-", Callback::job(JobCommand::BatchDown, id).format()?);

        Ok(if batch == 1 {
            vec![btn, up]
        } else if batch >= global.batch {
            vec![btn, down]
        } else {
            vec![btn, down, up]
        })
    }

    fn slippage_button(&self) -> Result<Vec<InlineKeyboardButton>> {
        let id = self.job_id();
        let slippage = self.slippage();
        let btn = InlineKeyboardButton::callback(
            format!("Slippage {}%", slippage),
            Callback::DoNothing.format()?,
        );

        let up = InlineKeyboardButton::callback(
            "+",
            Callback::job(JobCommand::SlippageUp, id).format()?,
        );

        let down = InlineKeyboardButton::callback(
            "-",
            Callback::job(JobCommand::SlippageDown, id).format()?,
        );

        Ok(if slippage == 1 {
            vec![btn, up]
        } else {
            vec![btn, down, up]
        })
    }

    fn tx_fee_button(&self) -> Result<Vec<InlineKeyboardButton>> {
        let id = self.job_id();
        let btn = InlineKeyboardButton::callback(
            format!("Tx Fee {}", self.tx_fee().round(6)),
            Callback::DoNothing.format()?,
        );

        let up = InlineKeyboardButton::callback(
            "+",
            Callback::job(JobCommand::PriorityFeeUp, id).format()?,
        );

        let down = InlineKeyboardButton::callback(
            "-",
            Callback::job(JobCommand::PriorityFeeDown, id).format()?,
        );

        Ok(if self.priority_fee().le(&BigDecimal::zero()) {
            vec![btn, up]
        } else {
            vec![btn, down, up]
        })
    }

    fn amount_button(&self, global: &config::PumpmanGlobal) -> Result<Vec<InlineKeyboardButton>> {
        let id = self.job_id();
        let amount = self.amount().round(3);
        let btn = InlineKeyboardButton::callback(
            format!("Bump Amount {}", amount),
            Callback::DoNothing.format()?,
        );

        let up =
            InlineKeyboardButton::callback("+", Callback::job(JobCommand::AmountUp, id).format()?);

        let down = InlineKeyboardButton::callback(
            "-",
            Callback::job(JobCommand::AmountDown, id).format()?,
        );

        Ok(if amount.round(3) == global.amount.round(3) {
            vec![btn, up]
        } else {
            vec![btn, down, up]
        })
    }

    fn toggle_speed(&mut self) {
        *self.speed_mut() = match self.speed() {
            13 => 7,
            5 => 13,
            _ => 5,
        }
    }

    fn apply_command(&mut self, command: &JobCommand, global: &config::PumpmanGlobal) {
        match command {
            JobCommand::Start => self.set_active(true),
            JobCommand::Stop => self.set_active(false),
            JobCommand::AmountDown => *self.amount_mut() -= &global.amount_step,
            JobCommand::AmountUp => *self.amount_mut() += &global.amount_step,
            JobCommand::BatchDown => *self.batch_mut() -= 1,
            JobCommand::BatchUp => *self.batch_mut() += 1,
            JobCommand::PriorityFeeDown => *self.priority_fee_mut() -= &global.priority_fee_step,
            JobCommand::PriorityFeeUp => *self.priority_fee_mut() += &global.priority_fee_step,
            JobCommand::SlippageDown => *self.slippage_mut() -= 1,
            JobCommand::SlippageUp => *self.slippage_mut() += 1,
            JobCommand::Speed => self.toggle_speed(),
        }
    }
}

impl PumpmanJob for Pumpman {
    fn amount(&self) -> &BigDecimal {
        &self.amount
    }

    fn amount_mut(&mut self) -> &mut BigDecimal {
        &mut self.amount
    }

    fn priority_fee(&self) -> &BigDecimal {
        &self.priority_fee
    }

    fn priority_fee_mut(&mut self) -> &mut BigDecimal {
        &mut self.priority_fee
    }

    fn slippage(&self) -> i32 {
        self.slippage
    }

    fn slippage_mut(&mut self) -> &mut i32 {
        &mut self.slippage
    }

    fn batch(&self) -> i32 {
        self.batch
    }

    fn batch_mut(&mut self) -> &mut i32 {
        &mut self.batch
    }

    fn speed(&self) -> i32 {
        self.speed
    }

    fn speed_mut(&mut self) -> &mut i32 {
        &mut self.speed
    }

    fn active(&self) -> bool {
        self.active
    }

    fn set_active(&mut self, active: bool) {
        self.active = active
    }

    fn job_id(&self) -> Option<i64> {
        self.id
    }
}

impl PumpmanJob for PumpmanGlobal {
    fn amount(&self) -> &BigDecimal {
        &self.amount
    }

    fn amount_mut(&mut self) -> &mut BigDecimal {
        &mut self.amount
    }

    fn priority_fee(&self) -> &BigDecimal {
        &self.priority_fee
    }

    fn priority_fee_mut(&mut self) -> &mut BigDecimal {
        &mut self.priority_fee
    }

    fn slippage(&self) -> i32 {
        self.slippage
    }

    fn slippage_mut(&mut self) -> &mut i32 {
        &mut self.slippage
    }

    fn batch(&self) -> i32 {
        self.batch
    }

    fn batch_mut(&mut self) -> &mut i32 {
        &mut self.batch
    }

    fn speed(&self) -> i32 {
        self.speed
    }

    fn speed_mut(&mut self) -> &mut i32 {
        &mut self.speed
    }
}
