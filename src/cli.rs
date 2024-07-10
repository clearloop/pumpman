//! CLI operations

use crate::{
    api::{HttpClient, SolRpcApi},
    context::Context,
    service, Config,
};
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::{path::PathBuf, sync::Arc};

/// Sub commands
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Prints transaction from signature
    Sig { signature: String },
    /// Prints metadata of a token
    Coin { mint: String },
    /// Prints pairs of a token
    Dex { mint: String },
    /// Init database
    Init,
}

/// Replika command line interfaces
#[derive(Parser)]
pub struct Opt {
    /// Path of replika config
    #[clap(short, long, default_value = "config.toml")]
    config: PathBuf,
    /// Replika sub commands
    #[clap(subcommand)]
    command: Option<Command>,
    /// The verbosity level.
    #[clap(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

impl Opt {
    /// Run commands
    pub async fn run(self) -> Result<()> {
        let config = Config::load(self.config)?;
        let context = Arc::new(Context::new(&config)?);

        // pre-process
        context.init()?;

        let Some(command) = self.command else {
            let mut result = service::start(&config, context.clone()).await;
            while let Err(e) = result {
                tracing::error!("service broken: {e}");
                result = service::start(&config, context.clone()).await;
            }
            return Ok(());
        };

        // match commands
        match command {
            Command::Init => {}
            Command::Sig { signature } => {
                let tx = context.client.tx(&signature).await?;
                println!("{tx:#?}");
            }
            Command::Coin { mint } => {
                let con = &mut context.redis()?;
                let meta = context.client.coin(&mint, true, con).await?;
                println!("{meta:#?}");
            }
            Command::Dex { mint } => {
                let con = &mut context.redis()?;
                let pairs = context.client.tokens(&mint, true, con).await?;
                println!("{pairs:#?}");
            }
        }

        Ok(())
    }
}
