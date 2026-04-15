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
        match self.path.as_deref() {
            | Some(value) => match self.params.get("index") {
                | Some(Value::Number(index)) => format!("{}[{}]", value, index),
                | None | Some(_) => value.to_string(),
            },
            | None => self.code.to_string().to_uppercase(),
        }
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
                    let prefix = if path.is_empty() { "".to_string() } else { format!("{path}.") };
                    process(&format!("{prefix}{field}"), &kind)
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

#[cfg(test)]
mod tests {
    use super::{process, ErrorCode, ErrorKind, Location};
    use crate::analyzer::error::ValidatorIssue;
    use crate::prelude::HashMap;
    use serde_json::Value;
    use validator::{Validate, ValidationErrorsKind};

    #[derive(Validate)]
    struct FieldModel {
        #[validate(length(min = 3, max = 3))]
        name: String,
    }
    #[derive(Validate)]
    struct ChildModel {
        #[validate(email)]
        email: String,
    }
    #[derive(Validate)]
    struct ParentModel {
        #[validate(nested)]
        child: ChildModel,
    }
    #[derive(Validate)]
    struct ParentListModel {
        #[validate(nested)]
        children: Vec<ChildModel>,
    }
    #[derive(Validate)]
    struct ArrayParamModel {
        #[validate(length(min = 3))]
        tags: Vec<String>,
    }
    #[test]
    fn process_returns_field_issue_tuple() {
        let value = FieldModel { name: "x".to_string() };
        let errors = value.validate().expect_err("expected validation failure");
        let issues = ErrorKind::Validator(ValidationErrorsKind::Struct(Box::new(errors))).process("");
        assert_eq!(issues.len(), 1);
        let issue = &issues[0];
        assert!(matches!(issue.code, ErrorCode::Length));
        assert_eq!(issue.path.as_deref(), Some("name"));
        assert_eq!(issue.message, "length");
        assert_eq!(issue.params.get("value").map(|v| v.as_str()), Some(Some("x")));
        assert_eq!(issue.params.get("min").and_then(|v| v.as_i64()), Some(3));
        assert_eq!(issue.params.get("max").and_then(|v| v.as_i64()), Some(3));
    }
    #[test]
    fn process_returns_nested_struct_path() {
        let value = ParentModel {
            child: ChildModel {
                email: "not-an-email".to_string(),
            },
        };
        let errors = value.validate().expect_err("expected validation failure");
        let issues = ErrorKind::Validator(ValidationErrorsKind::Struct(Box::new(errors))).process("");
        assert_eq!(issues.len(), 1);
        let issue = &issues[0];
        assert!(matches!(issue.code, ErrorCode::Email));
        assert_eq!(issue.path.as_deref(), Some("child.email"));
        assert_eq!(issue.params.get("value").map(|v| v.as_str()), Some(Some("not-an-email")));
    }
    #[test]
    fn process_returns_nested_list_index_path() {
        let value = ParentListModel {
            children: vec![
                ChildModel {
                    email: "valid@example.com".to_string(),
                },
                ChildModel {
                    email: "invalid".to_string(),
                },
            ],
        };
        let errors = value.validate().expect_err("expected validation failure");
        let issues = ErrorKind::Validator(ValidationErrorsKind::Struct(Box::new(errors))).process("");
        assert_eq!(issues.len(), 1);
        let issue = &issues[0];
        assert!(matches!(issue.code, ErrorCode::Email));
        assert_eq!(issue.path.as_deref(), Some("children[1].email"));
        assert_eq!(issue.params.get("value").map(|v| v.as_str()), Some(Some("invalid")));
    }
    #[test]
    fn process_returns_none_path_for_root_field_errors() {
        let value = FieldModel { name: "x".to_string() };
        let errors = value.validate().expect_err("expected validation failure");
        let field_errors = errors
            .into_errors()
            .remove("name")
            .and_then(|kind| match kind {
                | ValidationErrorsKind::Field(errors) => Some(errors),
                | _ => None,
            })
            .expect("expected field-level validation errors for name");
        let kind = ValidationErrorsKind::Field(field_errors);
        let issues = process("", &kind);
        assert_eq!(issues.len(), 1);
        let issue = &issues[0];
        assert!(matches!(issue.code, ErrorCode::Length));
        assert_eq!(issue.path.as_deref(), None);
        assert_eq!(issue.message, "length");
        assert_eq!(issue.params.get("value").map(|v| v.as_str()), Some(Some("x")));
        assert_eq!(issue.params.get("min").and_then(|v| v.as_i64()), Some(3));
    }
    #[test]
    fn process_serializes_array_param_values() {
        let value = ArrayParamModel {
            tags: vec!["one".to_string(), "two".to_string()],
        };
        let errors = value.validate().expect_err("expected validation failure");
        let issues = ErrorKind::Validator(ValidationErrorsKind::Struct(Box::new(errors))).process("");
        assert_eq!(issues.len(), 1);
        let issue = &issues[0];
        assert!(matches!(issue.code, ErrorCode::Length));
        assert_eq!(issue.path.as_deref(), Some("tags"));
        assert_eq!(issue.message, "length");
        assert_eq!(
            issue.params.get("value").and_then(|v| v.as_array()).map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_str())
                    .map(|item| item.to_string())
                    .collect::<Vec<String>>()
            }),
            Some(vec!["one".to_string(), "two".to_string()])
        );
        assert_eq!(issue.params.get("min").and_then(|v| v.as_i64()), Some(3));
    }
    #[test]
    fn source_text_formats_array_values_as_markdown_list() {
        let mut params: HashMap<String, Value> = HashMap::new();
        params.insert("value".to_string(), serde_json::json!(["alpha", "beta"]));
        let issue = ValidatorIssue::init()
            .code(ErrorCode::Section)
            .maybe_path(Some("sections.impact".to_string()))
            .message("section")
            .params(params)
            .build();
        let source = issue.source_text();
        assert!(source.contains("- \"alpha\""));
        assert!(source.contains("- \"beta\""));
    }
}
