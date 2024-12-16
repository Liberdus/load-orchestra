use alloy::signers::local::PrivateKeySigner;
use crate::{ cli::verbose, crypto, rpc, transactions };
use std::sync::Arc;
use rand::{self, Rng};
use std::io::Write;

pub struct InjectionStats{
    pub total: usize,
    pub success: usize,
    pub failed: usize,
}

#[derive(Clone)]
pub struct LiberdusIdentity{
    pub alias: String,
    pub signer: PrivateKeySigner,
}

pub async fn transfer(tps: &usize, eoa: &usize, duration: &usize, rpc_url: &String, verbosity: &bool) {
    let shardus_crypto = crypto::ShardusCrypto::new("69fa4195670576c0160d660c3be36556ff8d504725be8a59b5a96509e0c994bc");
    let mut wallets = Vec::new();
    
    let rpc_url_cloned = rpc_url.clone();
    for _ in 0..*eoa {
        let signer = PrivateKeySigner::random();
        let tx = transactions::build_register_transaction(&shardus_crypto, &signer, &generate_random_string(10));
        match transactions::inject_transaction(&transactions::LiberdusTransactions::Register(tx), &rpc_url_cloned.clone()).await {
            Ok(resp) => {
                if resp.clone().result.is_none() || (resp.clone().result.unwrap().success == false) {
                    verbose(verbosity, &format!("Failed to register {:?}", signer.address()));
                    continue;
                }

                verbose(verbosity, &format!("Registered {:?}, TxId {:?}", signer.address(), resp.clone().result.unwrap().txId));
                wallets.push(signer);
            },
            Err(e) => {
                verbose(verbosity, &format!("Failed to register {:?}", signer.address()));
            }
        }
    }

    println!("Registered {} successful wallets", wallets.len());

    println!("Waiting for 30 seconds before injecting transactions");

    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

    println!("Injecting transactions");

    let duration = tokio::time::Duration::from_secs(*duration as u64);
    let start_time = tokio::time::Instant::now();
    let interval = tokio::time::Duration::from_secs_f64(1.0 / *tps as f64);
    let mut interval_timer = tokio::time::interval(interval);

    let (transmitter, mut receiver) = tokio::sync::mpsc::unbounded_channel::<
        (transactions::TransferTransaction, Result<rpc::RpcResponse,_>)
    >();

    let rpc_url_long_live = rpc_url.clone();
    tokio::spawn(async move {

        let long_live_transmitter = transmitter.clone();


        let sc = Arc::new(shardus_crypto);
        let long_live_wallet = wallets.clone();
        while start_time.elapsed() < duration {
            interval_timer.tick().await;

            let sc = Arc::clone(&sc);

            let wl = long_live_wallet.clone();

            // make sure the from and to are not the same
            let from = rand::thread_rng().gen_range(0..wl.len());
            let mut to = rand::thread_rng().gen_range(0..wl.len());

            while from == to {
                to = rand::thread_rng().gen_range(0..wl.len());
            }
            

            let transmitter = long_live_transmitter.clone();

            let rpc_url_for_detached_thread = rpc_url_long_live.clone();
            tokio::spawn(async move {
                let signers = wl[from].clone();
                let to = wl[to].clone();
                let tx = transactions::build_transfer_transaction(&*Arc::clone(&sc), &signers, &to.address(), 1);
                let resp = transactions::inject_transaction(
                    &transactions::LiberdusTransactions::Transfer(tx.clone()),
                    &rpc_url_for_detached_thread
                    ).await;

                transmitter.send((tx, resp)).unwrap();
            });

        }
    });

    let mut stats = InjectionStats{
        total: 0,
        success: 0,
        failed: 0,
    };

    while let Some((tx, resp)) = receiver.recv().await {
        let from = tx.from.clone();
        let to = tx.to.clone();
        stats.total += 1;
        match resp {
            Ok(resp) => {
                if resp.result.is_none() || (resp.result.unwrap().success == false) {
                    stats.failed += 1;
                    verbose(verbosity, &format!("Transfer failed from {}, to {}", from, to));
                    continue;
                }

                verbose(verbosity, &format!("Transfer success from {}, to {}", from, to));
                stats.success += 1;
            },
            Err(e) => {
                verbose(verbosity, &format!("Transfer failed from {}, to {}", from, to));
                stats.failed += 1;
            }
        }

        stdout_injection_stats(&stats, verbosity);
    }

    println!(
        "\rTotal: {:<10} Success: {:<10} Failed: {:<10}",
        stats.total, stats.success, stats.failed,
    );

}

pub async fn message(tps: &usize, eoa: &usize, duration: &usize, rpc_url: &String, verbosity: &bool) {
    let shardus_crypto = crypto::ShardusCrypto::new("69fa4195670576c0160d660c3be36556ff8d504725be8a59b5a96509e0c994bc");
    let mut identities = Vec::new();
    
    let rpc_url_cloned = rpc_url.clone();
    for _ in 0..*eoa {
        let signer = PrivateKeySigner::random();
        let alias = generate_random_string(10);
        let tx = transactions::build_register_transaction(&shardus_crypto, &signer, &alias);
        match transactions::inject_transaction(&transactions::LiberdusTransactions::Register(tx), &rpc_url_cloned.clone()).await {
            Ok(resp) => {
                let stringify = serde_json::to_value(resp.clone()).unwrap().to_string();
                if resp.clone().result.is_none() || (resp.clone().result.unwrap().success == false) {
                    verbose(verbosity, &format!("Failed to register {:?}, Error {:?}", signer.address(), stringify));
                    continue;
                }

                verbose(verbosity, &format!("Registered {:?}, TxId {:?}", signer.address(), resp.clone().result.unwrap().txId));
                identities.push(LiberdusIdentity{
                    alias,
                    signer,
                });
            },
            Err(e) => {
                verbose(verbosity, &format!("Failed to register {:?}", signer.address()));
            }
        }
    }


    println!("Registered {} successful wallets", identities.len());

    println!("Waiting for 30 seconds before injecting friend transactions");

    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

    println!("Injecting transactions");

    let duration = tokio::time::Duration::from_secs(*duration as u64);
    let start_time = tokio::time::Instant::now();
    let interval = tokio::time::Duration::from_secs_f64(1.0 / *tps as f64);
    let mut interval_timer = tokio::time::interval(interval);


    let (transmitter, mut receiver) = tokio::sync::mpsc::unbounded_channel::<
        (transactions::MessageTransaction, Result<rpc::RpcResponse,_>)
    >();

    let rpc_url_long_live = rpc_url.clone();

    tokio::spawn(async move {

        let long_live_transmitter = transmitter.clone();


        let sc = Arc::new(shardus_crypto);
        let long_live_wallet = Arc::new(identities).clone();
        while start_time.elapsed() < duration {
            interval_timer.tick().await;

            let sc = Arc::clone(&sc);

            let wl = long_live_wallet.clone();

            // make sure the from and to are not the same
            let from = rand::thread_rng().gen_range(0..wl.len());
            let mut to = rand::thread_rng().gen_range(0..wl.len());

            while from == to {
                to = rand::thread_rng().gen_range(0..wl.len());
            }
            

            let transmitter = long_live_transmitter.clone();

            let rpc_url_for_detached_thread = rpc_url_long_live.clone();
            tokio::spawn(async move {
                let from = &wl[from];
                let to = &wl[to];
                let message = generate_random_string(30);
                let tx = transactions::build_message_transaction(&*Arc::clone(&sc), &from.signer, &to.signer.address(), &message);
                let resp = transactions::inject_transaction(
                    &transactions::LiberdusTransactions::Message(tx.clone()),
                    &rpc_url_for_detached_thread
                    ).await;

                transmitter.send((tx, resp)).unwrap();
            });

        }
    });

    let mut stats = InjectionStats{
        total: 0,
        success: 0,
        failed: 0,
    };

    while let Some((tx, resp)) = receiver.recv().await {
        let from = tx.from.clone();
        let to = tx.to.clone();
        stats.total += 1;
        match resp {
            Ok(resp) => {
                if resp.result.is_none() || (resp.result.unwrap().success == false) {
                    stats.failed += 1;
                    verbose(verbosity, &format!("Message failed from {}, to {}", from, to));
                    continue;
                }

                verbose(verbosity, &format!("Message success from {}, to {}", from, to));
                stats.success += 1;
            },
            Err(e) => {
                verbose(verbosity, &format!("Message failed from {}, to {}", from, to));
                stats.failed += 1;
            }
        }

        stdout_injection_stats(&stats, verbosity);
    }

    println!(
        "\rTotal: {:<10} Success: {:<10} Failed: {:<10}",
        stats.total, stats.success, stats.failed,
    );
}



fn generate_random_string(length: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();

    // Generate a random string by selecting random characters from the CHARSET
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len()); // Generate a random index
            CHARSET[idx] as char
        })
        .collect()
}


pub fn stdout_injection_stats(stats: &InjectionStats, verbosity: &bool) {
    if *verbosity {
        return;
    }
    let failure_rates = (stats.failed as f64 / stats.total as f64) * 100.0;
    print!(
        "\rTotal: {:<10} Success: {:<10} Failed: {:<10} Failure: {:<10.2}%",
        stats.total, stats.success, stats.failed, failure_rates
    );
    std::io::stdout().flush().unwrap(); 
}

