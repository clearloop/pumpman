use anyhow::Result;
use clap::Parser;
use replika::Opt;

#[tokio::main]
async fn main() -> Result<()> {
    let mut opt = Opt::parse();
    opt.run().await
}
