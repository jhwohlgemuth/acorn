//! Schema extractors and builders by-schema
//!
//! Each schema module provides implementations of SchemaExtractor and SchemaBuilder
//! for its main types, enabling field-level conversion logic.

pub mod datacite;
pub mod dcat;
pub mod huwise;
pub mod invenio;

// TODO: cff extractor/builder
