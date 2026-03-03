//! Error module for handling check errors
//!
use super::ReadabilityType;
use crate::analyzer::vale::ValeOutputItem;
use core::fmt;
use derive_more::Display;
use serde_json::Value;

use validator::{ValidationError, ValidationErrorsKind};
// use color_eyre::owo_colors::OwoColorize;

/// Augmented list of validator crate error codes, with custom codes for ACORN-specific validation errors
#[derive(Clone, Debug, Display)]
#[display(rename_all = "lowercase")]
pub enum ErrorCode {
    /// Invalid ARK identifier
    Ark,
    /// Invalid ISO 8601 date
    Date,
    /// Invalid DOI
    Doi,
    /// Invalid email address
    Email,
    /// Invalid Unix epoch timestamp
    Epoch,
    /// Unsupported image file extension
    Image,
    /// Invalid IPv6 address
    Ip6,
    /// Invalid ISBN
    Isbn,
    /// Invalid kebab-case string
    KebabCase,
    /// Value exceeds allowed length
    Length,
    /// Invalid ORCiD
    Orcid,
    /// Other
    Other,
    /// Invalid patent identifier
    Patent,
    /// Invalid phone number
    Phone,
    /// Invalid RAiD identifier
    Raid,
    /// Value outside allowed range
    Range,
    /// Invalid ROR identifier
    Ror,
    /// Invalid URL
    Url,
    /// Invalid ISO 8601 year
    Year,
}
/// Error kind
#[derive(Clone, Debug)]
pub enum ErrorKind {
    /// Readability issue where calculated index exceeds threshold of associated metric
    Readability((f64, ReadabilityType)),
    /// Prose issue found by Vale
    Vale(Vec<ValeOutputItem>),
    /// Schema validation issue found by [validator crate]
    ///
    /// [validator crate]: https://crates.io/crates/validator
    Validator(ValidationErrorsKind),
}
impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#?}", self)
    }
}
impl From<&str> for ErrorCode {
    fn from(code: &str) -> Self {
        match code.to_lowercase().as_str() {
            | "ark" => Self::Ark,
            | "iso 8601 date" | "date" => Self::Date,
            | "doi" => Self::Doi,
            | "email" => Self::Email,
            | "epoch" | "unix epoch" => Self::Epoch,
            | "image" => Self::Image,
            | "ip6" => Self::Ip6,
            | "isbn" => Self::Isbn,
            | "kebabcase" => Self::KebabCase,
            | "length" => Self::Length,
            | "orcid" => Self::Orcid,
            | "patent" => Self::Patent,
            | "phone" => Self::Phone,
            | "raid" | "raìd" => Self::Raid,
            | "range" => Self::Range,
            | "ror" => Self::Ror,
            | "url" | "urls" => Self::Url,
            | "year" => Self::Year,
            | _ => Self::Other,
        }
    }
}
impl ErrorKind {
    /// Flatten errors to a list of strings
    pub fn flatten(self) -> Vec<String> {
        match self {
            | ErrorKind::Validator(kind) => process("", kind),
            | _ => vec![],
        }
    }
}
fn process(path: &str, kind: ValidationErrorsKind) -> Vec<String> {
    match kind {
        | ValidationErrorsKind::Field(errors) => {
            let result = errors
                .into_iter()
                .map(|error| {
                    let ValidationError { code, message, params } = error;
                    let prefix = if path.is_empty() { "".to_string() } else { path.to_string() };
                    let message = message.map(|m| m.to_string()).unwrap_or_else(|| code.to_string());
                    let value = match params.get("value").unwrap_or(&Value::Null) {
                        | Value::String(s) => s.clone(),
                        | Value::Array(items) => items.iter().map(|v| v.to_string()).collect::<Vec<String>>().join(", "),
                        | other => other.to_string(),
                    };
                    format!("{code};{prefix};{};{}", message, value)
                })
                .collect::<Vec<String>>();
            result
        }
        | ValidationErrorsKind::Struct(errors) => {
            let result: Vec<String> = errors
                .into_errors()
                .into_iter()
                .flat_map(|(field, kind)| {
                    let prefix = if path.is_empty() { "".to_string() } else { format!("{path}.") };
                    process(&format!("{prefix}{field}"), kind)
                })
                .collect();
            result
        }
        | ValidationErrorsKind::List(errors) => {
            let result: Vec<String> = errors
                .into_iter()
                .flat_map(|(index, error)| {
                    let prefix = if path.is_empty() { "".to_string() } else { path.to_string() };
                    let kind = ValidationErrorsKind::Struct(error);
                    process(&format!("{prefix}[{index}]"), kind)
                })
                .collect();
            result
        }
    }
}
