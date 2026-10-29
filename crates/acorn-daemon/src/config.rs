//! ACRE-backed daemon configuration loading and host-path validation.
use acorn::io::{
    config::{ApplicationConfiguration, DaemonConfiguration, DaemonJobConfiguration},
    InputOutput,
};
use acorn_core::validation::ValidationError;
use acorn_host::fs::{FsError, SafePath};
use acorn_schema::validation::{Validate, ValidationReport};
use acorn_service::{JobId, JobSchedule, WorkflowAction};
use alloc::collections::BTreeSet;
use core::net::SocketAddr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Failure while loading or validating daemon configuration.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// A job input exists outside its canonical project root.
    #[error("daemon job '{id}' input escapes project root: {path:?}")]
    InputEscape {
        /// Job being validated.
        id: JobId,
        /// Escaping input path.
        path: PathBuf,
    },
    /// A job input could not be canonicalized.
    #[error("daemon job '{id}' input is unavailable at {path:?}: {source}")]
    InputUnavailable {
        /// Job being validated.
        id: JobId,
        /// Unavailable input path.
        path: PathBuf,
        /// Filesystem failure.
        #[source]
        source: io::Error,
    },
    /// A job root is not an existing absolute directory.
    #[error("daemon job '{id}' root must be an existing absolute directory: {path:?}")]
    InvalidRoot {
        /// Job being validated.
        id: JobId,
        /// Rejected root path.
        path: PathBuf,
    },
    /// The ACRE document has no daemon section.
    #[error("ACRE configuration does not define a daemon section")]
    MissingDaemon,
    /// A job input failed shared portable-path validation.
    #[error("daemon job '{id}' input is invalid at {path:?}: {source}")]
    NormalizeInput {
        /// Job being validated.
        id: JobId,
        /// Rejected input path.
        path: PathBuf,
        /// Shared safe-path failure.
        #[source]
        source: Box<FsError>,
    },
    /// The ACRE document could not be read or decoded.
    #[error("failed to read ACRE configuration: {message}")]
    Source {
        /// Configuration diagnostic.
        message: String,
    },
    /// One or more portable configuration invariants failed.
    #[error(transparent)]
    Validation(#[from] ValidationReport),
}
/// Validated protocol listener addresses retained for server composition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BindSettings {
    grpc: Option<SocketAddr>,
    rpc: Option<SocketAddr>,
}
/// Validated daemon configuration ready for runtime composition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DaemonConfig {
    bind: BindSettings,
    jobs: Jobs,
    tls: Option<TlsSettings>,
}
/// Validated configuration for one fixed ACORN job.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JobConfig {
    id: JobId,
    input: PathBuf,
    kind: WorkflowAction,
    root: PathBuf,
    schedule: JobSchedule,
}
/// Validated daemon jobs retained in deterministic source order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Jobs(Vec<JobConfig>);
/// Paired certificate and private-key paths for explicit TLS listeners.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TlsSettings {
    cert: PathBuf,
    key: PathBuf,
}
impl BindSettings {
    /// Return the optional typed gRPC bind address.
    #[must_use]
    pub const fn grpc(&self) -> Option<SocketAddr> {
        self.grpc
    }
    /// Return the optional JSON-RPC bind address.
    #[must_use]
    pub const fn rpc(&self) -> Option<SocketAddr> {
        self.rpc
    }
}
impl Validate for BindSettings {
    fn validate(&self) -> Result<(), ValidationReport> {
        match self.grpc.is_some() || self.rpc.is_some() {
            | true => Ok(()),
            | false => Err(ValidationReport::from_error(
                "bind",
                ValidationError::new("listener").with_message("Enable at least one daemon server listener"),
            )),
        }
    }
}
impl DaemonConfig {
    /// Load and fully validate the daemon section of one ACRE configuration file.
    ///
    /// # Errors
    ///
    /// Returns a [`ConfigError`] when the file cannot be decoded or any daemon invariant fails.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        ApplicationConfiguration::read(path.as_ref().to_path_buf())
            .map_err(|source| ConfigError::Source { message: source.to_string() })
            .and_then(|configuration| configuration.daemon.ok_or(ConfigError::MissingDaemon))
            .and_then(Self::try_from)
    }
    /// Borrow validated protocol listener addresses.
    #[must_use]
    pub const fn bind(&self) -> &BindSettings {
        &self.bind
    }
    /// Borrow configured jobs in deterministic source order.
    #[must_use]
    pub fn jobs(&self) -> &[JobConfig] {
        &self.jobs.0
    }
    /// Borrow paired TLS paths when configured.
    #[must_use]
    pub const fn tls(&self) -> Option<&TlsSettings> {
        self.tls.as_ref()
    }
}
impl TryFrom<DaemonConfiguration> for DaemonConfig {
    type Error = ConfigError;

    fn try_from(value: DaemonConfiguration) -> Result<Self, Self::Error> {
        Jobs::try_from(value.jobs).and_then(|jobs| {
            let bind = BindSettings {
                grpc: value.bind.grpc,
                rpc: value.bind.rpc,
            };
            bind.validate().map_err(ConfigError::from).map(|()| Self {
                bind,
                jobs,
                tls: value.tls.map(|tls| TlsSettings {
                    cert: tls.cert,
                    key: tls.key,
                }),
            })
        })
    }
}
impl JobConfig {
    /// Borrow the stable job identifier.
    #[must_use]
    pub const fn id(&self) -> &JobId {
        &self.id
    }
    /// Borrow the portable project-relative input path.
    #[must_use]
    pub fn input(&self) -> &Path {
        &self.input
    }
    /// Return the registered workflow action.
    #[must_use]
    pub const fn kind(&self) -> WorkflowAction {
        self.kind
    }
    /// Borrow the canonical absolute project root.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }
    /// Return the portable trigger schedule.
    #[must_use]
    pub const fn schedule(&self) -> JobSchedule {
        self.schedule
    }
}
impl TryFrom<DaemonJobConfiguration> for JobConfig {
    type Error = ConfigError;

    fn try_from(value: DaemonJobConfiguration) -> Result<Self, Self::Error> {
        value.validate().map_err(ConfigError::from).and_then(|()| {
            let DaemonJobConfiguration {
                id,
                input,
                kind,
                root,
                triggers,
            } = value;
            let canonical_root = match root.is_absolute() && root.is_dir() {
                | true => fs::canonicalize(&root).map_err(|_| ConfigError::InvalidRoot {
                    id: id.clone(),
                    path: root.clone(),
                }),
                | false => Err(ConfigError::InvalidRoot {
                    id: id.clone(),
                    path: root.clone(),
                }),
            };
            let input = PathBuf::from(<&str>::from(&input));
            let safe_input = SafePath::new(&input)
                .map(SafePath::into_path_buf)
                .map_err(|source| ConfigError::NormalizeInput {
                    id: id.clone(),
                    path: input.clone(),
                    source: Box::new(source),
                });
            canonical_root
                .and_then(|root| safe_input.map(|input| (root, input)))
                .and_then(|(root, input)| {
                    let resolved = root.join(&input);
                    fs::canonicalize(&resolved)
                        .map_err(|source| ConfigError::InputUnavailable {
                            id: id.clone(),
                            path: resolved,
                            source,
                        })
                        .and_then(|canonical| match canonical.starts_with(&root) {
                            | true => Ok((root, input)),
                            | false => Err(ConfigError::InputEscape { id: id.clone(), path: input }),
                        })
                })
                .map(|(root, input)| Self {
                    id,
                    input,
                    kind,
                    root,
                    schedule: triggers,
                })
        })
    }
}
impl TryFrom<Vec<DaemonJobConfiguration>> for Jobs {
    type Error = ConfigError;

    fn try_from(values: Vec<DaemonJobConfiguration>) -> Result<Self, Self::Error> {
        values
            .into_iter()
            .map(JobConfig::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map(Self)
            .and_then(|jobs| jobs.validate().map_err(ConfigError::from).map(|()| jobs))
    }
}
impl Validate for Jobs {
    fn validate(&self) -> Result<(), ValidationReport> {
        let mut identifiers = BTreeSet::new();
        self.0
            .iter()
            .enumerate()
            .fold(ValidationReport::new(), |mut report, (index, job)| {
                if !identifiers.insert(job.id()) {
                    report.add(
                        format!("[{index}].id"),
                        ValidationError::new("duplicate").with_message(format!("Duplicate daemon job identifier '{}'", job.id())),
                    );
                }
                report
            })
            .finish()
    }
}
impl TlsSettings {
    /// Borrow the certificate-chain path.
    #[must_use]
    pub fn cert(&self) -> &Path {
        &self.cert
    }
    /// Borrow the private-key path.
    #[must_use]
    pub fn key(&self) -> &Path {
        &self.key
    }
}

#[cfg(test)]
mod tests;
