mod crypto;
mod transactions;
mod rpc;
mod cli;
mod load_injector;

use alloy::signers::local::PrivateKeySigner;
use tokio::runtime::Runtime;


#[tokio::main]
async fn main()  {
    let args = cli::get_commands().get_matches();

    cli::execute_command(&args).await;

}



