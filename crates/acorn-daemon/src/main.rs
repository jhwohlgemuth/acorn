//! Foreground entry point for the ACORN daemon.
use acorn_daemon::{run_foreground, TargetPaths};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Subcommand)]
enum Command {
    /// Validate configuration and wait in the foreground.
    Run {
        /// Optional ACRE JSONC path; platform defaults are used when omitted.
        #[arg(value_name = "CONFIG")]
        config: Option<PathBuf>,
    },
}
#[derive(Debug, Parser)]
#[command(about = "Run persistent ACORN APIs and configured jobs")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn core::error::Error>> {
    tracing_subscriber::fmt().init();
    match Cli::parse().command {
        | Command::Run { config } => {
            let path = config.map_or_else(|| TargetPaths::resolve().map(|targets| targets.config().to_path_buf()), Ok);
            match path {
                | Ok(path) => run_foreground(path).await.map_err(|error| Box::new(error) as Box<dyn core::error::Error>),
                | Err(error) => Err(Box::new(error) as Box<dyn core::error::Error>),
            }
        }
    }
}
