mod build;
mod dev;
mod dev_orchestrator;
mod init;

use clap::{Parser, Subcommand};
use std::process;

#[derive(Parser)]
#[command(name = "virust")]
#[command(about = "Virust CLI tool", version = "0.4.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Build {
        #[arg(short, long)]
        release: bool,
    },
    Dev,
    Init {
        /// Project name
        name: String,
        /// Template to use
        #[arg(short, long, default_value = "chat")]
        template: String,
    },
}

fn main() {
    let cli = Cli::parse();

    if let Err(e) = match cli.command {
        Commands::Build { release } => {
            build::execute(release)
        }
        Commands::Dev => {
            tokio::runtime::Runtime::new().unwrap().block_on(dev::execute())
        }
        Commands::Init { name, template } => {
            init::execute(&name, &template)
        }
    } {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
