use std::{collections::HashMap, net::IpAddr, sync::Arc};

use clap::{Subcommand,Parser};
use pulse_lib::discover::{DeviceInfo, broadcaster::broadcast_discover, listener::listen_for_discover};
use tokio::sync::Mutex;
#[derive(Parser)]
struct Cli{
    #[command(subcommand)]
    command: Commands
}
#[derive(Subcommand)]
enum Commands {
    Discover,
    Send,
    Listen,
}
#[tokio::main]
async fn main(){
    let cli = Cli::parse();
    let devices:Arc<Mutex<HashMap<String, (DeviceInfo,IpAddr)>>> = Arc::new(Mutex::new(HashMap::new()));
    match cli.command {
        Commands::Discover => {
            if let Err(err) = broadcast_discover(Arc::clone(&devices)).await {
                eprintln!("{}",err);
            }
        }

        Commands::Send => {
            println!("Send placeholder");
        }

        Commands::Listen => {
            if let Err(err) = listen_for_discover(Arc::clone(&devices)).await {
                eprintln!("{}",err)
            }
        }
    }
}
