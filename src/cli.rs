//! CLI operations
#![allow(clippy::get_first)]

use crate::{
    api::{DexScreenerApi, PumpApi, SolRpcApi},
    context::Context,
    model::{Alert, AlertTitle},
    schema::users,
    sol::{
        self,
        pump::{self, accounts::BondingCurve, SOL_SCALE},
    },
    Config,
};
use anchor_lang::AccountDeserialize;
use anyhow::Result;
use bigdecimal::{BigDecimal, ToPrimitive};
use clap::{Parser, Subcommand};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use solana_account_decoder::UiAccountEncoding;
use solana_client::rpc_config::{
    RpcSimulateTransactionAccountsConfig, RpcSimulateTransactionConfig,
};
use solana_sdk::{
    native_token::LAMPORTS_PER_SOL,
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
    /// Get details of token account
    TokenAccounts {
        acc: String,
        mint: String,
    },
    /// Get bonding curve of pumpfun coin
    BondingCurve {
        bonding_curve: String,
    },
    /// Verify signature
    Verify {
        account: String,
        message: String,
        sig: String,
    },
    /// Sign message
    Sign {
        message: String,
    },
    /// Simulate bump
    SimBump {
        mint: String,
        amount: BigDecimal,
        payer: PathBuf,
    },
    /// Simulate withdraw
    SimWithdraw {
        to: Pubkey,
        payer: PathBuf,
    },
    PumpFee,
    /// Get balance
    Balance {
        address: String,
    },
    Import {
        tgid: i64,
        wallet: PathBuf,
    },
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
            Command::PumpFee => {
                let fee = context.client.priority_fee().await?;
                println!("pump.fun recent priority fee: {}", fee)
            }
            Command::Import { tgid, wallet } => {
                let postgres = &mut context.postgres().await?;
                let pair =
                    Keypair::from_bytes(&serde_json::from_slice::<Vec<u8>>(&fs::read(wallet)?)?)?;
                let wallet = bs58::encode(pair.to_bytes()).into_string();
                diesel::update(users::table)
                    .filter(users::tgid.eq(tgid))
                    .set(users::wallet.eq(wallet))
                    .execute(postgres)
                    .await?;
                println!("update the wallet of {tgid} as {}", pair.pubkey());
            }
            Command::SimBump {
                mint,
                amount,
                payer,
            } => {
                let payer =
                    Keypair::from_bytes(&serde_json::from_slice::<Vec<u8>>(&fs::read(payer)?)?)?;
                let mint = Pubkey::from_str(mint)?;
                let pubkey = payer.pubkey();
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
                    .helius()
                    .get_fee_for_message(tx.message())
                    .await?;
                let bytes = bincode::serialize(&tx)?;
                let balance = context.client.helius().get_balance(&pubkey).await?;

                let resp = context
                    .client
                    .helius()
                    .simulate_transaction_with_config(
                        &tx,
                        RpcSimulateTransactionConfig {
                            accounts: Some(RpcSimulateTransactionAccountsConfig {
                                encoding: Some(UiAccountEncoding::JsonParsed),
                                addresses: vec![
                                    format!("{pubkey}"),
                                    // "8VJb5raJxzy8ForP1jArfKkM34UaagtVd7B61C1bSqhn".to_string(),
                                ],
                            }),
                            ..Default::default()
                        },
                    )
                    .await?;
                println!("{resp:#?}");

                let logs: Vec<pump::events::TradeEvent> =
                    sol::parse2(&resp.value.logs.expect("Logs not found"))?;

                println!("{logs:#?}");

                let accounts: Vec<_> = resp.value.accounts.unwrap();
                let sender = accounts.get(0).unwrap().clone().unwrap();
                // let receiver = accounts.get(1).unwrap().clone().unwrap();
                println!(
                    "Cost: {} SOL",
                    ((BigDecimal::from(balance) - sender.lamports) / LAMPORTS_PER_SOL).round(6)
                );
                println!("Fee {fee} ({} SOL)", BigDecimal::from(fee) / SOL_SCALE);
                // println!(
                //     "Received {fee} ({} SOL)",
                //     (BigDecimal::from(receiver.lamports) / LAMPORTS_PER_SOL).round(6)
                // );
                println!("Size: {}", bytes.len());
            }
            Command::SimWithdraw { to, payer } => {
                let payer =
                    Keypair::from_bytes(&serde_json::from_slice::<Vec<u8>>(&fs::read(payer)?)?)?;
                let pubkey = payer.pubkey();
                let tx = context.client.withdraw(&payer, to.to_string()).await?;
                let balance = context.client.helius().get_balance(&pubkey).await?;
                let resp = context
                    .client
                    .helius()
                    .simulate_transaction_with_config(
                        &tx,
                        RpcSimulateTransactionConfig {
                            accounts: Some(RpcSimulateTransactionAccountsConfig {
                                encoding: Some(UiAccountEncoding::JsonParsed),
                                addresses: vec![format!("{pubkey}"), to.to_string()],
                            }),
                            ..Default::default()
                        },
                    )
                    .await?;
                let fee = context
                    .client
                    .helius()
                    .get_fee_for_message(tx.message())
                    .await?;
                println!("{resp:#?}");

                let accounts: Vec<_> = resp.value.accounts.unwrap();
                let sender = accounts.get(0).unwrap().clone().unwrap();
                let receiver = accounts.get(1).unwrap().clone().unwrap();
                println!(
                    "Cost: {} SOL",
                    ((BigDecimal::from(balance) - sender.lamports) / LAMPORTS_PER_SOL).round(6)
                );
                println!(
                    "Received {fee} ({} SOL)",
                    (BigDecimal::from(receiver.lamports) / LAMPORTS_PER_SOL).round(6)
                );
                println!("Fee {fee} ({} SOL)", BigDecimal::from(fee) / SOL_SCALE);
            }
        }

        Ok(())
    }
}
