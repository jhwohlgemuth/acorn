//! Standard metadata schema models
//!
//! This module contains schema implementations for established metadata
//! standards including [DataCite](https://datacite.org/) and
//! [HuWise](https://www.opendatasoft.com/) (HuWise).
//!
//! The [`crosswalk`] submodule provides trait abstractions and utilities for
//! bidirectional conversion among these schemas.

pub mod cff;
pub mod crosswalk;
pub mod datacite;
pub mod dcat;
pub mod dublin_core;
pub mod huwise;
pub mod invenio;
pub mod text;
pub(crate) mod validate;

#[cfg(test)]
mod tests;
