//! CLI operations

use crate::{
    context::{Db, Redis, Telegram},
    telegram::TakeoverBot,
    Client, Config,
};
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::{fs, path::PathBuf};

/// Sub commands
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Prints transaction from signature
    Sig { signature: String },
    /// Start the service
    Start,
    /// Start the takeover service
    Takeover,
    /// Subscribe message to telegram channel
    Subscribe {
        /// Message to be subscribed to the main channel
        message: String,
    },
}

/// Replika command line interfaces
#[derive(Parser)]
pub struct Opt {
    /// Path of replika config
    #[clap(short, long, default_value = "config.toml")]
    config: PathBuf,
    /// Replika sub commands
    #[clap(subcommand)]
    command: Command,
    /// The verbosity level.
    #[clap(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

impl Opt {
    /// Run commands
    pub async fn run(self) -> Result<()> {
        let config: Config = toml::from_str(&fs::read_to_string(&self.config)?)?;
        let db = Db::new(&config.postgres, &config.redis)?;
        let telegram = Telegram::new(&config.telegram, db.redis.clone());
        let client = Client::new(&config.cluster, db.redis.clone(), telegram.clone()).await?;

        match self.command {
            Command::Sig { signature } => client.sig(&signature).await,
            Command::Start => client.subscribe().await,
            Command::Takeover => {
                let bot = TakeoverBot::new(
                    &config.telegram.takeover,
                    db,
                    format!("{}/15", config.redis),
                );
                bot.start().await
            }
            Command::Subscribe { message } => telegram.subscribe(message).await,
        }
    }
}
