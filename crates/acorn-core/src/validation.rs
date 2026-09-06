//! Portable validation failures shared by core values and schema validation.
use crate::prelude::alloc::String;
use alloc::borrow::Cow;
use alloc::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A scalar or custom validation failure before its field path is attached.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ValidationError {
    /// Stable machine-readable error code.
    pub code: String,
    /// Optional human-readable error message.
    pub message: Option<String>,
    /// Structured parameters used by diagnostics and source highlighting.
    pub params: BTreeMap<String, Value>,
}
impl core::error::Error for ValidationError {}
impl core::fmt::Display for ValidationError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(self.message.as_deref().unwrap_or(&self.code))
    }
}
impl ValidationError {
    /// Create an error with a stable code.
    pub fn new(code: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: None,
            params: BTreeMap::new(),
        }
    }
    /// Attach a human-readable message.
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }
    /// Replace the error code.
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = code.into();
        self
    }
    /// Add a serializable diagnostic parameter.
    pub fn add_param<T: Serialize + ?Sized>(&mut self, name: Cow<'static, str>, value: &T) {
        if let Ok(value) = serde_json::to_value(value) {
            self.params.insert(name.into_owned(), value);
        }
    }
}
/// Validate a lowercase SHA-256 digest including its algorithm prefix.
pub fn digest(value: &str) -> Result<(), ValidationError> {
    value
        .strip_prefix("sha256:")
        .filter(|hash| {
            let is_lowercase_hex = hash
                .chars()
                .all(|character| character.is_ascii_hexdigit() && !character.is_ascii_uppercase());
            hash.len() == 64 && is_lowercase_hex
        })
        .map(|_| ())
        .ok_or_else(|| ValidationError::new("digest").with_message("Provide a lowercase SHA-256 digest"))
}
