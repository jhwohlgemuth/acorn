//! Deterministic platform target paths shared by daemon installation phases.
use acorn::util::constants::app::{DAEMON_NAMESPACE, DEFAULT_CONFIG_FILENAMES, ORGANIZATION, QUALIFIER};
use acorn_core::util::constants::app::APPLICATION;
use directories::ProjectDirs;
use std::env;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Failure while resolving platform daemon targets.
#[derive(Debug, Error)]
pub enum TargetPathError {
    /// The running executable could not be resolved.
    #[error("failed to resolve the running executable: {source}")]
    CurrentExecutable {
        /// Environment lookup failure.
        #[source]
        source: io::Error,
    },
    /// The running executable has no parent directory.
    #[error("executable has no parent directory: {path:?}")]
    MissingExecutableParent {
        /// Executable path without a parent.
        path: PathBuf,
    },
    /// The operating system did not provide application directories.
    #[error("platform application directories are unavailable")]
    ProjectDirectoriesUnavailable,
}
/// Stable filesystem targets for the installed daemon and paired CLI.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetPaths {
    binary: PathBuf,
    cache: PathBuf,
    config: PathBuf,
    data: PathBuf,
}
impl TargetPaths {
    /// Resolve targets for the running daemon without creating them
    pub fn resolve() -> Result<Self, TargetPathError> {
        ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
            .ok_or(TargetPathError::ProjectDirectoriesUnavailable)
            .and_then(|directories| {
                env::current_exe()
                    .map_err(|source| TargetPathError::CurrentExecutable { source })
                    .and_then(|executable| Self::from_directories(directories, &executable))
            })
    }
    /// Borrow the co-installed application binary path.
    #[must_use]
    pub fn binary(&self) -> &Path {
        &self.binary
    }
    /// Borrow the daemon cache root.
    #[must_use]
    pub fn cache(&self) -> &Path {
        &self.cache
    }
    /// Borrow the default daemon configuration file path.
    #[must_use]
    pub fn config(&self) -> &Path {
        &self.config
    }
    /// Borrow the daemon data root.
    #[must_use]
    pub fn data(&self) -> &Path {
        &self.data
    }
    pub(crate) fn from_directories(directories: ProjectDirs, executable: &Path) -> Result<Self, TargetPathError> {
        executable
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .map(|parent| Self {
                binary: parent.join(format!("{APPLICATION}{}", env::consts::EXE_SUFFIX)),
                cache: directories.cache_dir().join(DAEMON_NAMESPACE),
                config: directories.config_dir().join(DEFAULT_CONFIG_FILENAMES[1]),
                data: directories.data_local_dir().join(DAEMON_NAMESPACE),
            })
            .ok_or_else(|| TargetPathError::MissingExecutableParent {
                path: executable.to_path_buf(),
            })
    }
}

#[cfg(test)]
mod tests;
