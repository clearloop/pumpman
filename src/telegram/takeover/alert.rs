use crate::{
    api::{DexScreenerApi, PumpApi, SolRpcApi},
    config,
    context::{Context, TaskCache},
    model::{Alert, AlertTitle},
};
use anyhow::Result;
use bigdecimal::BigDecimal;
use redis::Commands;
use teloxide::Bot;

/// pumpfun soldout alert
pub async fn alert(
    config: &config::Takeover,
    context: &Context,
    bot: &Bot,
    mint: String,
) -> Result<()> {
    let redis = &mut context.redis()?;
    let client = context.client.clone();

    let key = TaskCache::DevSoldOut(&mint);
    if redis.exists(&key)? {
        return Ok(());
    }

    // filter out mc less than $8k
    let coin = client.coin(&mint, false, redis).await?;
    if let Some(mc) = &coin.usd_market_cap {
        if *mc < BigDecimal::from(config.marketcap) {
            return Ok(());
        }
    }

    // check if dev is soldout
    let (_, soldout) = client
        .soldout(&coin.mint, &coin.creator, false, redis)
        .await?;

    if !soldout {
        return Ok(());
    }

    // check holders amount
    let holders = client
        .top_holders(&mint, false, redis)
        .await?
        .skip_bc(&coin.associated_bonding_curve);

    if holders.len() < config.holders {
        return Ok(());
    }

    let pairs = client.pairs(&mint, false, redis).await?;
    context.update_coin(coin.clone()).await?;
    Alert::new(AlertTitle::DevSoldOut, coin, soldout)
        .pairs(pairs)
        .holders(holders)
        .alert(bot, &config.subscription)
        .await?;
    redis.set(key, true)?;

    Ok(())
}
