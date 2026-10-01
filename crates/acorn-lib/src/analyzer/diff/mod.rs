//! Same-standard semantic comparison for supplied ACORN content
use super::Standard;
use acorn_core::util::{MarkdownSupport, MimeType};
use acorn_diff::{unified_diff, DiffValue, JsonPatch};
use acorn_schema::{
    modelkit::kitfile::Kitfile,
    pid::raid,
    research_activity::ResearchActivity,
    standard::{cff, datacite, dcat, huwise, invenio, zon},
};
use core::{error::Error, fmt};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;

/// Semantic diff representation to produce.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DiffFormat {
    /// RFC 6902 patch over the canonical JSON data model.
    JsonPatch,
    /// Unified text diff of canonical same-standard serialization.
    #[default]
    Unified,
}
/// Result of comparing two inputs.
#[derive(Clone, Debug, PartialEq)]
pub enum SemanticDiff {
    /// RFC 6902 JSON Patch; empty when inputs are semantically equal.
    JsonPatch(JsonPatch),
    /// Unified text diff; empty when inputs are semantically equal.
    Unified(String),
}
/// Failure to perform a requested semantic comparison.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SemanticDiffError {
    /// Content could not be parsed as the selected standard.
    Parse {
        /// Input label.
        label: String,
        /// Parser diagnostic.
        message: String,
    },
    /// Canonical content could not be serialized.
    Serialize(String),
    /// The two inputs use different standards.
    StandardMismatch {
        /// Detected standard of the original input.
        before: Standard,
        /// Detected standard of the revised input.
        after: Standard,
    },
    /// The content's standard could not be detected.
    UnknownStandard {
        /// Input label.
        label: String,
    },
    /// The selected standard does not have a semantic adapter.
    UnsupportedStandard(Standard),
}
/// Supplied text and metadata for one side of a comparison.
#[derive(Clone, Debug)]
pub struct DiffInput<'a> {
    /// Serialized content.
    pub content: &'a str,
    /// Label used in unified diff headers.
    pub label: &'a str,
    /// Serialization format of the content.
    pub media_type: MimeType,
}
impl DiffInput<'_> {
    fn parse_error(&self, message: String) -> SemanticDiffError {
        SemanticDiffError::Parse {
            label: self.label.to_string(),
            message,
        }
    }
    fn parse_typed<T>(&self) -> Result<Value, SemanticDiffError>
    where
        T: DeserializeOwned + Serialize,
    {
        self.parse_untyped().and_then(|value| {
            if value.is_array() {
                serde_json::from_value::<Vec<T>>(value)
                    .map_err(|error| self.parse_error(error.to_string()))
                    .and_then(to_value)
            } else {
                serde_json::from_value::<T>(value)
                    .map_err(|error| self.parse_error(error.to_string()))
                    .and_then(to_value)
            }
        })
    }
    fn parse_untyped(&self) -> Result<Value, SemanticDiffError> {
        let result = match &self.media_type {
            | MimeType::Json | MimeType::LdJson => serde_json::from_str(self.content).map_err(|error| error.to_string()),
            | MimeType::Jsonc => crate::io::jsonc_parse_value(self.content).map_err(|error| error.to_string()),
            | MimeType::JsonLines => self
                .content
                .lines()
                .filter(|line| !line.trim().is_empty())
                .map(serde_json::from_str)
                .collect::<Result<Vec<Value>, _>>()
                .map(Value::Array)
                .map_err(|error| error.to_string()),
            | MimeType::Yaml | MimeType::Cff => serde_norway::from_str(self.content).map_err(|error| error.to_string()),
            | MimeType::Zon => zon::decode_value(self.content).map_err(|error| error.to_string()),
            | _ => Err(format!("unsupported source format {}", self.media_type.file_type())),
        };
        result.map_err(|message| self.parse_error(message))
    }
}
impl SemanticDiff {
    /// Compares two inputs without converting either input to a different standard.
    ///
    /// Automatic unified comparison falls back to a raw text diff when semantic detection or parsing is unavailable.
    /// Explicit-standard and JSON Patch comparisons return the semantic error instead.
    pub fn compare(before: &DiffInput<'_>, after: &DiffInput<'_>, format: DiffFormat, standard: Option<Standard>) -> Result<Self, SemanticDiffError> {
        let semantic = Self::compare_semantic(before, after, format, standard);
        match (format, standard, semantic) {
            | (DiffFormat::Unified, None, Err(_)) => Ok(Self::Unified(unified_diff(before.content, after.content, before.label, after.label))),
            | (_, _, result) => result,
        }
    }
    fn compare_semantic(
        before: &DiffInput<'_>,
        after: &DiffInput<'_>,
        format: DiffFormat,
        requested: Option<Standard>,
    ) -> Result<Self, SemanticDiffError> {
        requested
            .map_or_else(|| Standard::detect(before), Ok)
            .and_then(|before_standard| {
                requested
                    .map_or_else(|| Standard::detect(after), Ok)
                    .map(|after_standard| (before_standard, after_standard))
            })
            .and_then(|(before_standard, after_standard)| match before_standard == after_standard {
                | true => before_standard.canonical(before).and_then(|before_value| {
                    after_standard
                        .canonical(after)
                        .map(|after_value| (before_standard, before_value, after_value))
                }),
                | false => Err(SemanticDiffError::StandardMismatch {
                    before: before_standard,
                    after: after_standard,
                }),
            })
            .and_then(|(standard, before_value, after_value)| match format {
                | DiffFormat::JsonPatch => Ok(Self::JsonPatch(before_value.diff(&after_value))),
                | DiffFormat::Unified => standard
                    .render_pair(&before_value, &after_value, &before.media_type)
                    .map(|(left, right)| Self::Unified(unified_diff(&left, &right, before.label, after.label))),
            })
    }
}
impl Error for SemanticDiffError {}
impl fmt::Display for SemanticDiffError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            | Self::Parse { label, message } => write!(formatter, "could not parse {label}: {message}"),
            | Self::Serialize(message) => write!(formatter, "could not serialize canonical content: {message}"),
            | Self::StandardMismatch { before, after } => write!(formatter, "standards do not match: {before} and {after}"),
            | Self::UnknownStandard { label } => write!(formatter, "could not detect a metadata standard for {label}"),
            | Self::UnsupportedStandard(standard) => write!(formatter, "semantic diff does not support {standard}"),
        }
    }
}
impl Standard {
    fn canonical(self, input: &DiffInput<'_>) -> Result<Value, SemanticDiffError> {
        match self {
            | Standard::CitationFileFormat => input.parse_typed::<cff::Cff>(),
            | Standard::Datacite => input.parse_typed::<datacite::Record>(),
            | Standard::Dcat => input.parse_typed::<dcat::Dataset>(),
            | Standard::Huwise => input.parse_typed::<huwise::Dataset>(),
            | Standard::Invenio => input.parse_typed::<invenio::Record>(),
            | Standard::Raid => input.parse_typed::<raid::Metadata>(),
            | Standard::ResearchActivityData if input.media_type.is_markdown() => ResearchActivity::from_markdown(input.content)
                .map_err(|message| input.parse_error(message))
                .and_then(to_value),
            | Standard::ResearchActivityData => input.parse_typed::<ResearchActivity>(),
            | Standard::Kitfile if input.media_type.is_yaml() || input.media_type.is_cff() => Kitfile::from_yaml(input.content)
                .map_err(|error| input.parse_error(error.to_string()))
                .and_then(to_value),
            | Standard::Kitfile | Standard::ModelKit => Kitfile::from_resolved_json(input.content.as_bytes())
                .map_err(|error| input.parse_error(error.to_string()))
                .and_then(to_value),
            | Standard::Document | Standard::DublinCore | Standard::Text => Err(SemanticDiffError::UnsupportedStandard(self)),
        }
    }
    fn detect(input: &DiffInput<'_>) -> Result<Self, SemanticDiffError> {
        match input.media_type.is_markdown() && ResearchActivity::is_markdown(input.content) {
            | true => Ok(Self::ResearchActivityData),
            | false => input
                .parse_untyped()
                .ok()
                .and_then(|value| Self::detect_value(&value))
                .ok_or_else(|| SemanticDiffError::UnknownStandard {
                    label: input.label.to_string(),
                }),
        }
    }
    fn detect_value(value: &Value) -> Option<Self> {
        match value {
            | Value::Object(object) => Self::detect_object(object),
            | Value::Array(values) => {
                let mut standards = values.iter().map(Self::detect_value);
                let first = standards.next().flatten();
                first.filter(|standard| standards.all(|candidate| candidate == Some(*standard)))
            }
            | _ => None,
        }
    }
    fn render(self, value: &Value, media_type: &MimeType) -> Result<String, SemanticDiffError> {
        match self == Self::ResearchActivityData && media_type.is_markdown() {
            | true => serde_json::from_value::<ResearchActivity>(value.clone())
                .map_err(|error| SemanticDiffError::Serialize(error.to_string()))
                .map(|activity| activity.to_markdown()),
            | false => match media_type {
                | MimeType::Json | MimeType::Jsonc | MimeType::LdJson => {
                    serde_json::to_string_pretty(value).map_err(|error| SemanticDiffError::Serialize(error.to_string()))
                }
                | MimeType::JsonLines => value
                    .as_array()
                    .ok_or_else(|| SemanticDiffError::Serialize("JSON Lines requires an array".to_string()))
                    .and_then(|values| {
                        values
                            .iter()
                            .map(serde_json::to_string)
                            .collect::<Result<Vec<_>, _>>()
                            .map(|lines| lines.join("\n") + "\n")
                            .map_err(|error| SemanticDiffError::Serialize(error.to_string()))
                    }),
                | MimeType::Yaml | MimeType::Cff => serde_norway::to_string(value).map_err(|error| SemanticDiffError::Serialize(error.to_string())),
                | MimeType::Zon => zon::encode(value).map_err(|error| SemanticDiffError::Serialize(error.to_string())),
                | _ => Err(SemanticDiffError::Serialize(format!(
                    "unsupported target format {}",
                    media_type.file_type()
                ))),
            },
        }
    }
    fn render_pair(self, before: &Value, after: &Value, media_type: &MimeType) -> Result<(String, String), SemanticDiffError> {
        self.render(before, media_type)
            .and_then(|left| self.render(after, media_type).map(|right| (left, right)))
    }
}
fn to_value<T: Serialize>(value: T) -> Result<Value, SemanticDiffError> {
    serde_json::to_value(value).map_err(|error| SemanticDiffError::Serialize(error.to_string()))
}

#[cfg(test)]
mod tests;
