use crate::telegram::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Job {
    tgid: i64,
    mint: String,
}

#[derive(Serialize, Deserialize)]
pub struct CallbackJob {
    job: Job,
    command: JobCommand,
}

#[derive(Serialize, Deserialize, Default)]
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

#[derive(Serialize, Deserialize)]
pub enum Callback {
    Job(CallbackJob),
    ListJobs,
    DoNothing,
}

impl Callback {
    /// Construct callback job
    pub fn job(tgid: i64, mint: &str, command: JobCommand) -> Self {
        Self::Job(CallbackJob {
            job: Job {
                tgid,
                mint: mint.into(),
            },
            command,
        })
    }

    pub fn format(&self) -> Result<String> {
        serde_json::to_string(&Callback::DoNothing).map_err(Into::into)
    }
}
