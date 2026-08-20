//! Handle identifier parsing and formatting.
use crate::prelude::*;
use crate::schema::pid::{PersistentIdentifier, PersistentIdentifierParse};
use crate::util::constants::app::HANDLE_RESOLVER_URI;
use crate::util::constants::RE_HANDLE;
use bon::Builder;
use core::fmt;
use core::str::FromStr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A Handle System persistent identifier
#[derive(Builder, Clone, Debug, Default, Deserialize, Eq, JsonSchema, Ord, PartialEq, PartialOrd, Serialize)]
#[builder(start_fn = init, on(String, into))]
#[serde(deny_unknown_fields)]
pub struct Handle {
    /// Handle naming authority prefix
    pub prefix: Option<String>,
    /// Local name assigned under the prefix
    pub suffix: Option<String>,
}
/// Error returned when a Handle cannot be parsed or validated
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandleParseError {
    message: String,
}
impl fmt::Display for Handle {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.identifier())
    }
}
impl fmt::Display for HandleParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}
impl core::error::Error for HandleParseError {}
impl FromStr for Handle {
    type Err = HandleParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        representation(value).and_then(|value| match value.split_once('/') {
            | Some((prefix, suffix)) => {
                let prefix_is_valid = !prefix.is_empty()
                    && prefix
                        .split('.')
                        .all(|segment| !segment.is_empty() && segment.chars().all(|character| character.is_ascii_digit()));
                let suffix_is_valid = !suffix.is_empty() && suffix.chars().all(|character| !character.is_control());
                match (prefix_is_valid, suffix_is_valid, value.to_ascii_lowercase().starts_with("swh:")) {
                    | (true, true, false) => Ok(Self {
                        prefix: Some(prefix.to_string()),
                        suffix: Some(suffix.to_string()),
                    }),
                    | (false, _, _) => Err(HandleParseError::new("Handle prefix is invalid")),
                    | (_, false, _) => Err(HandleParseError::new("Handle local name is invalid")),
                    | _ => Err(HandleParseError::new("Software Heritage identifiers are not generic Handles")),
                }
            }
            | None => Err(HandleParseError::new("Handle must contain a prefix and local name")),
        })
    }
}
impl PersistentIdentifier for Handle {
    fn new() -> Self {
        Self::default()
    }
    fn schema_uri(&self) -> String {
        HANDLE_RESOLVER_URI.to_string()
    }
    fn identifier(&self) -> String {
        self.prefix
            .as_ref()
            .zip(self.suffix.as_ref())
            .map_or_else(String::new, |(prefix, suffix)| format!("{prefix}/{suffix}"))
    }
    fn prefix(&self) -> Option<String> {
        self.prefix.clone()
    }
    fn suffix(&self) -> Option<String> {
        self.suffix.clone()
    }
    fn url(&self) -> String {
        let identifier = self.identifier();
        if identifier.is_empty() {
            String::new()
        } else {
            let encoded = identifier.split('/').map(urlencoding::encode).collect::<Vec<_>>().join("/");
            format!("{HANDLE_RESOLVER_URI}/{encoded}")
        }
    }
}
impl PersistentIdentifierParse for Handle {
    fn find_all(value: impl ToString) -> Vec<Self> {
        RE_HANDLE
            .find_iter(&value.to_string())
            .filter_map(Result::ok)
            .filter_map(|matched| trim_candidate(matched.as_str()).parse().ok())
            .collect()
    }
    fn format(value: impl ToString) -> String {
        value
            .to_string()
            .parse::<Self>()
            .map_or_else(|_| String::new(), |identifier| identifier.to_string())
    }
    fn from_string(value: impl ToString) -> Self {
        value.to_string().parse().unwrap_or_default()
    }
    fn is_valid(value: impl ToString) -> bool {
        value.to_string().parse::<Self>().is_ok()
    }
}
impl Handle {
    /// Find explicitly labeled Handles in unstructured text
    #[cfg(feature = "analysis")]
    pub(crate) fn find_explicit(value: &str) -> Vec<Self> {
        Self::find_all(value)
    }
}
impl HandleParseError {
    fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
}
fn representation(value: &str) -> Result<String, HandleParseError> {
    let value = value.trim();
    let lowercase = value.to_ascii_lowercase();
    let is_resolver_uri = ["https://hdl.handle.net/", "http://hdl.handle.net/"]
        .into_iter()
        .any(|prefix| lowercase.starts_with(prefix));
    match (lowercase.starts_with("https://") || lowercase.starts_with("http://")) && !is_resolver_uri {
        | true => Err(HandleParseError::new("Handle URI must use hdl.handle.net")),
        | false => {
            let (identifier, encoded) = ["https://hdl.handle.net/", "http://hdl.handle.net/", "hdl.handle.net/"]
                .into_iter()
                .find(|prefix| lowercase.starts_with(prefix))
                .map_or_else(
                    || {
                        lowercase
                            .strip_prefix("hdl:")
                            .map(|_| {
                                let path = value.get(4..).unwrap_or_default().trim_start().trim_start_matches("//");
                                (path.split(['?', '#']).next().unwrap_or_default(), true)
                            })
                            .unwrap_or((value, false))
                    },
                    |prefix| {
                        let path = value.get(prefix.len()..).unwrap_or_default();
                        (path.split(['?', '#']).next().unwrap_or_default(), true)
                    },
                );
            match (encoded, valid_percent_encoding(identifier)) {
                | (true, true) => urlencoding::decode(identifier)
                    .map(|decoded| decoded.into_owned())
                    .map_err(|_| HandleParseError::new("Handle URI is not correctly escaped")),
                | (true, false) => Err(HandleParseError::new("Handle URI is not correctly escaped")),
                | (false, _) => Ok(identifier.to_string()),
            }
        }
    }
}
fn trim_candidate(value: &str) -> &str {
    value.trim_matches(|character: char| matches!(character, '<' | '>' | '[' | ']' | '{' | '}' | '(' | ')' | ',' | ';' | '.'))
}
fn valid_percent_encoding(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.iter().enumerate().all(|(index, byte)| {
        *byte != b'%'
            || bytes
                .get(index.saturating_add(1)..index.saturating_add(3))
                .is_some_and(|pair| pair.iter().all(u8::is_ascii_hexdigit))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_idutils_handle_vectors() {
        [
            "10013/epic.10033",
            "hdl:10013/epic.10033",
            "hdl: 10013/epic.10033",
            "HDL:10013/epic.10033",
            "hdl.handle.net/10013/epic.10033",
            "http://hdl.handle.net/10013/epic.10033",
            "https://hdl.handle.net/10013/epic.10033",
        ]
        .into_iter()
        .for_each(|value| assert_eq!(Handle::format(value), "10013/epic.10033", "{value}"));
    }
    #[test]
    fn test_handle_preserves_case_unicode_and_components() {
        let handle = Handle::from_string("hdl:20.5000/Case/üникóδé");
        assert_eq!(handle.prefix(), Some("20.5000".to_string()));
        assert_eq!(handle.suffix(), Some("Case/üникóδé".to_string()));
        assert_eq!(handle.to_string(), "20.5000/Case/üникóδé");
        assert_eq!(Handle::format(handle.url()), handle.to_string());
    }
    #[test]
    fn test_handle_rejects_invalid_values() {
        [
            "",
            "10013",
            "/suffix",
            "10013/",
            "10..123/name",
            "docs/index.md",
            "https://example.org/10013/name",
            "swh:1/cnt",
        ]
        .into_iter()
        .for_each(|value| assert!(!Handle::is_valid(value), "{value}"));
    }
    #[test]
    fn test_handle_find_all_requires_explicit_form() {
        let found = Handle::find_all("See hdl: 10013/epic.10033. Ignore docs/index.md.");
        assert_eq!(found, vec![Handle::from_string("10013/epic.10033")]);
    }
    #[test]
    fn test_handle_uri_ignores_query_and_fragment() {
        [
            "hdl:10013/epic.10033?locatt=view:level2",
            "https://hdl.handle.net/10013/epic.10033#metadata",
        ]
        .into_iter()
        .for_each(|value| assert_eq!(Handle::format(value), "10013/epic.10033", "{value}"));
    }
}
