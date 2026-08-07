//! Error contracts for portable domain code and host integrations.
//! Use [`AcornError`] for small, owned domain failures that must compile without host services, and [`AcornResult`] as its result shorthand.
//! Under `std`, use `ApiResult` at I/O, API, and application boundaries that need a `color_eyre::eyre::Report` with rich diagnostic context.
use crate::prelude::String;
use core::fmt;

/// Result returned by portable ACORN domain operations
pub type AcornResult<T> = core::result::Result<T, AcornError>;
/// Result returned by host operations with rich diagnostic context
#[cfg(feature = "std")]
pub type ApiResult<T> = core::result::Result<T, color_eyre::eyre::Report>;
/// Error returned by portable ACORN domain operations
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AcornError {
    message: String,
}
impl AcornError {
    /// Create an error with a user-facing message
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
    /// Return the error message
    pub fn message(&self) -> &str {
        &self.message
    }
}
impl fmt::Display for AcornError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}
impl core::error::Error for AcornError {}
impl From<&str> for AcornError {
    fn from(message: &str) -> Self {
        Self::new(message)
    }
}
impl From<String> for AcornError {
    fn from(message: String) -> Self {
        Self::new(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn error_preserves_message() {
        let error = AcornError::new("portable failure");
        assert_eq!(error.message(), "portable failure");
        assert_eq!(error.to_string(), "portable failure");
    }
}
