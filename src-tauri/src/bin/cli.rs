use clap::{Subcommand,Parser};
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

fn main(){
    let cli = Cli::parse();

    match cli.command {
        Commands::Discover => {
            println!("Discover placeholder");
        }

        Commands::Send => {
            println!("Send placeholder");
        }

        Commands::Listen => {
            println!("Listen placeholder");
        }
    }
}
