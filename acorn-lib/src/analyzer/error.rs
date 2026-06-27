//! Error module for handling check errors
//!
use super::readability::ReadabilityType;
use crate::analyzer::vale::ValeOutputItem;
use crate::prelude::HashMap;
use crate::util::ToMarkdown;
use bon::Builder;
use core::fmt;
use derive_more::Display;
use serde_json::Value;
use validator::{ValidationError, ValidationErrorsKind};

/// Trait for types that can provide a locator string indicating where an issue is located
pub trait Location {
    /// Return a string indicating where the issue is located, for use in check titles, summaries, etc.
    fn locator(&self) -> String;
    /// Return source text
    fn source_text(&self) -> String;
}
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
    /// Invalid latitude
    Latitude,
    /// Invalid longitude
    Longitude,
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
    /// Invalid polygon
    Polygon,
    /// Invalid RAiD identifier
    Raid,
    /// Value outside allowed range
    Range,
    /// Invalid ROR identifier
    Ror,
    /// Research activity prose section (see [`schema::research_activity::Sections`])
    Section,
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
/// Flattened schema validation issue.
#[derive(Builder, Clone, Debug)]
#[builder(start_fn = init, on(String, into))]
pub struct ValidatorIssue {
    pub(crate) code: ErrorCode,
    pub(crate) path: Option<String>,
    pub(crate) message: String,
    pub(crate) params: HashMap<String, Value>,
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
            | "latitude" => Self::Latitude,
            | "longitude" => Self::Longitude,
            | "length" => Self::Length,
            | "orcid" => Self::Orcid,
            | "patent" => Self::Patent,
            | "phone" => Self::Phone,
            | "polygon" => Self::Polygon,
            | "raid" | "raìd" => Self::Raid,
            | "range" => Self::Range,
            | "ror" => Self::Ror,
            | "section" | "sections.approach" | "sections.areas" | "sections.capabilities" | "sections.impact" => Self::Section,
            | "url" | "urls" => Self::Url,
            | "year" => Self::Year,
            | _ => Self::Other,
        }
    }
}
impl ErrorKind {
    /// Process errors to a list of `(code, path, message, params)` tuples.
    pub fn process(self, prefix: &str) -> Vec<ValidatorIssue> {
        match self {
            | ErrorKind::Validator(kind) => process(prefix, &kind),
            | _ => vec![],
        }
    }
}
impl Location for ValidatorIssue {
    /// Return title-friendly locator — "where to find the issue"
    fn locator(&self) -> String {
        let value = match self.path.as_deref() {
            | Some(value) => match self.params.get("index") {
                | Some(Value::Number(index)) => format!("{}[{}]", value, index),
                | None | Some(_) => value.to_string(),
            },
            | None => self.code.to_string().to_uppercase(),
        };
        value.replace("r#", "")
    }
    /// Return issue source text used by schema highlighting
    fn source_text(&self) -> String {
        match self.params.get("value") {
            | Some(Value::Array(items)) => items.iter().map(|item| item.to_string()).collect::<Vec<String>>().to_markdown(),
            | Some(value) => value.to_string(),
            | None => " ".to_string(),
        }
    }
}
/// Process validator errors into a more manageable form
pub fn process(path: &str, kind: &ValidationErrorsKind) -> Vec<ValidatorIssue> {
    let is_wrapper_field = |value: &str| matches!(value, "license" | "month" | "year" | "postal_code");
    match kind {
        | ValidationErrorsKind::Field(errors) => {
            let result = errors
                .iter()
                .map(|error| {
                    let ValidationError { code, message, params } = error.clone();
                    let path = if path.is_empty() { None } else { Some(path.to_string()) };
                    let message = message.map(|m| m.to_string()).unwrap_or_else(|| code.to_string());
                    let params = params
                        .into_iter()
                        .map(|(key, value)| (key.to_string(), value))
                        .collect::<HashMap<String, Value>>();
                    ValidatorIssue::init()
                        .code(ErrorCode::from(code.as_ref()))
                        .maybe_path(path)
                        .message(message)
                        .params(params)
                        .build()
                })
                .collect::<Vec<ValidatorIssue>>();
            result
        }
        | ValidationErrorsKind::Struct(errors) => {
            let result: Vec<ValidatorIssue> = errors
                .clone()
                .into_errors()
                .into_iter()
                .flat_map(|(field, kind)| {
                    let is_wrapper_recursion = matches!(kind, ValidationErrorsKind::Field(_)) && path == field && is_wrapper_field(field.as_ref());
                    let next_path = match (path.is_empty(), field.is_empty(), is_wrapper_recursion) {
                        | (_, true, _) => path.to_string(),
                        | (true, false, _) => field.to_string(),
                        | (false, false, true) => path.to_string(),
                        | (false, false, false) => format!("{path}.{field}"),
                    };
                    process(&next_path, &kind)
                })
                .collect();
            result
        }
        | ValidationErrorsKind::List(errors) => {
            let result: Vec<ValidatorIssue> = errors
                .iter()
                .flat_map(|(index, error)| {
                    let prefix = if path.is_empty() { "".to_string() } else { path.to_string() };
                    let kind = ValidationErrorsKind::Struct(error.clone());
                    process(&format!("{prefix}[{index}]"), &kind)
                })
                .collect();
            result
        }
    }
}
