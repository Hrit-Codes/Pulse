use clap::{Subcommand,Parser};
use pulse_lib::discover::{broadcaster::broadcast_discover, listener::listen_for_discover};
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

    match cli.command {
        Commands::Discover => {
            if let Err(err) = broadcast_discover().await {
                eprintln!("{}",err);
            }
        }

        Commands::Send => {
            println!("Send placeholder");
        }

        Commands::Listen => {
            if let Err(err) = listen_for_discover().await {
                eprintln ("{}",err)
            }
        }
    }
}
