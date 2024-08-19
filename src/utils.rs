// Day in seconds
// pub const DAY: u64 = 86400;

// 3 hrs
pub const THOURS: u64 = 12800;

/// 1 hr
pub const HOUR: u64 = 3600;

/// 1 day
pub const DAY: u64 = 86400;

/// 5 minutes
pub const FIVE_MINS: u64 = 300;

pub mod base64 {
    use anyhow::Result;
    use base64::{engine::general_purpose::STANDARD, Engine as _};

    /// base64 decode
    pub fn decode(encoded: &str) -> Result<Vec<u8>> {
        STANDARD.decode(encoded).map_err(Into::into)
    }

    /// base64 encode
    pub fn encode(data: &[u8]) -> String {
        STANDARD.encode(data)
    }
}

pub mod sol {
    use anyhow::Result;
    use solana_account_decoder::UiAccountData;
    use solana_client::rpc_response::RpcKeyedAccount;

    /// Parse token accounts
    pub fn parse_token_accounts(accs: Vec<RpcKeyedAccount>) -> Result<Vec<(String, String)>> {
        Ok(accs
            .into_iter()
            .map(|acc| {
                let pubkey = acc.pubkey;
                let amount = (if let UiAccountData::Json(json) = &acc.account.data {
                    Some(json)
                } else {
                    None
                })
                .and_then(|json| {
                    json.parsed
                        .as_object()?
                        .get("info")?
                        .get("tokenAmount")?
                        .get("uiAmountString")?
                        .as_str()
                })
                .unwrap_or("0")
                .to_string();

                (pubkey, amount)
            })
            .collect())
    }
}
