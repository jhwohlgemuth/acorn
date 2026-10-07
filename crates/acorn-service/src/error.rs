//! Stable service error categories independent of transport details.
use acorn_core::prelude::{String, ToString};
use core::fmt;
use serde::{Deserialize, Serialize};

/// Result returned by portable service operations.
pub type ServiceResult<T> = Result<T, ServiceError>;
/// Stable category for a public service failure.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ServiceErrorKind {
    /// Supplied data is malformed or fails a required invariant.
    InvalidInput,
    /// The selected format or readability metric is unsupported.
    Unsupported,
    /// Invocation policy does not permit the requested operation.
    PolicyDenied,
    /// A configured input or output byte bound was exceeded.
    PayloadBounds,
    /// Durable operation state is unavailable in this composition.
    DurableOperationsUnavailable,
    /// An unexpected internal failure occurred.
    Internal,
}
/// Secret-safe, transport-neutral operation error.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ServiceError {
    /// Stable failure category.
    pub kind: ServiceErrorKind,
    /// Optional safe field or document path.
    pub path: Option<String>,
    /// Public diagnostic message.
    pub message: String,
    /// Optional operation name.
    pub operation: Option<String>,
}
impl core::error::Error for ServiceError {}
impl fmt::Display for ServiceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.path.as_deref() {
            | Some(path) => write!(formatter, "{path}: {}", self.message),
            | None => formatter.write_str(&self.message),
        }
    }
}
impl From<serde_json::Error> for ServiceError {
    fn from(error: serde_json::Error) -> Self {
        Self::new(ServiceErrorKind::InvalidInput, error.to_string())
    }
}
impl ServiceError {
    /// Create a public service error.
    pub fn new(kind: ServiceErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            operation: None,
            path: None,
        }
    }
    /// Attach an operation name without changing the public diagnostic.
    pub fn with_operation(self, operation: impl Into<String>) -> Self {
        Self {
            operation: Some(operation.into()),
            ..self
        }
    }
    /// Attach a safe field or document path.
    pub fn with_path(self, path: impl Into<String>) -> Self {
        Self {
            path: Some(path.into()),
            ..self
        }
    }
}
