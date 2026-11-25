//! Local daemon lifecycle and authenticated job control.
use crate::cli::arguments::DaemonCommand;
use acorn::io::{config::application::ApplicationConfiguration, InputOutput};
#[cfg(unix)]
use acorn::util::constants::app::DAEMON_NAMESPACE;
use acorn::util::constants::app::{APPLICATION, DEFAULT_CONFIG_FILENAMES, ORGANIZATION, QUALIFIER};
use acorn_cmd::{args, cmd};
use acorn_host::terminal::Label;
use acorn_server::client::JsonRpcClient;
#[cfg(unix)]
use acorn_service::Memory;
use color_eyre::{eyre, eyre::eyre, Result};
#[cfg(unix)]
use core::net::SocketAddr;
#[cfg(unix)]
use core::time::Duration;
use directories::ProjectDirs;
use serde_json::{json, Value};
use std::env;
#[cfg(unix)]
use std::fs;
use std::path::{Path, PathBuf};
use tracing::info;

fn project_directories() -> Result<ProjectDirs> {
    ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION).ok_or_else(|| eyre!("ACORN application directories are unavailable"))
}
fn config_path(override_path: Option<&Path>) -> Result<PathBuf> {
    override_path.map_or_else(
        || project_directories().map(|dirs| dirs.config_dir().join(DEFAULT_CONFIG_FILENAMES[1])),
        |path| Ok(path.to_path_buf()),
    )
}
#[cfg(unix)]
fn read_token(path: &Path) -> Result<String> {
    fs::symlink_metadata(path)
        .map_err(|error| eyre!("Cannot inspect daemon token {}: {error}", path.display()))
        .and_then(|metadata| {
            let private = {
                use std::os::unix::fs::PermissionsExt;
                metadata.permissions().mode() & 0o077 == 0
            };
            (metadata.file_type().is_file() && private)
                .then_some(())
                .ok_or_else(|| eyre!("Daemon token must be a private regular file: {}", path.display()))
        })
        .and_then(|()| fs::read_to_string(path).map_err(|error| eyre!("Cannot read daemon token {}: {error}", path.display())))
        .and_then(|contents| {
            let token = contents.trim();
            (!token.is_empty() && !token.chars().any(char::is_control))
                .then(|| token.to_string())
                .ok_or_else(|| eyre!("Daemon token is empty or malformed: {}", path.display()))
        })
}
#[cfg(unix)]
fn local_client(address: SocketAddr, directory: &Path) -> Result<JsonRpcClient> {
    acorn_server::local::verify_private_dir(directory)
        .map_err(|error| eyre!("Daemon control directory is not private: {error}"))
        .and_then(|()| read_token(&directory.join("token")))
        .and_then(|token| {
            let socket = acorn_server::local::socket_path(directory, address.port());
            JsonRpcClient::new_unix_socket(&socket, token, Duration::from_secs(5), Memory::mb(1)).map_err(eyre::Report::from)
        })
}
fn client(config: Option<&Path>) -> Result<JsonRpcClient> {
    config_path(config)
        .and_then(ApplicationConfiguration::read)
        .and_then(|application| {
            application
                .daemon
                .and_then(|daemon| daemon.bind.rpc)
                .filter(|address| address.ip().is_loopback())
                .ok_or_else(|| eyre!("Daemon configuration needs a local JSON-RPC listener"))
        })
        .and_then(|address| {
            #[cfg(unix)]
            let result = project_directories().and_then(|dirs| local_client(address, &dirs.data_local_dir().join(DAEMON_NAMESPACE)));
            #[cfg(not(unix))]
            let result = {
                let _ = address;
                Err(eyre!("Local daemon job control requires an owner-verified transport"))
            };
            result
        })
}
fn lifecycle(action: &str, config: Option<&Path>, linger: bool) -> Result<()> {
    env::current_exe()
        .map_err(|error| eyre!("Cannot locate the paired daemon executable: {error}"))
        .and_then(|executable| {
            executable
                .parent()
                .map(|parent| parent.join(format!("acornd{}", env::consts::EXE_SUFFIX)))
                .ok_or_else(|| eyre!("Cannot locate the paired daemon executable"))
        })
        .and_then(|binary| {
            let arguments = args![
                action,
                ..config.map_or_else(|| args![], |path| args!["--config", path]),
                ..if linger { args!["--linger"] } else { args![] },
            ];
            cmd!(status binary, arguments).map_err(|error| eyre!("Cannot run acornd {action}: {error}"))
        })
        .and_then(|status| status.success().then_some(()).ok_or_else(|| eyre!("acornd {action} failed: {status}")))
}
async fn call(method: &str, params: Value, config: Option<&Path>) -> Result<()> {
    match client(config) {
        | Err(error) => Err(error),
        | Ok(client) => client
            .call::<_, Value>(method, &params)
            .await
            .map_err(|error| eyre!("Daemon request failed: {error}"))
            .map(|value| info!("{} {value}", Label::output())),
    }
}
/// Execute a local daemon command.
pub async fn run(command: &DaemonCommand) -> Result<()> {
    match command {
        | DaemonCommand::Install { config, linger } => lifecycle("install", Some(config), *linger),
        | DaemonCommand::Uninstall => lifecycle("uninstall", None, false),
        | DaemonCommand::Start => lifecycle("start", None, false),
        | DaemonCommand::Stop => lifecycle("stop", None, false),
        | DaemonCommand::Restart => lifecycle("restart", None, false),
        | DaemonCommand::ServiceStatus => lifecycle("status", None, false),
        | DaemonCommand::List { config } => call("acorn.jobs.list", json!({}), config.as_deref()).await,
        | DaemonCommand::Status { job_id, config } => call("acorn.jobs.status", json!({"jobId":job_id}), config.as_deref()).await,
        | DaemonCommand::History { job_id, limit, config } => {
            call("acorn.jobs.history", json!({"jobId":job_id,"limit":limit}), config.as_deref()).await
        }
        | DaemonCommand::Run { job_id, config } => call("acorn.jobs.run", json!({"jobId":job_id}), config.as_deref()).await,
        | DaemonCommand::Pause { job_id, config } => call("acorn.jobs.pause", json!({"jobId":job_id}), config.as_deref()).await,
        | DaemonCommand::Resume { job_id, config } => call("acorn.jobs.resume", json!({"jobId":job_id}), config.as_deref()).await,
        | DaemonCommand::Reload { config } => call("acorn.jobs.reload", json!({}), config.as_deref()).await,
    }
}

#[cfg(test)]
mod tests;
