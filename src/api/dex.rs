//! Dexscreener APis
use crate::{api::HttpClient, model::DexScreenerPair, utils::THOURS};
use anyhow::Result;
use redis::Connection;
use serde::Deserialize;

const DEX_SCREENER: &str = "https://api.dexscreener.com/latest/dex";

/// Dexscreener API
pub trait DexScreenerApi: HttpClient {
    /// dexscreener pairs
    async fn pairs(
        &self,
        mint: &str,
        update: bool,
        con: &mut Connection,
    ) -> Result<Vec<DexScreenerPair>> {
        let tokens: DexScreenerTokensResult = self
            .cget(
                &format!("{DEX_SCREENER}/tokens/{mint}"),
                update,
                THOURS,
                con,
            )
            .await?;
        Ok(tokens.pairs.unwrap_or_default())
    }

    /// Get dexscreener url
    async fn pair(&self, mint: &str, update: bool, con: &mut Connection) -> Option<String> {
        self.pairs(mint, update, con)
            .await
            .ok()?
            .first()
            .map(|p| p.url.clone())
    }
}

#[derive(Clone, Deserialize, Debug)]
pub struct DexScreenerTokensResult {
    pub pairs: Option<Vec<DexScreenerPair>>,
}
