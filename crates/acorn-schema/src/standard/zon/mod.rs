//! Portable encoder and decoder for ACORN's pinned ZON profile
//!
//! Parser offsets are bounded by validated document limits and guarded parser state;
//! decimal casts are performed only in sign-checked branches.
#![allow(clippy::arithmetic_side_effects, clippy::cast_sign_loss, clippy::indexing_slicing)]
mod decode;
mod encode;
#[cfg(test)]
mod tests;

use self::ZonErrorCode::{E101, E103, E104, E301, E302, E303, E304};
use self::{decode::decode_value_with_limits, encode::encode_value_with_limits};
use crate::validation::rules;
use alloc::string::String;
use bon::Builder;
use core::fmt;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;

const LIMITS: Limits = Limits {
    array_items: MAX_ARRAY_ITEMS,
    depth: MAX_DEPTH,
    document_bytes: MAX_DOCUMENT_BYTES,
    line_bytes: MAX_LINE_BYTES,
    object_keys: MAX_OBJECT_KEYS,
};
const MAX_ARRAY_ITEMS: usize = 1_000_000;
const MAX_DEPTH: usize = 100;
const MAX_DOCUMENT_BYTES: usize = 100 * 1024 * 1024;
const MAX_LINE_BYTES: usize = 1024 * 1024;
const MAX_OBJECT_KEYS: usize = 100_000;
/// Stable error code emitted by the pinned ZON profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ZonErrorCode {
    /// A table contains fewer rows than declared.
    #[error("E001")]
    E001,
    /// A table row has the wrong number of fields.
    #[error("E002")]
    E002,
    /// A table header is malformed.
    #[error("E003")]
    E003,
    /// A table declares an invalid column list.
    #[error("E004")]
    E004,
    /// A string contains an invalid escape or unsupported character.
    #[error("E101")]
    E101,
    /// A quoted string or escape sequence is unterminated.
    #[error("E102")]
    E102,
    /// A ZON value or document is malformed.
    #[error("E103")]
    E103,
    /// An object key is empty.
    #[error("E104")]
    E104,
    /// A document exceeds the byte limit.
    #[error("E301")]
    E301,
    /// A line exceeds the byte limit.
    #[error("E302")]
    E302,
    /// An array exceeds the item limit.
    #[error("E303")]
    E303,
    /// An object exceeds the key limit.
    #[error("E304")]
    E304,
}
#[derive(Clone, Copy)]
struct Limits {
    array_items: usize,
    depth: usize,
    document_bytes: usize,
    line_bytes: usize,
    object_keys: usize,
}
/// Error returned while encoding or decoding ZON data.
#[derive(Builder, Clone, Debug, Eq, PartialEq)]
#[builder(on(String, into))]
pub struct ZonError {
    code: ZonErrorCode,
    column: Option<usize>,
    context: Option<String>,
    line: Option<usize>,
}
impl ZonError {
    fn at(code: ZonErrorCode, line: usize, column: usize, context: &str) -> Self {
        Self::builder()
            .code(code)
            .column(column)
            .context(context.chars().take(160).collect::<String>())
            .line(line)
            .build()
    }
    /// Return the stable upstream error code.
    pub fn code(&self) -> &str {
        self.code.as_str()
    }
    /// Return the one-based source column when available.
    pub fn column(&self) -> Option<usize> {
        self.column
    }
    /// Return a bounded excerpt of the failing source line when available.
    pub fn context(&self) -> Option<&str> {
        self.context.as_deref()
    }
    /// Return the one-based source line when available.
    pub fn line(&self) -> Option<usize> {
        self.line
    }
    /// Return the human-readable error message.
    pub const fn message(&self) -> &'static str {
        self.code.message()
    }
}
impl core::error::Error for ZonError {}
impl fmt::Display for ZonError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.line, self.column) {
            | (Some(line), Some(column)) => write!(formatter, "{} at {line}:{column}: {}", self.code, self.message()),
            | _ => write!(formatter, "{}: {}", self.code, self.message()),
        }
    }
}
impl From<ZonErrorCode> for ZonError {
    fn from(code: ZonErrorCode) -> Self {
        Self::builder().code(code).build()
    }
}
impl ZonErrorCode {
    const fn as_str(&self) -> &'static str {
        match self {
            | Self::E001 => "E001",
            | Self::E002 => "E002",
            | Self::E003 => "E003",
            | Self::E004 => "E004",
            | Self::E101 => "E101",
            | Self::E102 => "E102",
            | Self::E103 => "E103",
            | Self::E104 => "E104",
            | Self::E301 => "E301",
            | Self::E302 => "E302",
            | Self::E303 => "E303",
            | Self::E304 => "E304",
        }
    }
    /// Return the human-readable description for this error code.
    pub const fn message(&self) -> &'static str {
        match self {
            | Self::E001 => "A table contains fewer rows than declared.",
            | Self::E002 => "A table row has the wrong number of fields.",
            | Self::E003 => "A table header is malformed.",
            | Self::E004 => "A table declares an invalid column list.",
            | Self::E101 => "A string contains an invalid escape or unsupported character.",
            | Self::E102 => "A quoted string or escape sequence is unterminated.",
            | Self::E103 => "A ZON value or document is malformed.",
            | Self::E104 => "An object key is empty.",
            | Self::E301 => "A document exceeds the byte limit.",
            | Self::E302 => "A line exceeds the byte limit.",
            | Self::E303 => "An array exceeds the item limit.",
            | Self::E304 => "An object exceeds the key limit.",
        }
    }
}
/// Decode a ZON document into a typed value.
pub fn decode<T>(source: &str) -> Result<T, ZonError>
where
    T: DeserializeOwned,
{
    decode_value(source).and_then(|value| serde_json::from_value(value).map_err(|_| ZonError::from(E103)))
}
/// Decode a ZON document into its JSON data-model representation.
pub fn decode_value(source: &str) -> Result<Value, ZonError> {
    decode_value_with_limits(source, LIMITS)
}
/// Encode a serializable value using ACORN's canonical ZON profile.
pub fn encode<T>(value: &T) -> Result<String, ZonError>
where
    T: Serialize + ?Sized,
{
    serde_json::to_value(value)
        .map_err(|_| ZonError::from(E103))
        .and_then(|value| encode_value_with_limits(&value, LIMITS))
}
fn validate_encoded(source: &str, limits: Limits) -> Result<(), ZonError> {
    match source.len() > limits.document_bytes {
        | true => Err(ZonError::from(E301)),
        | false if source.starts_with('\u{feff}') => Err(ZonError::at(E103, 1, 1, "<BOM>")),
        | false if source.as_bytes().windows(2).any(|pair| pair[0] == b'\r' && pair[1] != b'\n') || source.ends_with('\r') => {
            Err(ZonError::from(E103))
        }
        | false => source
            .lines()
            .enumerate()
            .try_for_each(|(index, line)| match line.len() > limits.line_bytes {
                | true => Err(ZonError::at(E302, index + 1, 1, line)),
                | false => Ok(()),
            }),
    }
}
fn validate_value(value: &Value, depth: usize, limits: Limits) -> Result<(), ZonError> {
    match depth > limits.depth {
        | true => Err(ZonError::from(E103)),
        | false => match value {
            | Value::Array(values) if values.len() > limits.array_items => Err(ZonError::from(E303)),
            | Value::Array(values) => values.iter().try_for_each(|value| validate_value(value, depth + 1, limits)),
            | Value::Object(fields) if fields.len() > limits.object_keys => Err(ZonError::from(E304)),
            | Value::Object(fields) => fields.iter().try_for_each(|(key, value)| match key.is_empty() {
                | true => Err(ZonError::from(E104)),
                | false => rules::zon(key)
                    .map_err(|_| ZonError::from(E101))
                    .and_then(|()| validate_value(value, depth + 1, limits)),
            }),
            | Value::String(value) => rules::zon(value).map_err(|_| ZonError::from(E101)),
            | _ => Ok(()),
        },
    }
}
