use crate::{
    cli::verbose,
    crypto::{self, ShardusCrypto},
    proxy,
    transactions::{self},
    utils,
};
use alloy::signers::local::PrivateKeySigner;
use rand::{self, Rng};
use std::sync::Arc;

#[derive(Debug)]
pub struct LoadInjectParams {
    pub tx_type: String,
    pub tps: usize,
    pub duration: usize,
    pub eoa: usize,
    pub eoa_tps: usize,
    pub gateway_url: String,
    pub verbosity: bool,
}

pub async fn transfer(load_inject_params: LoadInjectParams) {
    let LoadInjectParams {
        tps,
        duration,
        eoa,
        gateway_url,
        verbosity,
        eoa_tps,
        ..
    } = load_inject_params;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let log_file_path = format!("./artifacts/test_transfer_{}.txt", now);
    let shardus_crypto = Arc::new(crypto::ShardusCrypto::new(
        "69fa4195670576c0160d660c3be36556ff8d504725be8a59b5a96509e0c994bc",
    ));

    let gateway_url_cloned = gateway_url.clone();

    let mut wallets = generate_register_wallets(
        &eoa_tps,
        &eoa,
        &gateway_url_cloned,
        Arc::clone(&shardus_crypto),
        &verbosity,
    )
    .await;

    println!("Waiting for 30 seconds before injecting transactions");

    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

    wallets = validate_filter_failed_register(wallets, &gateway_url, &verbosity).await;

    if wallets.len() < 2 {
        println!("Couldn't register enough wallets to conduct test, shuting down...");
        return;
    }

    println!("Registered {} successful wallets", wallets.len());

    println!("Injecting transactions");

    let duration = tokio::time::Duration::from_secs(duration as u64);
    let start_time = tokio::time::Instant::now();
    let interval = tokio::time::Duration::from_secs_f64(1.0 / tps as f64);
    let mut interval_timer = tokio::time::interval(interval);

    let (transmitter, mut receiver) = tokio::sync::mpsc::unbounded_channel::<(
        transactions::TransferTransaction,
        Result<transactions::InjectedTxResp, String>,
    )>();

    let gateway_url_long_live = gateway_url.clone();
    tokio::spawn(async move {
        // uses ARC internally
        let http_client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap();

        let long_live_transmitter = transmitter.clone();

        let sc = Arc::clone(&shardus_crypto);
        let long_live_wallet = wallets.clone();
        while start_time.elapsed() < duration {
            interval_timer.tick().await;
            let sc = Arc::clone(&sc);
            let wl = long_live_wallet.clone();
            let http_client = http_client.clone();

            // make sure the from and to are not the same
            let from = rand::thread_rng().gen_range(0..wl.len());
            let mut to = rand::thread_rng().gen_range(0..wl.len());

            while from == to {
                to = rand::thread_rng().gen_range(0..wl.len());
            }

            let transmitter = long_live_transmitter.clone();

            let gateway_url_for_detached_thread = gateway_url_long_live.clone();
            tokio::spawn(async move {
                let signers = wl[from].clone();
                let to = wl[to].clone();
                let tx = transactions::build_transfer_transaction(
                    &Arc::clone(&sc),
                    &signers,
                    &to.address(),
                    1,
                );
                let resp = match transactions::inject_transaction(
                    http_client,
                    &transactions::LiberdusTransactions::Transfer(tx.clone()),
                    &gateway_url_for_detached_thread,
                    &verbosity,
                )
                .await
                {
                    Ok(resp) => Ok(resp),
                    Err(e) => Err(e.to_string()),
                };

                transmitter.send((tx, resp)).unwrap();
            });
        }
    });

    let mut stats = utils::InjectionStats {
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
                    if resp.success {
                        stats.success += 1;
                    }
                    else {
                        stats.failed += 1;
                    }
                    resp
                },
                Err(e_str) => {
                    verbose(&verbosity, &format!("Transfer failed from {}, to {}", from, to));
                    stats.failed += 1;

                    transactions::InjectedTxResp {
                        success: false,
                        reason: e_str,
                        status: 500,
                        txId: None
                    }
                }
            }

        });

        let _ = utils::append_json_to_file(&log_file_path, &dump);
        utils::stdout_injection_stats(&stats, &verbosity);
    }

    println!(
        "\rTotal: {:<10} Success: {:<10} Failed: {:<10}",
        stats.total, stats.success, stats.failed,
    );
}

pub async fn message(load_inject_params: LoadInjectParams) {
    let LoadInjectParams {
        tps,
        duration,
        eoa,
        gateway_url,
        verbosity,
        eoa_tps,
        ..
    } = load_inject_params;
    let shardus_crypto = Arc::new(crypto::ShardusCrypto::new(
        "69fa4195670576c0160d660c3be36556ff8d504725be8a59b5a96509e0c994bc",
    ));

    let gateway_url_cloned = gateway_url.clone();

    let wallets = {
        let mut w = generate_register_wallets(
            &eoa_tps,
            &eoa,
            &gateway_url_cloned,
            Arc::clone(&shardus_crypto),
            &verbosity,
        )
        .await;
        println!("Waiting for 30 seconds before injecting Message transactions");

        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        w = validate_filter_failed_register(w, &gateway_url, &verbosity).await;
        w
    };

    println!("Registered {} successful wallets", wallets.len());

    if wallets.len() < 2 {
        println!("Not Enough Wallets to conduct testing...., Killing Process");
        return;
    }

    println!("Injecting transactions");

    let duration = tokio::time::Duration::from_secs(duration as u64);
    let start_time = tokio::time::Instant::now();
    let interval = tokio::time::Duration::from_secs_f64(1.0 / tps as f64);
    let mut interval_timer = tokio::time::interval(interval);

    let (transmitter, mut receiver) = tokio::sync::mpsc::unbounded_channel::<(
        transactions::MessageTransaction,
        Result<transactions::InjectedTxResp, String>,
    )>();

    let rpc_url_long_live = gateway_url.clone();

    tokio::spawn(async move {
        let long_live_transmitter = transmitter.clone();

        let sc = Arc::clone(&shardus_crypto);
        let long_live_wallet = Arc::new(wallets).clone();
        let http_client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap();
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
            let http = http_client.clone();
            tokio::spawn(async move {
                let from = &wl[from];
                let to = &wl[to];
                let message = utils::generate_random_string(30);
                let tx = transactions::build_message_transaction(
                    &Arc::clone(&sc),
                    from,
                    &to.address(),
                    &message,
                );
                let resp = match transactions::inject_transaction(
                    http,
                    &transactions::LiberdusTransactions::Message(tx.clone()),
                    &rpc_url_for_detached_thread,
                    &verbosity,
                )
                .await
                {
                    Ok(resp) => Ok(resp),
                    Err(e) => Err(e.to_string()),
                };

                transmitter.send((tx, resp)).unwrap();
            });
        }
    });

    let mut stats = utils::InjectionStats {
        total: 0,
        success: 0,
        failed: 0,
    };

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let log_file_path = format!("./artifacts/test_message_{}.txt", now);

    while let Some((tx, resp)) = receiver.recv().await {
        let from = tx.from.clone();
        let to = tx.to.clone();
        stats.total += 1;

        let dump = serde_json::json!({
            "tx": serde_json::to_value(&tx).expect(""),
            "result": match resp {
                Ok(resp) => {
                    if resp.success {
                        stats.success += 1;
                    }
                    else {
                        stats.failed += 1;
                    };
                    resp
                },
                Err(e_str) => {
                    verbose(&verbosity, &format!("Message failed from {}, to {}", from, to));
                    stats.failed += 1;
                    transactions::InjectedTxResp {
                        success: false,
                        reason: e_str,
                        status: 500,
                        txId: None
                    }
                }
            }


        });

        let _ = utils::append_json_to_file(&log_file_path, &dump);
        utils::stdout_injection_stats(&stats, &verbosity);
    }

    println!(
        "\rTotal: {:<10} Success: {:<10} Failed: {:<10}",
        stats.total, stats.success, stats.failed,
    );
}

pub async fn generate_register_wallets(
    tps: &usize,
    eoa: &usize,
    gateway_url: &String,
    shardus_crypto: Arc<ShardusCrypto>,
    verbosity: &bool,
) -> Vec<PrivateKeySigner> {
    let mut signers = Vec::new();
    let interval = tokio::time::Duration::from_secs_f64(1.0 / *tps as f64);
    let mut interval_timer = tokio::time::interval(interval);

    let (transmitter, mut receiver) = tokio::sync::mpsc::unbounded_channel::<(
        PrivateKeySigner,
        Result<transactions::InjectedTxResp, String>,
    )>();

    let gateway_url = gateway_url.clone();
    let eoa_moved = *eoa;
    let verbosity = *verbosity;
    tokio::spawn(async move {
        let transmitter = transmitter.clone();
        let http_client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap();
        for _ in 0..eoa_moved {
            interval_timer.tick().await;
            let crypto = Arc::clone(&shardus_crypto);
            let url = gateway_url.clone();

            let transmitter = transmitter.clone();
            let http_client = http_client.clone();
            tokio::spawn(async move {
                let signer = PrivateKeySigner::random();
                let tx = transactions::build_register_transaction(
                    &Arc::clone(&crypto),
                    &signer,
                    &utils::generate_random_string(10),
                );
                let resp = match transactions::inject_transaction(
                    http_client,
                    &transactions::LiberdusTransactions::Register(tx.clone()),
                    &url,
                    &verbosity,
                )
                .await
                {
                    Ok(resp) => Ok(resp),
                    Err(e) => Err(e.to_string()),
                };

                transmitter.send((signer, resp)).unwrap();
            });
        }
    });

    while let Some((signer, resp)) = receiver.recv().await {
        match resp {
            Ok(resp) => {
                if resp.success {
                    signers.push(signer);
                    utils::stdout_register_progress(*eoa, signers.len());
                }
                if !resp.success {
                    verbose(
                        &verbosity,
                        &format!("Failed to register wallet: {}", resp.reason),
                    );
                }
            }
            Err(e_str) => {
                verbose(
                    &verbosity,
                    &format!(
                        "Failed to register wallet: {} Transaction Object likely malformed",
                        e_str
                    ),
                );
            }
        };
    }

    signers
}

async fn validate_filter_failed_register(
    wallets: Vec<PrivateKeySigner>,
    gateway_url: &str,
    verbosity: &bool,
) -> Vec<PrivateKeySigner> {
    verbose(
        verbosity,
        "Filtering wallets that failed to register"
            .to_string()
            .as_str(),
    );

    let mut filtered_wallets = vec![];

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<PrivateKeySigner>();

    for wallet in wallets.iter() {
        let wallet = wallet.clone();
        let transmitter = tx.clone();
        let gateway_url_long_live = gateway_url.to_string();

        tokio::spawn(async move {
            let addr = utils::to_shardus_address(&wallet.address().to_string());
            let url = format!("{}/account/{}", &gateway_url_long_live, addr);
            let resp = proxy::get_request(None, &url).await;

            if let Ok(resp) = resp {
                let json: proxy::GetAccountResp =
                    serde_json::from_value(resp).expect("Failed to parse gateway response");

                if json.account.is_some() {
                    transmitter.send(wallet.clone()).unwrap();
                }
            };

            drop(transmitter);
        });
    }

    drop(tx);

    while let Some(wallet) = rx.recv().await {
        verbose(
            verbosity,
            format!(
                "Successful Register {}",
                utils::to_shardus_address(&wallet.address().to_string())
            )
            .as_str(),
        );
        filtered_wallets.push(wallet);
    }

    filtered_wallets
}
