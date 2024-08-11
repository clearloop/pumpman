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
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    signer::Signer,
};
use std::{path::PathBuf, str::FromStr};

/// Replika command line interfaces
#[derive(Parser)]
pub struct Opt {
    /// Path of replika config
    #[clap(short, long, default_value = "config.toml")]
    config: PathBuf,
    /// Replika sub commands
    #[clap(subcommand)]
    command: Option<Command>,
    /// Disabled takeover service
    #[clap(short, long)]
    disable_takeover: bool,
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
        let mut config = Config::load(self.config)?;
        let context = Context::new(&config)?;

        if self.disable_takeover {
            config.takeover.disabled = true;
        }

        // pre-process
        context.init().await?;

        let Some(command) = self.command else {
            return service::start(&config, context.clone()).await;
        };

        command.run(context, self.update).await
    }
}

/// Sub commands
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Prints transaction from signature
    Sig { signature: String },
    /// Prints metadata of a token
    Coin { mint: String },
    /// Prints pairs of a token
    Dex { mint: String },
    /// Get alert info of a token
    Info { mint: String },
    /// Get details of token account
    TokenAccounts { acc: String, mint: String },
    /// Get bonding curve of pumpfun coin
    BondingCurve { mint: String },
    /// Verify signature
    Verify {
        account: String,
        message: String,
        sig: String,
    },
    /// Sign message
    Sign { message: String },
    /// Init database
    Init,
}

impl Command {
    /// Run command
    pub async fn run(&self, context: Context, update: bool) -> Result<()> {
        // match commands
        match &self {
            Command::Init => {}
            Command::Sig { signature } => {
                let tx = context.client.tx(signature).await?;
                println!("{tx:#?}");
            }
            Command::Coin { mint } => {
                let con = &mut context.redis()?;
                let coin = context.client.coin(mint, update, con).await?;
                println!("{coin:#?}");
            }
            Command::Dex { mint } => {
                let con = &mut context.redis()?;
                let pairs = context.client.pairs(mint, update, con).await?;
                println!("{pairs:#?}");
            }
            Command::TokenAccounts { acc, mint } => {
                let con = &mut context.redis()?;
                let accs = context
                    .client
                    .token_account(
                        Pubkey::from_str(mint)?,
                        &Pubkey::from_str(acc)?,
                        update,
                        con,
                    )
                    .await?;
                println!("{accs:#?}");
            }
            Command::Info { mint } => {
                let con = &mut context.redis()?;
                let coin = context.client.coin(mint, update, con).await?;
                let pairs = context.client.pairs(mint, update, con).await?;
                let soldout = context
                    .client
                    .soldout(&coin.mint, &coin.creator, false, con)
                    .await?;
                let holders = context
                    .client
                    .top_holders(mint, update, con)
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
            Command::Verify {
                account,
                message,
                sig,
            } => {
                let pk = Pubkey::from_str(account)?;
                let result = Signature::from_str(sig)?.verify(&pk.to_bytes(), message.as_bytes());
                println!("{result:?}");
            }
            Command::Sign { message } => {
                let pair = Keypair::new();
                let pubkey = pair.pubkey();
                println!("pubkey: {pubkey}");

                let r = pair.sign_message(&message.as_bytes());
                println!("{r:?}");
            }
        }

        Ok(())
    }
}
