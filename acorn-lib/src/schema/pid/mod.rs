//!
//! Module for working with persistent identifiers (PID) such as [ORCiDs], [`DOI`]s, and [RAiDs]
//!
//! [ORCiDs]: https://orcid.org/
//! [RAiDs]: https://www.raid.org/
use crate::constants::{RE_DOI, RE_DOI_TEXT};
use crate::util::regex_capture_lookup;
use bon::{builder, Builder};

pub mod raid;

/// Digital Object Identifier (DOI)
///
/// See <https://www.doi.org/doi-handbook/HTML/index.html> for more information
#[derive(Builder, Clone, Debug)]
#[builder(start_fn = init)]
pub struct DOI {
    /// Schema URI (e.g., <https://doi.org/>)
    pub schema_uri: Option<String>,
    /// Directory indicator
    /// ### Rules
    /// - Can contain only numeric values
    /// - usually 10 but other indicators may be designated as compliant by the DOI Foundation
    pub directory_indicator: Option<String>,
    /// Registrant code
    /// ### Rules
    /// - Can contain only numeric values and one or several full stops which are used to subdivide the code
    /// - If the directory indicator is 10 then a registrant code is mandatory
    pub registrant_code: Option<String>,
    /// Suffix
    /// ### Rules
    /// - Shall be unique to the prefix element that precedes it
    /// - Can be a sequential number
    /// - Can be an identifier generated from or based on another system used by the registrant
    /// - No length limit is set to the suffix by the DOI System
    pub suffix: Option<String>,
}
impl DOI {
    /// Convenience method for easily parsing and formatting a DOI from a string value
    /// ### Example
    /// ```rust
    /// use acorn_lib::schema::validate::DOI;
    ///
    /// assert_eq!(DOI::format("https://doi.org/10.1000/182"), "10.1000/182");
    /// assert_eq!(DOI::format("10.1000/182"), "10.1000/182");
    /// ```
    pub fn format<S>(value: S) -> String
    where
        S: AsRef<str>,
    {
        DOI::from_string(value).to_string()
    }
    /// Create new DOI by parsing raw string value
    /// ### Example
    /// ```rust
    /// use acorn_lib::schema::validate::DOI;
    ///
    /// let doi = DOI::from_string("https://doi.org/10.1000/182");
    /// assert_eq!(doi.prefix(), "10.1000");
    /// assert_eq!(doi.suffix(), "182");
    /// ```
    pub fn from_string<S>(value: S) -> DOI
    where
        S: AsRef<str>,
    {
        let names = ["schema_uri", "directory_indicator", "registrant_code", "suffix"];
        let lookup = regex_capture_lookup(RE_DOI_TEXT, value.as_ref(), names.to_vec());
        DOI::init()
            .maybe_schema_uri(lookup.get("schema_uri").cloned())
            .maybe_directory_indicator(lookup.get("directory_indicator").cloned())
            .maybe_registrant_code(lookup.get("registrant_code").cloned())
            .maybe_suffix(lookup.get("suffix").cloned())
            .build()
    }
    /// Check if value is a valid DOI
    /// ### Example
    /// ```rust
    /// use acorn_lib::schema::validate::DOI;
    ///
    /// assert!(DOI::is_valid("https://doi.org/10.1000/182"));
    /// ```
    pub fn is_valid<S>(value: S) -> bool
    where
        S: AsRef<str>,
    {
        match RE_DOI.is_match(value.as_ref()) {
            | Ok(x) if x && !value.as_ref().contains("10.5555/") => true,
            | _ => false,
        }
    }
    /// Get DOI prefix (i.e., "{directory_indicator}.{registrant_code}")
    pub fn prefix(&self) -> String {
        let values = [
            self.directory_indicator.as_ref().cloned().unwrap_or_default(),
            self.registrant_code.as_ref().cloned().unwrap_or_default(),
        ];
        values
            .iter()
            .filter(|x| !x.is_empty())
            .map(String::from)
            .collect::<Vec<String>>()
            .join(".")
    }
    /// Get DOI suffix
    pub fn suffix(&self) -> String {
        self.suffix.as_ref().cloned().unwrap_or_default()
    }
}
impl std::fmt::Display for DOI {
    /// Format a DOI into a standard format of `"{prefix}/{suffix}"`
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let values = [self.prefix(), self.suffix()];
        let result = values
            .iter()
            .filter(|x| !x.is_empty())
            .map(String::from)
            .collect::<Vec<String>>()
            .join("/");
        write!(f, "{result}")
    }
}

#[cfg(test)]
mod tests;
