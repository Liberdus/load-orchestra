use std::path::PathBuf;
use clap::{arg, command, value_parser, ArgAction, Command};
use crate::load_injector;


pub fn get_commands() -> Command  {
    command!() // requires `cargo` feature
    .arg(arg!(
            --tx_type <TYPE> "Type of Transaction to test"
        )
        .required(true)
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
            --verbose <BOOL> "Std out verbosity, true, false"
        )
        .required(false)
        .value_parser(|s: &str|{
            s.parse::<bool>()
            .map_err(|_| format!("'{}' is not a valid boolean", s))
        }),
    )
    .arg(
        arg!(
            --rpc_url <STRING> "Url of the rpc, (default: http://localhost:8545)"
        )
        .required(false)
        .value_parser(|s: &str| {
            s.parse::<String>()
            .map_err(|_| format!("'{}' is not a valid string", s))
        }),
    )
    .subcommand(
        Command::new("tui")
            .about("Starts the TUI, (still in development)")
            .arg(arg!(-l --list "lists test values").action(ArgAction::SetTrue)),
    )
}

pub async fn execute_command(matches: &clap::ArgMatches) {

    let tx_type = match matches.get_one::<String>("tx_type") {
        Some(tx_type) => tx_type.to_string(),
        None => panic!("No tx_type provided"),

    };

    let tps = match matches.get_one::<usize>("tps") {
        Some(tps) => tps,
        None => &1,
    };

    let duration = match matches.get_one::<usize>("duration") {
        Some(duration) => duration,
        None => &60,
    };

    let eoa = match matches.get_one::<usize>("eoa") {
        Some(eoa) => eoa,
        None => {
            let total_tx = (tps * duration) as f64;

            &((total_tx / 2.0).round() as usize)
        },
    };

    let rpc_url = match matches.get_one::<String>("rpc_url") {
        Some(rpc_url) => rpc_url,
        None => &"http://localhost:8545".to_string(),
    };

    let verbosity = match matches.get_one::<bool>("verbose") {
        Some(verbosity) => verbosity,
        None => &false,
    };

    let eoa_tps = match matches.get_one::<usize>("eoa_tps") {
        Some(eoa_tps) => eoa_tps,
        None => &4,
    };

    let args = load_injector::LoadInjectParams {
        tx_type,
        eoa_tps: *eoa_tps,
        tps: *tps,
        duration: *duration,
        eoa: *eoa,
        rpc_url: rpc_url.to_string(),
        verbosity: *verbosity,
    };

    match args.tx_type.as_str() {
        "transfer" => {
            load_injector::transfer(args).await;
        },
        "message" => {
            load_injector::message(args).await;
        },
        _ => {
            panic!("Invalid tx_type provided");
        }
    }
}


pub fn verbose(verbosity: &bool, message: &str) {
    if *verbosity {
        println!("{}", message);
    }
}
