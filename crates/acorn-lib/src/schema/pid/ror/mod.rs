//! Research Organization Registry identifier parsing and formatting.
use crate::prelude::{format, String, ToString, Vec};
use crate::schema::namespaces::DEFAULT_ROR_SCHEMA_URI;
use crate::schema::pid::{mod_97_10_check_digit, PersistentIdentifier, PersistentIdentifierParse};
use crate::util::constants::{RE_ROR, RE_ROR_TEXT};
use crate::util::regex_capture_lookup;
use bon::Builder;
use core::fmt;

/// Research Organization Registry (ROR)[^ror]
///
/// A global, community-led registry of open persistent identifiers for research and funding organizations
///
/// [^ror]: <https://ror.org/>
#[derive(Builder, Clone, Debug)]
#[builder(start_fn = init, on(String, into))]
pub struct ROR {
    /// Schema URI (e.g., <https://ror.org/>)
    pub schema_uri: Option<String>,
    /// ROR identifier value
    pub identifier: Option<String>,
    /// The last two integers are a zero-padded checksum, 01 -98
    /// ### Note
    /// Check digits should be verified IAW [ISO 7064](https://www.iso.org/standard/31531.html)
    pub check_digit: Option<String>,
}
impl Default for ROR {
    fn default() -> Self {
        Self::new()
    }
}
impl fmt::Display for ROR {
    /// Format a ROR into a standard format of `"{schema_uri}{identifier}"`
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let schema_uri = self.schema_uri();
        let result = self.identifier();
        if result.is_empty() {
            write!(f, "")
        } else {
            write!(f, "{schema_uri}{result}")
        }
    }
}
impl PersistentIdentifier for ROR {
    fn new() -> Self {
        ROR::init().build()
    }
    fn schema_uri(&self) -> String {
        let processed = self
            .schema_uri
            .as_ref()
            .cloned()
            .unwrap_or_else(|| DEFAULT_ROR_SCHEMA_URI.to_string())
            .trim_end_matches("/")
            .replace(" ", "")
            .to_string();
        format!("{processed}/")
    }
    fn identifier(&self) -> String {
        self.identifier.clone().unwrap_or_default()
    }
    fn suffix(&self) -> Option<String> {
        self.identifier.clone()
    }
    fn check_digit(&self) -> Option<Vec<char>> {
        self.identifier().get(1..).and_then(mod_97_10_check_digit)
    }
}
impl PersistentIdentifierParse for ROR {
    /// Find all [`ROR`] values present in a string
    fn find_all(value: impl ToString) -> Vec<Self> {
        let re = &RE_ROR;
        re.find_iter(&value.to_string())
            .filter_map(Result::ok)
            .filter(|value| ROR::is_valid(value.as_str()))
            .map(|m| ROR::from_string(m.as_str()))
            .collect()
    }
    /// Convenience method for easily parsing and formatting a [`ROR`] from a string value
    /// ### Example
    /// ```rust
    /// use acorn::schema::pid::{ROR, PersistentIdentifierParse};
    ///
    /// assert_eq!(ROR::format("https://ror.org/01qz5mb56"), "https://ror.org/01qz5mb56");
    /// assert_eq!(ROR::format("01qz5mb56"), "https://ror.org/01qz5mb56");
    /// ```
    fn format(value: impl ToString) -> String {
        ROR::from_string(value.to_string()).to_string()
    }
    /// Create new [`ROR`] by parsing raw string value
    /// ### Example
    /// ```rust
    /// use acorn::schema::pid::{ROR, PersistentIdentifier, PersistentIdentifierParse};
    ///
    /// let ror = ROR::from_string("https://ror.org/01qz5mb56");
    /// assert_eq!(ror.identifier(), "01qz5mb56");
    /// ```
    fn from_string(value: impl ToString) -> Self {
        let groups = ["schema_uri", "identifier", "check_digit"];
        let pattern = format!("^{RE_ROR_TEXT}$");
        let text = value.to_string();
        let lookup = regex_capture_lookup(pattern.as_ref(), text.as_ref(), groups.to_vec());
        ROR::init()
            .maybe_schema_uri(lookup.get("schema_uri").cloned())
            .maybe_identifier(lookup.get("identifier").cloned())
            .maybe_check_digit(lookup.get("check_digit").cloned())
            .build()
    }
    /// Check if value is a valid [`ROR`]
    /// ### Conditions
    /// - Exactly 9 characters long
    /// - Must have a valid check digits (last two characters are zero-padded checksum, 01-98) (see `mod_97_10_check_digit`)
    /// - [Base32 Crockford](https://www.crockford.com/base32.html) encoded (i.e., digits 0-9 and letters A-Z except for I, L, O, and U)
    /// - Value can be valid with or without schema URI[^format]
    /// ### Example
    /// ```rust
    /// use acorn::schema::pid::{ROR, PersistentIdentifierParse};
    ///
    /// assert!(ROR::is_valid("https://ror.org/01qz5mb56"));
    /// assert!(ROR::is_valid("01qz5mb56"));
    /// ```
    ///
    /// [^format]: Use `ROR::format(value)` to ensure value is formatted correctly
    fn is_valid(value: impl ToString) -> bool {
        let pid = ROR::from_string(value.to_string());
        let identifier = pid.identifier();
        let last_two = identifier.chars().rev().take(2).collect::<String>().chars().rev().collect::<String>();
        if identifier.is_empty() {
            false
        } else {
            match mod_97_10_check_digit(&identifier[1..]) {
                | Some(check_digit) => {
                    if identifier.len() == 9 {
                        let calculated_last_two = check_digit.iter().collect::<String>();
                        calculated_last_two == last_two
                    } else {
                        false
                    }
                }
                | _ => false,
            }
        }
    }
}

#[cfg(test)]
mod tests;
