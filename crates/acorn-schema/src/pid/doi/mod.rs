//! Digital Object Identifier parsing and formatting
use crate::namespaces::{ARXIV_DATACITE_REGISTRANT_CODE, DATACITE_DOI_DIRECTORY_INDICATOR, DEFAULT_DOI_SCHEMA_URI};
use crate::pid::{Arxiv, PersistentIdentifier, PersistentIdentifierParse, ISBN};
use crate::util::constants::{RE_DOI, RE_DOI_TEXT};
use acorn_core::prelude::alloc::{format, String, ToString, Vec};
use acorn_core::util::{regex_capture_lookup, trim_unmatched_trailing_parentheses, StringExt};
use bon::Builder;
use core::fmt;

/// Digital Object Identifier (DOI)
///
/// DOIs consist of a DOI name which is resolved at <https://doi.org>, with the full URI formulated according to the pattern `https://doi.org/{DOI_name}`. DOI names in turn consist of a prefix and a suffix, separated by a forward slash. The prefix is a code indicating the registrant who issues the DOI, e.g., Harvard University Dataverse - 10.7910; Dryad Digital Repository - 10.5061. The suffix is the identifier, in any form, assigned by the registrant.[^doi]
///
/// See <https://www.doi.org/doi-handbook/HTML/index.html> for more information
///
/// [^doi]: `N. Juty, S. M. Wimalaratne, S. Soiland-Reyes, J. Kunze, C. A. Goble, and T. Clark, "Unique, Persistent, Resolvable: Identifiers as the Foundation of FAIR," Data Intelligence, vol. 2, no. 1-2, pp. 30-39, Jan. 2020, doi: 10.1162/dint_a_00025.`
#[derive(Builder, Clone, Debug)]
#[builder(start_fn = init, on(String, into))]
pub struct DOI {
    /// Schema URI (i.e., <https://doi.org/>)
    pub schema_uri: Option<String>,
    /// Directory indicator
    /// ### Rules
    /// - Can contain only numeric values
    /// - Usually 10 but other indicators may be designated as compliant by the DOI Foundation
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
impl Default for DOI {
    fn default() -> Self {
        Self::new()
    }
}
impl fmt::Display for DOI {
    /// Format a DOI into a standard format of `"{prefix}/{suffix}"`
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let result = self.identifier();
        write!(f, "{result}")
    }
}
impl PersistentIdentifier for DOI {
    fn new() -> Self {
        DOI::init().build()
    }
    fn schema_uri(&self) -> String {
        self.schema_uri.as_ref().cloned().unwrap_or_default().trim_end_matches("/").to_string()
    }
    fn identifier(&self) -> String {
        let values = [self.prefix(), self.suffix()];
        values
            .iter()
            .flatten()
            .filter(|x| !x.is_empty())
            .map(String::from)
            .collect::<Vec<String>>()
            .join("/")
    }
    /// Get DOI prefix (i.e., "{directory_indicator}.{registrant_code}")
    fn prefix(&self) -> Option<String> {
        let values = [
            self.directory_indicator.as_ref().cloned().unwrap_or_default(),
            self.registrant_code.as_ref().cloned().unwrap_or_default(),
        ];
        let result = values
            .iter()
            .filter(|x| !x.is_empty())
            .map(String::from)
            .collect::<Vec<String>>()
            .join(".");
        Some(result)
    }
    /// Get DOI suffix
    fn suffix(&self) -> Option<String> {
        fn postprocess(mut value: String) -> String {
            if value.ends_with(".") {
                value.pop();
            }
            value
        }
        let result = self.suffix.as_ref().cloned().unwrap_or_default();
        if !result.is_empty() {
            Some(postprocess(result))
        } else {
            None
        }
    }
    fn url(&self) -> String {
        let identifier = self.identifier();
        if identifier.is_empty() {
            String::new()
        } else {
            let uri = self.schema_uri();
            let schema = if uri.is_empty() { DEFAULT_DOI_SCHEMA_URI } else { &uri };
            format!("{}/{}", schema, identifier)
        }
    }
}
impl PersistentIdentifierParse for DOI {
    /// Find all [`DOI`] values present in a string
    fn find_all(value: impl ToString) -> Vec<Self> {
        let re = &RE_DOI;
        re.find_iter(&value.to_string())
            .filter_map(Result::ok)
            .map(|m| DOI::from_string(trim_unmatched_trailing_parentheses(m.as_str())))
            .collect()
    }
    /// Convenience method for easily parsing and formatting a [`DOI`] from a string value
    /// ### Example
    /// ```rust
    /// use acorn_schema::pid::{DOI, PersistentIdentifierParse};
    ///
    /// assert_eq!(DOI::format("https://doi.org/10.1000/182"), "10.1000/182");
    /// assert_eq!(DOI::format("10.1000/182"), "10.1000/182");
    /// ```
    fn format(value: impl ToString) -> String {
        DOI::from_string(value).to_string()
    }
    /// Create new [`DOI`] by parsing raw string value
    /// ### Example
    /// ```rust
    /// use acorn_schema::pid::{DOI, PersistentIdentifier, PersistentIdentifierParse};
    ///
    /// let doi = DOI::from_string("https://doi.org/10.1000/182");
    /// assert_eq!(doi.prefix(), Some("10.1000".into()));
    /// assert_eq!(doi.suffix(), Some("182".into()));
    /// ```
    fn from_string(value: impl ToString) -> Self {
        let groups = ["schema_uri", "directory_indicator", "prefix_element", "registrant_code", "suffix"];
        let pattern = format!("^{RE_DOI_TEXT}$");
        let text = value.to_string().trim().to_string();
        let lookup = regex_capture_lookup(pattern.as_ref(), text.as_ref(), groups.to_vec());
        DOI::init()
            .maybe_schema_uri(lookup.get("schema_uri").cloned())
            .maybe_directory_indicator(lookup.get("directory_indicator").cloned())
            .maybe_registrant_code(lookup.get("registrant_code").cloned())
            .maybe_suffix(lookup.get("suffix").cloned())
            .build()
    }
    /// Check if value is a valid [`DOI`]
    /// ### Conditions
    /// - Must match DOI regular expression (see [`RE_DOI_TEXT`])
    /// - Is valid with or without schema URI[^format]
    /// - `10.5555/` is not a valid DOI prefix
    /// ### Example
    /// ```rust
    /// use acorn_schema::pid::{DOI, PersistentIdentifierParse};
    ///
    /// assert!(DOI::is_valid("https://doi.org/10.1000/182"));
    /// assert!(DOI::is_valid("10.1000/182"));
    /// assert!(!DOI::is_valid("10.5555/182"));
    /// ```
    ///
    /// [^format]: Use `DOI::format(value)` to ensure value is formatted correctly
    fn is_valid(value: impl ToString) -> bool {
        let pid = DOI::from_string(value.to_string());
        let prefix_is_valid = match pid.prefix() {
            | Some(x) => x.replace(".", "").is_numeric() && !x.eq("10.5555"),
            | _ => false,
        };
        let suffix_is_valid = pid.suffix().is_some();
        prefix_is_valid && suffix_is_valid
    }
}
impl From<ISBN> for DOI {
    fn from(isbn: ISBN) -> Self {
        DOI::init()
            .schema_uri(DEFAULT_DOI_SCHEMA_URI)
            .directory_indicator(DATACITE_DOI_DIRECTORY_INDICATOR)
            .maybe_registrant_code(isbn.prefix())
            .maybe_suffix(isbn.suffix())
            .build()
    }
}
impl From<Arxiv> for DOI {
    fn from(arxiv: Arxiv) -> Self {
        let suffix = arxiv.work_identifier().trim_start_matches("arXiv:").to_string();
        DOI::init()
            .schema_uri(DEFAULT_DOI_SCHEMA_URI)
            .directory_indicator(DATACITE_DOI_DIRECTORY_INDICATOR)
            .registrant_code(ARXIV_DATACITE_REGISTRANT_CODE)
            .suffix(format!("arXiv.{suffix}"))
            .build()
    }
}

#[cfg(all(test, feature = "std"))]
mod tests;
