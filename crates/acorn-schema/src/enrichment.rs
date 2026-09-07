//! Portable enrichment provider identities.
use crate::util::stable_alphabetic_hash;
use acorn_core::prelude::alloc::{format, String};
use convert_case::{Case, Casing};
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
impl Provider {
    /// Build a deterministic kebab-case identifier for a discovered value.
    pub fn discovered_identifier(self, source: &str) -> String {
        let normalized = format!("{self}-{source}")
            .chars()
            .fold((String::new(), false), |(mut output, separator), character| {
                match character.to_ascii_lowercase() {
                    | character if character.is_ascii_alphanumeric() => {
                        if separator && !output.is_empty() {
                            output.push('-');
                        }
                        output.push(character);
                        (output, false)
                    }
                    | _ => (output, true),
                }
            })
            .0
            .trim_matches('-')
            .to_case(Case::Kebab);
        match normalized.len() <= 100 {
            | true => normalized,
            | false => format!(
                "{}-{}",
                normalized.chars().take(83).collect::<String>().trim_matches('-'),
                stable_alphabetic_hash(&normalized)
            ),
        }
    }
}
