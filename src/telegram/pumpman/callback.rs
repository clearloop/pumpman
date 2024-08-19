use super::{message, state::State, BotDialogue, PumpmanContext};
use crate::{
    model::PumpmanJob,
    schema::{pumpman_global, pumpmen},
    telegram::Result,
    utils::base64,
};
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
    DoNothing,
    Withdraw,
    BackToList,
    Job { job: i64, command: JobCommand },
    Global(JobCommand),
    ShowJob(i64),
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
                command.handle_job(bot, dialogue, context, msg, *job).await
            }
            Callback::Global(command) => command.handle_global(bot, context, msg).await,
            Callback::BackToList => Self::back(bot, context, msg).await,
            Callback::ShowJob(id) => Self::show_job(bot, dialogue, context, msg, *id).await,
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

    async fn show_job(
        bot: Bot,
        dialogue: BotDialogue,
        context: PumpmanContext,
        msg: Message,
        id: i64,
    ) -> Result<()> {
        let job = context.job_by_id(id).await?;
        dialogue.update(State::BackToList).await?;
        let mut markup = job.markup(&context.global)?;
        markup
            .inline_keyboard
            .push(vec![InlineKeyboardButton::callback(
                "Back",
                Callback::BackToList.format()?,
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

#[derive(Debug, Serialize, Deserialize, Default)]
pub enum JobCommand {
    #[default]
    Start,
    Stop,
    AmountUp,
    AmountDown,
    BatchUp,
    BatchDown,
    PriorityFeeUp,
    PriorityFeeDown,
    Speed,
}

impl JobCommand {
    pub async fn handle_job(
        &self,
        bot: Bot,
        dialogue: BotDialogue,
        context: PumpmanContext,
        msg: Message,
        job_id: i64,
    ) -> Result<()> {
        let mut job = context.job_by_id(job_id).await?;
        job.apply_command(&self, &context.global);

        diesel::update(pumpmen::table)
            .filter(pumpmen::id.eq(job.id.unwrap_or_default()))
            .set(&job)
            .execute(&mut context.postgres().await?)
            .await?;

        let mut markup = job.markup(&context.global)?;
        if dialogue.get().await? == Some(State::BackToList) {
            markup
                .inline_keyboard
                .push(vec![InlineKeyboardButton::callback(
                    "Back",
                    Callback::BackToList.format()?,
                )]);
        }

        bot.edit_message_reply_markup(msg.chat.id, msg.id)
            .reply_markup(markup)
            .await?;
        Ok(())
    }

    pub async fn handle_global(
        &self,
        bot: Bot,
        context: PumpmanContext,
        msg: Message,
    ) -> Result<()> {
        let mut global = context.pglobal(msg.chat.id.0).await?;
        global.apply_command(&self, &context.global);

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
