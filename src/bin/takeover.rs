use anyhow::Result;
use clap::Parser;
use replika::{service, Config, Context};
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

/// Replika command line interfaces
#[derive(Parser)]
pub struct Opt {
    /// Path of replika config
    #[clap(short, long, default_value = "config.toml")]
    config: PathBuf,
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

        service::takeover(&config, context.clone()).await
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let app = Opt::parse();
    let env: EnvFilter =
        EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new(match app.verbose {
            0 => "info",
            1 => "info,replika=debug",
            2 => "info,replika=trace",
            3 => "debug,replika=trace",
            _ => "trace",
        }));

    tracing_subscriber::fmt().with_env_filter(env).init();
    app.run().await
}
