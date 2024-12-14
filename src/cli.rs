use std::path::PathBuf;
use clap::{arg, command, value_parser, ArgAction, Command};
use crate::load_injector;

pub fn get_commands() -> Command  {
    command!() // requires `cargo` feature
    .arg(arg!(
            --tx_type <TYPE> "Type of Transaction to test"
        )
        .required(true)
        .value_parser([ "transfer", "register" ])
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
            --duration <SEC> "Duration of the test in seconds. (default: 60)"
        )
        .required(false)
        .value_parser(|s: &str| {
            s.parse::<usize>()
            .map_err(|_| format!("'{}' is not a valid number", s))
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

            &((total_tx / 25.0).round() as usize)
        },
    };

    match tx_type.as_str() {
        "transfer" => {
            load_injector::transfer(tps, eoa, duration).await;
        },
        _ => {
            panic!("Invalid tx_type provided");
        }
    }
}

