use std::{collections::HashMap, net::{IpAddr, SocketAddr}, path::Path, str::FromStr, sync::Arc};

use clap::{Subcommand,Parser};
use pulse_lib::{discover::{DeviceInfo, broadcaster::broadcast_discover, listener::listen_for_discover}, transfer::{receiver::receive_file, sender::send_file}};
use tokio::sync::Mutex;
#[derive(Parser)]
struct Cli{
    #[command(subcommand)]
    command: Commands
}
#[derive(Subcommand)]
enum Commands {
    Discover,
    Send{ip: String, #[arg(short, long)] file: String},
    Receive { #[arg(short, long, default_value = "9000")] port: u16 },
    ListenDiscover,
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

        Commands::Send{ip,file} => {
            let ip_addr = match IpAddr::from_str(&ip) {
                Ok(v) => v,
                Err(err) => {
                    eprintln!("invalid ip address: {}", err);
                    return;
                }
            }; 
            let socket_addr = SocketAddr::new(ip_addr,9000);
            let path = Path::new(&file);
            if let Err(err) = send_file(socket_addr, path).await {
                eprintln!("error sending file: {}",err);
            }
        }

        Commands::Receive{port} => {
            let addr = SocketAddr::new(IpAddr::from_str("0.0.0.0").unwrap(), port);
            if let Err(err) = receive_file(addr).await {
                eprintln!("{}", err);
            } 
        }
        Commands::ListenDiscover=>{ 
            if let Err(err) = listen_for_discover(Arc::clone(&devices)).await {
                eprintln!("{}", err);
            }
        }
    }
}
