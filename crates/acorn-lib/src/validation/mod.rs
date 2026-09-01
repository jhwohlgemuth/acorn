//! Portable, deterministic validation for ACORN schema values.
//!
//! The [`Validate`] derive collects every independent failure in declaration
//! order and records paths such as `outputs[2].title`. Rules do not normalize
//! input or perform filesystem, clock, or network access, except that `year`
//! rejects future years when the `std` feature supplies a clock.
//!
//! # Combinators
//!
//! | Attribute | Purpose |
//! |---|---|
//! | `schema(function = "path")` | Run a synchronous whole-value validator. |
//! | `async(function = "path")` | Run an asynchronous field or whole-value validator. |
//! | `nested` | Append failures from another [`Validate`] implementation. |
//! | `each(rule)` | Apply one scalar rule to every collection item. |
//! | `required` | Require an `Option<T>` to contain a value. |
//! | `not(rule)` | Reject values accepted by one synchronous scalar rule. |
//! | `or(rule-a, rule-b, ...)` | Accept values matching at least one of two or more synchronous scalar rules. Plain nested `or` rules are flattened; nested overrides are rejected. Usable inside `each`. |
//!
//! # Validators
//!
//! | Attribute | Purpose |
//! |---|---|
//! | `length(min = ..., max = ..., equal = ...)` | Constrain collection or string length. |
//! | `range(min = ..., max = ...)` | Constrain an ordered numeric value. |
//! | `contains(pattern = ...)` | Require a string or collection pattern. |
//! | `date`, `timestamp`, `year` | Validate supported calendar representations. |
//! | `email`, `url`, `nonempty`, `version` | Validate common scalar text. |
//! | `ark`, `arxiv`, `doi`, `handle`, `isbn`, `isni`, `orcid`, `patent`, `raid`, `ror`, `swhid` | Delegate persistent identifiers to canonical schema validators. |
//! | `phone`, `commit`, `latitude`, `longitude`, `state`, `kebabcase`, `camelCase`, `snake_case` | Validate ACORN scalar shapes. |
//! | `custom(function = "path")` | Call a synchronous scalar validator. |
//!
//! `code = "..."` and `message = "..."` override a rule's diagnostic. Schema
//! and async rules also accept `skip_on_field_errors = false`; the default is
//! to skip them after an earlier field failure. When every `or` alternative
//! fails, validation emits one `or` issue with the offending value rather than
//! aggregating the alternatives' errors.
//!
//! # Whole-value schema rules
//!
//! A `schema(...)` or whole-value `async(...)` validator returns
//! [`Result<(), ValidationReport>`](ValidationReport) so it can report several
//! ordered issues, each at its own sub-path. The generated call always forwards
//! a context as a second argument: `#function(self, &__context)`. Supply a typed
//! context with `context = "expr"` (default `&()`), remap every returned issue's
//! code and message with `code = "..."` / `message = "..."`, and conditionally
//! skip the whole check with `if = "bool_expression"`. Build reports
//! immutably with [`ValidationError::into_report`], [`ValidationReport::merge`],
//! and [`report_of`]:
//!
//! ```
//! # mod validation { pub use acorn::validation::*; }
//! # mod prelude { pub use acorn::prelude::*; }
//! # mod schema { pub use acorn::schema::*; }
//! use acorn::validation::{report_of, Validate, ValidationError, ValidationReport};
//! use serde::Serialize;
//!
//! #[derive(Clone, Copy, Serialize)]
//! struct Context;
//!
//! #[derive(Serialize, Validate)]
//! #[validate(
//!     schema(
//!         function = "check_identity",
//!         context = "self.ctx",
//!         if = "self.enabled",
//!         code = "identity",
//!         skip_on_field_errors = false,
//!     )
//! )]
//! struct Member {
//!     enabled: bool,
//!     ctx: Context,
//!     #[validate(length(min = 1))]
//!     id: String,
//!     name: String,
//! }
//!
//! fn validate_id(value: &Member, _ctx: &Context) -> Result<(), ValidationReport> {
//!     (!value.id.is_empty())
//!         .then_some(())
//!         .ok_or_else(|| ValidationError::new("id"))
//!         .map_err(|error| error.into_report("id"))
//! }
//!
//! fn validate_name(value: &Member) -> Result<(), ValidationReport> {
//!     (!value.name.is_empty())
//!         .then_some(())
//!         .ok_or_else(|| ValidationError::new("name"))
//!         .map_err(|error| error.into_report("name"))
//! }
//!
//! fn check_identity(value: &Member, ctx: &Context) -> Result<(), ValidationReport> {
//!     ValidationReport::default()
//!         .merge("", report_of(validate_id(value, ctx)))
//!         .merge("", report_of(validate_name(value)))
//!         .finish()
//! }
//!
//! fn main() {
//!     let active = Member { enabled: true, ctx: Context, id: String::new(), name: "A".into() };
//!     assert!(active.validate().is_err());
//!     let skipped = Member { enabled: false, ctx: Context, id: "id-1".into(), name: "A".into() };
//!     assert!(skipped.validate().is_ok());
//! }
//! ```
//!
//! # Examples
//!
//! Combine scalar, optional, collection, and negated validation:
//!
//! ```
//! # mod validation { pub use acorn::validation::*; }
//! # mod prelude { pub use acorn::prelude::*; }
//! # mod schema { pub use acorn::schema::*; }
//! use acorn::validation::{Validate, ValidationError};
//! use serde::Serialize;
//!
//! fn known_channel(value: &str) -> Result<(), ValidationError> {
//!     matches!(value, "web" | "email")
//!         .then_some(())
//!         .ok_or_else(|| ValidationError::new("channel"))
//! }
//!
//! #[derive(Serialize, Validate)]
//! struct Contact {
//!     #[validate(nonempty, length(max = 80))]
//!     name: String,
//!     #[validate(required)]
//!     address: Option<String>,
//!     #[validate(or(email, url))]
//!     destination: String,
//!     #[validate(each(email))]
//!     emails: Vec<String>,
//!     #[validate(not(url), custom(function = "known_channel"))]
//!     channel: String,
//! }
//!
//! fn main() {
//! let value = Contact {
//!     name: "ACORN".into(),
//!     address: Some("research@example.org".into()),
//!     destination: "https://example.org/contact".into(),
//!     emails: vec!["research@example.org".into()],
//!     channel: "email".into(),
//! };
//! assert!(value.validate().is_ok());
//! }
//! ```
//!
//! Whole-value and nested rules compose without losing paths:
//!
//! ```
//! # mod validation { pub use acorn::validation::*; }
//! # mod prelude { pub use acorn::prelude::*; }
//! # mod schema { pub use acorn::schema::*; }
//! use acorn::validation::{Validate, ValidationError, ValidationReport};
//! use serde::Serialize;
//!
//! #[derive(Serialize, Validate)]
//! struct Child {
//!     #[validate(range(min = 1, max = 10))]
//!     score: u8,
//! }
//!
//! fn has_children(value: &Parent, _context: &()) -> Result<(), ValidationReport> {
//!     (!value.children.is_empty())
//!         .then_some(())
//!         .ok_or_else(|| ValidationError::new("children"))
//!         .map_err(|error| error.into_report(""))
//! }
//!
//! #[derive(Serialize, Validate)]
//! #[validate(schema(function = "has_children"))]
//! struct Parent {
//!     #[validate(nested)]
//!     children: Vec<Child>,
//! }
//!
//! fn main() {
//! assert!(Parent { children: vec![Child { score: 5 }] }.validate().is_ok());
//! }
//! ```
use crate::prelude::alloc::{format, vec, Box, String, ToString, Vec};
use crate::util::Checksum;
pub use acorn_macros::Validate;
use alloc::borrow::Cow;
use alloc::collections::BTreeMap;
use core::fmt;
use core::future;
use core::slice::Iter;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Collection abstraction used by `#[validate(each(...))]`.
pub trait Each {
    /// Item validated at each collection index.
    type Item;
    /// Iterator over collection items.
    type Iter<'a>: Iterator<Item = &'a Self::Item>
    where
        Self: 'a;
    /// Iterate over collection items in input order.
    fn each_iter(&self) -> Self::Iter<'_>;
}
/// Validate a value and return every issue in deterministic declaration order.
pub trait Validate {
    /// Validate this value using synchronous rules only.
    fn validate(&self) -> Result<(), ValidationReport>;
    /// Validate this value with all synchronous and asynchronous rules.
    fn validate_async(&self) -> impl future::Future<Output = Result<(), ValidationReport>> + '_ {
        future::ready(self.validate())
    }
}
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
/// One validation failure with its structured document path.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ValidationIssue {
    /// Path rendered as `field.subfield[2]`.
    pub path: String,
    /// Scalar validation failure.
    pub error: ValidationError,
}
/// Ordered, flat collection of validation issues.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct ValidationReport(Vec<ValidationIssue>);
impl Validate for Checksum {
    fn validate(&self) -> Result<(), ValidationReport> {
        Ok(())
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
    pub fn with_message(mut self, message: Cow<'static, str>) -> Self {
        self.message = Some(message.into_owned());
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
    /// Convert a single scalar failure into a one-issue report at `path`.
    pub fn into_report(self, path: impl Into<String>) -> ValidationReport {
        ValidationReport::from_error(path, self)
    }
}
impl ValidationIssue {
    /// Create an issue at a structured path.
    pub fn new(path: impl Into<String>, error: ValidationError) -> Self {
        Self { path: path.into(), error }
    }
}
impl ValidationReport {
    /// Create an empty report.
    pub const fn new() -> Self {
        Self(Vec::new())
    }
    /// Create a report containing one scalar failure at `path`.
    pub fn from_error(path: impl Into<String>, error: ValidationError) -> Self {
        Self(vec![ValidationIssue::new(path, error)])
    }
    /// Return whether the report contains no issues.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    /// Return the number of issues.
    pub fn len(&self) -> usize {
        self.0.len()
    }
    /// Iterate over issues in validation order.
    pub fn iter(&self) -> Iter<'_, ValidationIssue> {
        self.0.iter()
    }
    /// Append a scalar failure at `path`.
    pub fn add(&mut self, path: impl Into<String>, error: ValidationError) {
        self.0.push(ValidationIssue::new(path, error));
    }
    /// Append another report while prefixing every path.
    pub fn append_prefixed(&mut self, prefix: &str, report: Self) {
        self.0.extend(report.0.into_iter().map(|issue| ValidationIssue {
            path: match (
                matches!(issue.path.as_str(), "license" | "month" | "year" | "postal_code"),
                prefix.rsplit('.').next() == Some(issue.path.as_str()),
            ) {
                | (true, true) => prefix.to_string(),
                | _ => join_path(prefix, &issue.path),
            },
            error: issue.error,
        }));
    }
    /// Consume this report and append `other`, prefixing every one of its paths.
    pub fn merge(mut self, prefix: &str, other: Self) -> Self {
        self.append_prefixed(prefix, other);
        self
    }
    /// Convert the report into its ordered issues.
    pub fn into_issues(self) -> Vec<ValidationIssue> {
        self.0
    }
    /// Convert an accumulator into the public validation result.
    #[doc(hidden)]
    pub fn finish(self) -> Result<(), Self> {
        match self.is_empty() {
            | true => Ok(()),
            | false => Err(self),
        }
    }
}
impl fmt::Display for ValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message.as_deref().unwrap_or(&self.code))
    }
}
impl fmt::Display for ValidationIssue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.path.is_empty() {
            | true => write!(formatter, "{}", self.error),
            | false => write!(formatter, "{}: {}", self.path, self.error),
        }
    }
}
impl fmt::Display for ValidationReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().enumerate().try_for_each(|(index, issue)| match index {
            | 0 => write!(formatter, "{issue}"),
            | _ => write!(formatter, "\n{issue}"),
        })
    }
}
impl core::error::Error for ValidationReport {}
impl<T> Each for Vec<T> {
    type Item = T;
    type Iter<'a>
        = Iter<'a, T>
    where
        T: 'a;
    fn each_iter(&self) -> Self::Iter<'_> {
        self.iter()
    }
}
impl<T, const N: usize> Each for [T; N] {
    type Item = T;
    type Iter<'a>
        = Iter<'a, T>
    where
        T: 'a;
    fn each_iter(&self) -> Self::Iter<'_> {
        self.iter()
    }
}
impl<T> Each for [T] {
    type Item = T;
    type Iter<'a>
        = Iter<'a, T>
    where
        T: 'a;
    fn each_iter(&self) -> Self::Iter<'_> {
        self.iter()
    }
}
impl<C> Each for &C
where
    C: Each + ?Sized,
{
    type Item = C::Item;
    type Iter<'a>
        = C::Iter<'a>
    where
        Self: 'a;
    fn each_iter(&self) -> Self::Iter<'_> {
        (*self).each_iter()
    }
}
impl<K, V> Each for BTreeMap<K, V> {
    type Item = V;
    type Iter<'a>
        = alloc::collections::btree_map::Values<'a, K, V>
    where
        K: 'a,
        V: 'a;
    fn each_iter(&self) -> Self::Iter<'_> {
        self.values()
    }
}
impl<T: Validate> Validate for Vec<T> {
    fn validate(&self) -> Result<(), ValidationReport> {
        validate_sequence(self.iter())
    }
    fn validate_async(&self) -> impl future::Future<Output = Result<(), ValidationReport>> + '_ {
        validate_sequence_async(self.iter())
    }
}
impl<T: Validate, const N: usize> Validate for [T; N] {
    fn validate(&self) -> Result<(), ValidationReport> {
        validate_sequence(self.iter())
    }
    fn validate_async(&self) -> impl future::Future<Output = Result<(), ValidationReport>> + '_ {
        validate_sequence_async(self.iter())
    }
}
impl<T: Validate + ?Sized> Validate for Box<T> {
    fn validate(&self) -> Result<(), ValidationReport> {
        (**self).validate()
    }
    fn validate_async(&self) -> impl future::Future<Output = Result<(), ValidationReport>> + '_ {
        (**self).validate_async()
    }
}
fn validate_sequence<'a, T: Validate + 'a>(items: impl Iterator<Item = &'a T>) -> Result<(), ValidationReport> {
    items
        .enumerate()
        .fold(ValidationReport::new(), |mut report, (index, item)| {
            if let Err(nested) = item.validate() {
                report.append_prefixed(&format!("[{index}]"), nested);
            }
            report
        })
        .finish()
}
async fn validate_sequence_async<'a, T: Validate + 'a>(items: impl Iterator<Item = &'a T>) -> Result<(), ValidationReport> {
    let mut report = ValidationReport::new();
    for (index, item) in items.enumerate() {
        if let Err(nested) = item.validate_async().await {
            report.append_prefixed(&format!("[{index}]"), nested);
        }
    }
    report.finish()
}
/// Append all indexed failures produced by an `each` rule.
#[doc(hidden)]
pub fn apply_each<C, F>(report: &mut ValidationReport, path: &str, collection: &C, mut validate: F)
where
    C: Each + ?Sized,
    C::Item: Serialize,
    F: FnMut(&C::Item) -> Result<(), ValidationError>,
{
    collection.each_iter().enumerate().for_each(|(index, item)| {
        if let Err(mut error) = validate(item) {
            error.add_param("value".into(), item);
            report.add(format!("{path}[{index}]"), error);
        }
    });
}
/// Join a field or index prefix to an existing validation path.
#[doc(hidden)]
pub fn join_path(prefix: &str, path: &str) -> String {
    match (prefix.is_empty(), path.is_empty(), path.starts_with('[')) {
        | (true, _, _) => path.to_string(),
        | (_, true, _) => prefix.to_string(),
        | (_, _, true) => format!("{prefix}{path}"),
        | _ => format!("{prefix}.{path}"),
    }
}
/// Lift a whole-value validation result into an immutable report for chaining.
pub fn report_of(result: Result<(), ValidationReport>) -> ValidationReport {
    result.err().unwrap_or_default()
}
/// Built-in scalar validation rules used by the derive macro.
pub mod rules {
    use super::ValidationError;
    use crate::prelude::alloc::{format, ToString};
    use crate::schema::geonames::{CodeFormat, GeonamesParser};
    use crate::schema::pid::{Patent, PersistentIdentifierConvert};
    use crate::util::constants::{RE_FAKE_PHONE, RE_PHONE, RE_UNIX_EPOCH};
    use crate::util::{Constant, SemanticVersion};
    use convert_case::{Case, Casing};
    use core::borrow::Borrow;
    use core::net::{IpAddr, Ipv4Addr, Ipv6Addr};
    use core::str::FromStr;
    use fluent_uri::UriRef;
    use idna::domain_to_ascii;
    #[cfg(feature = "std")]
    use jiff::tz::TimeZone;
    use jiff::{civil::Date, fmt::temporal::Pieces, Timestamp};

    fn identifier(value: bool, code: &'static str) -> Result<(), ValidationError> {
        value
            .then_some(())
            .ok_or_else(|| ValidationError::new(code).with_message(format!("Provide valid {code} value").into()))
    }
    /// Validate an ARK identifier with ACORN's canonical parser.
    pub fn ark(value: &str) -> Result<(), ValidationError> {
        identifier(value.is_ark(), "ARK")
    }
    /// Validate an arXiv identifier with ACORN's canonical parser.
    pub fn arxiv(value: &str) -> Result<(), ValidationError> {
        identifier(value.is_arxiv(), "arXiv")
    }
    /// Validate a DOI with ACORN's canonical parser.
    pub fn doi(value: &str) -> Result<(), ValidationError> {
        identifier(value.is_doi(), "DOI")
    }
    /// Validate a Handle identifier with ACORN's canonical parser.
    pub fn handle(value: &str) -> Result<(), ValidationError> {
        identifier(value.is_handle(), "Handle")
    }
    /// Validate an ISBN with ACORN's canonical parser.
    pub fn isbn(value: &str) -> Result<(), ValidationError> {
        identifier(value.is_isbn(), "ISBN")
    }
    /// Validate an ISNI with ACORN's canonical parser.
    pub fn isni(value: &str) -> Result<(), ValidationError> {
        identifier(value.is_isni(), "ISNI")
    }
    /// Validate an ORCID with ACORN's canonical parser.
    pub fn orcid(value: &str) -> Result<(), ValidationError> {
        identifier(value.is_orcid(), "ORCiD")
    }
    /// Validate a patent identifier with ACORN's canonical parser.
    pub fn patent(value: &str) -> Result<(), ValidationError> {
        Patent::is_valid(value)
            .then_some(())
            .ok_or_else(|| ValidationError::new("patent").with_message("Provide valid patent identifier".into()))
    }
    /// Validate a RAiD identifier with ACORN's canonical parser.
    pub fn raid(value: &str) -> Result<(), ValidationError> {
        identifier(value.is_raid(), "RAiD")
    }
    /// Validate a ROR identifier with ACORN's canonical parser.
    pub fn ror(value: &str) -> Result<(), ValidationError> {
        identifier(value.is_ror(), "ROR")
    }
    /// Validate an SWHID with ACORN's canonical parser.
    pub fn swhid(value: &str) -> Result<(), ValidationError> {
        identifier(value.is_swhid(), "SWHID")
    }
    /// Validate a commit hash containing 7-64 hexadecimal characters.
    pub fn commit(value: &str) -> Result<(), ValidationError> {
        ((7..=64).contains(&value.len()) && value.chars().all(|character| character.is_ascii_hexdigit()))
            .then_some(())
            .ok_or_else(|| ValidationError::new("commit").with_message("Provide valid commit hash (7-64 hexadecimal characters)".into()))
    }
    /// Validate an ISO 3166-1 alpha-2 country code.
    pub fn country(value: impl ToString) -> Result<(), ValidationError> {
        let value = value.to_string();
        (value.len() == 2 && Constant::country_codes(CodeFormat::Alpha2).contains(&value.to_lowercase()))
            .then_some(())
            .ok_or_else(|| ValidationError::new("country").with_message("Provide valid ISO 3166-1 alpha-2 country code".into()))
    }
    /// Validate a lowercase SHA-256 digest including its algorithm prefix.
    pub fn digest(value: &str) -> Result<(), ValidationError> {
        value
            .strip_prefix("sha256:")
            .filter(|hash| {
                hash.len() == 64
                    && hash
                        .chars()
                        .all(|character| character.is_ascii_hexdigit() && !character.is_ascii_uppercase())
            })
            .map(|_| ())
            .ok_or_else(|| ValidationError::new("digest").with_message("Provide a lowercase SHA-256 digest".into()))
    }
    /// Validate kebab-case text.
    pub fn kebabcase(value: &str) -> Result<(), ValidationError> {
        let valid = value.to_case(Case::Kebab);
        valid
            .eq(value)
            .then_some(())
            .ok_or_else(|| ValidationError::new("kebabcase").with_message(format!("Provide ID in kebab-case format (e.g., {valid})").into()))
    }
    /// Validate camelCase text.
    pub fn camelcase(value: &str) -> Result<(), ValidationError> {
        let valid = value.to_case(Case::Camel);
        valid
            .eq(value)
            .then_some(())
            .ok_or_else(|| ValidationError::new("camelcase").with_message(format!("Provide ID in camelCase format (e.g., {valid})").into()))
    }
    /// Validate a latitude from -90 through 90.
    pub fn latitude(value: impl ToString) -> Result<(), ValidationError> {
        value
            .to_string()
            .parse::<f64>()
            .ok()
            .filter(|value| (-90.0..=90.0).contains(value))
            .map(|_| ())
            .ok_or_else(|| ValidationError::new("latitude").with_message("Provide valid latitude (-90 to 90)".into()))
    }
    /// Validate a longitude from -180 through 180.
    pub fn longitude(value: impl ToString) -> Result<(), ValidationError> {
        value
            .to_string()
            .parse::<f64>()
            .ok()
            .filter(|value| (-180.0..=180.0).contains(value))
            .map(|_| ())
            .ok_or_else(|| ValidationError::new("longitude").with_message("Provide valid longitude (-180 to 180)".into()))
    }
    /// Validate a phone number while rejecting known placeholder values.
    pub fn phone(value: &str) -> Result<(), ValidationError> {
        match (RE_PHONE.is_match(value).unwrap_or(false), RE_FAKE_PHONE.is_match(value).unwrap_or(false)) {
            | (true, false) => Ok(()),
            | (_, true) => Err(ValidationError::new("phone").with_message("Provide real phone number, not a placeholder".into())),
            | _ => Err(ValidationError::new("phone").with_message("Provide valid phone number".into())),
        }
    }
    /// Validate a two-letter US state or Canadian province abbreviation.
    pub fn state(value: impl ToString) -> Result<(), ValidationError> {
        let value = value.to_string();
        (value.len() == 2 && value.chars().all(|character| character.is_ascii_alphabetic()))
            .then_some(())
            .ok_or_else(|| {
                ValidationError::new("state").with_message("Provide valid US state or Canadian province abbreviation (e.g., CA, NY, ON)".into())
            })
    }
    /// Validate snake_case text.
    pub fn snake_case(value: &str) -> Result<(), ValidationError> {
        let valid = value.to_case(Case::Snake);
        valid
            .eq(value)
            .then_some(())
            .ok_or_else(|| ValidationError::new("snake_case").with_message(format!("Provide ID in snake_case format (e.g., {valid})").into()))
    }

    /// Validate an ACORN-supported ISO 8601 calendar date.
    ///
    /// # Examples
    ///
    /// ```
    /// use acorn::validation::rules;
    /// assert!(rules::date("2025-01-01").is_ok());
    /// assert!(rules::date("1949-12-31").is_err());
    /// ```
    pub fn date(value: &str) -> Result<(), ValidationError> {
        let parts = value.as_bytes();
        let year = value.get(0..4).and_then(|part| part.parse::<u16>().ok());
        let month = value.get(5..7).and_then(|part| part.parse::<u8>().ok());
        let day = value.get(8..10).and_then(|part| part.parse::<u8>().ok());
        let max_day = match month {
            | Some(2) => 29,
            | Some(4 | 6 | 9 | 11) => 30,
            | Some(1 | 3 | 5 | 7 | 8 | 10 | 12) => 31,
            | None | Some(_) => 0,
        };
        let valid = parts.len() == 10
            && parts.get(4) == Some(&b'-')
            && parts.get(7) == Some(&b'-')
            && parts
                .iter()
                .enumerate()
                .all(|(index, byte)| matches!(index, 4 | 7) || byte.is_ascii_digit())
            && year.is_some_and(|year| (1950..=2029).contains(&year))
            && day.is_some_and(|day| (1..=max_day).contains(&day));
        valid
            .then_some(())
            .ok_or_else(|| ValidationError::new("date").with_message("Provide valid ISO 8601 date (e.g., YYYY-MM-DD)".into()))
    }
    /// Validate a complete RFC 3339 timestamp with an explicit UTC offset.
    ///
    /// # Examples
    ///
    /// ```
    /// use acorn::validation::rules;
    /// assert!(rules::timestamp("2026-08-29T12:30:00Z").is_ok());
    /// assert!(rules::timestamp("2026-08-29").is_err());
    /// ```
    pub fn timestamp(value: &str) -> Result<(), ValidationError> {
        let bytes = value.as_bytes();
        let numeric_offset = bytes.len().checked_sub(6).and_then(|start| bytes.get(start..)).is_some_and(|offset| {
            offset.len() == 6
                && matches!(offset.first(), Some(b'+' | b'-'))
                && offset
                    .iter()
                    .skip(1)
                    .enumerate()
                    .all(|(index, byte)| if index == 2 { *byte == b':' } else { byte.is_ascii_digit() })
        });
        let structure = Pieces::parse(value).ok();
        let valid = structure.is_some_and(|parts| {
            parts.time().is_some()
                && parts.time_zone_annotation().is_none()
                && (value.ends_with('Z') || numeric_offset)
                && value.parse::<Timestamp>().is_ok()
        });
        valid
            .then_some(())
            .ok_or_else(|| ValidationError::new("timestamp").with_message("Provide a complete RFC 3339 timestamp with an explicit UTC offset".into()))
    }
    /// Validate a ten-digit Unix epoch timestamp.
    pub fn unix_epoch(value: impl Borrow<usize>) -> Result<(), ValidationError> {
        RE_UNIX_EPOCH
            .is_match(&value.borrow().to_string())
            .is_ok_and(core::convert::identity)
            .then_some(())
            .ok_or_else(|| ValidationError::new("epoch").with_message("Provide valid Unix epoch timestamp".into()))
    }
    /// Validate text containing at least one non-whitespace Unicode scalar.
    ///
    /// # Examples
    ///
    /// ```
    /// use acorn::validation::rules;
    /// assert!(rules::nonempty(" ACORN ").is_ok());
    /// assert!(rules::nonempty("\u{2003}\n").is_err());
    /// ```
    pub fn nonempty(value: &str) -> Result<(), ValidationError> {
        value
            .chars()
            .any(|character| !character.is_whitespace())
            .then_some(())
            .ok_or_else(|| ValidationError::new("nonempty").with_message("Provide text containing a non-whitespace character".into()))
    }
    /// Validate an email address using the established ACORN-compatible shape.
    ///
    /// # Examples
    ///
    /// ```
    /// use acorn::validation::rules;
    /// assert!(rules::email("research@example.org").is_ok());
    /// assert!(rules::email("not an email").is_err());
    /// ```
    pub fn email(value: &str) -> Result<(), ValidationError> {
        let valid_user = |user: &str| {
            !user.is_empty()
                && user.chars().count() <= 64
                && user
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric() || ".!#$%&'*+/=?^_`{|}~-".contains(character))
        };
        let valid_ascii_domain = |domain: &str| {
            if let Some(literal) = domain.strip_prefix('[').and_then(|value| value.strip_suffix(']')) {
                return IpAddr::from_str(literal).is_ok();
            }
            !domain.is_empty()
                && domain.split('.').all(|label| {
                    !label.is_empty()
                        && label.len() <= 63
                        && label.as_bytes().first().is_some_and(u8::is_ascii_alphanumeric)
                        && label.as_bytes().last().is_some_and(u8::is_ascii_alphanumeric)
                        && label.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
                })
        };
        let valid = value.rsplit_once('@').is_some_and(|(user, domain)| {
            valid_user(user)
                && domain.chars().count() <= 255
                && (valid_ascii_domain(domain) || domain_to_ascii(domain).is_ok_and(|domain| valid_ascii_domain(&domain)))
        });
        valid.then_some(()).ok_or_else(|| ValidationError::new("email"))
    }
    /// Validate an absolute URL.
    ///
    /// # Examples
    ///
    /// ```
    /// use acorn::validation::rules;
    /// assert!(rules::url("https://example.org/research").is_ok());
    /// assert!(rules::url("relative/path").is_err());
    /// ```
    pub fn url(value: &str) -> Result<(), ValidationError> {
        UriRef::parse(value)
            .ok()
            .filter(|uri| uri.scheme().is_some())
            .map(|_| ())
            .ok_or_else(|| ValidationError::new("url"))
    }
    /// Validate ACORN's exact abbreviated semantic-version syntax.
    ///
    /// # Examples
    ///
    /// ```
    /// use acorn::validation::rules;
    /// assert!(rules::version("1.2.3").is_ok());
    /// assert!(rules::version("v1.2.3").is_err());
    /// ```
    pub fn version(value: &str) -> Result<(), ValidationError> {
        SemanticVersion::parse_exact(value)
            .map(|_| ())
            .map_err(|_| ValidationError::new("version").with_message("Provide valid semantic version".into()))
    }
    /// Validate an IPv4 address
    /// ### Examples
    /// ```
    /// use acorn::validation::rules;
    ///
    /// assert!(rules::ipv4("192.168.1.1").is_ok());
    /// assert!(rules::ipv4("256.1.1.1").is_err());
    /// ```
    pub fn ipv4(value: &str) -> Result<(), ValidationError> {
        value
            .parse::<Ipv4Addr>()
            .map(|_| ())
            .map_err(|_| ValidationError::new("ipv4").with_message("Provide valid IPv4 address".into()))
    }
    /// Validate an IPv6 address
    /// ### Examples
    /// ```
    /// use acorn::validation::rules;
    ///
    /// assert!(rules::ipv6("2001:db8::1").is_ok());
    /// assert!(rules::ipv6("192.168.1.1").is_err());
    /// ```
    pub fn ipv6(value: &str) -> Result<(), ValidationError> {
        value
            .parse::<Ipv6Addr>()
            .map(|_| ())
            .map_err(|_| ValidationError::new("ipv6").with_message("Provide valid IPv6 address".into()))
    }
    /// Validate a four-digit year that is not in the future when a clock is available.
    ///
    /// # Examples
    ///
    /// ```
    /// use acorn::validation::rules;
    /// assert!(rules::year(2020).is_ok());
    /// assert!(rules::year("20").is_err());
    /// ```
    pub fn year(value: impl ToString) -> Result<(), ValidationError> {
        let value = value.to_string();
        let parsed = (value.len() == 4)
            .then(|| value.parse::<i16>().ok())
            .flatten()
            .and_then(|year| Date::new(year, 1, 1).ok())
            .map(|date| i32::from(date.year()));
        #[cfg(feature = "std")]
        let valid = parsed.is_some_and(|year| year <= i32::from(Timestamp::now().to_zoned(TimeZone::UTC).year()));
        #[cfg(not(feature = "std"))]
        let valid = parsed.is_some();
        valid
            .then_some(())
            .ok_or_else(|| ValidationError::new("year").with_message("Provide valid ISO 8601 year (e.g., YYYY)".into()))
    }
}
#[cfg(test)]
mod tests;
