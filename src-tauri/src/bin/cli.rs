use std::{collections::HashMap, net::{IpAddr, SocketAddr}, path::Path, str::FromStr, sync::Arc};

use clap::{Subcommand,Parser};
use pulse_lib::{discover::{DeviceInfo, broadcaster::broadcast_discover, get_devices, listener::listen_for_discover},
    storage::{TransferStore, change_name}, transfer::{receiver::{receive_file, resume_transfer}, sender::{request_to_send_file, 
        run_resume_listener}}};
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
    Resume,
    Run,
    ChangeName
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
            if let Err(err) = broadcast_discover(Arc::clone(&devices),device_info,None).await {
                eprintln!("{}",err);
            }
            println!("{:?}",get_devices(Arc::clone(&devices)).await);
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
            if let Err(err) = request_to_send_file(device_info.id,socket_addr, path,Arc::clone(&store),None).await {
                eprintln!("error sending file: {}",err);
            }
        }

        Commands::Receive{port} => {
            let addr = SocketAddr::new(IpAddr::from_str("0.0.0.0").unwrap(), port);
            if let Err(err) = receive_file(addr,None).await {
                eprintln!("error occurred {}", err);
            } 
        }
        Commands::ListenDiscover=>{ 
            if let Err(err) = listen_for_discover(Arc::clone(&devices),device_info,None).await {
                eprintln!("error occurred {}", err);
            }
        }
        Commands::Resend=> {
            if let Err(err) = run_resume_listener(SocketAddr::new(IpAddr::from_str("0.0.0.0").unwrap(), 9000),
                Arc::clone(&store),None).await{
                eprintln!("error {}",err);
            }
        }
        Commands::Resume => {
            let mut devices:HashMap<String, (DeviceInfo,IpAddr)>= HashMap::new(); 
            devices.insert("76e326b3-c74b-442c-b477-7e0a43d7f5d3".to_string(), 
                (DeviceInfo::new("76e326b3-c74b-442c-b477-7e0a43d7f5d3".to_string(),
                    "User2432".to_string(), 9000),IpAddr::from_str("127.0.0.1").unwrap()));
            let devices = Arc::new(Mutex::new(devices));
            if let Err(err) = resume_transfer("0e069818-6921-40ef-967b-ec66c8027ecc".to_string(), Arc::clone(&store),
                Arc::clone(&devices),None).await {
                eprintln!("error occurred {}",err);
            }
        }
        Commands::Run => {
            println!("peer starting as: {} {}",device_info.name,device_info.id);
        }
        Commands::ChangeName => {
            if let Err(err) = change_name(Arc::clone(&store), "Rochak's Macbook") {
                eprintln!("error changing name: {}",err);
            }

        }
    }
}
