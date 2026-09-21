//! Caller-owned JSON and YAML configuration loading.
use acorn_core::util::MimeType;
use serde::de::DeserializeOwned;
use std::{
    fs::read_to_string,
    path::{Path, PathBuf},
};
use thiserror::Error;

/// Failure while loading structured configuration.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// The path does not identify a supported format.
    #[error("unsupported configuration format: {0:?}")]
    UnsupportedFormat(PathBuf),
    /// Configuration bytes could not be read.
    #[error("failed to read configuration {path:?}: {source}")]
    Read {
        /// Configuration path.
        path: PathBuf,
        /// Underlying filesystem error.
        #[source]
        source: std::io::Error,
    },
    /// JSON configuration could not be decoded.
    #[error("invalid JSON configuration {path:?}: {source}")]
    Json {
        /// Configuration path.
        path: PathBuf,
        /// Decoder error.
        #[source]
        source: serde_json::Error,
    },
    /// YAML configuration could not be decoded.
    #[error("invalid YAML configuration {path:?}: {source}")]
    Yaml {
        /// Configuration path.
        path: PathBuf,
        /// Decoder error.
        #[source]
        source: serde_norway::Error,
    },
}
/// Read JSON configuration into a caller-owned type.
pub fn read_json<T>(path: impl AsRef<Path>) -> Result<T, ConfigError>
where
    T: DeserializeOwned,
{
    let path = path.as_ref();
    read_to_string(path)
        .map_err(|source| ConfigError::Read {
            path: path.to_path_buf(),
            source,
        })
        .and_then(|content| {
            serde_json::from_str(&content).map_err(|source| ConfigError::Json {
                path: path.to_path_buf(),
                source,
            })
        })
}
/// Read YAML configuration into a caller-owned type.
pub fn read_yaml<T>(path: impl AsRef<Path>) -> Result<T, ConfigError>
where
    T: DeserializeOwned,
{
    let path = path.as_ref();
    read_to_string(path)
        .map_err(|source| ConfigError::Read {
            path: path.to_path_buf(),
            source,
        })
        .and_then(|content| {
            serde_norway::from_str(&content).map_err(|source| ConfigError::Yaml {
                path: path.to_path_buf(),
                source,
            })
        })
}
/// Read JSON, YAML, or YML configuration based on the filename extension.
pub fn read<T>(path: impl AsRef<Path>) -> Result<T, ConfigError>
where
    T: DeserializeOwned,
{
    let path = path.as_ref();
    match MimeType::from(path.to_string_lossy().as_ref()) {
        | MimeType::Json => read_json(path),
        | MimeType::Yaml => read_yaml(path),
        | _ => Err(ConfigError::UnsupportedFormat(path.to_path_buf())),
    }
}
