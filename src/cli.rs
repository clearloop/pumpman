//! CLI operations

use crate::{
    api::{DexScreenerApi, SolRpcApi},
    context::Context,
    telegram::TakeoverBot,
    Config,
};
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Sub commands
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Prints transaction from signature
    Sig { signature: String },
    /// Prints metadata of a token
    Metadata { mint: String },
    /// Prints pairs of a token
    Dex { mint: String },
    /// Start the takeover service
    Takeover,
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
        let config = Config::load(self.config)?;
        let context = Context::new(&config)?;

        // pre-process
        context.init().await?;

        // match commands
        match self.command {
            Command::Sig { signature } => {
                let tx = context.client.tx(&signature).await?;
                println!("{tx:#?}");
                Ok(())
            }
            Command::Metadata { mint } => {
                let meta = context.client.coin(&mint).await?;
                println!("{meta:#?}");
                Ok(())
            }
            Command::Dex { mint } => {
                let pairs = context.client.tokens(&mint).await?;
                println!("{pairs:#?}");
                Ok(())
            }
            Command::Takeover => {
                let bot = TakeoverBot::new(
                    &config.telegram.takeover,
                    context,
                    format!("{}/15", config.redis),
                );

                let mut result = bot.start().await;
                while let Err(e) = result {
                    tracing::error!("takeover bot broken: {e}");
                    result = bot.start().await;
                }

                Ok(())
            }
        }
    }
}
