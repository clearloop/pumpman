use anyhow::Result;
use clap::Parser;
use pumpman::Opt;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    let app = Opt::parse();
    let env: EnvFilter =
        EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new(match app.verbose {
            0 => "info",
            1 => "info,pumpman=debug",
            2 => "info,pumpman=trace",
            3 => "debug,pumpman=trace",
            _ => "trace",
        }));

    tracing_subscriber::fmt().with_env_filter(env).init();
    app.run().await
}
