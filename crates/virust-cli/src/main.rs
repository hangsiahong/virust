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
        /// Build mode: 'cargo' or 'ssg'
        #[arg(long, default_value = "cargo")]
        mode: String,

        /// Output directory (for SSG mode)
        #[arg(short, long)]
        output: Option<String>,

        /// Number of parallel jobs (for SSG mode)
        #[arg(short, long)]
        jobs: Option<usize>,

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
        Commands::Build { mode, output, jobs, release } => {
            let build_mode = match mode.as_str() {
                "ssg" => build::BuildMode::Ssg,
                "cargo" => build::BuildMode::Cargo,
                _ => {
                    eprintln!("Error: Invalid build mode '{}'. Expected 'cargo' or 'ssg'", mode);
                    process::exit(1);
                }
            };

            let cmd = build::BuildCommand {
                mode: build_mode,
                output,
                jobs,
                release,
            };
            cmd.execute()
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
