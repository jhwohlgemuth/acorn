//! Caller-owned JSON, JSONC, and YAML configuration loading.
use acorn_core::util::{MimeType, ParseOptionsExt};
use jsonc_parser::{errors::ParseError, parse_to_serde_value, ParseOptions};
use serde::de::DeserializeOwned;
use serde_json::Value;
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
    /// A parsed JSONC value could not be decoded into the caller-owned type.
    #[error("invalid JSONC configuration {path:?}: {source}")]
    JsoncDeserialize {
        /// Configuration path.
        path: PathBuf,
        /// Decoder error.
        #[source]
        source: serde_json::Error,
    },
    /// JSONC configuration did not satisfy the shared strict syntax policy.
    #[error("invalid JSONC syntax {path:?}: {source}")]
    JsoncParse {
        /// Configuration path.
        path: PathBuf,
        /// Parser error.
        #[source]
        source: ParseError,
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
/// Read JSON, JSONC, YAML, or YML configuration based on the filename extension.
pub fn read<T>(path: impl AsRef<Path>) -> Result<T, ConfigError>
where
    T: DeserializeOwned,
{
    let path = path.as_ref();
    match MimeType::from(path.to_string_lossy().as_ref()) {
        | MimeType::Json => read_json(path),
        | MimeType::Jsonc => read_jsonc(path),
        | MimeType::Yaml => read_yaml(path),
        | _ => Err(ConfigError::UnsupportedFormat(path.to_path_buf())),
    }
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
/// Read strict JSON-with-comments configuration into a caller-owned type.
pub fn read_jsonc<T>(path: impl AsRef<Path>) -> Result<T, ConfigError>
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
            parse_to_serde_value::<Value>(&content, &ParseOptions::strict_jsonc()).map_err(|source| ConfigError::JsoncParse {
                path: path.to_path_buf(),
                source,
            })
        })
        .and_then(|value| {
            serde_json::from_value(value).map_err(|source| ConfigError::JsoncDeserialize {
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
/// Resolve an explicit configuration path or the first regular default candidate.
pub fn resolve_config_path(explicit: Option<&Path>, directory: &Path, candidates: &[&str]) -> Option<PathBuf> {
    explicit.map(Path::to_path_buf).or_else(|| {
        candidates
            .iter()
            .map(|candidate| directory.join(candidate))
            .find(|candidate| candidate.is_file())
    })
}
