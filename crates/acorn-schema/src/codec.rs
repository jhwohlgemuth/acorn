//! Portable content codecs for schema-owned document types.
use crate::modelkit::kitfile::Kitfile;
use crate::pid::raid;
use crate::research_activity::ResearchActivity;
use crate::standard::{cff::Cff, ckan, datacite, invenio, text, zon};
#[cfg(feature = "std")]
use crate::standard::{dcat, huwise};
use crate::types::OneOrMany;
use acorn_core::prelude::{format, vec, String, ToString, Vec};
use acorn_core::util::{MarkdownDecode, MarkdownSupport, MimeType};
use acorn_macros::With;
use bon::Builder;
use core::fmt;
use serde::de::DeserializeOwned;
use serde::Serialize;

/// Decode and encode a schema document without performing host I/O.
pub trait DocumentCodec: Sized {
    /// Human-readable document name used in codec diagnostics.
    const NAME: &'static str;
    /// Decode complete document content in the selected format.
    fn decode(content: &str, format: &MimeType) -> Result<Self, Vec<CodecDiagnostic>>;
    /// Encode the document in the selected format.
    fn encode(&self, format: &MimeType) -> Result<String, CodecDiagnostic>;
}
/// One portable document decoding or encoding diagnostic.
#[derive(Builder, Clone, Debug, Eq, PartialEq, With)]
pub struct CodecDiagnostic {
    /// Stable diagnostic code.
    pub code: String,
    /// Human-readable diagnostic message.
    pub message: String,
    /// Optional document field path.
    pub path: Option<String>,
}
impl DocumentCodec for Cff {
    const NAME: &'static str = "CFF";
    fn decode(content: &str, format: &MimeType) -> Result<Self, Vec<CodecDiagnostic>> {
        match format {
            | MimeType::Json => eserde::json::from_str::<Self>(content).map_err(|errors| {
                errors
                    .iter()
                    .map(|error| CodecDiagnostic::deserialize(error.message()).with_path(error.path().map(ToString::to_string)))
                    .collect()
            }),
            | MimeType::Cff | MimeType::Yaml => {
                serde_norway::from_str(content).map_err(|error| vec![CodecDiagnostic::deserialize(error.to_string())])
            }
            | _ => Err(vec![CodecDiagnostic::unsupported(format, Self::NAME)]),
        }
    }
    fn encode(&self, format: &MimeType) -> Result<String, CodecDiagnostic> {
        encode_structured(self, format)
    }
}
impl DocumentCodec for ckan::Package {
    const NAME: &'static str = "CKAN package";
    fn decode(content: &str, format: &MimeType) -> Result<Self, Vec<CodecDiagnostic>> {
        ckan::Package::parse(content, format.clone())
            .map_err(|error| vec![CodecDiagnostic::deserialize(error.to_string())])
            .and_then(|packages| match packages {
                | OneOrMany::One(package) => Ok(package),
                | OneOrMany::Many(mut packages) if packages.len() == 1 => {
                    packages.pop().ok_or_else(|| vec![CodecDiagnostic::expected_one(Self::NAME, 0)])
                }
                | OneOrMany::Many(packages) => Err(vec![CodecDiagnostic::expected_one(Self::NAME, packages.len())]),
            })
    }
    fn encode(&self, format: &MimeType) -> Result<String, CodecDiagnostic> {
        encode_structured(self, format)
    }
}
impl CodecDiagnostic {
    /// Create a deserialization diagnostic.
    pub fn deserialize(message: impl Into<String>) -> Self {
        Self::builder().code("deserialize".to_string()).message(message.into()).build()
    }
    /// Create a diagnostic for a document-count mismatch.
    pub fn expected_one(label: &str, count: usize) -> Self {
        Self::builder()
            .code("document-count".to_string())
            .message(format!("Expected one {label} but found {count}"))
            .build()
    }
    /// Create a serialization diagnostic.
    pub fn serialize(message: impl Into<String>) -> Self {
        Self::builder().code("serialize".to_string()).message(message.into()).build()
    }
    /// Create an unsupported-format diagnostic.
    pub fn unsupported(format: &MimeType, label: &str) -> Self {
        Self::builder()
            .code("unsupported-format".to_string())
            .message(format!("Unsupported {label} format '{format}'"))
            .build()
    }
}
impl fmt::Display for CodecDiagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.path.as_deref() {
            | Some(path) => write!(formatter, "{path}: {}", self.message),
            | None => formatter.write_str(&self.message),
        }
    }
}
impl DocumentCodec for Kitfile {
    const NAME: &'static str = "Kitfile";
    fn decode(content: &str, format: &MimeType) -> Result<Self, Vec<CodecDiagnostic>> {
        decode_structured(content, format)
    }
    fn encode(&self, format: &MimeType) -> Result<String, CodecDiagnostic> {
        match format {
            | MimeType::Json => self.to_resolved_json().map_err(|error| CodecDiagnostic::deserialize(error.to_string())),
            | MimeType::Yaml => self.to_yaml().map_err(|error| CodecDiagnostic::deserialize(error.to_string())),
            | _ => Err(CodecDiagnostic::unsupported(format, Self::NAME)),
        }
    }
}
impl DocumentCodec for raid::Metadata {
    const NAME: &'static str = "RAiD metadata";
    fn decode(content: &str, format: &MimeType) -> Result<Self, Vec<CodecDiagnostic>> {
        match format {
            | MimeType::Json => eserde::json::from_str::<Self>(content).map_err(|errors| {
                errors
                    .iter()
                    .map(|error| CodecDiagnostic::deserialize(error.message()).with_path(error.path().map(ToString::to_string)))
                    .collect()
            }),
            | MimeType::Yaml => serde_norway::from_str(content).map_err(|error| vec![CodecDiagnostic::deserialize(error.to_string())]),
            | _ => Err(vec![CodecDiagnostic::unsupported(format, Self::NAME)]),
        }
    }
    fn encode(&self, format: &MimeType) -> Result<String, CodecDiagnostic> {
        encode_structured(self, format)
    }
}
impl DocumentCodec for ResearchActivity {
    const NAME: &'static str = "research activity";
    fn decode(content: &str, format: &MimeType) -> Result<Self, Vec<CodecDiagnostic>> {
        match format {
            | MimeType::Json => decode_research_activity_json(content),
            | MimeType::Jsonc => jsonc_parser::parse_to_serde_value::<serde_json::Value>(content, &Default::default())
                .map_err(|error| vec![CodecDiagnostic::deserialize(error.to_string())])
                .and_then(|value| {
                    serde_json::to_string(&value)
                        .map_err(|error| vec![CodecDiagnostic::deserialize(error.to_string())])
                        .and_then(|json| decode_research_activity_json(&json))
                }),
            | MimeType::Markdown => Self::from_markdown(content).map_err(|error| vec![CodecDiagnostic::deserialize(error)]),
            | MimeType::Yaml => serde_norway::from_str(content).map_err(|error| vec![CodecDiagnostic::deserialize(error.to_string())]),
            | MimeType::Zon => zon::decode_value(content)
                .map_err(|error| vec![CodecDiagnostic::deserialize(error.to_string())])
                .and_then(|value| {
                    serde_json::to_string(&value)
                        .map_err(|error| vec![CodecDiagnostic::deserialize(error.to_string())])
                        .and_then(|json| decode_research_activity_json(&json))
                }),
            | _ => Err(vec![CodecDiagnostic::unsupported(format, Self::NAME)]),
        }
    }
    fn encode(&self, format: &MimeType) -> Result<String, CodecDiagnostic> {
        match format {
            | MimeType::Json | MimeType::Jsonc => serde_json::to_string_pretty(self).map_err(|error| CodecDiagnostic::serialize(error.to_string())),
            | MimeType::Markdown => Ok(self.to_markdown()),
            | MimeType::Yaml => serde_json::to_value(self)
                .map_err(|error| CodecDiagnostic::serialize(error.to_string()))
                .and_then(|value| serde_norway::to_string(&value).map_err(|error| CodecDiagnostic::serialize(error.to_string()))),
            | MimeType::Zon => zon::encode(self).map_err(|error| CodecDiagnostic::serialize(error.to_string())),
            | _ => Err(CodecDiagnostic::unsupported(format, Self::NAME)),
        }
    }
}
impl DocumentCodec for text::Text {
    const NAME: &'static str = "text";
    fn decode(content: &str, format: &MimeType) -> Result<Self, Vec<CodecDiagnostic>> {
        match format {
            | MimeType::Markdown | MimeType::Text => Ok(Self::from(content)),
            | _ => decode_structured(content, format),
        }
    }
    fn encode(&self, format: &MimeType) -> Result<String, CodecDiagnostic> {
        match format {
            | MimeType::Markdown | MimeType::Text => Ok(self.content.clone()),
            | _ => encode_structured(self, format),
        }
    }
}
macro_rules! single_document_codec {
    ($($kind:ty => $label:literal),+ $(,)?) => {
        $(impl DocumentCodec for $kind {
            const NAME: &'static str = $label;
            fn decode(content: &str, format: &MimeType) -> Result<Self, Vec<CodecDiagnostic>> {
                decode_single(content, format)
            }
            fn encode(&self, format: &MimeType) -> Result<String, CodecDiagnostic> {
                encode_structured(self, format)
            }
        })+
    };
}
single_document_codec!(datacite::Record => "DataCite record", invenio::Record => "Invenio record");
#[cfg(feature = "std")]
single_document_codec!(dcat::Dataset => "DCAT dataset", huwise::Dataset => "HuWise dataset");

fn decode_research_activity_json(content: &str) -> Result<ResearchActivity, Vec<CodecDiagnostic>> {
    eserde::json::from_str::<ResearchActivity>(content).map_err(|errors| {
        errors
            .iter()
            .map(|error| CodecDiagnostic::deserialize(error.message()).with_path(error.path().map(ToString::to_string)))
            .collect()
    })
}
fn decode_single<T: DeserializeOwned + DocumentCodec>(content: &str, format: &MimeType) -> Result<T, Vec<CodecDiagnostic>> {
    let decoded = match format {
        | MimeType::Json => serde_json::from_str::<OneOrMany<T>>(content).map_err(|error| vec![CodecDiagnostic::deserialize(error.to_string())]),
        | MimeType::Yaml => serde_norway::from_str::<OneOrMany<T>>(content).map_err(|error| vec![CodecDiagnostic::deserialize(error.to_string())]),
        | _ => Err(vec![CodecDiagnostic::unsupported(format, T::NAME)]),
    };
    decoded.and_then(|value| match value {
        | OneOrMany::One(value) => Ok(value),
        | OneOrMany::Many(mut values) if values.len() == 1 => values.pop().ok_or_else(|| vec![CodecDiagnostic::expected_one(T::NAME, 0)]),
        | OneOrMany::Many(values) => Err(vec![CodecDiagnostic::expected_one(T::NAME, values.len())]),
    })
}
fn decode_structured<T: DeserializeOwned + DocumentCodec>(content: &str, format: &MimeType) -> Result<T, Vec<CodecDiagnostic>> {
    match format {
        | MimeType::Json => serde_json::from_str(content).map_err(|error| vec![CodecDiagnostic::deserialize(error.to_string())]),
        | MimeType::Jsonc => jsonc_parser::parse_to_serde_value::<serde_json::Value>(content, &Default::default())
            .map_err(|error| vec![CodecDiagnostic::deserialize(error.to_string())])
            .and_then(|value| serde_json::from_value(value).map_err(|error| vec![CodecDiagnostic::deserialize(error.to_string())])),
        | MimeType::Cff | MimeType::Yaml => serde_norway::from_str(content).map_err(|error| vec![CodecDiagnostic::deserialize(error.to_string())]),
        | MimeType::Zon => zon::decode(content).map_err(|error| vec![CodecDiagnostic::deserialize(error.to_string())]),
        | _ => Err(vec![CodecDiagnostic::unsupported(format, T::NAME)]),
    }
}
/// Decode a generic structured value in a portable schema format.
pub fn decode_value(content: &str, format: &MimeType) -> Result<serde_json::Value, CodecDiagnostic> {
    match format {
        | MimeType::Json => serde_json::from_str(content).map_err(|error| CodecDiagnostic::deserialize(error.to_string())),
        | MimeType::Jsonc => {
            jsonc_parser::parse_to_serde_value(content, &Default::default()).map_err(|error| CodecDiagnostic::deserialize(error.to_string()))
        }
        | MimeType::Yaml => serde_norway::from_str(content).map_err(|error| CodecDiagnostic::deserialize(error.to_string())),
        | MimeType::Zon => zon::decode_value(content).map_err(|error| CodecDiagnostic::deserialize(error.to_string())),
        | _ => Err(CodecDiagnostic::unsupported(format, "structured document")),
    }
}
fn encode_structured<T: DocumentCodec + Serialize>(value: &T, format: &MimeType) -> Result<String, CodecDiagnostic> {
    match format {
        | MimeType::Json | MimeType::Jsonc => serde_json::to_string_pretty(value).map_err(|error| CodecDiagnostic::serialize(error.to_string())),
        | MimeType::Cff | MimeType::Yaml => serde_norway::to_string(value).map_err(|error| CodecDiagnostic::serialize(error.to_string())),
        | MimeType::Zon => zon::encode(value).map_err(|error| CodecDiagnostic::serialize(error.to_string())),
        | _ => Err(CodecDiagnostic::unsupported(format, T::NAME)),
    }
}
/// Encode a generic structured value in a portable schema format.
pub fn encode_value(value: &serde_json::Value, format: &MimeType) -> Result<String, CodecDiagnostic> {
    match format {
        | MimeType::Json | MimeType::Jsonc => serde_json::to_string_pretty(value).map_err(|error| CodecDiagnostic::serialize(error.to_string())),
        | MimeType::Yaml => serde_norway::to_string(value).map_err(|error| CodecDiagnostic::serialize(error.to_string())),
        | MimeType::Zon => zon::encode(value).map_err(|error| CodecDiagnostic::serialize(error.to_string())),
        | _ => Err(CodecDiagnostic::unsupported(format, "structured document")),
    }
}
