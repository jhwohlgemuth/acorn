//! Foreground entry point for the ACORN daemon.
#[cfg(target_os = "linux")]
use acorn_daemon::install::linux;
#[cfg(target_os = "macos")]
use acorn_daemon::install::macos;
use acorn_daemon::{run_foreground, TargetPaths};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Subcommand)]
enum Command {
    /// Serve authenticated ACORN APIs and scheduled jobs in the foreground.
    Run {
        /// Optional ACRE JSONC path; platform defaults are used when omitted.
        #[arg(value_name = "CONFIG")]
        config: Option<PathBuf>,
    },
    /// Register and start the platform service for this user.
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    Install {
        /// ACRE configuration path.
        #[arg(long)]
        config: PathBuf,
        /// Keep this user's service manager after logout.
        #[arg(long)]
        linger: bool,
    },
    /// Remove this user's registered service.
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    Uninstall,
    /// Start this user's registered service.
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    Start,
    /// Stop this user's registered service.
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    Stop,
    /// Restart this user's registered service.
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    Restart,
    /// Show this user's registered service status.
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    Status,
}
#[derive(Debug, Parser)]
#[command(about = "Run persistent ACORN APIs and configured jobs", version)]
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
        #[cfg(target_os = "linux")]
        | Command::Install { config, linger } => linux::install(&config, linger).map_err(|error| Box::new(error) as Box<dyn core::error::Error>),
        #[cfg(target_os = "linux")]
        | Command::Uninstall => linux::uninstall().map_err(|error| Box::new(error) as Box<dyn core::error::Error>),
        #[cfg(target_os = "linux")]
        | Command::Start => linux::start().map_err(|error| Box::new(error) as Box<dyn core::error::Error>),
        #[cfg(target_os = "linux")]
        | Command::Stop => linux::stop().map_err(|error| Box::new(error) as Box<dyn core::error::Error>),
        #[cfg(target_os = "linux")]
        | Command::Restart => linux::restart().map_err(|error| Box::new(error) as Box<dyn core::error::Error>),
        #[cfg(target_os = "linux")]
        | Command::Status => linux::status().map_err(|error| Box::new(error) as Box<dyn core::error::Error>),
        #[cfg(target_os = "macos")]
        | Command::Install { config, linger } => match linger {
            | true => Err(Box::new(std::io::Error::other("--linger applies only to Linux")) as Box<dyn core::error::Error>),
            | false => macos::install(&config).map_err(|error| Box::new(error) as Box<dyn core::error::Error>),
        },
        #[cfg(target_os = "macos")]
        | Command::Uninstall => macos::uninstall().map_err(|error| Box::new(error) as Box<dyn core::error::Error>),
        #[cfg(target_os = "macos")]
        | Command::Start => macos::start().map_err(|error| Box::new(error) as Box<dyn core::error::Error>),
        #[cfg(target_os = "macos")]
        | Command::Stop => macos::stop().map_err(|error| Box::new(error) as Box<dyn core::error::Error>),
        #[cfg(target_os = "macos")]
        | Command::Restart => macos::restart().map_err(|error| Box::new(error) as Box<dyn core::error::Error>),
        #[cfg(target_os = "macos")]
        | Command::Status => macos::status().map_err(|error| Box::new(error) as Box<dyn core::error::Error>),
    }
}
