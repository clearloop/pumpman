//! CLI operations

use crate::Client;
use anyhow::Result;
use clap::{Parser, Subcommand};

/// Sub commands
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Prints transaction from signature
    Sig {
        signature: String,
    },
    Sub,
}

/// CLI Operations
#[derive(Parser)]
pub struct Opt {
    #[clap(subcommand)]
    command: Command,
}

impl Opt {
    /// Run commands
    pub async fn run(&mut self) -> Result<()> {
        let client = Client::new("ws://api.mainnet-beta.solana.com").await?;

        match &self.command {
            Command::Sig { signature } => client.sig(&signature).await,
            Command::Sub => client.subscribe().await,
        }
    }
}
