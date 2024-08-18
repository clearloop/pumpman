//! CLI operations

use crate::{
    api::{DexScreenerApi, PumpApi, SolRpcApi},
    context::Context,
    model::{Alert, AlertTitle},
    sol::pump::{accounts::BondingCurve, SOL_SCALE},
    Config,
};
use anchor_lang::AccountDeserialize;
use anyhow::Result;
use bigdecimal::{BigDecimal, ToPrimitive};
use clap::{Parser, Subcommand};
use solana_sdk::{
    pubkey::Pubkey,
    signature::Signature,
    signer::{keypair::Keypair, Signer},
};
use std::{fs, path::PathBuf, str::FromStr};

/// Replika command line interfaces
#[derive(Parser)]
pub struct Opt {
    /// Path of replika config
    #[clap(short, long, default_value = "config.toml")]
    config: PathBuf,
    /// Replika sub commands
    #[clap(subcommand)]
    command: Command,
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
        self.command.run(context, self.update).await
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
    BondingCurve { bonding_curve: String },
    /// Verify signature
    Verify {
        account: String,
        message: String,
        sig: String,
    },
    /// Sign message
    Sign { message: String },
    /// Simulate bump
    SimBump {
        mint: String,
        amount: BigDecimal,
        payer: PathBuf,
    },
    /// Get balance
    Balance { address: String },
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
            Command::BondingCurve { bonding_curve } => {
                let pk = bonding_curve.parse()?;
                let data = context.client.rpc().get_account_data(&pk).await?;
                let bc = BondingCurve::try_deserialize(&mut data.as_ref())?;
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

                let r = pair.sign_message(message.as_bytes());
                println!("{r:?}");
            }
            Command::Balance { address } => {
                let pk = Pubkey::from_str(address)?;
                let balance = context.client.rpc().get_balance(&pk).await?;
                println!(
                    "{address} balance: {} SOL",
                    BigDecimal::from(balance) / SOL_SCALE
                );
            }
            Command::SimBump {
                mint,
                amount,
                payer,
            } => {
                let payer =
                    Keypair::from_bytes(&serde_json::from_slice::<Vec<u8>>(&fs::read(payer)?)?)?;
                let mint = Pubkey::from_str(mint)?;

                let exists = context.client.check_auser(mint, payer.pubkey()).await;
                let tx = context
                    .client
                    .bump(
                        &mint,
                        (amount * SOL_SCALE)
                            .to_u64()
                            .expect("Failed to convert sol amount"),
                        &payer,
                        exists,
                    )
                    .await?;

                let fee = context
                    .client
                    .rpc()
                    .get_fee_for_message(tx.message())
                    .await?;
                println!("Fee {fee} ({} SOL)", BigDecimal::from(fee) / SOL_SCALE);

                let bytes = bincode::serialize(&tx)?;
                println!("Size: {}", bytes.len());

                let resp = context.client.helius().simulate_transaction(&tx).await?;
                println!("{resp:#?}");

                // let _logs: Vec<pump::events::TradeEvent> =
                //     sol::parse2(&resp.value.logs.expect("Logs not found"))?;
                // println!("{logs:#?}");
            }
        }

        Ok(())
    }
}
