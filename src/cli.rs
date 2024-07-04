//! CLI operations

use crate::{context::Telegram, Client, Config, Redis};
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
        let redis = Redis::new(config.redis)?;
        let telegram = Telegram::new(&config.telegram, redis.clone());
        let client = Client::new(&config.cluster, redis, telegram.clone()).await?;

        match self.command {
            Command::Sig { signature } => client.sig(&signature).await,
            Command::Start => client.subscribe().await,
            Command::Subscribe { message } => telegram.subscribe(message).await,
        }
    }
}
