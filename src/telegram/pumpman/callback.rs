use super::{message, state::State, BotDialogue, PumpmanContext};
use crate::{
    model::{Pumpman, PumpmanJob},
    schema::{pumpman_global, pumpmen},
    telegram::Result,
    utils::base64,
};
use diesel::ExpressionMethods;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use teloxide::{
    payloads::EditMessageTextSetters,
    prelude::Message,
    requests::Requester,
    types::{CallbackQuery, InlineKeyboardButton, ParseMode},
    Bot,
};

#[derive(Debug, Serialize, Deserialize)]
pub enum Callback {
    DoNothing,
    Withdraw,
    BackToList,
    Job { job: i64, command: JobCommand },
    Global(JobCommand),
    List(ListCallback),
}

impl Callback {
    pub async fn handle(
        bot: Bot,
        dialogue: BotDialogue,
        context: PumpmanContext,
        q: CallbackQuery,
    ) -> Result<()> {
        let cb = Callback::from_callback(&q)?;
        let Some(msg) = q.message else { return Ok(()) };

        cb.run(bot, dialogue, context, msg).await
    }

    pub async fn run(
        &self,
        bot: Bot,
        dialogue: BotDialogue,
        context: PumpmanContext,
        msg: Message,
    ) -> Result<()> {
        match self {
            Callback::Job { command, job } => {
                command
                    .handle_job(&bot, dialogue, &context, &msg, *job)
                    .await
            }
            Callback::Global(command) => command.handle_global(bot, context, msg).await,
            Callback::List(l) => l.handle(bot, dialogue, context, msg).await,
            Callback::BackToList => Self::back(bot, context, msg).await,
            _ => Ok(()),
        }
    }

    pub fn job(command: JobCommand, job: Option<i64>) -> Self {
        if let Some(job) = job {
            Self::Job { job, command }
        } else {
            Self::Global(command)
        }
    }

    pub fn list(cb: ListCallback) -> Self {
        Self::List(cb)
    }

    pub fn from_callback(cb: &CallbackQuery) -> Result<Self> {
        let Some(data) = &cb.data else {
            return Ok(Self::DoNothing);
        };

        bitcode::deserialize(&base64::decode(data)?).map_err(Into::into)
    }

    pub fn format(&self) -> Result<String> {
        let r = base64::encode(&bitcode::serialize(&self)?);
        Ok(r)
    }

    /// Only back to the list atm
    async fn back(bot: Bot, context: PumpmanContext, msg: Message) -> Result<()> {
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
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub enum JobCommand {
    #[default]
    Start,
    Stop,
    AmountUp,
    AmountDown,
    AmountRandom,
    AmountReset,
    BatchUp,
    BatchDown,
    BatchRandom,
    BatchReset,
    PriorityFeeUp,
    PriorityFeeDown,
    PriorityFeeRandom,
    PriorityFeeReset,
    Speed,
}

impl JobCommand {
    pub async fn show_job(
        bot: &Bot,
        dialogue: BotDialogue,
        context: &PumpmanContext,
        msg: &Message,
        job: Pumpman,
    ) -> Result<()> {
        let state = dialogue.get().await?;
        if state == Some(State::NoUpdateMarkup) {
            dialogue.update(State::Start).await?;
            return Ok(());
        }

        let mut markup = job.markup(&context.global)?;
        if state == Some(State::BackToList) {
            dialogue.update(State::Start).await?;
            markup
                .inline_keyboard
                .push(vec![InlineKeyboardButton::callback(
                    "Back",
                    Callback::BackToList.format()?,
                )]);
        }

        let chat = msg.chat.id;
        let message = msg.id;

        bot.edit_message_text(chat, message, message::job(context, &job).await?)
            .parse_mode(ParseMode::Html)
            .reply_markup(markup)
            .await?;
        Ok(())
    }

    pub async fn handle_job(
        &self,
        bot: &Bot,
        dialogue: BotDialogue,
        context: &PumpmanContext,
        msg: &Message,
        job_id: i64,
    ) -> Result<()> {
        let mut job = context.job_by_id(job_id).await?;
        job.apply_command(self, &context.global);

        diesel::update(pumpmen::table)
            .filter(pumpmen::id.eq(job.id.unwrap_or_default()))
            .set(&job)
            .execute(&mut context.postgres().await?)
            .await?;

        Self::show_job(bot, dialogue, context, msg, job).await
    }

    pub async fn handle_global(
        &self,
        bot: Bot,
        context: PumpmanContext,
        msg: Message,
    ) -> Result<()> {
        let mut global = context.pglobal(msg.chat.id.0).await?;
        global.apply_command(self, &context.global);

        diesel::update(pumpman_global::table)
            .filter(pumpman_global::owner.eq(global.owner))
            .set(&global)
            .execute(&mut context.postgres().await?)
            .await?;

        bot.edit_message_text(
            msg.chat.id,
            msg.id,
            message::config(&context, &global).await?,
        )
        .parse_mode(ParseMode::Html)
        .reply_markup(global.markup(&context.global)?)
        .await?;

        Ok(())
    }
}

/// list call back
#[derive(Debug, Serialize, Deserialize)]
pub enum ListCallback {
    ShowJob(i64),
    Start(i64),
    Stop(i64),
    StartAll,
    StopAll,
}

impl ListCallback {
    /// handle list callbacks
    pub async fn handle(
        &self,
        bot: Bot,
        dialogue: BotDialogue,
        context: PumpmanContext,
        msg: Message,
    ) -> Result<()> {
        match self {
            ListCallback::StartAll => Self::all(bot, context, msg, true).await,
            ListCallback::StopAll => Self::all(bot, context, msg, false).await,
            ListCallback::ShowJob(id) => {
                dialogue.update(State::BackToList).await?;

                let job = context.job_by_id(*id).await?;
                JobCommand::show_job(&bot, dialogue, &context, &msg, job).await
            }
            ListCallback::Start(id) => {
                dialogue.update(State::BackToList).await?;
                JobCommand::handle_job(&JobCommand::Start, &bot, dialogue, &context, &msg, *id)
                    .await
            }
            ListCallback::Stop(id) => {
                dialogue.update(State::NoUpdateMarkup).await?;
                JobCommand::handle_job(&JobCommand::Stop, &bot, dialogue, &context, &msg, *id)
                    .await?;
                Self::update_list(bot, context, msg).await
            }
        }
    }

    async fn update_list(bot: Bot, context: PumpmanContext, msg: Message) -> Result<()> {
        let jobs = context.jobs(msg.chat.id.0).await?;
        bot.edit_message_text(msg.chat.id, msg.id, message::list(&jobs))
            .parse_mode(ParseMode::Html)
            .reply_markup(message::list_markup(&context, &jobs).await?)
            .await?;
        Ok(())
    }

    /// Start or stop all bots
    async fn all(bot: Bot, context: PumpmanContext, msg: Message, start: bool) -> Result<()> {
        let postgres = &mut context.postgres().await?;
        diesel::update(pumpmen::table)
            .filter(pumpmen::owner.eq(msg.chat.id.0))
            .set(pumpmen::active.eq(start))
            .execute(postgres)
            .await?;

        Self::update_list(bot, context, msg).await
    }
}
