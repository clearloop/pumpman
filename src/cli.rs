//! CLI operations

use crate::{
    api::{HttpClient, SolRpcApi},
    context::Context,
    model::{Alert, AlertTitle},
    service,
    sol::pump::accounts::BondingCurve,
    Config,
};
use anchor_lang::AnchorDeserialize;
use anyhow::Result;
use clap::{Parser, Subcommand};
use solana_sdk::pubkey::Pubkey;
use std::{path::PathBuf, str::FromStr};

/// Sub commands
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Prints transaction from signature
    Sig {
        signature: String,
    },
    /// Prints metadata of a token
    Coin {
        mint: String,
    },
    /// Prints pairs of a token
    Dex {
        mint: String,
    },
    /// Get alert info of a token
    Info {
        mint: String,
    },
    TokenAccounts {
        acc: String,
        mint: String,
    },
    BondingCurve {
        mint: String,
    },
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
    /// If update cache
    #[clap(short, long)]
    update: bool,
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

        let Some(command) = self.command else {
            return service::start(&config, context.clone()).await;
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
                let coin = context.client.coin(&mint, self.update, con).await?;
                println!("{coin:#?}");
            }
            Command::Dex { mint } => {
                let con = &mut context.redis()?;
                let pairs = context.client.pairs(&mint, self.update, con).await?;
                println!("{pairs:#?}");
            }
            Command::TokenAccounts { acc, mint } => {
                let con = &mut context.redis()?;
                let accs = context
                    .client
                    .token_account(
                        Pubkey::from_str(&mint)?,
                        &Pubkey::from_str(&acc)?,
                        self.update,
                        con,
                    )
                    .await?;
                println!("{accs:#?}");
            }
            Command::Info { mint } => {
                let con = &mut context.redis()?;
                let coin = context.client.coin(&mint, self.update, con).await?;
                let pairs = context.client.pairs(&mint, self.update, con).await?;
                let soldout = context
                    .client
                    .soldout(&coin.mint, &coin.creator, false, con)
                    .await?;
                let holders = context
                    .client
                    .top_holders(&mint, self.update, con)
                    .await?
                    .skip_bc(&coin.associated_bonding_curve);

                println!(
                    "{}",
                    Alert::new(AlertTitle::DevSoldOut, coin, soldout.1)
                        .pairs(pairs)
                        .holders(holders)
                );
            }
            Command::BondingCurve { mint } => {
                let pk = mint.parse()?;
                let data = context.client.rpc().get_account_data(&pk).await?;
                let bc = BondingCurve::deserialize(&mut data.as_ref())?;
                println!("{bc:#?}");
            }
        }

        Ok(())
    }
}
