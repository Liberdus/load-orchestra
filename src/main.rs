mod crypto;
mod transactions;
mod rpc;
mod cli;
mod load_injector;
mod proxy;
mod utils;
mod stake;
mod monitor_server;
mod change_config;



#[tokio::main]
async fn main()  {
    let args = cli::get_commands().get_matches();

    cli::execute_command(&args).await;

}



