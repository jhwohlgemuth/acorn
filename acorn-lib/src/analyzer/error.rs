//! Error module for handling check errors
//!
use super::ReadabilityType;
use crate::analyzer::vale::ValeOutputItem;
use validator::ValidationErrorsKind;

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
impl ErrorKind {
    /// Convert error kind to string for display purposes
    pub fn to_string(self) -> String {
        // format!("{:#?}", self.flatten())
        format!("{:#?}", self)
    }
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
        use validator::{ValidationError, ValidationErrorsKind};
        match self {
            | ErrorKind::Validator(kind) => match kind {
                | ValidationErrorsKind::Field(errors) => {
                    let result = errors
                        .into_iter()
                        .map(|error| {
                            let ValidationError { code, message, params } = error;
                            let message = message.map(|m| m.to_string()).unwrap_or_else(|| code.to_string());
                            dbg!(&params);
                            message
                        })
                        .collect::<Vec<String>>();
                    result
                }
                | ValidationErrorsKind::Struct(errors) => {
                    let result = errors
                        .into_errors()
                        .into_iter()
                        .map(|(field, error)| format!("{}: {:?}", field, error))
                        .collect();
                    result
                }
                | ValidationErrorsKind::List(_) => vec![],
            },
            | _ => vec![],
        }
    }
}
