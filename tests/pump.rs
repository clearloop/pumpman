use anyhow::Result;
use borsh::BorshDeserialize;
use replika::sol::pump::{accounts::Global, TradeAccount};
use solana_client::rpc_client::RpcClient;
use solana_sdk::instruction::AccountMeta;

#[test]
fn global_account_matched() -> Result<()> {
    let client = RpcClient::new("https://api.mainnet-beta.solana.com");
    let global_pda = AccountMeta::from(TradeAccount::Global).pubkey;
    let global = &client.get_account_data(&global_pda)?[8..];

    println!("{:#?}", Global::deserialize(&mut global.as_ref()));
    Ok(())
}
