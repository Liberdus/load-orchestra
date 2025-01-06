mod crypto;
mod transactions;
mod rpc;
mod cli;
mod load_injector;
mod proxy;
mod utils;



#[tokio::main]
async fn main()  {
    let args = cli::get_commands().get_matches();

    cli::execute_command(&args).await;

}



