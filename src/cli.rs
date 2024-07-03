use anyhow::Result;
/// CLI operations
use clap::{Parser, Subcommand};

use crate::Client;

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Prints transaction from signature
    Sig { signature: String },
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
            Command::Sig { signature } => client.get_sig(&signature).await,
        }
    }
}
