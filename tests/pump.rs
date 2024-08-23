use anchor_lang::AccountDeserialize;
use anyhow::Result;
use replika::{
    api::PumpApi,
    config::Cluster,
    context::{Client, Redis},
    sol::pump::{
        self,
        accounts::{BondingCurve, Global},
        GLOBAL, SOL_SCALE,
    },
};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signature::Keypair};

fn cluster() -> Cluster {
    Cluster {
        helius: "https://mainnet.helius-rpc.com/?api-key=a4174161-7e9c-40ab-83ce-d7f288335380"
            .parse()
            .unwrap(),
        http: "https://api.mainnet-beta.solana.com".parse().unwrap(),
        ws: "wss://api.mainnet-beta.solana.com".parse().unwrap(),
    }
}

#[test]
fn test_keys() {
    assert_eq!(b"global", &[103, 108, 111, 98, 97, 108]);
    assert_eq!(
        b"bonding-curve",
        &[98, 111, 110, 100, 105, 110, 103, 45, 99, 117, 114, 118, 101]
    );
}

#[test]
fn trade_accounts() -> Result<()> {
    let client = RpcClient::new("https://api.mainnet-beta.solana.com");
    let mint: Pubkey = "8CTjSbj6h3pAMx1UJcQXLwA4KXAwRF6nQ1JVMkBjpump".parse()?;
    let data = &client.get_account_data(&GLOBAL)?;
    let global = Global::try_deserialize(&mut data.as_ref())?;
    let accs = pump::Buy::new(0, 0)
        .ix(&global, mint, Default::default())
        .accounts;

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

#[test]
fn bonding_curve_calc() -> Result<()> {
    let client = RpcClient::new("https://api.mainnet-beta.solana.com");
    let data = &client.get_account_data(&GLOBAL)?;
    let global = Global::try_deserialize(&mut data.as_ref())?;
    println!("{:#?}", global);

    let bc = global.init();
    assert_eq!(global.buy(bc.real_sol_reserves, SOL_SCALE)?, 34612903225806);
    Ok(())
}

#[tokio::test]
async fn login() -> Result<()> {
    let client = Client::new(&cluster())?;
    let pair = Keypair::new();
    let redis = Redis::new(&"redis://localhost".parse()?)?;

    let auth_token = client.auth(pair, &mut redis.con()?).await?;
    println!("{auth_token}");
    Ok(())
}
