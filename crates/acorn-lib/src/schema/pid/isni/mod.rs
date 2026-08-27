//! International Standard Name Identifier parsing and formatting.
use crate::prelude::alloc::{format, String, ToString, Vec};
use crate::schema::namespaces::DEFAULT_ISNI_SCHEMA_URI;
use crate::schema::pid::{mod_11_2_check_digit, PersistentIdentifier, PersistentIdentifierParse, ORCID};
use crate::util::constants::{RE_ISNI, RE_ISNI_TEXT};
use crate::util::regex_capture_lookup;
use bon::Builder;
use core::fmt;

/// International Standard Name Identifier (ISNI)
///
/// ISNIs identify public identities of people and organizations using a 16-character ISO 7064 MOD 11-2 identifier.
#[derive(Builder, Clone, Debug)]
#[builder(start_fn = init, on(String, into))]
pub struct ISNI {
    /// Resolver URI (i.e., <https://isni.org/isni/>)
    pub schema_uri: Option<String>,
    /// The 16-character ISNI value
    pub identifier: Option<String>,
    /// The final ISNI check character
    pub check_digit: Option<String>,
}
impl Default for ISNI {
    fn default() -> Self {
        Self::new()
    }
}
impl fmt::Display for ISNI {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let identifier = self.identifier();
        match identifier.is_empty() {
            | true => write!(f, ""),
            | false => write!(f, "{}/{}", self.schema_uri(), identifier),
        }
    }
}
impl PersistentIdentifier for ISNI {
    fn new() -> Self {
        ISNI::init().build()
    }
    fn schema_uri(&self) -> String {
        DEFAULT_ISNI_SCHEMA_URI.to_string()
    }
    fn identifier(&self) -> String {
        self.identifier
            .as_ref()
            .map(|value| value.replace([' ', '-'], "").to_ascii_uppercase())
            .unwrap_or_default()
    }
    fn suffix(&self) -> Option<String> {
        Some(self.identifier())
    }
    fn check_digit(&self) -> Option<Vec<char>> {
        mod_11_2_check_digit(self.identifier())
    }
}
impl PersistentIdentifierParse for ISNI {
    /// Find all [`ISNI`] values present in a string
    fn find_all(value: impl ToString) -> Vec<Self> {
        RE_ISNI
            .find_iter(&value.to_string())
            .filter_map(Result::ok)
            .filter(|matched| ISNI::is_valid(matched.as_str()) && !ORCID::is_valid(matched.as_str()))
            .map(|matched| ISNI::from_string(matched.as_str()))
            .collect()
    }
    /// Parse and format an [`ISNI`] as its canonical resolver URL
    fn format(value: impl ToString) -> String {
        ISNI::from_string(value).to_string()
    }
    /// Create an [`ISNI`] by parsing a resolver URL or bare identifier
    fn from_string(value: impl ToString) -> Self {
        let groups = ["schema_uri", "identifier", "check_digit"];
        let pattern = format!("^{RE_ISNI_TEXT}$");
        let text = value.to_string();
        let lookup = regex_capture_lookup(pattern.as_ref(), text.as_ref(), groups.to_vec());
        ISNI::init()
            .maybe_schema_uri(lookup.get("schema_uri").cloned())
            .maybe_identifier(lookup.get("identifier").cloned())
            .maybe_check_digit(lookup.get("check_digit").cloned())
            .build()
    }
    /// Check whether a value is a valid ISNI with a MOD 11-2 check character
    fn is_valid(value: impl ToString) -> bool {
        let pid = ISNI::from_string(value);
        let identifier = pid.identifier();
        let last = identifier.chars().last().unwrap_or_default();
        identifier.len() == 16 && mod_11_2_check_digit(&identifier).is_some_and(|digits| digits.contains(&last))
    }
}

#[cfg(test)]
mod tests;
