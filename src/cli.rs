use crate::{
    change_config,
    load_injector::{self},
    monitor_server, proxy, stake, transactions,
};
use alloy::signers::local::PrivateKeySigner;
use clap::{arg, command, ArgAction, ArgGroup, Command};

pub fn get_commands() -> Command {
    command!() // requires `cargo` feature
        .subcommand(loadtest_subcommand())
        .subcommand(staking_subcommand())
        .subcommand(change_config_subcommand())
        .subcommand(
            Command::new("tui")
                .about("Starts the TUI, (still in development)")
                .arg(arg!(-l --list "lists test values").action(ArgAction::SetTrue)),
        )
}

pub async fn execute_command(matches: &clap::ArgMatches) {
    match matches.subcommand() {
        Some(("sustain_load", sub_m)) => {
            execute_loadtest_subcommand(sub_m).await;
        }
        Some(("stake", sub_m)) => {
            execute_staking_subcommand(sub_m).await;
        }
        Some(("change_config", sub_m)) => {
            execute_change_config_subcommand(sub_m).await;
        }
        _ => {
            panic!("Invalid subcommand provided");
        }
    }
}

pub fn verbose(verbosity: &bool, message: &str) {
    if *verbosity {
        println!("{}", message);
    }
}

fn change_config_subcommand() -> Command {
    Command::new("change_config")
        .about("Change the configuration of the network")
        .arg(
            arg!(
                --gateway_url <URL> "RPC URL to use. (default: http://0.0.0.0:3030)"
            )
            .required(false)
            .value_parser(|s: &str| s.parse::<String>()),
        )
}

async fn execute_change_config_subcommand(matches: &clap::ArgMatches) {
    let gateway_url = match matches.get_one::<String>("gateway_url") {
        Some(url) => url,
        None => &"http://0.0.0.0:3030".to_string(),
    };

    let full_url = format!("{}/netconfig", gateway_url);
    let resp = proxy::get_request(None, &full_url).await.unwrap();

    let change = match change_config::init(resp["config"].clone()) {
        Ok(Some(v)) => {
            println!("Config: {:?}", v);
            serde_json::to_string(&v).expect("Failed to serialize")
        }
        Ok(None) => {
            panic!("No config selected");
        }
        Err(e) => {
            panic!("Failed to initialize config: {}", e);
        }
    };

    println!("{:?}", change);

    let shardus_crypto = crate::crypto::ShardusCrypto::new(
        "69fa4195670576c0160d660c3be36556ff8d504725be8a59b5a96509e0c994bc",
    );
    let wallet = PrivateKeySigner::random();
    let tx = transactions::build_change_config_transaction(&shardus_crypto, &wallet, -1, &change);

    println!("Transaction: {:?}", tx);

    let client = reqwest::Client::new();
    let resp = match transactions::inject_transaction(
        client,
        &transactions::LiberdusTransactions::ChangeConfig(tx),
        gateway_url,
        &false,
    )
    .await
    {
        Ok(resp) => resp,
        Err(e) => {
            panic!("Failed to inject transaction: {}", e);
        }
    };

    println!("Response: {:?}", resp);
}

fn staking_subcommand() -> Command {
    Command::new("stake")
        .about("Staking nodes")
        .arg(
            arg!(
                --amount <NUMBER> "Staking amount (default: 10)"
            )
            .required(false)
            .value_parser(|s: &str| {
                s.parse::<u128>()
                    .map_err(|_| format!("'{}' is not a valid number", s))
            }),
        )
        .arg(
            arg!(
                --joining "Stake all joining nodes"
            )
            .required(false)
            .action(ArgAction::SetTrue),
        )
        .arg(
            arg!(
                --active "Stake all active nodes"
            )
            .required(false)
            .action(ArgAction::SetTrue),
        )
        .arg(
            arg!(
                --file <PATH> "Stake all nodes in the nodelist file"
            )
            .required(false)
            .value_parser(|s: &str| {
                s.parse::<String>()
                    .map_err(|_| format!("'{}' is not a valid string", s))
            }),
        )
        // .arg(
        //     arg!(
        //         --nominee <STRING> "Address of a particular node's account"
        //     )
        //     .required(false)
        //     .value_parser(|s: &str| {
        //         s.parse::<String>()
        //         .map_err(|_| format!("'{}' is not a valid string", s))
        //     }),
        //
        // )
        .group(
            ArgGroup::new("target")
                .args(["joining", "active", "file"])
                .required(true), // Ensure one of these is required
        )
        .arg(
            arg!(
                --verbose <BOOL> "Std out verbosity"
            )
            .required(false)
            .action(ArgAction::SetTrue),
        )
        .arg(
            arg!(
                --gateway_url <URL> "Gateway URL to use. (default: http://0.0.0.0:3030)"
            )
            .required(false)
            .value_parser(|s: &str| s.parse::<String>()),
        )
        .arg(
            arg!(
                --monitor_url <URL> "Monitor URL to use. (default: http://0.0.0.0:3000)"
            )
            .required(false)
            .value_parser(|s: &str| s.parse::<String>()),
        )
}

fn loadtest_subcommand() -> Command {
    Command::new("sustain_load")
    .about("Inject Transactions for a duration")
    .arg(arg!(
            --tx_type <TYPE> "Type of Transaction to test"
        )
        .required(false)
        .value_parser([ "transfer", "register", "message" ]),
    )
    .arg(
        arg!(
            --tps <NUMBER> "Transactions per second. (default: 1)"
        )
        .required(false)
        .value_parser(|s: &str| {
            s.parse::<usize>()
            .map_err(|_| format!("'{}' is not a valid number", s))
        }),

    )
    .arg(
        arg!(
            --eoa <NUMBER> "Number of address to create. (default: auto). When auto is used, it will be calculated based on the tps and duration"
        ).required(false)
        .value_parser(|s: &str| {
            s.parse::<usize>()
            .map_err(|_| format!("'{}' is not a valid number", s))
        }),
    )
    .arg(
        arg!(
            --eoa_tps <NUMBER> "Tps for registering addresses (default: 4)"
        ).required(false)
        .value_parser(|s: &str| {
            s.parse::<usize>()
            .map_err(|_| format!("'{}' is not a valid number", s))
        }),
    )
    .arg(
        arg!(
            --duration <SEC> "Duration of the test in seconds. (default: 60)"
        )
        .required(false)
        .value_parser(|s: &str| {
            s.parse::<usize>()
            .map_err(|_| format!("'{}' is not a valid number", s))
         }),
    )
    .arg(
        arg!(
            --verbose <BOOL> "Std out verbosity"
        )
        .required(false)
        .action(ArgAction::SetTrue),
    )
    .arg(
        arg!(
            --gateway_url <URL> "Gateway URL to use. (default: http://0.0.0.0:3030)"
        )
        .required(false)
        .value_parser(|s: &str| {
            s.parse::<String>()
        })
    )
}

async fn execute_loadtest_subcommand(matches: &clap::ArgMatches) {
    let tx_type = match matches.get_one::<String>("tx_type") {
        Some(tx_type) => tx_type.to_string(),
        None => panic!("No tx_type provided"),
    };

    let tps = matches.get_one::<usize>("tps").unwrap_or(&1);

    let duration = matches.get_one::<usize>("duration").unwrap_or(&60);

    let eoa = match matches.get_one::<usize>("eoa") {
        Some(eoa) => eoa,
        None => {
            let total_tx = (tps * duration) as f64;

            &((total_tx / 2.0).round() as usize)
        }
    };

    let gateway_url = match matches.get_one::<String>("gateway_url") {
        Some(gateway_url) => gateway_url,
        None => "http://0.0.0.0:3030",
    };

    let verbosity = matches.get_one::<bool>("verbose").unwrap_or(&false);

    let eoa_tps = matches.get_one::<usize>("eoa_tps").unwrap_or(&4);

    let args = load_injector::LoadInjectParams {
        tx_type,
        eoa_tps: *eoa_tps,
        tps: *tps,
        duration: *duration,
        eoa: *eoa,
        gateway_url: gateway_url.to_string(),
        verbosity: *verbosity,
    };

    println!("{:?}", args);

    match args.tx_type.as_str() {
        "transfer" => {
            load_injector::transfer(args).await;
        }
        "message" => {
            load_injector::message(args).await;
        }
        _ => {
            panic!("Invalid tx_type provided");
        }
    }
}

async fn execute_staking_subcommand(matches: &clap::ArgMatches) {
    let amount = matches.get_one::<u128>("amount").unwrap_or(&10);

    let joining = matches.get_flag("joining");

    let active = matches.get_flag("active");

    let mut nominees = match matches.get_one::<String>("file") {
        Some(file) => match stake::load_nominee(file) {
            Ok(nominees) => {
                let mut n = Vec::new();
                for nominee in nominees {
                    n.push(nominee.publicKey);
                }
                n
            }
            Err(e) => {
                panic!("Failed to load nominees: {}", e);
            }
        },
        None => Vec::new(),
    };

    let verbosity = matches.get_one::<bool>("verbose").unwrap_or(&false);

    let monitor_url = match matches.get_one::<String>("monitor_url") {
        Some(monitor_url) => monitor_url,
        None => &"http://0.0.0.0:3000".to_string(),
    };

    let gateway_url = match matches.get_one::<String>("gateway_url") {
        Some(rpc_url) => rpc_url,
        None => &"http://0.0.0.0:3030".to_string(),
    };

    match (joining, active) {
        (true, false) => {
            let joining = monitor_server::collect_joining(monitor_url).await;
            nominees.extend(joining);
        }
        (false, true) => {
            let active = monitor_server::collect_active(monitor_url).await;
            nominees.extend(active);
        }
        _ => {}
    }

    let args = stake::StakingParams {
        gateway_url: gateway_url.to_string(),
        verbose: *verbosity,
        stake_amount: *amount,
    };

    let _ = stake::stake(nominees, &args).await;
}
