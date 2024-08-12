use anchor_lang::AccountDeserialize;
use anyhow::Result;
use replika::sol::pump::{
    self,
    accounts::{BondingCurve, Global},
    GLOBAL,
};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

#[test]
fn test_keys() {
    assert_eq!(b"global", &[103, 108, 111, 98, 97, 108]);
    assert_eq!(
        b"bonding-curve",
        &[98, 111, 110, 100, 105, 110, 103, 45, 99, 117, 114, 118, 101]
    );
}

#[test]
fn global_account_matched() -> Result<()> {
    let client = RpcClient::new("https://api.mainnet-beta.solana.com");
    let global = &client.get_account_data(&GLOBAL)?;
    println!("{:#?}", Global::try_deserialize(&mut global.as_ref()));
    Ok(())
}

#[test]
fn trade_account() -> Result<()> {
    let client = RpcClient::new("https://api.mainnet-beta.solana.com");
    let mint: Pubkey = "8CTjSbj6h3pAMx1UJcQXLwA4KXAwRF6nQ1JVMkBjpump".parse()?;
    let accs = pump::trade_accounts(mint, Default::default(), Default::default());

    // check bonding curve account
    let bc = accs[3].pubkey;
    assert_eq!(bc, "8ic1aqhr9n4hvEUK5JSxAJS778tDeMjki6TmZD5qLQxV".parse()?);

    // 2. get bonding curve account
    let data = client.get_account_data(&bc)?;
    let bc_info = BondingCurve::try_deserialize(&mut data.as_ref())?;
    println!("{:#?}", bc_info);

    // 3. check associated bonding curve account
    let abc = accs[4].pubkey;
    assert_eq!(abc, "CSeFMTTDFoDJwhohpVSMccjKTBmhSAtPeYGuvk2a7Vze".parse()?);
    Ok(())
}
