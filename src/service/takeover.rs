//! Task processor

use crate::{
    config,
    context::Context,
    service::{Event, PumpEvent},
    telegram::takeover,
};
use anyhow::Result;
use std::time::Duration;
use teloxide::Bot;
use tokio::sync::mpsc::Receiver;

/// Pump events handler
pub struct Takeover {
    config: config::Takeover,
    context: Context,
    rx: Receiver<Event>,
}

impl Takeover {
    /// Create new takeover service
    pub fn new(config: &config::Takeover, context: Context, rx: Receiver<Event>) -> Self {
        Self {
            config: config.clone(),
            context,
            rx,
        }
    }

    /// Start the reporter service
    pub async fn start(&mut self, redis: String) -> Result<()> {
        let Some(reporter) = self.config.bot.take() else {
            tracing::warn!("Takeover alerts is disabled since the bot token is not set.");
            loop {
                tokio::time::sleep(Duration::from_secs(86400)).await
            }
        };

        tracing::trace!("Starting takeover service ...");
        let bot = Bot::new(reporter);
        if self.config.registry {
            takeover::start(&bot, self.context.clone(), redis).await?;
        }

        while let Some(event) = self.rx.recv().await {
            tracing::trace!("Received event: {event:?}");
            if let Err(e) = match event {
                Event::Pump(PumpEvent::DevSoldout(mints)) => self.soldout(&bot, mints).await,
            } {
                tracing::warn!("Failed to process event, error: {e}");
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }

        Ok(())
    }

    /// split mints into windows
    async fn soldout(&self, bot: &Bot, mints: Vec<String>) -> Result<()> {
        let len = mints.len();
        let mut ptr = 0;

        while ptr < len {
            let to = (ptr + self.config.batch).min(len);
            futures::future::join_all(
                mints[ptr..to]
                    .iter()
                    .map(|mint| takeover::alert(&self.config, &self.context, bot, mint.to_string()))
                    .collect::<Vec<_>>(),
            )
            .await
            .into_iter()
            .collect::<Result<Vec<_>>>()?;

            tokio::time::sleep(Duration::from_millis(500)).await;
            ptr = to;
        }

        Ok(())
    }
}
