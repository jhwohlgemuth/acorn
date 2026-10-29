//! Persistent ACORN daemon configuration and foreground process composition.
extern crate alloc;

use std::io;
use std::path::Path;
use thiserror::Error;
use tracing::info;

pub mod config;
pub mod install;

pub use config::{BindSettings, ConfigError, DaemonConfig, JobConfig, Jobs, TlsSettings};
pub use install::{TargetPathError, TargetPaths};

/// Result returned by daemon process operations.
pub type DaemonResult<T> = Result<T, DaemonError>;
/// Failure while running the foreground daemon process.
#[derive(Debug, Error)]
pub enum DaemonError {
    /// Configuration loading or validation failed.
    #[error(transparent)]
    Config(#[from] ConfigError),
    /// Process shutdown notification failed.
    #[error("failed to wait for daemon shutdown: {source}")]
    Shutdown {
        /// Signal adapter failure.
        #[source]
        source: io::Error,
    },
}
/// Validate configuration and wait for foreground shutdown without starting services
pub async fn run_foreground(path: impl AsRef<Path>) -> DaemonResult<()> {
    let path = path.as_ref();
    match DaemonConfig::load(path) {
        | Ok(config) => {
            info!(config = %path.display(), jobs = config.jobs().len(), "Daemon configuration validated");
            tokio::signal::ctrl_c().await.map_err(|source| DaemonError::Shutdown { source })
        }
        | Err(error) => Err(DaemonError::Config(error)),
    }
}
