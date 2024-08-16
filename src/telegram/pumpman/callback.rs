use std::str::FromStr;

use super::{BotDialogue, PumpmanContext};
use crate::{schema::pumpmen, telegram::Result, utils::base64};
use bigdecimal::BigDecimal;
use diesel::ExpressionMethods;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use teloxide::{
    payloads::EditMessageReplyMarkupSetters, prelude::Message, requests::Requester,
    types::CallbackQuery, Bot,
};

#[derive(Debug, Serialize, Deserialize)]
pub enum Callback {
    Job(CallbackJob),
    ListJobs,
    DoNothing,
    Withdraw(i64),
}

impl Callback {
    /// Construct callback job
    pub fn job(job: i64, command: JobCommand) -> Self {
        Self::Job(CallbackJob { job, command })
    }

    pub fn from_callback(cb: &CallbackQuery) -> Result<Self> {
        let Some(data) = &cb.data else {
            return Ok(Self::DoNothing);
        };

        bitcode::deserialize(&base64::decode(&data)?).map_err(Into::into)
    }

    pub fn format(&self) -> Result<String> {
        let r = base64::encode(&bitcode::serialize(&self)?);
        Ok(r)
    }

    pub async fn run(&self, bot: Bot, context: PumpmanContext, msg: Message) -> Result<()> {
        match self {
            Callback::DoNothing => {}
            Callback::Job(j) => return j.run(bot, context, msg).await,
            _ => {}
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CallbackJob {
    job: i64,
    command: JobCommand,
}

impl CallbackJob {
    pub async fn run(&self, bot: Bot, context: PumpmanContext, msg: Message) -> Result<()> {
        let mut job = context.job_by_id(self.job).await?;
        match self.command {
            JobCommand::Start => job.active = true,
            JobCommand::Stop => job.active = false,
            JobCommand::AmountDown => job.amount = job.amount - BigDecimal::from_str("0.005")?,
            JobCommand::AmountUp => job.amount = job.amount + BigDecimal::from_str("0.005")?,
            JobCommand::BatchDown => job.batch = job.batch - 1,
            JobCommand::BatchUp => job.batch = job.batch + 1,
            JobCommand::TxFeeDown => job.tx_fee = job.tx_fee - BigDecimal::from_str("0.000010")?,
            JobCommand::TxFeeUp => job.tx_fee = job.tx_fee + BigDecimal::from_str("0.000010")?,
            JobCommand::Speed => job.toggle_speed(),
        }

        diesel::update(pumpmen::table)
            .filter(pumpmen::id.eq(job.id.unwrap_or_default()))
            .set(&job)
            .execute(&mut context.postgres().await?)
            .await?;

        bot.edit_message_reply_markup(msg.chat.id, msg.id)
            .reply_markup(job.markup()?)
            .await?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub enum JobCommand {
    #[default]
    Start,
    Stop,
    AmountUp,
    AmountDown,
    BatchUp,
    BatchDown,
    TxFeeUp,
    TxFeeDown,
    Speed,
}

pub async fn handle(
    bot: Bot,
    context: PumpmanContext,
    _dialogue: BotDialogue,
    q: CallbackQuery,
) -> Result<()> {
    let cb = Callback::from_callback(&q)?;
    let Some(msg) = q.message else { return Ok(()) };

    cb.run(bot, context, msg).await
}
