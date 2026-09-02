//! Portable enrichment provider identities.
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString};

/// An enrichment provider available to ACORN workflows.
#[derive(Clone, Copy, Debug, Deserialize, Display, EnumIter, EnumString, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
#[strum(ascii_case_insensitive, serialize_all = "lowercase")]
pub enum Provider {
    /// DOE CODE / OSTI software metadata supplied by a host adapter.
    Osti,
    /// Scholarly metadata supplied by a host adapter.
    OpenAlex,
    /// Citation metadata supplied by a host adapter.
    CiteAs,
    /// Researcher identities supplied by a host adapter.
    Orcid,
    /// Research Organization Registry identities supplied by a host adapter.
    Ror,
}
