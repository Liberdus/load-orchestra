use alloy::signers::local::PrivateKeySigner;
use crate::{ cli::verbose, crypto::{self, ShardusCrypto}, rpc, transactions };
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

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let log_file_path = format!("./artifacts/test_transfer_{}.txt", now.to_string());
    let shardus_crypto = Arc::new(crypto::ShardusCrypto::new("69fa4195670576c0160d660c3be36556ff8d504725be8a59b5a96509e0c994bc"));
    
    let rpc_url_cloned = rpc_url.clone();

    let wallets = generate_register_wallets(&4, eoa, &rpc_url_cloned, shardus_crypto.clone()).await;

    println!("Registered {} successful wallets", wallets.len());

    if wallets.len() < 2 {
        println!("Couldn't register enough wallets to conduct test, shuting down...");
        return;
    }

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


        let sc = Arc::clone(&shardus_crypto);
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

        let dump = serde_json::json!({
            "tx": tx,
            "result": match resp {
                Ok(resp) => {
                    let resp_cloned = resp.clone();
                    if resp.result.is_none() || (resp.result.unwrap().success == false) {
                        stats.failed += 1;
                        verbose(verbosity, &format!("Transfer failed from {}, to {}", from, to));
                    }
                    else{
                        verbose(verbosity, &format!("Transfer success from {}, to {}", from, to));
                        stats.success += 1;
                    }
                    resp_cloned

                },
                Err(e) => {
                    verbose(verbosity, &format!("Transfer failed from {}, to {}", from, to));
                    stats.failed += 1;
                    
                    rpc::RpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: 1,
                        result: None,
                        error: Some(rpc::RpcError{
                            code: 0,
                            message: e.to_string(),
                        })
                    }
                }
            }

        });

        let _ = append_json_to_file(&log_file_path, &dump);
        stdout_injection_stats(&stats, verbosity);
    }

    println!(
        "\rTotal: {:<10} Success: {:<10} Failed: {:<10}",
        stats.total, stats.success, stats.failed,
    );

}

pub async fn message(tps: &usize, eoa: &usize, duration: &usize, rpc_url: &String, verbosity: &bool) {
    let shardus_crypto = Arc::new(crypto::ShardusCrypto::new("69fa4195670576c0160d660c3be36556ff8d504725be8a59b5a96509e0c994bc"));
    
    let rpc_url_cloned = rpc_url.clone();


    let wallets = generate_register_wallets(&4, eoa, &rpc_url_cloned, shardus_crypto.clone()).await;


    println!("Registered {} successful wallets", wallets.len());

    if wallets.len() < 2  {
        println!("Not Enough Wallets to conduct testing...., Killing Process");
        return;
    }

    println!("Waiting for 30 seconds before injecting Message transactions");

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


        let sc = Arc::clone(&shardus_crypto);
        let long_live_wallet = Arc::new(wallets).clone();
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
                let tx = transactions::build_message_transaction(&*Arc::clone(&sc), &from, &to.address(), &message);
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

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let log_file_path = format!("./artifacts/test_message_{}.txt", now.to_string());

    while let Some((tx, resp)) = receiver.recv().await {
        let from = tx.from.clone();
        let to = tx.to.clone();
        stats.total += 1;

        let dump = serde_json::json!({
            "tx": serde_json::to_value(&tx).expect(""),
            "result": match resp {
                Ok(resp) => {
                    if resp.result.clone().is_none() || (resp.result.clone().unwrap().success == false) {
                        stats.failed += 1;
                        verbose(verbosity, &format!("Message failed from {}, to {}", from, to));
                    }
                    else{
                        verbose(verbosity, &format!("Message success from {}, to {}", from, to));
                        stats.success += 1;
                    }


                    resp
                },
                Err(e) => {
                    verbose(verbosity, &format!("Message failed from {}, to {}", from, to));
                    stats.failed += 1;

                    rpc::RpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: 1,
                        result: None,
                        error: Some(rpc::RpcError{
                            code: 0,
                            message: e.to_string(),
                        })
                    }
                }
            }


        });

        let _ = append_json_to_file(&log_file_path, &dump);
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

fn stdout_register_progress(max: usize, progress: usize) {
    let percentage = (progress as f64 / max as f64) * 100.0;
    print!(
        "\rRegistering {:?} / {:?} Wallets. ({:<.2}%)",
        progress, max, percentage
    );
    std::io::stdout().flush().unwrap(); 
}


fn append_json_to_file(file_path: &str, json_value: &serde_json::Value) -> std::io::Result<()> {
    let path = std::path::Path::new(file_path);

    // Ensure the parent directories exist
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?; // Creates all directories in the path
    }
    let file = std::fs::OpenOptions::new()
        .create(true)   
        .append(true)   
        .open(file_path)?;

    let mut writer = std::io::BufWriter::new(file);

    let json_string = serde_json::to_string(json_value)?;

    writeln!(writer, "{}", json_string)?;

    Ok(())
}



async fn generate_register_wallets(tps: &usize, eoa: &usize, rpc_url: &String, shardus_crypto: Arc<ShardusCrypto>) -> Vec<PrivateKeySigner> {

    let mut signers = Vec::new();
    let interval = tokio::time::Duration::from_secs_f64(1.0 / *tps as f64);
    let mut interval_timer = tokio::time::interval(interval);

    let (transmitter, mut receiver) = tokio::sync::mpsc::unbounded_channel::<
        (PrivateKeySigner, Result<rpc::RpcResponse,_>)
    >();


    let rpc_url = rpc_url.clone();
    let eoa_moved = eoa.clone();
    tokio::spawn(async move {
        let transmitter = transmitter.clone();
        for _ in 0..eoa_moved {
            interval_timer.tick().await;
            let crypto = Arc::clone(&shardus_crypto);
            let rpc_url = rpc_url.clone();


            let transmitter = transmitter.clone();
            tokio::spawn(async move {
                let signer = PrivateKeySigner::random();
                let tx = transactions::build_register_transaction(&*Arc::clone(&crypto), &signer, &generate_random_string(10));
                let resp = transactions::inject_transaction(
                    &transactions::LiberdusTransactions::Register(tx.clone()),
                    &rpc_url
                    ).await;

                transmitter.send((signer, resp)).unwrap();

            });

        }
    });

    while let Some((signer, resp)) = receiver.recv().await {
        match resp {
            Ok(resp) => {
                if resp.result.is_some() && resp.result.unwrap().success == true{
                    signers.push(signer);
                    stdout_register_progress(*eoa, signers.len());
                }
            },
            Err(e) => {

            }

        };
    }

    signers
}
