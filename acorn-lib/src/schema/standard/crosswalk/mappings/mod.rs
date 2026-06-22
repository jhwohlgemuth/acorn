//! Field mapping rules for cross-schema conversions
//!
//! Each conversion pair gets a FieldMapping that defines how to transform fields.
//! These mappings are reused by From<> impls and custom Crosswalk impls.

pub mod datacite_to_dcat;
pub mod dcat_to_datacite;
pub mod invenio_to_datacite;
pub mod datacite_to_invenio;
pub mod huwise_to_datacite;
pub mod datacite_to_huwise;
