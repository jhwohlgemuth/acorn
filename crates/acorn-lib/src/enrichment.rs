//! Portable enrichment provider identities.
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString};

/// An enrichment provider available to ACORN workflows.
#[derive(Clone, Copy, Debug, Deserialize, Display, EnumIter, EnumString, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
#[strum(ascii_case_insensitive, serialize_all = "lowercase")]
pub enum Provider {
    /// DOE CODE / OSTI software metadata from [`crate::io::api::osti`].
    Osti,
    /// Scholarly metadata from [`crate::io::api::openalex`].
    OpenAlex,
    /// Citation metadata from [`crate::io::api::citeas`].
    CiteAs,
    /// Researcher identities from [`crate::io::api::orcid`].
    Orcid,
    /// Research Organization Registry identities from [`crate::io::api::ror`].
    Ror,
}
