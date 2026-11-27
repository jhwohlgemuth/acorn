//! SSH delivery and host-side control for signed Linux daemon releases.

use super::bundle;
use crate::cli::arguments::DaemonCommand;
use acorn::io::Remote;
use acorn_cmd::{args, cmd};
use acorn_host::terminal::Label;
use color_eyre::{eyre::eyre, Result};
use data_encoding::HEXLOWER;
use fluent_uri::Uri;
use ring::digest::{Context, SHA256};
use std::ffi::OsString;
use std::fs::{self, File};
use std::io::Read;
#[cfg(unix)]
use std::os::unix::fs::DirBuilderExt;
use std::path::{Path, PathBuf};
use tracing::info;

const CONTROL: &str = "$HOME/.local/share/acorn/remote-control/acorn";
const MAX_ARCHIVE_BYTES: u64 = 512 * 1024 * 1024;
const MAX_SIGNATURE_BYTES: u64 = 16 * 1024;
const MAX_STDERR_BYTES: usize = 2048;
pub(super) const MAX_VISIBLE_BYTES: usize = 1024 * 1024;
pub(super) struct Endpoint {
    pub(super) target: String,
    pub(super) port: Option<u16>,
    pub(super) program: PathBuf,
}
pub(super) struct Staging(PathBuf);
impl Endpoint {
    pub(super) fn parse(remote: &Remote) -> Result<Self> {
        Uri::parse(remote.as_ref())
            .map_err(|error| eyre!("Invalid SSH host: {error}"))
            .and_then(|uri| {
                let path = uri.path().as_str();
                (path.is_empty() || path == "/")
                    .then_some(())
                    .ok_or_else(|| eyre!("Daemon SSH host must not include a socket path"))
                    .and_then(|()| uri.authority().ok_or_else(|| eyre!("Daemon SSH host has no authority")))
                    .and_then(|authority| {
                        let host = authority.host();
                        let user = authority.userinfo().map(|value| value.as_str());
                        let target = user.map_or_else(|| host.to_string(), |user| format!("{user}@{host}"));
                        authority
                            .port_to_u16()
                            .map_err(|error| eyre!("Invalid SSH port: {error}"))
                            .map(|port| Self {
                                target,
                                port,
                                program: PathBuf::from("ssh"),
                            })
                    })
            })
    }
    pub(super) fn execute(&self, script: &str, input: Option<&Path>) -> Result<String> {
        self.execute_with_output(script, input, false)
    }
    pub(super) fn execute_visible(&self, script: &str, input: Option<&Path>) -> Result<String> {
        self.execute_with_output(script, input, true)
    }
    fn execute_with_output(&self, script: &str, input: Option<&Path>, visible_stderr: bool) -> Result<String> {
        let port = self.port.map_or_else(|| args![], |port| args!["-p", port.to_string()]);
        let arguments = args![
            "-o",
            "BatchMode=yes",
            ..port,
            "--",
            self.target.as_str(),
            format!("exec /bin/sh -c {}", quote(script))
        ];
        let program = self.program.to_string_lossy();
        let output = match input {
            | Some(path) => cmd!(program.as_ref(), &arguments; stdin_file: path),
            | None => cmd!(program.as_ref(), &arguments),
        };
        output.map_err(|error| eyre!("Cannot start SSH: {error}")).and_then(|output| {
            output
                .status
                .success()
                .then(|| {
                    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
                    if visible_stderr {
                        format!("{stdout}{}", visible_excerpt(&output.stderr))
                    } else {
                        stdout
                    }
                })
                .ok_or_else(|| match stderr_excerpt(&output.stderr) {
                    | Some(diagnostic) => eyre!("Remote command failed ({}): {diagnostic}", output.status),
                    | None => eyre!("Remote command failed ({})", output.status),
                })
        })
    }
}
impl Drop for Staging {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
impl Staging {
    fn create() -> Result<Self> {
        let path = std::env::temp_dir().join(format!("acorn-ssh-{}", nanoid::nanoid!()));
        #[cfg(unix)]
        let created = fs::DirBuilder::new().mode(0o700).create(&path);
        #[cfg(not(unix))]
        let created = fs::create_dir(&path);
        created
            .map_err(|error| eyre!("Cannot create private bundle staging directory: {error}"))
            .map(|()| Self(path))
    }
}
fn control_arguments(command: &DaemonCommand) -> Result<Vec<OsString>> {
    let fields = match command {
        | DaemonCommand::Uninstall => Ok(("uninstall", None, None, None)),
        | DaemonCommand::Start => Ok(("start", None, None, None)),
        | DaemonCommand::Stop => Ok(("stop", None, None, None)),
        | DaemonCommand::Restart => Ok(("restart", None, None, None)),
        | DaemonCommand::ServiceStatus => Ok(("service-status", None, None, None)),
        | DaemonCommand::List { config } => Ok(("list", None, config.as_deref(), None)),
        | DaemonCommand::Status { job_id, config } => Ok(("status", Some(job_id.as_str()), config.as_deref(), None)),
        | DaemonCommand::History { job_id, limit, config } => Ok(("history", Some(job_id.as_str()), config.as_deref(), *limit)),
        | DaemonCommand::Run { job_id, config } => Ok(("run", Some(job_id.as_str()), config.as_deref(), None)),
        | DaemonCommand::Pause { job_id, config } => Ok(("pause", Some(job_id.as_str()), config.as_deref(), None)),
        | DaemonCommand::Resume { job_id, config } => Ok(("resume", Some(job_id.as_str()), config.as_deref(), None)),
        | DaemonCommand::Reload { config } => Ok(("reload", None, config.as_deref(), None)),
        | _ => Err(eyre!("This daemon command cannot run over SSH")),
    };
    fields.map(|(action, job, config, limit)| {
        args![
            action,
            ..job.map_or_else(|| args![], |job| args![job]),
            ..config.map_or_else(|| args![], |path| args!["--config", path.as_os_str()]),
            ..limit.map_or_else(|| args![], |limit| args!["--limit", limit.to_string()]),
        ]
    })
}
fn digest(path: &Path) -> Result<String> {
    File::open(path)
        .map_err(|error| eyre!("Cannot read staged bundle: {error}"))
        .and_then(|mut file| {
            let mut context = Context::new(&SHA256);
            let mut buffer = [0u8; 64 * 1024];
            let mut result = Ok(());
            loop {
                match file.read(&mut buffer) {
                    | Ok(0) => break,
                    | Ok(count) => match buffer.get(..count) {
                        | Some(bytes) => context.update(bytes),
                        | None => {
                            result = Err(eyre!("Invalid staged bundle read length"));
                            break;
                        }
                    },
                    | Err(error) => {
                        result = Err(eyre!("Cannot hash staged bundle: {error}"));
                        break;
                    }
                }
            }
            result.map(|()| HEXLOWER.encode(context.finish().as_ref()))
        })
}
fn download(url: &str, path: &Path, max_bytes: u64) -> Result<()> {
    let size = max_bytes.to_string();
    let arguments = args![
        "--fail",
        "--location",
        "--silent",
        "--show-error",
        "--proto",
        "=https",
        "--max-filesize",
        size,
        "--output",
        path.as_os_str(),
        url,
    ];
    cmd!(status "curl", arguments)
        .map_err(|error| eyre!("Cannot start release download: {error}"))
        .and_then(|status| {
            status.success().then_some(()).ok_or_else(|| {
                eyre!(
                    "Release download failed ({status}); access-controlled registries require --bundle with a locally downloaded archive and .minisig"
                )
            })
        })
}
pub(super) fn quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}
pub(super) fn release_base(override_url: Option<&str>) -> Result<&str> {
    override_url
        .or_else(|| {
            option_env!("CI_COMMIT_TAG")
                .filter(|tag| *tag == format!("v{}", env!("CARGO_PKG_VERSION")))
                .and(option_env!("PKG_REGISTRY_URL"))
        })
        .ok_or_else(|| eyre!("A source-built CLI requires --release-url or --bundle for remote install"))
        .and_then(|url| {
            Uri::parse(url)
                .ok()
                .filter(|uri| {
                    uri.scheme().as_str() == "https"
                        && uri
                            .authority()
                            .is_some_and(|authority| authority.userinfo().is_none() && !authority.host().is_empty())
                        && uri.query().is_none()
                        && uri.fragment().is_none()
                })
                .map(|_| url)
                .ok_or_else(|| eyre!("Release URL must be an absolute HTTPS URL"))
        })
}
/// Execute fixed daemon management commands on the SSH host.
pub(super) fn run(command: &DaemonCommand, remote: &Remote, offline: bool) -> Result<()> {
    Endpoint::parse(remote).and_then(|endpoint| match command {
        | DaemonCommand::Install {
            config,
            bundle,
            release_url,
            linger,
        } => config
            .to_string_lossy()
            .starts_with('/')
            .then_some(())
            .ok_or_else(|| eyre!("Remote daemon config must be an absolute Linux path"))
            .and_then(|()| target(&endpoint))
            .and_then(|target| stage(bundle.as_deref(), release_url.as_deref(), target, offline))
            .and_then(|(_staging, archive, signature)| {
                let name = archive
                    .file_name()
                    .and_then(|value| value.to_str())
                    .ok_or_else(|| eyre!("Staged bundle has no name"));
                name.and_then(|name| transfer(&endpoint, name, &archive, &signature, config, *linger))
            }),
        | _ => control_arguments(command).and_then(|arguments| {
            let quoted = arguments
                .iter()
                .map(|value| quote(&value.to_string_lossy()))
                .collect::<Vec<_>>()
                .join(" ");
            let script = format!("exec \"{CONTROL}\" daemon {quoted}");
            endpoint
                .execute_visible(&script, None)
                .map(|output| info!("{} {output}", Label::output()))
        }),
    })
}
pub(super) fn stage(bundle_path: Option<&Path>, release_url: Option<&str>, target: &str, offline: bool) -> Result<(Staging, PathBuf, PathBuf)> {
    let name = bundle_path
        .and_then(|path| path.file_name().and_then(|value| value.to_str()).map(str::to_owned))
        .unwrap_or_else(|| format!("acornd-v{}-{target}.tar.gz", env!("CARGO_PKG_VERSION")));
    Staging::create().and_then(|staging| {
        let archive = staging.0.join(&name);
        let signature = staging.0.join(format!("{name}.minisig"));
        let acquired = match bundle_path {
            | Some(source) => {
                let source_name = source.file_name().and_then(|value| value.to_str());
                (source_name == Some(name.as_str()) && name.ends_with(&format!("-{target}.tar.gz")))
                    .then_some(())
                    .ok_or_else(|| eyre!("Bundle target or version does not match remote host"))
                    .and_then(|()| {
                        fs::metadata(source)
                            .map_err(|error| eyre!("Cannot inspect bundle: {error}"))
                            .and_then(|metadata| {
                                (metadata.len() <= MAX_ARCHIVE_BYTES)
                                    .then_some(())
                                    .ok_or_else(|| eyre!("Bundle exceeds the size limit"))
                            })
                    })
                    .and_then(|()| {
                        fs::copy(source, &archive)
                            .map(|_| ())
                            .map_err(|error| eyre!("Cannot stage bundle: {error}"))
                    })
                    .and_then(|()| {
                        let source_signature = PathBuf::from(format!("{}.minisig", source.display()));
                        fs::metadata(&source_signature)
                            .map_err(|error| eyre!("Cannot inspect bundle signature: {error}"))
                            .and_then(|metadata| {
                                (metadata.len() <= MAX_SIGNATURE_BYTES)
                                    .then_some(())
                                    .ok_or_else(|| eyre!("Bundle signature exceeds the size limit"))
                            })
                            .and_then(|()| {
                                fs::copy(&source_signature, &signature)
                                    .map(|_| ())
                                    .map_err(|error| eyre!("Cannot stage bundle signature: {error}"))
                            })
                    })
            }
            | None if offline => Err(eyre!("Offline remote install requires --bundle")),
            | None => release_base(release_url).and_then(|base| {
                let url = format!("{}/{target}/v{}/{name}", base.trim_end_matches('/'), env!("CARGO_PKG_VERSION"));
                download(&url, &archive, MAX_ARCHIVE_BYTES).and_then(|()| download(&format!("{url}.minisig"), &signature, MAX_SIGNATURE_BYTES))
            }),
        };
        acquired
            .and_then(|()| bundle::verify_path(&archive))
            .map(|()| (staging, archive, signature))
    })
}
pub(super) fn stderr_excerpt(stderr: &[u8]) -> Option<String> {
    let prefix = stderr.get(..stderr.len().min(MAX_STDERR_BYTES)).unwrap_or_default();
    let decoded = String::from_utf8_lossy(prefix);
    let lower = decoded.to_ascii_lowercase();
    let sensitive = ["authorization", "bearer", "token", "password", "secret", "private key", "piv pin"]
        .iter()
        .any(|marker| lower.contains(marker));
    let excerpt = if sensitive {
        "[redacted]".to_string()
    } else {
        decoded
            .lines()
            .take(3)
            .map(|line| line.chars().filter(|character| !character.is_control()).take(256).collect::<String>())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join(" | ")
    };
    (!excerpt.is_empty()).then_some(excerpt)
}
pub(super) fn target(endpoint: &Endpoint) -> Result<&'static str> {
    endpoint.execute("uname -s; uname -m", None).and_then(|output| {
        let mut lines = output.lines();
        match (lines.next(), lines.next(), lines.next()) {
            | (Some("Linux"), Some("x86_64"), None) => Ok("x86_64-unknown-linux-musl"),
            | (Some("Linux"), Some("aarch64" | "arm64"), None) => Ok("aarch64-unknown-linux-musl"),
            | _ => Err(eyre!("Unsupported remote daemon target: {}", output.trim())),
        }
    })
}
pub(super) fn transfer(endpoint: &Endpoint, name: &str, archive: &Path, signature: &Path, config: &Path, linger: bool) -> Result<()> {
    let transfer = format!("acorn-transfer-{}", nanoid::nanoid!());
    let remote_dir = format!("$HOME/.cache/{transfer}");
    let remote_archive = format!("{remote_dir}/{name}");
    let remote_signature = format!("{remote_archive}.minisig");
    let control_next = format!("$HOME/.local/share/acorn/remote-control/acorn.{transfer}.next");
    let make = format!("umask 077; mkdir -p \"$HOME/.cache\" && mkdir \"{remote_dir}\"");
    endpoint.execute(&make, None).and_then(|_| {
        let result = endpoint
            .execute(&format!("umask 077; cat > \"{remote_archive}\""), Some(archive))
            .and_then(|_| endpoint.execute(&format!("umask 077; cat > \"{remote_signature}\""), Some(signature)))
            .and_then(|_| digest(archive).and_then(|archive_hash| digest(signature).map(|signature_hash| (archive_hash, signature_hash))))
            .and_then(|(archive_hash, signature_hash)| {
                let linger_flag = if linger { " --linger" } else { "" };
                let script = format!(
                    "set -eu; umask 077; a=\"{remote_archive}\"; s=\"{remote_signature}\"; \
                     test \"$(sha256sum \"$a\" | awk '{{print $1}}')\" = '{archive_hash}'; \
                     test \"$(sha256sum \"$s\" | awk '{{print $1}}')\" = '{signature_hash}'; \
                     tar -xzf \"$a\" -C \"{remote_dir}\" acorn; chmod 700 \"{remote_dir}/acorn\"; \
                     \"{remote_dir}/acorn\" daemon verify-bundle \"$a\"; \
                     \"{remote_dir}/acorn\" daemon install --config {} --bundle \"$a\"{linger_flag}; \
                     mkdir -p \"$HOME/.local/share/acorn/remote-control\"; \
                     chmod 700 \"$HOME/.local/share/acorn/remote-control\"; \
                     cp \"{remote_dir}/acorn\" \"{control_next}\"; \
                     chmod 700 \"{control_next}\"; \
                     mv \"{control_next}\" \"{CONTROL}\"",
                    quote(&config.to_string_lossy())
                );
                endpoint.execute_visible(&script, None)
            });
        let cleanup = endpoint.execute(&format!("rm -rf \"{remote_dir}\""), None);
        result.and_then(|output| cleanup.map(|_| info!("{} {output}", Label::output())))
    })
}
pub(super) fn visible_excerpt(stderr: &[u8]) -> String {
    let prefix = stderr.get(..stderr.len().min(MAX_VISIBLE_BYTES)).unwrap_or_default();
    let decoded = String::from_utf8_lossy(prefix);
    let lower = decoded.to_ascii_lowercase();
    let sensitive = ["authorization", "bearer", "token", "password", "secret", "private key", "piv pin"]
        .iter()
        .any(|marker| lower.contains(marker));
    if sensitive {
        "[redacted]".to_string()
    } else {
        decoded.into_owned()
    }
}
