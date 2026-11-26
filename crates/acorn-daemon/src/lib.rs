//! Persistent ACORN daemon configuration and foreground process composition.
extern crate alloc;

use acorn_store::daemon::{DaemonStore, StoreError};
use alloc::sync::Arc;
pub use config::{BindSettings, ConfigError, DaemonConfig, JobConfig, Jobs, TlsSettings};
use control::JobControl;
use core::future::Future;
pub use install::{TargetPathError, TargetPaths};
use runner::RunLimits;
use scheduler::{Scheduler, SchedulerError};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::sync::oneshot;
use tracing::info;

pub mod config;
pub mod control;
pub mod install;
pub mod reload;
pub mod runner;
pub mod scheduler;

const TOKEN_FILE: &str = "token";
/// Result returned by daemon process operations.
pub type DaemonResult<T> = Result<T, DaemonError>;
/// Failure while running the foreground daemon process.
#[derive(Debug, Error)]
pub enum DaemonError {
    /// Configuration loading or validation failed.
    #[error(transparent)]
    Config(#[from] ConfigError),
    /// The daemon data directory could not be created.
    #[error("failed to prepare daemon data directory: {source}")]
    DataDirectory {
        /// Filesystem failure.
        #[source]
        source: io::Error,
    },
    /// Platform daemon paths could not be resolved.
    #[error(transparent)]
    Target(#[from] TargetPathError),
    /// Durable daemon history could not be initialized.
    #[error(transparent)]
    Store(#[from] StoreError),
    /// The daemon token file could not be read.
    #[error("failed to read protected daemon token: {source}")]
    TokenRead {
        /// Filesystem failure.
        #[source]
        source: io::Error,
    },
    /// The daemon token file is empty or its protection cannot be verified.
    #[error("daemon token file must be a nonempty owner-only regular file with verifiable permissions")]
    TokenProtection,
    /// An existing or daemon job method could not be registered.
    #[error(transparent)]
    Service(#[from] acorn_service::ServiceError),
    /// Protocol listener setup or serving failed.
    #[error("daemon server failed: {source}")]
    Server {
        /// Protocol error.
        #[source]
        source: Box<dyn core::error::Error + Send + Sync>,
    },
    /// The resolved server listeners differ from the validated daemon configuration.
    #[error("daemon listener settings were overridden by the environment")]
    ListenerOverride,
    /// The scheduler or worker failed.
    #[error(transparent)]
    Scheduler(#[from] SchedulerError),
    /// The scheduler thread panicked.
    #[error("daemon scheduler thread panicked")]
    WorkerPanicked,
    /// The worker exited without announcing successful initialization.
    #[error("daemon worker stopped before initialization")]
    WorkerStoppedBeforeReady,
}
fn read_token(path: &Path) -> DaemonResult<String> {
    let protection = fs::symlink_metadata(path)
        .map_err(|source| DaemonError::TokenRead { source })
        .and_then(|metadata| {
            let regular = metadata.file_type().is_file();
            #[cfg(unix)]
            let private = {
                use std::os::unix::fs::PermissionsExt;
                metadata.permissions().mode() & 0o077 == 0
            };
            #[cfg(not(unix))]
            let private = false;
            (regular && private).then_some(()).ok_or(DaemonError::TokenProtection)
        });
    protection.and_then(|()| {
        fs::read_to_string(path)
            .map_err(|source| DaemonError::TokenRead { source })
            .and_then(|contents| {
                let token = contents.trim();
                (!token.is_empty() && !token.chars().any(char::is_control))
                    .then(|| token.to_string())
                    .ok_or(DaemonError::TokenProtection)
            })
    })
}
async fn run_composed<F>(
    config: DaemonConfig,
    config_path: PathBuf,
    store: Arc<DaemonStore>,
    binary: PathBuf,
    token: String,
    local_socket: Option<PathBuf>,
    shutdown: F,
) -> DaemonResult<()>
where
    F: Future<Output = io::Result<()>>,
{
    let server = server_config(&config, token);
    match server {
        | Err(error) => Err(error),
        | Ok(server) => {
            let (scheduler, handle) = Scheduler::new(config.jobs().to_vec(), Arc::clone(&store), binary, RunLimits::default());
            let (ready_sender, ready_receiver) = oneshot::channel();
            let (ended_sender, ended_receiver) = oneshot::channel();
            let worker = tokio::task::spawn_blocking(move || {
                let result = scheduler.run_with_ready(|| {
                    let _ = ready_sender.send(());
                });
                let _ = ended_sender.send(());
                result
            });
            match ready_receiver.await {
                | Err(_) => worker
                    .await
                    .map_err(|_| DaemonError::WorkerPanicked)
                    .and_then(|result| result.map_err(DaemonError::from))
                    .and(Err(DaemonError::WorkerStoppedBeforeReady)),
                | Ok(()) => {
                    let registry = JobControl::new(config_path, config, handle.clone(), store).registry();
                    let served = match registry {
                        | Err(error) => Err(DaemonError::Service(error)),
                        | Ok(registry) => {
                            let stop = async move {
                                tokio::select! {
                                    signal = shutdown => signal.map_err(|error| Box::new(error) as Box<dyn core::error::Error + Send + Sync>),
                                    _ = ended_receiver => Err("daemon worker stopped unexpectedly".into()),
                                }
                            };
                            #[cfg(unix)]
                            let result = match local_socket {
                                | Some(path) => acorn_server::serve_with_registry_on_unix(server, registry, path, stop).await,
                                | None => acorn_server::serve_with_registry(server, registry, stop).await,
                            };
                            #[cfg(not(unix))]
                            let result = {
                                let _ = local_socket;
                                acorn_server::serve_with_registry(server, registry, stop).await
                            };
                            result.map_err(|source| DaemonError::Server { source })
                        }
                    };
                    handle.stop();
                    worker
                        .await
                        .map_err(|_| DaemonError::WorkerPanicked)
                        .and_then(|result| result.map_err(DaemonError::from))
                        .and(served)
                }
            }
        }
    }
}

/// Validate configuration, run scheduled jobs, and wait for foreground shutdown.
pub async fn run_foreground(path: impl AsRef<Path>) -> DaemonResult<()> {
    let path = path.as_ref();
    match DaemonConfig::load(path) {
        | Ok(config) => {
            let prepared = TargetPaths::resolve()
                .map_err(DaemonError::from)
                .and_then(|targets| {
                    prepare_data_directory(targets.data())
                        .map_err(|source| DaemonError::DataDirectory { source })
                        .map(|()| targets)
                })
                .and_then(|targets| read_token(&targets.data().join(TOKEN_FILE)).map(|token| (targets, token)))
                .and_then(|(targets, token)| {
                    DaemonStore::open(targets.data().join("history.sqlite3"))
                        .map_err(DaemonError::from)
                        .map(|store| (targets, token, store))
                });
            match prepared {
                | Err(error) => Err(error),
                | Ok((targets, token, store)) => {
                    info!(config = %path.display(), jobs = config.jobs().len(), "Daemon configuration validated");
                    #[cfg(unix)]
                    let local_socket = config
                        .bind()
                        .rpc()
                        .map(|address| acorn_server::local::socket_path(targets.data(), address.port()));
                    #[cfg(not(unix))]
                    let local_socket = None;
                    #[cfg(unix)]
                    let shutdown = shutdown_signal();
                    #[cfg(not(unix))]
                    let shutdown = tokio::signal::ctrl_c();
                    run_composed(
                        config,
                        path.to_path_buf(),
                        Arc::new(store),
                        targets.binary().to_path_buf(),
                        token,
                        local_socket,
                        shutdown,
                    )
                    .await
                }
            }
        }
        | Err(error) => Err(DaemonError::Config(error)),
    }
}
#[cfg(unix)]
async fn shutdown_signal() -> io::Result<()> {
    match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
        | Ok(mut terminate) => tokio::select! {
            result = tokio::signal::ctrl_c() => result,
            _ = terminate.recv() => Ok(()),
        },
        | Err(error) => Err(error),
    }
}
fn prepare_data_directory(path: &Path) -> io::Result<()> {
    #[cfg(unix)]
    let result = acorn_server::local::prepare_private_dir(path);
    #[cfg(not(unix))]
    let result = fs::create_dir_all(path);
    result
}
fn server_config(config: &DaemonConfig, token: String) -> DaemonResult<acorn_server::ServerConfig> {
    let rpc = config.bind().rpc();
    let grpc = config.bind().grpc();
    let args = acorn_server::ServerArgs {
        allow_mutation: rpc.is_some_and(|address| address.ip().is_loopback()),
        disable_grpc: grpc.is_none(),
        disable_json_rpc: rpc.is_none(),
        grpc_bind: grpc,
        json_rpc_bind: rpc,
        tls_cert: config.tls().map(|tls| tls.cert().to_path_buf()),
        tls_key: config.tls().map(|tls| tls.key().to_path_buf()),
        ..acorn_server::ServerArgs::default()
    };
    acorn_server::ServerConfig::resolve_with_token(args, token)
        .map_err(|source| DaemonError::Server { source })
        .and_then(|mut server| {
            let matching = server.json_rpc_bind == rpc
                && server.grpc_bind == grpc
                && server.tls_cert.as_deref() == config.tls().map(|tls| tls.cert())
                && server.tls_key.as_deref() == config.tls().map(|tls| tls.key());
            matching.then_some(()).ok_or(DaemonError::ListenerOverride).map(|()| {
                server.allow_mutation = rpc.is_some_and(|address| address.ip().is_loopback());
                server
            })
        })
}
#[cfg(test)]
mod tests;
