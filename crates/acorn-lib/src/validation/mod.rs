//! Portable validation contracts and built-in rules.
use crate::prelude::alloc::{format, vec, Box, String, ToString, Vec};
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
/// Built-in scalar validation rules used by the derive macro.
pub mod rules {
    use super::ValidationError;
    use crate::prelude::alloc::ToString;
    use crate::util::SemanticVersion;
    use core::net::IpAddr;
    use core::str::FromStr;
    use fluent_uri::UriRef;
    use idna::domain_to_ascii;
    use jiff::civil::Date;
    #[cfg(feature = "std")]
    use jiff::Timestamp;

    /// Validate an ACORN-supported ISO 8601 calendar date.
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
    /// Validate an email address using the established ACORN-compatible shape.
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
    pub fn url(value: &str) -> Result<(), ValidationError> {
        UriRef::parse(value)
            .ok()
            .filter(|uri| uri.scheme().is_some())
            .map(|_| ())
            .ok_or_else(|| ValidationError::new("url"))
    }
    /// Validate ACORN's exact abbreviated semantic-version syntax.
    pub fn version(value: &str) -> Result<(), ValidationError> {
        SemanticVersion::parse_exact(value)
            .map(|_| ())
            .map_err(|_| ValidationError::new("version").with_message("Provide valid semantic version".into()))
    }
    /// Validate a four-digit year that is not in the future when a clock is available.
    pub fn year(value: impl ToString) -> Result<(), ValidationError> {
        let value = value.to_string();
        let parsed = (value.len() == 4)
            .then(|| value.parse::<i16>().ok())
            .flatten()
            .and_then(|year| Date::new(year, 1, 1).ok())
            .map(|date| i32::from(date.year()));
        #[cfg(feature = "std")]
        let valid = parsed.is_some_and(|year| year <= i32::from(Timestamp::now().to_zoned(jiff::tz::TimeZone::UTC).year()));
        #[cfg(not(feature = "std"))]
        let valid = parsed.is_some();
        valid
            .then_some(())
            .ok_or_else(|| ValidationError::new("year").with_message("Provide valid ISO 8601 year (e.g., YYYY)".into()))
    }
}
#[cfg(test)]
mod tests;
