use clap::{Arg, ArgGroup, arg, command, ArgAction, Command};
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
        Arg::new("rpc")
            .long("rpc")
            .help("Use RPC server to inject transactions")
            .action(clap::ArgAction::SetTrue)
    )
    .arg(
        Arg::new("proxy")
            .long("proxy")
            .help("Use Proxy server to inject transactions")
            .action(clap::ArgAction::SetTrue),
    )
    .group(
        ArgGroup::new("gateway")
            .args(&["rpc", "proxy"])
            .required(false) // Ensure one of these is required
    )
    .arg(
        arg!(
            --gateway_url <URL> "Gateway URL to use. (default: http://0.0.0.0:8545 for rpc) Or (default: http://0.0.0.0:3030 for proxy server)"
        )
        .required(false)
        .value_parser(|s: &str| {
            s.parse::<String>()
        })
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

    let gateway_type;
    if matches.get_flag("proxy") {
        gateway_type = load_injector::GatewayType::Proxy;

    }else{
        gateway_type = load_injector::GatewayType::Rpc;
    }

    let gateway_url = match matches.get_one::<String>("gateway_url") {
        Some(url) => url,
        None => { 
            match gateway_type {
                load_injector::GatewayType::Rpc => "http://0.0.0.0:8545",
                load_injector::GatewayType::Proxy => "http://0.0.0.0:3030",
            }
        },
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
        gateway_url: gateway_url.to_string(),
        gateway_type,
        verbosity: *verbosity,
    };

    println!("{:?}", args);

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
