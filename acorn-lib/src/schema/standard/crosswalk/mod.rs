//! Generalized metadata schema crosswalk infrastructure
//!
//! This module provides trait abstractions and utilities for bidirectional conversion
//! among metadata standards: CFF, DataCite, DCAT, InvenioRDM, and HuWise.
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
//! See [`field_map`] for the canonical intermediate representation.
//! Field mappings are defined in submodules and reused across conversions.
//!
//! # Schema Extractors & Builders
//!
//! Each schema provides implementations of [`SchemaExtractor`] and [`SchemaBuilder`]
//! in the [`extractors`] and [`builders`] submodules.
use crate::prelude::*;
use core::fmt;

pub mod mapping;

pub use mapping::{FieldMapping, FieldRule, FieldValue, Fields};

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
impl ConversionWarning {
    /// Create a new warning for a field with no equivalent in target schema
    pub fn no_equivalent(from: &'static str, to: &'static str, field: impl Into<String>) -> Self {
        Self {
            from,
            to,
            field: field.into(),
            details: None,
            loss_type: FieldLossType::NoEquivalent,
        }
    }
    /// Create a new warning for an approximated mapping
    pub fn approximated(from: &'static str, to: &'static str, field: impl Into<String>, details: impl Into<String>) -> Self {
        Self {
            from,
            to,
            field: field.into(),
            details: Some(details.into()),
            loss_type: FieldLossType::Approximated,
        }
    }
    /// Create a new warning for a simplified mapping
    pub fn simplified(from: &'static str, to: &'static str, field: impl Into<String>, details: impl Into<String>) -> Self {
        Self {
            from,
            to,
            field: field.into(),
            details: Some(details.into()),
            loss_type: FieldLossType::Simplified,
        }
    }
    /// Create a new warning for an omitted field
    pub fn omitted(from: &'static str, to: &'static str, field: impl Into<String>, details: impl Into<String>) -> Self {
        Self {
            from,
            to,
            field: field.into(),
            details: Some(details.into()),
            loss_type: FieldLossType::Omitted,
        }
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
impl From<String> for CrosswalkError {
    fn from(err: String) -> Self {
        CrosswalkError::BuildFailed(err)
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
/// Convert between metadata schemas with tracking of warnings and data loss
///
/// Implementers provide error context and track lossy conversions via ConversionWarning.
///
/// # Example
///
/// ```ignore
/// use acorn::schema::standard::crosswalk::Crosswalk;
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
/// Extract normalized fields from a schema type
///
/// Implementers decompose their type into a `FieldMap`, enabling schema-agnostic transformations.
pub trait SchemaExtractor {
    /// Extract all fields into normalized form
    fn extract_fields(&self) -> Fields;
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
    let warnings = Vec::new();
    mapping.apply(&source_fields, &mut target_fields)?;
    let target = T::build_from_fields(&target_fields)?;
    Ok((target, warnings))
}

#[cfg(test)]
mod tests;
