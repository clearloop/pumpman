use std::str::FromStr;

use super::{message, BotDialogue, PumpmanContext};
use crate::{
    model::Pumpman,
    schema::{pumpman_global, pumpmen},
    telegram::Result,
    utils::base64,
};
use bigdecimal::BigDecimal;
use diesel::ExpressionMethods;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use teloxide::{
    payloads::{EditMessageReplyMarkupSetters, EditMessageTextSetters},
    prelude::Message,
    requests::Requester,
    types::{CallbackQuery, InlineKeyboardButton, ParseMode},
    Bot,
};

#[derive(Debug, Serialize, Deserialize)]
pub enum Callback {
    Job(CallbackJob),
    Global(CallbackGlobal),
    ListJobs,
    DoNothing,
    ShowJob,
    Withdraw(i64),
}

impl Callback {
    /// Construct callback job
    pub fn job(job: i64, command: JobCommand) -> Self {
        Self::Job(CallbackJob { job, command })
    }

    /// Construct callback job
    pub fn global(global: i64, command: JobCommand) -> Self {
        Self::Global(CallbackGlobal { global, command })
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
            Callback::Global(g) => return g.run(bot, context, msg).await,
            _ => {}
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CallbackJob {
    /// pumpman id
    pub job: i64,
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
            JobCommand::ShowJob => return self.show_job(bot, context, msg, job).await,
            JobCommand::Back => return self.back(bot, context, msg).await,
        };

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

    /// Only back to the list atm
    async fn back(&self, bot: Bot, context: PumpmanContext, msg: Message) -> Result<()> {
        let chat = msg.chat.id;
        let message = msg.id;
        let jobs = context.jobs(msg.chat.id.0).await?;
        let markup = message::list_markup(&context, &jobs).await?;

        bot.edit_message_text(chat, message, message::list(&jobs))
            .parse_mode(ParseMode::Html)
            .reply_markup(markup)
            .await?;

        Ok(())
    }

    async fn show_job(
        &self,
        bot: Bot,
        context: PumpmanContext,
        msg: Message,
        job: Pumpman,
    ) -> Result<()> {
        let mut markup = job.markup()?;
        markup
            .inline_keyboard
            .push(vec![InlineKeyboardButton::callback(
                "Back",
                Callback::job(job.id(), JobCommand::Back).format()?,
            )]);

        let chat = msg.chat.id;
        let message = msg.id;

        bot.edit_message_text(chat, message, message::job(&context, &job).await?)
            .parse_mode(ParseMode::Html)
            .reply_markup(markup)
            .await?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CallbackGlobal {
    global: i64,
    command: JobCommand,
}

impl CallbackGlobal {
    pub async fn run(&self, bot: Bot, context: PumpmanContext, msg: Message) -> Result<()> {
        let mut global = context.global_by_id(self.global).await?;
        match self.command {
            JobCommand::AmountDown => {
                global.amount = global.amount - BigDecimal::from_str("0.005")?
            }
            JobCommand::AmountUp => global.amount = global.amount + BigDecimal::from_str("0.005")?,
            JobCommand::BatchDown => global.batch = global.batch - 1,
            JobCommand::BatchUp => global.batch = global.batch + 1,
            JobCommand::TxFeeDown => {
                global.tx_fee = global.tx_fee - BigDecimal::from_str("0.000010")?
            }
            JobCommand::TxFeeUp => {
                global.tx_fee = global.tx_fee + BigDecimal::from_str("0.000010")?
            }
            JobCommand::Speed => global.toggle_speed(),
            _ => {}
        };

        diesel::update(pumpman_global::table)
            .filter(pumpman_global::owner.eq(global.owner))
            .set(&global)
            .execute(&mut context.postgres().await?)
            .await?;

        bot.edit_message_reply_markup(msg.chat.id, msg.id)
            .reply_markup(global.markup()?)
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
    Back,
    ShowJob,
}

/// Back to previous state
#[derive(Debug, Serialize, Deserialize, Default)]
pub enum Back {
    /// Back to list
    #[default]
    List,
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
