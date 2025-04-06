mod change_config;
mod cli;
mod crypto;
mod load_injector;
mod monitor_server;
mod proxy;
mod stake;
mod transactions;
mod utils;

#[tokio::main]
async fn main() {
    let args = cli::get_commands().get_matches();

    cli::execute_command(&args).await;
}
