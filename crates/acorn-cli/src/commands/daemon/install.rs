//! Transactional activation of a signed Linux daemon bundle.

use super::{
    bundle::{extract_verified, read_signature, verify_bundle},
    client, project_directories,
};
use acorn::util::constants::app::DAEMON_NAMESPACE;
use color_eyre::{eyre::eyre, Result};
use data_encoding::HEXLOWER;
use directories::BaseDirs;
use ring::rand::{SecureRandom, SystemRandom};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

const TARGET: &str = if cfg!(target_arch = "aarch64") {
    "aarch64-unknown-linux-musl"
} else if cfg!(target_arch = "x86_64") {
    "x86_64-unknown-linux-musl"
} else {
    "unsupported-linux-architecture"
};

pub(super) fn active_directory() -> Result<Option<PathBuf>> {
    project_directories().and_then(|directories| {
        let data = directories.data_local_dir().join(DAEMON_NAMESPACE);
        active_name(&data).map(|name| name.map(|name| data.join("releases").join(name)))
    })
}

fn active_name(data: &Path) -> Result<Option<String>> {
    match fs::read_to_string(data.join("active-release")) {
        | Ok(name) => {
            let name = name.trim();
            let valid = !name.is_empty()
                && name.bytes().next().is_some_and(|byte| byte.is_ascii_alphanumeric())
                && name
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'));
            valid
                .then(|| Some(name.to_string()))
                .ok_or_else(|| eyre!("Active release marker is invalid"))
        }
        | Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        | Err(error) => Err(eyre!("Cannot read active release marker: {error}")),
    }
}

pub(super) fn clear_active() -> Result<()> {
    project_directories().and_then(|directories| {
        let path = directories.data_local_dir().join(DAEMON_NAMESPACE).join("active-release");
        match fs::remove_file(&path) {
            | Ok(()) => Ok(()),
            | Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            | Err(error) => Err(eyre!("Cannot clear active release marker: {error}")),
        }
    })
}

fn daemon_action(binary: &Path, action: &str, config: Option<&Path>, linger: bool) -> Result<()> {
    let mut command = Command::new(binary);
    command.arg(action);
    if let Some(config) = config {
        command.arg("--config").arg(config);
    }
    if linger {
        command.arg("--linger");
    }
    command
        .status()
        .map_err(|error| eyre!("Cannot run staged acornd {action}: {error}"))
        .and_then(|status| {
            status
                .success()
                .then_some(())
                .ok_or_else(|| eyre!("Staged acornd {action} failed: {status}"))
        })
}

fn ensure_token(data: &Path) -> Result<()> {
    let path = data.join("token");
    match fs::symlink_metadata(&path) {
        | Ok(_) => super::read_token(&path).map(|_| ()),
        | Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let mut random = [0u8; 32];
            SystemRandom::new()
                .fill(&mut random)
                .map_err(|_| eyre!("Cannot generate daemon token"))
                .and_then(|()| private_file(&path, HEXLOWER.encode(&random).as_bytes(), false))
                .or_else(|error| {
                    if path.exists() {
                        super::read_token(&path).map(|_| ())
                    } else {
                        Err(error)
                    }
                })
        }
        | Err(error) => Err(eyre!("Cannot inspect daemon token: {error}")),
    }
}

fn preflight_existing_service(previous: Option<&str>) -> Result<()> {
    match previous {
        | Some(_) => Ok(()),
        | None => BaseDirs::new()
            .map(|directories| directories.config_dir().join("systemd/user/acornd.service"))
            .ok_or_else(|| eyre!("Cannot inspect the current user service"))
            .and_then(|path| match fs::symlink_metadata(&path) {
                | Ok(_) => Err(eyre!(
                    "An existing daemon service has no signed release marker; uninstall it before installing a bundle"
                )),
                | Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
                | Err(error) => Err(eyre!("Cannot inspect the current user service: {error}")),
            }),
    }
}

fn private_file(path: &Path, bytes: &[u8], executable: bool) -> Result<()> {
    let mode = if executable { 0o700 } else { 0o600 };
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(mode)
        .open(path)
        .and_then(|mut file| file.write_all(bytes).and_then(|()| file.sync_all()))
        .map_err(|error| eyre!("Cannot stage {}: {error}", path.display()))
}

async fn ready(config: &Path) -> Result<()> {
    let mut last = eyre!("Daemon did not become ready");
    let mut success = false;
    for _ in 0..20 {
        let result = match client(Some(config)) {
            | Ok(client) => client
                .call::<_, serde_json::Value>("acorn.jobs.list", &serde_json::json!({}))
                .await
                .map(|_| ())
                .map_err(color_eyre::Report::from),
            | Err(error) => Err(error),
        };
        match result {
            | Ok(()) => {
                success = true;
                break;
            }
            | Err(error) => {
                last = eyre!("Daemon readiness check failed: {error}");
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }
    }
    if success {
        Ok(())
    } else {
        Err(last)
    }
}

async fn rollback(data: &Path, previous: Option<&str>, new_binary: &Path) -> Result<()> {
    match previous {
        | Some(previous) => {
            let directory = data.join("releases").join(previous);
            let config = directory.join("config.jsonc");
            match daemon_action(&directory.join("acornd"), "install", Some(&config), false) {
                | Ok(()) => ready(&config).await,
                | Err(error) => Err(error),
            }
        }
        | None => daemon_action(new_binary, "uninstall", None, false),
    }
}

/// Verify, stage, and activate a signed Linux daemon release with rollback.
pub(super) async fn run(bundle: &Path, config: &Path, linger: bool) -> Result<()> {
    let prepared = version_from_name(bundle).and_then(|version| {
        project_directories().and_then(|directories| {
            let data = directories.data_local_dir().join(DAEMON_NAMESPACE);
            acorn_server::local::prepare_private_dir(&data)
                .map_err(|error| eyre!("Cannot protect daemon data directory: {error}"))
                .and_then(|()| active_name(&data))
                .and_then(|previous| preflight_existing_service(previous.as_deref()).map(|()| previous))
                .and_then(|previous| {
                    let releases = data.join("releases");
                    acorn_server::local::prepare_private_dir(&releases)
                        .map_err(|error| eyre!("Cannot protect release directory: {error}"))
                        .and_then(|()| {
                            let name = format!("{version}-{TARGET}-{}", nanoid::nanoid!());
                            let directory = releases.join(&name);
                            fs::DirBuilder::new()
                                .mode(0o700)
                                .create(&directory)
                                .map_err(|error| eyre!("Cannot create staged release: {error}"))
                                .map(|()| (data, directory, name, previous, version))
                        })
                })
        })
    });
    match prepared {
        | Err(error) => Err(error),
        | Ok((data, directory, name, previous, version)) => {
            let binary = directory.join("acornd");
            let staged_config = directory.join("config.jsonc");
            let staged = stage_archive(bundle, &directory.join("bundle.tar.gz"), &version)
                .and_then(|()| fs::read(config).map_err(|error| eyre!("Cannot read daemon configuration: {error}")))
                .and_then(|bytes| private_file(&staged_config, &bytes, false))
                .and_then(|()| ensure_token(&data));
            match staged {
                | Err(error) => {
                    let _ = fs::remove_dir_all(&directory);
                    Err(error)
                }
                | Ok(()) => {
                    let activated = daemon_action(&binary, "install", Some(&staged_config), linger);
                    let activated = match activated {
                        | Ok(()) => ready(&staged_config).await,
                        | Err(error) => Err(error),
                    };
                    match activated.and_then(|()| set_active(&data, &name)) {
                        | Ok(()) => Ok(()),
                        | Err(error) => match rollback(&data, previous.as_deref(), &binary).await {
                            | Ok(()) => {
                                let _ = fs::remove_dir_all(&directory);
                                Err(error)
                            }
                            | Err(rollback_error) => Err(eyre!("Activation failed: {error}; rollback failed: {rollback_error}")),
                        },
                    }
                }
            }
        }
    }
}

fn set_active(data: &Path, name: &str) -> Result<()> {
    let temporary = data.join(format!(".active-release-{}", nanoid::nanoid!()));
    private_file(&temporary, name.as_bytes(), false)
        .and_then(|()| fs::rename(&temporary, data.join("active-release")).map_err(|error| eyre!("Cannot activate release marker: {error}")))
}

fn stage_archive(bundle: &Path, destination: &Path, version: &str) -> Result<()> {
    let signature_path = PathBuf::from(format!("{}.minisig", bundle.display()));
    read_signature(&signature_path).and_then(|signature| {
        fs::metadata(bundle)
            .map_err(|error| eyre!("Cannot inspect bundle: {error}"))
            .and_then(|metadata| {
                (metadata.len() <= 512 * 1024 * 1024)
                    .then_some(())
                    .ok_or_else(|| eyre!("Bundle exceeds the size limit"))
            })
            .and_then(|()| fs::copy(bundle, destination).map_err(|error| eyre!("Cannot stage bundle: {error}")))
            .and_then(|_| File::open(destination).map_err(|error| eyre!("Cannot open staged bundle: {error}")))
            .and_then(|mut archive| {
                verify_bundle(&mut archive, &signature, version, TARGET)
                    .and_then(|_| extract_verified(&mut archive, destination.parent().unwrap_or_else(|| Path::new("."))))
            })
    })
}

fn version_from_name(path: &Path) -> Result<String> {
    path.file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| name.strip_prefix("acornd-v"))
        .and_then(|name| name.strip_suffix(&format!("-{TARGET}.tar.gz")))
        .filter(|version| {
            version.split('.').count() == 3
                && version
                    .split('.')
                    .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
        })
        .map(str::to_owned)
        .ok_or_else(|| eyre!("Bundle name must be acornd-vVERSION-{TARGET}.tar.gz"))
}
