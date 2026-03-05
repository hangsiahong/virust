mod build;
mod dev;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::process;

#[derive(Parser)]
#[command(name = "virust")]
#[command(about = "Virust CLI tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Build { release: bool },
    Dev { port: u16 },
}

fn main() {
    let cli = Cli::parse();

    if let Err(e) = match cli.command {
        Commands::Build { release } => {
            build::execute(release)
        }
        Commands::Dev { port } => {
            dev::execute(port)
        }
    } {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
