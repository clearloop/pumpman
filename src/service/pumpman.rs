use crate::{
    api::PumpApi,
    context::Context,
    model::{Pumpman, Speed},
    schema::pumpmen,
    telegram::{self, pumpman::PumpmanContext},
    Config,
};
use anyhow::Result;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use std::time::Duration;
use teloxide::Bot;
use tokio::{signal, task};

/// Start pumpman service
pub async fn start(mut config: Config, context: Context) -> Result<()> {
    let Some(mut pumpman) = config.pumpman.take() else {
        tracing::error!("pumpman config not found");
        return Ok(());
    };

    let Some(bot) = pumpman.bot.take() else {
        tracing::error!("pumpman bot not found");
        return Ok(());
    };

    let bot = Bot::new(&bot);
    let context = PumpmanContext::new(context, pumpman.global.clone());
    tracing::info!("Starting pumpman service ...");

    loop {
        let r = tokio::select! {
            _ = signal::ctrl_c() => break,
            r = bumping(context.clone(), Speed::Fast) => r,
            r = bumping(context.clone(), Speed::Normal) => r,
            r = bumping(context.clone(), Speed::Low) => r,
            r = async {
                loop {
                    if let Err(e) = telegram::pumpman::start(&bot, context.clone(), config.database.redis.to_string()).await {
                        tracing::error!("{e}");
                        tokio::time::sleep(Duration::from_secs(3)).await;
                    }
                }
            }=> r,
        };

        if let Err(e) = r {
            tracing::error!("{e}");
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }

    Ok(())
}

/// loop for bumping coins
async fn bumping(context: PumpmanContext, speed: Speed) -> Result<()> {
    let postgres = &mut context.postgres().await?;
    loop {
        let jobs = pumpmen::table
            .filter(pumpmen::active)
            .filter(pumpmen::speed.eq(speed.secs()))
            .get_results::<Pumpman>(postgres)
            .await?;

        let global = context.client.global().await?;
        for job in jobs {
            let context = context.clone();
            task::spawn(async move {
                let job_id = job.id();
                if let Err(e) = context.bump(&global, &job).await {
                    tracing::warn!("job {job_id} failed: {e:?}");

                    if let Err(e) = context.stop(job.id()).await {
                        tracing::error!("Failed to stop job {job_id}: {e:?}");
                    }
                }
            });
        }

        tokio::time::sleep(Duration::from_secs(speed.secs() as u64)).await;
    }
}
