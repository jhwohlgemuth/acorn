//! Standard metadata schema models
//!
//! This module contains schema implementations for established metadata
//! standards including [DataCite](https://datacite.org/) and
//! [OpenDataSoft](https://www.opendatasoft.com/) (ODS).

pub mod datacite;
pub mod dublin_core;
pub mod ods;

#[cfg(test)]
mod tests;