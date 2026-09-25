use std::{collections::HashMap, net::{IpAddr, SocketAddr}, path::Path, str::FromStr, sync::Arc};

use clap::{Subcommand,Parser};
use pulse_lib::{discover::{DeviceInfo, broadcaster::broadcast_discover, listener::listen_for_discover},
    storage::TransferStore, transfer::{receiver::{receive_file, resume_transfer}, sender::{run_resume_listener, send_file}}};
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
    Resend,
    Resume
}
#[tokio::main]
async fn main(){
    let cli = Cli::parse();
    let device_info;
    let store:Arc<TransferStore>;
    match TransferStore::new() {
        Ok(s) => {
            match s.get_or_create_identity() {
                Ok((id,name)) => {
                    device_info = DeviceInfo::new(id,name,9000);
                },
                Err(err) => {
                    eprintln!("{}",err);
                    return;
                }
            }
            store = Arc::new(s);
        },
        Err(err) => {
            eprintln!("{}",err);
            return;
        }
    }
    let devices:Arc<Mutex<HashMap<String, (DeviceInfo,IpAddr)>>> = Arc::new(Mutex::new(HashMap::new()));
    match cli.command {
        Commands::Discover => {
            if let Err(err) = broadcast_discover(Arc::clone(&devices),device_info).await {
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
            if let Err(err) = send_file(device_info.id,socket_addr, path,Arc::clone(&store)).await {
                eprintln!("error sending file: {}",err);
            }
        }

        Commands::Receive{port} => {
            let addr = SocketAddr::new(IpAddr::from_str("0.0.0.0").unwrap(), port);
            if let Err(err) = receive_file(addr).await {
                eprintln!("error occurred {}", err);
            } 
        }
        Commands::ListenDiscover=>{ 
            if let Err(err) = listen_for_discover(Arc::clone(&devices),device_info).await {
                eprintln!("error occurred {}", err);
            }
        }
        Commands::Resend=> {
            if let Err(err) = run_resume_listener(SocketAddr::new(IpAddr::from_str("0.0.0.0").unwrap(), 9000),
                Arc::clone(&store)).await{
                eprintln!("error {}",err);
            }
        }
        Commands::Resume => {
            let mut devices:HashMap<String, (DeviceInfo,IpAddr)>= HashMap::new(); 
            devices.insert("b72d466e-7d6b-4c70-97f0-6eb50e9a6f23".to_string(), 
                (DeviceInfo::new("b72d466e-7d6b-4c70-97f0-6eb50e9a6f23".to_string(),
                    "Rochak's Macbook".to_string(), 9000),IpAddr::from_str("127.0.0.1").unwrap()));
            let devices = Arc::new(Mutex::new(devices));
            if let Err(err) = resume_transfer("66fbe046-a711-4190-bbe3-0890609b9a4e".to_string(), Arc::clone(&store),
                Arc::clone(&devices)).await {
                eprintln!("error occurred {}",err);
            }
        }
    }
}
