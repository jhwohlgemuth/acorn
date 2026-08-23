//! Open Researcher and Contributor Identifier parsing and formatting
use crate::prelude::{format, vec, String, ToString, Vec};
use crate::schema::namespaces::DEFAULT_ORCID_SCHEMA_URI;
use crate::schema::pid::{mod_11_2_check_digit, PersistentIdentifier, PersistentIdentifierParse};
use crate::util::constants::{RE_ORCID, RE_ORCID_TEXT};
use crate::util::{regex_capture_lookup, ToStringChunks};
use bon::Builder;
use core::fmt;

/// Open Researcher and Contributor ID (ORCiD)[^orcid]
///
/// Disambiguates researchers, and connects people with their research activities. This includes employment affiliations, research outputs, funding, peer review activity, research resources, society membership, distinctions and other scholarly infrastructure.
///
/// See <https://orcid.org/> for more information
///
/// [^orcid]: `L. L. Haak, M. Fenner, L. Paglione, E. Pentz, and H. Ratner, "ORCID: a system to uniquely identify researchers," Learned Publishing, vol. 25, no. 4, pp. 259-264, 2012, doi: 10.1087/20120404.`
#[derive(Builder, Clone, Debug)]
#[builder(start_fn = init, on(String, into))]
pub struct ORCID {
    /// Schema URI (i.e., <https://orcid.org/>)
    pub schema_uri: Option<String>,
    /// 16 digit string with hyphens every 4 digits (for readability)
    /// <div class="warning">This value can be stored with or without hyphens. To ensure compliancy, use <code>ORCID::identifier</code> method to access ORCiD identifier.</div>
    pub identifier: Option<String>,
    /// The check digit is the last (16th) digit of the identifier
    /// ### Note
    /// Check digit should be verified IAW [ISO 7064, MOD 11-2](https://www.iso.org/standard/31531.html).
    pub check_digit: Option<String>,
}
impl Default for ORCID {
    fn default() -> Self {
        Self::new()
    }
}
impl fmt::Display for ORCID {
    /// Format a ORCiD into a standard format of `"{schema_uri}{identifier}"`
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let schema_uri = self.schema_uri();
        let identifier = self.identifier();
        let uri = if schema_uri.is_empty() { DEFAULT_ORCID_SCHEMA_URI } else { &schema_uri };
        let values = match &self.identifier {
            | Some(_) => [uri, &identifier].to_vec(),
            | None => vec![],
        };
        let result = values
            .into_iter()
            .filter(|x| !x.is_empty())
            .map(String::from)
            .collect::<Vec<String>>()
            .join("/");
        write!(f, "{result}")
    }
}
impl PersistentIdentifier for ORCID {
    fn new() -> Self {
        ORCID::init().build()
    }
    /// Get ORCID schema URI
    /// ### Notes
    /// - Should always be "<https://orcid.org/>"
    fn schema_uri(&self) -> String {
        self.schema_uri.as_ref().cloned().unwrap_or_default().trim_end_matches("/").to_string()
    }
    /// Get ORCID identifier
    /// ### Notes
    /// - Will return an empty string if no identifier is present
    /// - Will always return a 19 character string with a hyphen every 4 characters (i.e., "0000-0000-0000-0000")
    fn identifier(&self) -> String {
        let stripped = self.identifier.as_ref().cloned().unwrap_or_default().replace("-", "");
        stripped.chunk(4).join("-")
    }
    fn suffix(&self) -> Option<String> {
        Some(self.identifier())
    }
    fn check_digit(&self) -> Option<Vec<char>> {
        mod_11_2_check_digit(self.identifier())
    }
}
impl PersistentIdentifierParse for ORCID {
    /// Find all [`ORCID`] values present in a string
    fn find_all(value: impl ToString) -> Vec<Self> {
        let re = &RE_ORCID;
        re.find_iter(&value.to_string())
            .filter_map(Result::ok)
            .map(|m| ORCID::from_string(m.as_str()))
            .collect()
    }
    /// Convenience method for easily parsing and formatting a [`ORCID`] from a string value
    /// ### Example
    /// ```rust
    /// use acorn::schema::pid::{ORCID, PersistentIdentifierParse};
    ///
    /// assert_eq!(ORCID::format("https://orcid.org/0000-0002-2057-9115"), "https://orcid.org/0000-0002-2057-9115");
    /// assert_eq!(ORCID::format("0000-0002-2057-9115"), "https://orcid.org/0000-0002-2057-9115");
    /// ```
    fn format(value: impl ToString) -> String {
        ORCID::from_string(value).to_string()
    }
    /// Create new [`ORCID`] by parsing raw string value
    /// ### Example
    /// ```rust
    /// use acorn::schema::pid::{ORCID, PersistentIdentifier, PersistentIdentifierParse};
    ///
    /// let orcid = ORCID::from_string("https://orcid.org/0000-0002-2057-9115");
    /// assert_eq!(orcid.identifier(), "0000-0002-2057-9115");
    /// ```
    fn from_string(value: impl ToString) -> Self {
        let groups = ["schema_uri", "identifier", "check_digit"];
        let pattern = format!("^{RE_ORCID_TEXT}$");
        let text = value.to_string();
        let lookup = regex_capture_lookup(pattern.as_ref(), text.as_ref(), groups.to_vec());
        ORCID::init()
            .maybe_schema_uri(lookup.get("schema_uri").cloned())
            .maybe_identifier(lookup.get("identifier").cloned())
            .maybe_check_digit(lookup.get("check_digit").cloned())
            .build()
    }
    /// Check if value is a valid [`ORCID`]
    /// ### Conditions
    /// - ORCiD identifier must be 16 characters, 0 thru 9, or "X"
    /// - Last character of identifier must be a valid ISO 7064 check digit (see `mod_11_2_check_digit`)
    /// - Value can be valid with or without hyphens in the ORCiD identifier[^format]
    /// - Value can be valid with or without schema URI[^format]
    /// ### Example
    /// ```rust
    /// use acorn::schema::pid::{ORCID, PersistentIdentifierParse};
    ///
    /// assert!(ORCID::is_valid("https://orcid.org/0000-0002-2057-9115"));
    /// assert!(ORCID::is_valid("0000-0002-2057-9115"));
    /// assert!(ORCID::is_valid("0000000220579115"));
    /// ```
    ///
    /// [^format]: Use `ORCID::format(value)` to ensure value is formatted correctly
    fn is_valid(value: impl ToString) -> bool {
        let pid = ORCID::from_string(value.to_string());
        let identifier = pid.identifier();
        let last = identifier.chars().last().unwrap_or_default();
        match mod_11_2_check_digit(identifier.as_str()) {
            | Some(check_digit) => {
                if check_digit.contains(&last) {
                    identifier.len() == 19
                } else {
                    false
                }
            }
            | _ => false,
        }
    }
}

#[cfg(test)]
mod tests;
