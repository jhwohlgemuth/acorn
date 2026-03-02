//! Error module for handling check errors
//!
use super::ReadabilityType;
use crate::analyzer::vale::ValeOutputItem;
use core::fmt;
use serde_json::Value;
use validator::{ValidationError, ValidationErrorsKind};
// use color_eyre::owo_colors::OwoColorize;

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
impl ErrorKind {
    /// `Field(Vec<ValidationError>)`
    /// ```ignore
    /// Validator(
    ///     Field(
    ///         [
    ///             ValidationError {
    ///                 code: "length",
    ///                 message: Some(
    ///                     "Subtitle is too long, please reduce the length below 75.",
    ///                 ),
    ///                 params: {
    ///                     "max": Number(75),
    ///                     "value": String("Subtitle for this project that is way too long and should trigger a validation error because it exceeds the maximum length allowed for subtitles in the ACORN schema."),
    ///                 },
    ///             },
    ///         ],
    ///     ),
    /// )
    /// ```
    /// `Struct(Box<ValidationErrors>)`
    /// ```ignore
    /// Validator(
    ///    Struct(
    ///        ValidationErrors(
    ///            {
    ///                "email": Field(
    ///                    [
    ///                        ValidationError {
    ///                            code: "email",
    ///                            message: Some(
    ///                                "Please provide a valid email",
    ///                            ),
    ///                            params: {
    ///                                "value": String("Not an email"),
    ///                            },
    ///                        },
    ///                    ],
    ///                ),
    ///                "telephone": Field(
    ///                    [
    ///                        ValidationError {
    ///                            code: "phone",
    ///                            message: Some(
    ///                                "Please provide a valid phone number",
    ///                            ),
    ///                            params: {
    ///                                "value": String("42"),
    ///                            },
    ///                        },
    ///                    ],
    ///                ),
    ///            },
    ///        ),
    ///    ),
    /// )
    /// ```
    /// `List(BTreeMap<usize, Box<ValidationErrors>>)`
    /// ```ignore
    /// Validator(
    ///     Struct(
    ///         ValidationErrors(
    ///             {
    ///                 "websites": List(
    ///                     {
    ///                         1: ValidationErrors(
    ///                             {
    ///                                 "url": Field(
    ///                                     [
    ///                                         ValidationError {
    ///                                             code: "url",
    ///                                             message: Some(
    ///                                                 "Please provide a valid URL",
    ///                                             ),
    ///                                             params: {
    ///                                                 "value": String("Not a URL"),
    ///                                             },
    ///                                         },
    ///                                     ],
    ///                                 ),
    ///                             },
    ///                         ),
    ///                     },
    ///                 ),
    ///             },
    ///         ),
    ///     ),
    /// )
    /// ```
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
                        | Value::String(s) => s.clone().replace("\"", "").replace("\\", ""),
                        | Value::Array(items) => items
                            .iter()
                            .map(|v| v.to_string())
                            .collect::<Vec<String>>()
                            .join(", ")
                            .replace("\"", "")
                            .replace("\\", ""),
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
                    // format!("{prefix}[{index}] - {:?}", process(&format!("{prefix}."), kind))
                    process(&format!("{prefix}[{index}]"), kind)
                })
                .collect();
            result
        }
    }
}
