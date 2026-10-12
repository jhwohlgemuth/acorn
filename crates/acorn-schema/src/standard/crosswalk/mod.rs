//! Generalized metadata schema crosswalk infrastructure
//!
//! This module provides trait abstractions and utilities for bidirectional conversion
//! among metadata standards including Dublin Core, DataCite, DCAT, InvenioRDM, and HuWise.
//!
//! # Design Principles
//!
//! - **Field-level abstraction**: Conversions decompose into field extraction, mapping, and building
//! - **Lossy conversion tracking**: ConversionWarning types document data loss
//! - **Registry-driven discovery**: SchemaRegistry enables dynamic conversion discovery
//! - **Canonical intermediate format**: FieldMap allows type-safe conversions without serde_json
//! - **Trait-based extensibility**: Extractors and builders enable schema-specific logic
//!
//! # Conversion Strategies
//!
//! Two usage patterns:
//!
//! 1. **TryFrom trait (infallible at type level, propagates errors)**:
//!    ```ignore
//!    let dcat_dataset: dcat::Dataset = datacite_record.try_into()?;
//!    ```
//!
//! 2. **Crosswalk trait (error-aware)**:
//!    ```ignore
//!    let (dcat_dataset, warnings) = datacite_record.crosswalk()?;
//!    for warning in warnings {
//!        eprintln!("Data loss: {}", warning);
//!    }
//!    ```
//!
//! # Field Mapping
//!
//! See [`mapping`] for the canonical intermediate representation.
//! Field mappings are defined in submodules and reused across conversions.
//!
//! # Schema Extractors & Builders
//!
//! Each schema provides implementations of [`SchemaExtractor`] and [`SchemaBuilder`] in its standard-specific module.
use crate::codec::CodecDiagnostic;
use acorn_core::prelude::alloc::{format, String, ToString, Vec};
use core::fmt;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

macro_rules! via_datacite {
    ($from:ty => $target:ty) => {
        impl $crate::standard::crosswalk::Crosswalk<$target> for $from {
            fn crosswalk(
                &self,
            ) -> Result<($target, Vec<$crate::standard::crosswalk::ConversionWarning>), $crate::standard::crosswalk::CrosswalkError> {
                <Self as $crate::standard::crosswalk::Crosswalk<$crate::standard::datacite::Record>>::crosswalk(self)
                    .and_then(|(record, warnings)| <$target>::try_from(record).map(|target| (target, warnings)))
            }
        }
        impl $crate::standard::crosswalk::Crosswalk<$from> for $target {
            fn crosswalk(&self) -> Result<($from, Vec<$crate::standard::crosswalk::ConversionWarning>), $crate::standard::crosswalk::CrosswalkError> {
                $crate::standard::datacite::Record::try_from(self.clone())
                    .and_then(|record| <$crate::standard::datacite::Record as $crate::standard::crosswalk::Crosswalk<$from>>::crosswalk(&record))
            }
        }
    };
}
pub(crate) use via_datacite;

mod dublin_core;
pub mod mapping;
mod research_activity;
mod rrid;

pub use dublin_core::DublinCoreImportOptions;
pub use mapping::{FieldMapping, FieldRule, FieldValue, Fields};
pub use research_activity::{RaidExportOptions, RaidImportOptions};
/// Convert between metadata schemas with tracking of warnings and data loss
///
/// Implementers provide error context and track lossy conversions via ConversionWarning.
///
/// # Example
///
/// ```ignore
/// use acorn_schema::standard::crosswalk::Crosswalk;
///
/// let datacite_record = /* ... */;
/// let (dcat_dataset, warnings) = datacite_record.crosswalk()?;
/// for warning in warnings {
///     eprintln!("Note: {}", warning);
/// }
/// ```
pub trait Crosswalk<T>: Sized {
    /// Convert self to target type, tracking data loss via warnings
    ///
    /// Returns `Ok((target, warnings))` on success, even if some fields were unmapped.
    /// Returns `Err` only for missing required fields or transformation errors.
    fn crosswalk(&self) -> Result<(T, Vec<ConversionWarning>), CrosswalkError>;
}
/// Reconstruct a schema type from normalized fields
///
/// Implementers build their type from a `FieldMap`, enabling schema-agnostic transformation targets.
pub trait SchemaBuilder: Sized {
    /// Build from normalized fields
    ///
    /// Returns `Err` if required fields are missing or invalid.
    fn build_from_fields(fields: &Fields) -> Result<Self, CrosswalkError>;
}
/// Extract normalized fields from a schema type
///
/// Implementers decompose their type into a `FieldMap`, enabling schema-agnostic transformations.
pub trait SchemaExtractor {
    /// Extract all fields into normalized form
    fn extract_fields(&self) -> Fields;
}
/// Error type for crosswalk operations
#[derive(Clone, Debug)]
pub enum CrosswalkError {
    /// Field required by target schema is missing in source
    MissingRequiredField(String),
    /// Field transformation failed (e.g., invalid date format)
    TransformationFailed {
        /// The field name that failed to transform
        field: String,
        /// The reason for the transformation failure
        reason: String,
    },
    /// Build failed (e.g., struct construction error)
    BuildFailed(String),
    /// Content parsing failed (JSON/YAML)
    ParseFailed(String),
    /// Output serialization failed
    SerializeFailed(String),
}
#[derive(Clone, Copy, Debug)]
enum CrosswalkSchema {
    DataCite,
    DublinCore,
    Rad,
    RadOutput,
    Raid,
}
/// Metadata standards exposed by the shared schema-crosswalk interface.
#[derive(Clone, Copy, Debug, Deserialize, Display, EnumString, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum CrosswalkStandard {
    /// DataCite metadata.
    Datacite,
    /// Data Catalog Vocabulary metadata.
    Dcat,
    /// Dublin Core metadata.
    #[serde(alias = "dcmi", rename = "dublin-core")]
    #[strum(serialize = "dcmi", serialize = "dublin-core")]
    DublinCore,
    /// HuWise metadata.
    Huwise,
    /// InvenioRDM metadata.
    Invenio,
}
/// Type of data loss during conversion
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FieldLossType {
    /// No equivalent field in target schema
    NoEquivalent,
    /// Mapped to similar field but semantics differ
    Approximated,
    /// Complex structure simplified to scalar or vice versa
    Simplified,
    /// Field intentionally omitted (e.g., requires special handling)
    Omitted,
    /// Field mapping exists but target-specific block not available (HuWise only)
    BlockNotSelected,
}
/// A converted value paired with warnings describing any lossy mappings.
#[derive(Clone, Debug)]
pub struct Conversion<T> {
    value: T,
    warnings: Vec<ConversionWarning>,
}
/// Documents data loss during a schema conversion
#[derive(Clone, Debug)]
pub struct ConversionWarning {
    /// Source schema name (e.g., "datacite", "dcat")
    pub from: &'static str,
    /// Target schema name
    pub to: &'static str,
    /// Source field name that was not fully mapped
    pub field: String,
    /// Type of data loss
    pub loss_type: FieldLossType,
    /// Optional details about the loss (e.g., "sizes and formats omitted")
    pub details: Option<String>,
}
impl<T> Conversion<T> {
    /// Create a conversion from its value and warnings.
    pub fn new(value: T, warnings: Vec<ConversionWarning>) -> Self {
        Self { value, warnings }
    }
    /// Consume the conversion and return its value and warnings.
    pub fn into_parts(self) -> (T, Vec<ConversionWarning>) {
        (self.value, self.warnings)
    }
    /// Consume the conversion and return its value.
    pub fn into_value(self) -> T {
        self.value
    }
    /// Borrow the converted value.
    pub fn value(&self) -> &T {
        &self.value
    }
    /// Borrow the conversion warnings.
    pub fn warnings(&self) -> &[ConversionWarning] {
        &self.warnings
    }
}
impl<T> From<(T, Vec<ConversionWarning>)> for Conversion<T> {
    fn from((value, warnings): (T, Vec<ConversionWarning>)) -> Self {
        Self::new(value, warnings)
    }
}
impl ConversionWarning {
    /// Create a new warning for a field with no equivalent in target schema
    pub fn no_equivalent(from: impl Into<&'static str>, to: impl Into<&'static str>, field: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            field: field.into(),
            details: None,
            loss_type: FieldLossType::NoEquivalent,
        }
    }
    /// Create a new warning for an approximated mapping
    pub fn approximated(from: impl Into<&'static str>, to: impl Into<&'static str>, field: impl Into<String>, details: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            field: field.into(),
            details: Some(details.into()),
            loss_type: FieldLossType::Approximated,
        }
    }
    /// Create a new warning for a simplified mapping
    pub fn simplified(from: impl Into<&'static str>, to: impl Into<&'static str>, field: impl Into<String>, details: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            field: field.into(),
            details: Some(details.into()),
            loss_type: FieldLossType::Simplified,
        }
    }
    /// Create a new warning for an omitted field
    pub fn omitted(from: impl Into<&'static str>, to: impl Into<&'static str>, field: impl Into<String>, details: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            field: field.into(),
            details: Some(details.into()),
            loss_type: FieldLossType::Omitted,
        }
    }
    /// Rebase the warning source and replace the first matching field prefix.
    pub fn rebase(self, from: impl Into<&'static str>, fields: &[(&str, &str)]) -> Self {
        let field = fields
            .iter()
            .find_map(|(source, target)| self.field.strip_prefix(source).map(|suffix| format!("{target}{suffix}")))
            .unwrap_or(self.field);
        Self {
            from: from.into(),
            field,
            ..self
        }
    }
}
impl fmt::Display for ConversionWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} → {}: {} ({}) {}",
            self.from,
            self.to,
            self.field,
            self.loss_type,
            self.details.as_deref().unwrap_or("")
        )
    }
}
impl fmt::Display for CrosswalkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            | Self::MissingRequiredField(field) => write!(f, "missing required field — {field}"),
            | Self::TransformationFailed { field, reason } => {
                write!(f, "failed to transform field '{field}' — {reason}")
            }
            | Self::BuildFailed(reason) => write!(f, "build failed — {reason}"),
            | Self::ParseFailed(reason) => write!(f, "content parse failed — {reason}"),
            | Self::SerializeFailed(reason) => write!(f, "output serialize failed — {reason}"),
        }
    }
}
impl From<CodecDiagnostic> for CrosswalkError {
    fn from(error: CodecDiagnostic) -> Self {
        Self::SerializeFailed(error.to_string())
    }
}
impl From<String> for CrosswalkError {
    fn from(err: String) -> Self {
        CrosswalkError::BuildFailed(err)
    }
}
impl From<Vec<CodecDiagnostic>> for CrosswalkError {
    fn from(errors: Vec<CodecDiagnostic>) -> Self {
        Self::ParseFailed(errors.into_iter().map(|error| error.to_string()).collect::<Vec<_>>().join("\n"))
    }
}
impl fmt::Display for FieldLossType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            | Self::NoEquivalent => write!(f, "no equivalent field"),
            | Self::Approximated => write!(f, "approximated"),
            | Self::Simplified => write!(f, "simplified"),
            | Self::Omitted => write!(f, "omitted"),
            | Self::BlockNotSelected => write!(f, "metadata block not selected"),
        }
    }
}
impl From<CrosswalkSchema> for &'static str {
    fn from(schema: CrosswalkSchema) -> Self {
        match schema {
            | CrosswalkSchema::DataCite => "datacite",
            | CrosswalkSchema::DublinCore => "dublin-core",
            | CrosswalkSchema::Rad => "rad",
            | CrosswalkSchema::RadOutput => "rad-output",
            | CrosswalkSchema::Raid => "raid",
        }
    }
}
/// Execute a schema conversion using the pipeline
///
/// # Steps
///
/// 1. Extract all fields from source using SchemaExtractor
/// 2. Look up FieldMapping for source→target conversion
/// 3. Apply mapping rules to transform fields
/// 4. Build target schema using SchemaBuilder
/// 5. Return target and list of lossy conversions
///
/// # Errors
///
/// Returns error if:
/// - Required fields are missing
/// - Field transformation fails
/// - Target schema cannot be built from fields
pub fn convert<S, T>(source: &S, mapping: &FieldMapping) -> Result<(T, Vec<ConversionWarning>), CrosswalkError>
where
    S: SchemaExtractor,
    T: SchemaBuilder,
{
    let source_fields = source.extract_fields();
    let mut target_fields = Fields::new();
    mapping.apply(&source_fields, &mut target_fields).and_then(|missing| {
        let warnings = missing
            .into_iter()
            .map(|field| {
                ConversionWarning::omitted(
                    mapping.source,
                    mapping.target,
                    field.clone(),
                    format!("optional '{field}' not present in source; target '{}' remains empty", field),
                )
            })
            .collect();
        T::build_from_fields(&target_fields).map(|target| (target, warnings))
    })
}

#[cfg(test)]
mod tests;
