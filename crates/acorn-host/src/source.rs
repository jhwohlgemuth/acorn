//! Local and remote source loading.
use crate::{
    fs::{file_uri_to_path, FsError},
    http::{redact_url, HttpClient, HttpError},
};
use acorn_cmd::CommandSpec;
use acorn_core::{Location, Scheme};
use alloc::string::FromUtf8Error;
use core::time::Duration;
use std::{io, path::PathBuf};
use thiserror::Error;

/// Failure while opening or probing a source.
#[derive(Debug, Error)]
pub enum SourceError {
    /// A filesystem source could not be opened.
    #[error(transparent)]
    File(#[from] FsError),
    /// An HTTP source could not be opened.
    #[error(transparent)]
    Http(#[from] HttpError),
    /// Source bytes are not valid UTF-8.
    #[error("source content is not valid UTF-8")]
    Utf8(#[from] FromUtf8Error),
    /// The location scheme is not supported by this adapter.
    #[error("unsupported source location: {0}")]
    Unsupported(String),
    /// Runtime policy disabled remote source access.
    #[error("remote source access is disabled while offline: {0}")]
    Offline(String),
    /// Git is unavailable on this host.
    #[error("git executable is unavailable")]
    GitUnavailable,
    /// A bounded Git reachability probe failed.
    #[error("Git reachability probe failed for {url}: {kind}")]
    GitProbe {
        /// Redacted remote URL.
        url: String,
        /// Stable failure category.
        kind: &'static str,
    },
}
/// Runtime policy for opening a source.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SourcePolicy {
    offline: bool,
}
impl SourcePolicy {
    /// Create the default online source policy.
    pub fn new() -> Self {
        Self::default()
    }
    /// Enable or disable remote source access.
    pub fn with_offline(mut self, offline: bool) -> Self {
        self.offline = offline;
        self
    }
}
/// Read source bytes from a local path, file URI, or HTTP(S) location.
pub async fn read_bytes(client: &HttpClient, location: &Location, policy: SourcePolicy) -> Result<Vec<u8>, SourceError> {
    let raw: &str = location.into();
    match location.scheme() {
        | Scheme::HTTP | Scheme::HTTPS if policy.offline => Err(SourceError::Offline(redact_url(raw))),
        | Scheme::HTTP | Scheme::HTTPS => client.get_bytes(raw).await.map_err(Into::into),
        | Scheme::File => match file_uri_to_path(raw) {
            | Ok(path) => read_local(path).await,
            | Err(error) => Err(error.into()),
        },
        | Scheme::Unsupported if location.is_path() => read_local(PathBuf::from(raw)).await,
        | Scheme::OCI | Scheme::SSH | Scheme::Unsupported => Err(SourceError::Unsupported(redact_url(raw))),
    }
}
/// Read UTF-8 source text from a local path, file URI, or HTTP(S) location.
pub async fn read_text(client: &HttpClient, location: &Location, policy: SourcePolicy) -> Result<String, SourceError> {
    read_bytes(client, location, policy)
        .await
        .and_then(|bytes| String::from_utf8(bytes).map_err(Into::into))
}
/// Probe a Git remote without allowing an interactive credential prompt.
pub fn probe_git(remote: &str, timeout: Duration) -> Result<(), SourceError> {
    let command = CommandSpec::new("git")
        .args(["ls-remote", "--exit-code", "--", remote])
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GCM_INTERACTIVE", "Never");
    match command.available() {
        | false => Err(SourceError::GitUnavailable),
        | true => command
            .checked_output_with_timeout(timeout)
            .map(|_| ())
            .map_err(|error| SourceError::GitProbe {
                url: redact_url(remote),
                kind: if error.kind() == io::ErrorKind::TimedOut {
                    "probe timed out"
                } else {
                    "remote was not reachable"
                },
            }),
    }
}
async fn read_local(path: PathBuf) -> Result<Vec<u8>, SourceError> {
    tokio::fs::read(&path)
        .await
        .map_err(|source| FsError::Io { path, source })
        .map_err(Into::into)
}
