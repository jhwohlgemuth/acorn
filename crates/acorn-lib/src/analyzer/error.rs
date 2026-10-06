//! Error module for handling check errors
use super::readability::score::ReadabilityScore;
use crate::analyzer::model::ValeOutputItem;
#[cfg(feature = "analysis")]
use crate::io::document::{DocumentPath, DocumentQuery};
use acorn_core::prelude::{format, HashMap, String, ToString, Vec};
use acorn_core::util::MarkdownSupport;
use acorn_schema::validation::{join_path, ValidationReport};
use bon::Builder;
use core::fmt;
use derive_more::Display;
use serde_json::Value;

/// Trait for findings that can identify their target within a source document.
pub trait DocumentTarget {
    /// Return a string indicating where the issue is located, for use in check titles, summaries, etc.
    fn locator(&self) -> String;
    /// Return source text
    fn source_text(&self) -> String;
    /// Return format-neutral criteria for resolving the finding in its original document (host).
    #[cfg(feature = "analysis")]
    fn query(&self) -> DocumentQuery;
}
/// ACORN validation error codes used by analyzer diagnostics.
#[derive(Clone, Debug, Display)]
#[display(rename_all = "lowercase")]
pub enum ErrorCode {
    /// Invalid ARK identifier
    Ark,
    /// Invalid arXiv identifier
    Arxiv,
    /// Invalid camelCase string
    CamelCase,
    /// Invalid commit hash
    Commit,
    /// Invalid ISO 3166-1 alpha-2 country code
    Country,
    /// Invalid ISO 8601 date
    Date,
    /// Invalid DOI
    Doi,
    /// Invalid content digest
    Digest,
    /// Invalid email address
    Email,
    /// Invalid Unix epoch timestamp
    Epoch,
    /// Unsupported image file extension
    Image,
    /// Invalid Handle identifier
    Handle,
    /// Invalid IPv4 address
    Ipv4,
    /// Invalid IPv6 address
    Ipv6,
    /// Invalid ISBN
    Isbn,
    /// Invalid ISNI
    Isni,
    /// Invalid kebab-case string
    KebabCase,
    /// Invalid latitude
    Latitude,
    /// Invalid longitude
    Longitude,
    /// Value exceeds allowed length
    Length,
    /// Text contains no non-whitespace characters
    Nonempty,
    /// Value satisfies a forbidden validation rule
    Not,
    /// Value satisfies none of the alternative validation rules
    Or,
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
    /// Invalid Software Heritage identifier
    Swhid,
    /// Research activity prose section (see [`schema::research_activity::Sections`])
    Section,
    /// Invalid snake_case string
    SnakeCase,
    /// Invalid state or province abbreviation
    State,
    /// Invalid RFC 3339 timestamp
    Timestamp,
    /// Invalid URL
    Url,
    /// Invalid semantic version
    Version,
    /// Invalid ISO 8601 year
    Year,
}
/// Error kind
#[derive(Clone, Debug)]
pub enum ErrorKind {
    /// Readability issue where calculated index exceeds threshold of associated metric
    Readability(ReadabilityScore),
    /// Prose issue found by Vale
    Vale(Vec<ValeOutputItem>),
    /// Schema validation issues found by ACORN validation.
    Validator(ValidationReport),
}
/// Flattened schema validation issue.
#[derive(Builder, Clone, Debug)]
#[builder(start_fn = init, on(String, into))]
#[cfg_attr(not(feature = "analysis"), allow(dead_code))]
pub struct ValidatorIssue {
    pub(crate) code: ErrorCode,
    pub(crate) path: Option<String>,
    pub(crate) message: String,
    pub(crate) params: HashMap<String, Value>,
}
/// Ordered analyzer representation of a portable validation report.
#[derive(Clone, Debug, Default)]
pub struct ValidatorIssues(Vec<ValidatorIssue>);
impl From<&str> for ErrorCode {
    fn from(code: &str) -> Self {
        match code.to_lowercase().as_str() {
            | "ark" => Self::Ark,
            | "arxiv" => Self::Arxiv,
            | "camelcase" => Self::CamelCase,
            | "commit" => Self::Commit,
            | "country" => Self::Country,
            | "iso 8601 date" | "date" => Self::Date,
            | "doi" => Self::Doi,
            | "digest" => Self::Digest,
            | "email" => Self::Email,
            | "epoch" | "unix epoch" => Self::Epoch,
            | "image" => Self::Image,
            | "handle" => Self::Handle,
            | "ip4" | "ipv4" => Self::Ipv4,
            | "ip6" | "ipv6" => Self::Ipv6,
            | "isbn" => Self::Isbn,
            | "isni" => Self::Isni,
            | "kebabcase" => Self::KebabCase,
            | "latitude" => Self::Latitude,
            | "longitude" => Self::Longitude,
            | "length" => Self::Length,
            | "nonempty" => Self::Nonempty,
            | "not" => Self::Not,
            | "or" => Self::Or,
            | "orcid" => Self::Orcid,
            | "patent" => Self::Patent,
            | "phone" => Self::Phone,
            | "polygon" => Self::Polygon,
            | "raid" | "raìd" => Self::Raid,
            | "range" => Self::Range,
            | "ror" => Self::Ror,
            | "swhid" => Self::Swhid,
            | "section" | "sections.approach" | "sections.areas" | "sections.capabilities" | "sections.impact" => Self::Section,
            | "snake_case" => Self::SnakeCase,
            | "state" => Self::State,
            | "timestamp" => Self::Timestamp,
            | "url" | "urls" => Self::Url,
            | "version" => Self::Version,
            | "year" => Self::Year,
            | _ => Self::Other,
        }
    }
}
impl ErrorKind {
    /// Convert validation errors into ordered analyzer issues with `prefix` applied.
    pub fn process(self, prefix: &str) -> ValidatorIssues {
        match self {
            | ErrorKind::Validator(report) => ValidatorIssues::from(report).with_prefix(prefix),
            | _ => ValidatorIssues::default(),
        }
    }
}
impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#?}", self)
    }
}
impl DocumentTarget for ValidatorIssue {
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
    #[cfg(feature = "analysis")]
    fn query(&self) -> DocumentQuery {
        let locator = self.locator();
        let source_path = self.params.get("source_path").and_then(Value::as_str).unwrap_or(&locator);
        let query = DocumentQuery::new().with_path(DocumentPath::parse(source_path));
        let value = match self.params.get("value") {
            | Some(Value::Array(values)) => self
                .params
                .get("index")
                .and_then(Value::as_u64)
                .and_then(|index| values.get(index as usize)),
            | value => value,
        };

        match value {
            | Some(Value::String(value)) => query.with_value(value),
            | Some(value) if !value.is_array() && !value.is_object() => query.with_value(value.to_string()),
            | None | Some(_) => query,
        }
    }
}
impl From<ValidationReport> for ValidatorIssues {
    fn from(report: ValidationReport) -> Self {
        Self(
            report
                .into_issues()
                .into_iter()
                .map(|issue| {
                    let path = (!issue.path.is_empty()).then_some(issue.path);
                    let code = ErrorCode::from(issue.error.code.as_str());
                    let message = issue.error.message.unwrap_or(issue.error.code);
                    let params = issue.error.params.into_iter().collect::<HashMap<String, Value>>();
                    ValidatorIssue::init().code(code).maybe_path(path).message(message).params(params).build()
                })
                .collect(),
        )
    }
}
impl IntoIterator for ValidatorIssues {
    type Item = ValidatorIssue;
    type IntoIter = alloc::vec::IntoIter<ValidatorIssue>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
impl ValidatorIssues {
    pub(crate) fn with_prefix(mut self, prefix: &str) -> Self {
        self.0.iter_mut().for_each(|issue| {
            let path = join_path(prefix, issue.path.as_deref().unwrap_or_default());
            issue.path = (!path.is_empty()).then_some(path);
        });
        self
    }
    /// Return the number of analyzer issues.
    pub fn len(&self) -> usize {
        self.0.len()
    }
    /// Return whether there are no analyzer issues.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    /// Iterate over analyzer issues in report order.
    pub fn iter(&self) -> core::slice::Iter<'_, ValidatorIssue> {
        self.0.iter()
    }
}
