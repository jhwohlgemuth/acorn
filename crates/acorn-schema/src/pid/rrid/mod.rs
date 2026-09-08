//! Research Resource Identifier (RRID) parsing and formatting.
use crate::pid::{PersistentIdentifier, PersistentIdentifierParse};
use crate::util::constants::app::{MAX_RRID_PAYLOAD_LENGTH, N2T_URI, RRID_SCHEMA_URI, SCICRUNCH_RRID_RESOLVER_URI};
use crate::util::constants::RE_RRID;
use crate::validation::{Validate, ValidationReport};
use acorn_core::prelude::alloc::{format, String, ToString, Vec};
use acorn_core::validation::ValidationError;
use core::{fmt, str::FromStr};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A Research Resource Identifier with an opaque registry-assigned payload.
#[derive(Clone, Debug, Default, Deserialize, Eq, JsonSchema, Ord, PartialEq, PartialOrd, Serialize, Validate)]
#[serde(deny_unknown_fields)]
#[validate(schema(function = "RRID::has_valid_payload", skip_on_field_errors = false))]
pub struct RRID {
    /// Registry-assigned payload, excluding the `RRID:` label.
    pub payload: String,
}
/// Error returned when an RRID cannot be parsed or validated.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RridParseError {
    message: String,
}
impl RRID {
    fn has_valid_payload(&self, _context: &()) -> Result<(), ValidationReport> {
        let length = self.payload.chars().count();
        let nested_label = self.payload.get(..5).is_some_and(|prefix| prefix.eq_ignore_ascii_case("RRID:"));
        let visible = self
            .payload
            .chars()
            .all(|character| !character.is_control() && !character.is_whitespace());
        let no_encoded_delimiter = !self.payload.contains('%');
        let no_url_suffix = !self.payload.contains(['?', '#']);
        let no_prose_suffix = !self
            .payload
            .chars()
            .last()
            .is_some_and(|character| matches!(character, '.' | ',' | ';' | ':' | '!' | ')' | ']' | '}' | '>'));
        let message = match (length, nested_label, visible, no_encoded_delimiter, no_url_suffix, no_prose_suffix) {
            | (0, _, _, _, _, _) => Some("RRID payload cannot be empty"),
            | (length, _, _, _, _, _) if length > MAX_RRID_PAYLOAD_LENGTH => Some("RRID payload exceeds 255 characters"),
            | (_, true, _, _, _, _) => Some("RRID payload cannot contain a second RRID label"),
            | (_, _, false, _, _, _) => Some("RRID payload cannot contain whitespace or control characters"),
            | (_, _, _, false, _, _) => Some("RRID payload cannot retain encoded delimiters"),
            | (_, _, _, _, false, _) => Some("RRID payload cannot contain URL query or fragment delimiters"),
            | (_, _, _, _, _, false) => Some("RRID payload cannot end with prose punctuation"),
            | _ => None,
        };
        message.map_or(Ok(()), |message| {
            Err(ValidationReport::from_error(
                "payload",
                ValidationError::new("RRID").with_message(message),
            ))
        })
    }
}
impl fmt::Display for RRID {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "RRID:{}", self.payload)
    }
}
impl FromStr for RRID {
    type Err = RridParseError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let value = value.trim();
        let lowercase = value.to_ascii_lowercase();
        let prefixes = [
            format!("{N2T_URI}/").to_ascii_lowercase(),
            format!("{SCICRUNCH_RRID_RESOLVER_URI}/").to_ascii_lowercase(),
            "http://scicrunch.org/resolver/".to_string(),
        ];
        let has_query_or_fragment = value.contains(['?', '#']);
        let prefix = prefixes.iter().find(|prefix| lowercase.starts_with(prefix.as_str()));
        let representation = match (has_query_or_fragment, prefix) {
            | (true, _) => Err(RridParseError::new("RRID resolver URLs cannot contain query or fragment data")),
            | (false, Some(prefix)) => value
                .get(prefix.len()..)
                .map(ToString::to_string)
                .ok_or_else(|| RridParseError::new("RRID resolver URL does not contain an identifier")),
            | (false, None) if lowercase.starts_with("http://") || lowercase.starts_with("https://") => {
                Err(RridParseError::new("unsupported RRID resolver URL"))
            }
            | (false, None) => Ok(value.to_string()),
        };
        representation
            .and_then(|value| {
                value
                    .split_once(':')
                    .filter(|(label, _)| label.trim().eq_ignore_ascii_case("RRID"))
                    .map(|(_, payload)| payload.trim_matches([' ', '\t']))
                    .and_then(|payload| urlencoding::decode(payload).ok())
                    .map(|payload| payload.into_owned())
                    .ok_or_else(|| RridParseError::new("RRID must use an explicit RRID label and valid percent encoding"))
            })
            .map(|payload| Self { payload })
            .and_then(|rrid| rrid.validate().map(|()| rrid).map_err(|report| RridParseError::new(report.to_string())))
    }
}
impl PersistentIdentifier for RRID {
    fn new() -> Self {
        Self::default()
    }
    fn schema_uri(&self) -> String {
        RRID_SCHEMA_URI.to_string()
    }
    fn identifier(&self) -> String {
        self.to_string()
    }
    fn prefix(&self) -> Option<String> {
        Some("RRID".to_string())
    }
    fn suffix(&self) -> Option<String> {
        (!self.payload.is_empty()).then(|| self.payload.clone())
    }
    fn url(&self) -> String {
        if !self.payload.is_empty() {
            format!("{N2T_URI}/{self}")
        } else {
            Default::default()
        }
    }
}
impl PersistentIdentifierParse for RRID {
    fn find_all(value: impl ToString) -> Vec<Self> {
        RE_RRID
            .find_iter(&value.to_string())
            .filter_map(Result::ok)
            .filter_map(|matched| matched.as_str().trim_end_matches('.').parse().ok())
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
impl core::error::Error for RridParseError {}
impl fmt::Display for RridParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}
impl RridParseError {
    fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
}
#[cfg(all(test, feature = "std"))]
mod tests;
