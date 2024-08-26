//! Comment service
use crate::{sol, sol::pump};
use anyhow::Result;
use fantoccini::{Client, ClientBuilder, Locator};
use futures::StreamExt;
use rand::{thread_rng, Rng};
use redis::{Client as Redis, Commands, Connection};
use solana_client::{
    nonblocking::pubsub_client::PubsubClient,
    rpc_config::{RpcTransactionLogsConfig, RpcTransactionLogsFilter},
};
use solana_sdk::{commitment_config::CommitmentConfig, signature::Keypair};
use std::{
    io,
    sync::{
        atomic::{AtomicU16, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{
    signal,
    sync::{
        mpsc,
        mpsc::{Receiver, Sender},
    },
};

const WEB_DRIVER: &str = "http://localhost:8888";
const REDIS: &str = "redis://localhost";
const SOL_WS: &str = "wss://api.mainnet-beta.solana.com";
const BIO: &str = "Bump tokens with https://t.me/pumpmaniobot (~ 0.000275 SOL per bump). Get takeover alerts at https://t.me/takeoveralerts";

/// Pumpfun commenter
pub struct Commenter {
    client: Client,
    count: Arc<AtomicU16>,
}

impl Commenter {
    /// Create a new commenter
    pub async fn new() -> Result<Self> {
        Ok(Self {
            client: ClientBuilder::native().connect(WEB_DRIVER).await?,
            count: Arc::new(AtomicU16::new(0)),
        })
    }

    pub async fn start() -> Result<()> {
        let cmt = Self::new().await?;
        cmt.setup().await?;
        tracing::info!("Start commenting ...");

        let redis = Redis::open(REDIS)?;

        loop {
            let (tx, rx) = mpsc::channel::<String>(50);
            let con1 = &mut redis.get_connection()?;
            let con2 = &mut redis.get_connection()?;
            if let Err(e) = tokio::select! {
                _ = signal::ctrl_c() => break,
                r = cmt.sub(con1, tx) => r,
                r = cmt.comment(con2, rx) => r,
            } {
                tracing::error!("{e:?}");
            }
        }
        Ok(())
    }

    /// Set up pumpfun cookies
    async fn setup(&self) -> Result<()> {
        self.client.goto("https://pump.fun").await?;

        // close the popup
        let ready = r#"//button[contains(string(), "m ready to pump]")]"#;
        let ready = self
            .client
            .wait()
            .for_element(Locator::XPath(ready))
            .await?;
        ready.click().await?;

        // reject cookies
        let reject = r#"//button[contains(string(), "Reject All")]"#;
        let reject = self
            .client
            .wait()
            .for_element(Locator::XPath(reject))
            .await?;
        reject.click().await?;

        // waiting for setup
        let wallet = bs58::encode(Keypair::new().to_bytes()).into_string();
        self.pause(&format!(
            r#"
phantom ext: https://chromewebstore.google.com/detail/phantom/bfnaelmomeimhlpmgjnjophhpkkoljpa?hl=ja

wallet secret: {wallet}

BIO: {BIO}

please setup your phantom wallet and press [Enter] to continue"#
        ))?;
        Ok(())
    }

    async fn comment(&self, con: &mut Connection, mut rx: Receiver<String>) -> Result<()> {
        let mut rng = thread_rng();

        while let Some(mint) = rx.recv().await {
            if con.hexists("comment", &mint)? {
                continue;
            }

            let url = format!("https://pump.fun/{mint}");
            tracing::info!(
                "#{} Commenting on {url}",
                self.count.load(Ordering::Relaxed)
            );
            self.client.goto(&url).await?;

            // get the post button
            let par = r#"//div[text() = "[Post a reply]"]"#;
            let par = self.client.wait().for_element(Locator::XPath(par)).await?;
            par.click().await?;

            // input text
            let comment = r#"//textarea[@placeholder='comment']"#;
            let comment = self
                .client
                .wait()
                .for_element(Locator::XPath(comment))
                .await?;
            comment
                .send_keys(
                    &format!(r#"Keep this token on the first page via @pumpmaniobot on tg"#).trim(),
                )
                .await?;

            // send post
            let post = r#"//button[contains(string(), "post reply")]"#;
            let post = self.client.wait().for_element(Locator::XPath(post)).await?;
            post.click().await?;

            self.count.fetch_add(1, Ordering::Release);
            con.hset("comment", mint, true)?;
            tokio::time::sleep(Duration::from_secs(rng.gen_range(12..24))).await;
        }

        Ok(())
    }

    fn pause(&self, info: &str) -> Result<()> {
        println!("{info}");
        let mut buffer = String::new();
        let stdin = io::stdin();
        stdin.read_line(&mut buffer)?;
        Ok(())
    }

    async fn sub(&self, con: &mut Connection, tx: Sender<String>) -> Result<()> {
        let pubsub = PubsubClient::new(SOL_WS).await?;
        let mut sub = pubsub
            .logs_subscribe(
                RpcTransactionLogsFilter::Mentions(vec![pump::ID.to_string()]),
                RpcTransactionLogsConfig {
                    commitment: Some(CommitmentConfig::finalized()),
                },
            )
            .await?;

        while let Some(resp) = sub.0.next().await {
            if resp.value.err.is_some() {
                continue;
            }

            if let Some(event) = sol::parse::<pump::events::TradeEvent>(&resp.value.logs) {
                let mint = event.mint.to_string();

                if con.hexists("comment", &mint)? {
                    continue;
                }

                tx.send(mint).await?;
            }
        }

        Ok(())
    }
}
