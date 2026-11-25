//! Owner-scoped systemd user service registration.
use crate::config::DaemonConfig;
use crate::install::TargetPaths;
use acorn_cmd::{args, cmd};
use directories::BaseDirs;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const UNIT_NAME: &str = "acornd.service";
const MARKER: &str = "# Managed by ACORN";
fn failure(message: impl Into<String>) -> io::Error {
    io::Error::other(message.into())
}
fn unit_path() -> io::Result<PathBuf> {
    BaseDirs::new()
        .map(|dirs| dirs.config_dir().join("systemd/user").join(UNIT_NAME))
        .ok_or_else(|| failure("Cannot resolve the user systemd directory"))
}
pub(super) fn quoted(path: &Path) -> io::Result<String> {
    let value = path
        .to_str()
        .filter(|value| !value.chars().any(char::is_control))
        .ok_or_else(|| failure("Service path must be UTF-8 without control characters"));
    value.map(|value| {
        format!(
            "\"{}\"",
            value.replace('%', "%%").replace('$', "$$").replace('\\', "\\\\").replace('"', "\\\"")
        )
    })
}
pub(super) fn render(binary: &Path, config: &Path) -> io::Result<String> {
    quoted(binary).and_then(|binary| {
        quoted(config).map(|config| {
            format!(
                "{MARKER}\n[Unit]\nDescription=ACORN persistent service\nAfter=network-online.target\n\n[Service]\nType=simple\nExecStart={binary} run {config}\nRestart=on-failure\nRestartSec=5s\nUMask=0077\n\n[Install]\nWantedBy=default.target\n"
            )
        })
    })
}
fn systemctl(args: &[&str]) -> io::Result<()> {
    cmd!(status "systemctl", args!["--user", ..args.iter().copied()]).and_then(|status| {
        status
            .success()
            .then_some(())
            .ok_or_else(|| failure(format!("systemctl --user {} failed: {status}", args.join(" "))))
    })
}
pub(super) fn managed_unit(path: &Path) -> io::Result<bool> {
    match fs::symlink_metadata(path) {
        | Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        | Err(error) => Err(error),
        | Ok(metadata) if !metadata.file_type().is_file() => Err(failure("Service unit path is not a regular file")),
        | Ok(_) => fs::read_to_string(path).and_then(|source| {
            source
                .starts_with(MARKER)
                .then_some(true)
                .ok_or_else(|| failure("Refusing to replace a unit not managed by ACORN"))
        }),
    }
}
fn write_unit(path: &Path, source: &str) -> io::Result<()> {
    path.parent()
        .ok_or_else(|| failure("Service unit has no parent directory"))
        .and_then(fs::create_dir_all)
        .and_then(|()| {
            let temporary = path.with_extension(format!("service.{}.tmp", std::process::id()));
            let opened = OpenOptions::new().write(true).create_new(true).open(&temporary);
            opened.and_then(|mut file| {
                #[cfg(unix)]
                let protected = {
                    use std::os::unix::fs::PermissionsExt;
                    file.set_permissions(fs::Permissions::from_mode(0o600))
                };
                #[cfg(not(unix))]
                let protected = Ok(());
                let written = protected
                    .and_then(|()| file.write_all(source.as_bytes()))
                    .and_then(|()| file.sync_all())
                    .and_then(|()| fs::rename(&temporary, path));
                if written.is_err() {
                    let _ = fs::remove_file(&temporary);
                }
                written
            })
        })
}
fn enable_linger() -> io::Result<()> {
    cmd!("id", args!["-un"])
        .and_then(|output| {
            output
                .status
                .success()
                .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
                .filter(|user| !user.is_empty())
                .ok_or_else(|| failure("Cannot determine the current user"))
        })
        .and_then(|user| {
            cmd!(status "loginctl", args!["enable-linger", user]).and_then(|status| {
                status
                    .success()
                    .then_some(())
                    .ok_or_else(|| failure(format!("loginctl enable-linger failed: {status}")))
            })
        })
}
pub(super) fn protected_token(path: &Path) -> io::Result<()> {
    fs::symlink_metadata(path)
        .and_then(|metadata| {
            use std::os::unix::fs::PermissionsExt;
            (metadata.file_type().is_file() && metadata.permissions().mode() & 0o077 == 0)
                .then_some(())
                .ok_or_else(|| failure(format!("Daemon token must be a private regular file: {}", path.display())))
        })
        .and_then(|()| fs::read_to_string(path))
        .and_then(|contents| {
            let token = contents.trim();
            (!token.is_empty() && !token.chars().any(char::is_control))
                .then_some(())
                .ok_or_else(|| failure(format!("Daemon token is empty or malformed: {}", path.display())))
        })
}
fn preflight_token() -> io::Result<()> {
    TargetPaths::resolve()
        .map_err(|error| failure(error.to_string()))
        .map(|paths| paths.data().join("token"))
        .and_then(|path| protected_token(&path))
}
/// Install or update the current user's service and start it.
pub fn install(config: &Path, linger: bool) -> io::Result<()> {
    preflight_token()
        .and_then(|()| fs::canonicalize(config))
        .and_then(|config| {
            DaemonConfig::load(&config)
                .map_err(|error| failure(format!("Invalid daemon configuration: {error}")))
                .map(|_| config)
        })
        .and_then(|config| env::current_exe().and_then(|binary| render(&binary, &config)))
        .and_then(|source| unit_path().and_then(|path| managed_unit(&path).and_then(|_| write_unit(&path, &source))))
        .and_then(|()| systemctl(&["daemon-reload"]))
        .and_then(|()| systemctl(&["enable", UNIT_NAME]))
        .and_then(|()| systemctl(&["restart", UNIT_NAME]))
        .and_then(|()| if linger { enable_linger() } else { Ok(()) })
}
/// Start the registered user service.
pub fn start() -> io::Result<()> {
    systemctl(&["start", UNIT_NAME])
}
/// Stop the registered user service.
pub fn stop() -> io::Result<()> {
    systemctl(&["stop", UNIT_NAME])
}
/// Restart the registered user service.
pub fn restart() -> io::Result<()> {
    systemctl(&["restart", UNIT_NAME])
}
/// Show the registered user service's status.
pub fn status() -> io::Result<()> {
    systemctl(&["status", "--no-pager", UNIT_NAME])
}
/// Stop, disable, and remove only an ACORN-managed user service.
pub fn uninstall() -> io::Result<()> {
    unit_path().and_then(|path| {
        managed_unit(&path).and_then(|present| {
            match present {
                | false => Ok(()),
                | true => systemctl(&["disable", "--now", UNIT_NAME]).and_then(|()| fs::remove_file(path)),
            }
            .and_then(|()| systemctl(&["daemon-reload"]))
        })
    })
}
