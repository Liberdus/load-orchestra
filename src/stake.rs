use crate::{
    cli, crypto, load_injector::{self, GatewayType}, monitor_server, transactions
};
use alloy::signers::{
    k256::ecdsa::SigningKey, local::LocalSigner, SignerSync
};
use std::sync::Arc;
use reqwest::Client;

#[derive(Debug)]
pub struct StakingParams {
    pub rpc_url: String,
    pub verbose: bool,
    pub stake_amount: u128,
    pub monitor_url: String,

}

#[allow(non_snake_case)]
#[derive(Debug, serde::Deserialize)]
pub struct Nominee{
    pub id: String,
    pub ip: String,
    pub port: u16,
    pub publicKey: String,
}




pub async fn stake_node(
    params: &StakingParams, 
    nominee: &String, 
    nominator: &LocalSigner<SigningKey>, 
    crypto: &crypto::ShardusCrypto
) -> Result<transactions::InjectedTxResp, Box<dyn std::error::Error>> {
    let tx = transactions::build_deposite_stake_transaction(
        crypto,
        nominator,
        nominee,
        params.stake_amount,
        );


    let client = Client::new();
    transactions::inject_transaction(
        client,
        &transactions::LiberdusTransactions::DepositStake(tx.clone()),
        &GatewayType::Rpc,
        &params.rpc_url,
        &params.verbose,
    ).await
}




pub fn load_nominee(
    path: &str,
) -> Result<Vec<Nominee>, Box<dyn std::error::Error>> {
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    let nominees: Vec<Nominee> = serde_json::from_reader(reader)?;
    Ok(nominees)
}


pub async fn stake(
    nominees: Vec<String>,
    params: &StakingParams,
) -> Result<(), Box<dyn std::error::Error>> {
    let crypto =  Arc::new(
        crypto::ShardusCrypto::new("69fa4195670576c0160d660c3be36556ff8d504725be8a59b5a96509e0c994bc")
    );

    let target_wallet_count = nominees.len();

    let mut wallet = Vec::new();

    cli::verbose(&params.verbose, "Generating wallets for staking...");

    loop {
       let w = load_injector::generate_register_wallets(
            &1,
            &target_wallet_count,
            &GatewayType::Rpc,
            &params.rpc_url,
            crypto.clone(),
            &params.verbose,
        ).await;

       wallet.extend(w);
       if wallet.len() >= target_wallet_count { 
           break;
       }
    }

    println!("Sleeping for 30 seconds to let register transactions propagate...");
    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;


    for nominee in nominees {
        let nominator = wallet.pop().unwrap();
        match stake_node(params, &nominee, &nominator, &crypto).await {
            Ok(_) => {
                println!("Staked node: {} by {}", nominee, nominator.address());
            },
            Err(e) => {
                eprintln!("Failed to stake node: {}", e);
            }
       }
    } 

    Ok(())
}

