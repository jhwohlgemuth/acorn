//!
//! Module for working with persistent identifiers (PID) such as [ORCiDs], [`DOI`]s, and [RAiDs]
//!
//! [ORCiDs]: https://orcid.org/
//! [RAiDs]: https://www.raid.org/
use crate::constants::{RE_DOI, RE_DOI_TEXT};
use crate::util::regex_capture_lookup;
use bon::{builder, Builder};

pub mod raid;

/// Add coercion to persistent identifier (PID) functionality to string values
pub trait PersistentIdentifier<T: AsRef<str>> {
    /// Coerce `self` into given PID type.
    /// ```
    /// use acorn_lib::schema::pid::{PID, PersistentIdentifier};
    ///
    /// let pid = "https://doi.org/10.1234/5678".to_pid(PID::DOI);
    /// assert_eq!(pid.suffix(), "5678");
    /// ```
    fn to_pid(&self, pid_type: PID) -> PersistentIdentifierInternal;
    /// Determines if `self` is of the given PID type.
    /// ```
    /// use acorn_lib::schema::pid::{PID, PersistentIdentifier};
    /// assert!("https://doi.org/10.1234/5678".is_pid(PID::DOI));
    /// ```
    fn is_pid(&self, pid_type: PID) -> bool;
    /// Determines if `self` is a DOI
    /// ```
    /// use acorn_lib::schema::pid::{PID, PersistentIdentifier};
    ///
    /// assert!("https://doi.org/10.1234/5678".is_doi());
    /// ```
    fn is_doi(&self) -> bool;
}
/// Trait for formatting string values
pub trait StringFormat {
    /// Convert `self` into a string with a standard format
    fn format(&self) -> String;
}
/// Internal representation of a persistent identifier
#[derive(Default)]
pub struct PersistentIdentifierInternal {
    /// Raw string content of the (possible) PID
    value: String,
    /// Type of PID
    pid_type: PID,
}
/// Persistent Identifier (PID) types
///
/// PIDs are globally unique identifiers, resolvable on the Web, and associated with a set of additional descriptive metadata (ex. [`raid::Metadata`])
#[derive(Clone, Debug, Default)]
pub enum PID {
    /// Archival Resource Key (ARK)
    ///
    /// Widely used persistent identifier, supported by the California Digital Library [21], in collaboration with DuraSpace. ARKs work similarly to DOIs, but are more permissive in design.[^ark]
    ///
    /// [^ark]: `M. Stocker et al., “Persistent Identification of Instruments,” Data Science Journal, vol. 19, p. 18, May 2020, doi: 10.5334/dsj-2020-018.`
    ARK,
    /// Digital Object Identifier (DOI)
    ///
    /// See [`DOI`]
    DOI,
    /// Open Researcher and Contributor ID (ORCiD)[^orcid]
    ///
    /// Disambiguates researchers, and connects people with their research activities. This includes employment affiliations, research outputs, funding, peer review activity, research resources, society membership, distinctions and other scholarly infrastructure.
    ///
    /// See <https://orcid.org/> for more information
    ///
    /// [^orcid]: `L. L. Haak, M. Fenner, L. Paglione, E. Pentz, and H. Ratner, “ORCID: a system to uniquely identify researchers,” Learned Publishing, vol. 25, no. 4, pp. 259–264, 2012, doi: 10.1087/20120404.`
    ORCID,
    /// Persistent Identification of Instruments (PIDINST)
    /// ### Citation
    /// ```text
    /// M. Stocker et al., “Persistent Identification of Instruments,” Data Science Journal, vol. 19, p. 18, May 2020, doi: 10.5334/dsj-2020-018.
    /// ```
    PIDINST,
    /// Research Activity Identifier (RAiD)
    ///
    /// Developed by tthe Australian Research Data Commons (ARDC), used to identify research projects and activities for access by research communities worldwide
    ///
    /// The ARDC and [DataCite](https://datacite.org/) have entered an agreement to use DataCite [`DOI`]s as RAiD identifiers
    ///
    /// See [`raid`] module
    RAID,
    /// Research Organization Registry (ROR)
    ///
    /// Global, community-led registry of open persistent identifiers for research organizations
    ///
    /// See <https://www.ror.org/> for more information
    ROR,
    /// Unknown PID
    #[default]
    Unknown,
}
/// Digital Object Identifier (DOI)
///
/// DOIs consist of a DOI name which is resolved at https://doi.org, with the full URI formulated according to the pattern https://doi.org/<DOI_name>. DOI names in turn consist of a prefix and a suffix, separated by a forward slash. The prefix is a code indicating the registrant who issues the DOI, e.g., Harvard University Dataverse – 10.7910; Dryad Digital Repository – 10.5061. The suffix is the identifier, in any form, assigned by the registrant.[^doi]
///
/// See <https://www.doi.org/doi-handbook/HTML/index.html> for more information
///
/// [^doi]: `N. Juty, S. M. Wimalaratne, S. Soiland-Reyes, J. Kunze, C. A. Goble, and T. Clark, “Unique, Persistent, Resolvable: Identifiers as the Foundation of FAIR,” Data Intellegence, vol. 2, no. 1–2, pp. 30–39, Jan. 2020, doi: 10.1162/dint_a_00025.`
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
impl Default for DOI {
    fn default() -> Self {
        DOI::init().build()
    }
}
impl StringFormat for DOI {
    fn format(&self) -> String {
        self.to_string()
    }
}
impl<T: AsRef<str>> PersistentIdentifier<T> for T
where
    T: ToString,
{
    fn to_pid(&self, _pid_type: PID) -> PersistentIdentifierInternal {
        match _pid_type {
            | PID::DOI => PersistentIdentifierInternal {
                value: self.as_ref().to_string(),
                pid_type: PID::DOI,
            },
            | _ => PersistentIdentifierInternal::default(),
        }
    }
    fn is_pid(&self, pid_type: PID) -> bool {
        match pid_type {
            | PID::DOI => self.is_doi(),
            | _ => false,
        }
    }
    fn is_doi(&self) -> bool {
        DOI::is_valid(self.as_ref())
    }
}
impl PersistentIdentifierInternal {
    /// Convert a `PersistentIdentifierInternal` to a `DOI`
    pub fn to_doi(&self) -> DOI {
        let PersistentIdentifierInternal { value, pid_type } = self;
        match pid_type {
            | PID::DOI => DOI::from_string(value),
            | _ => DOI::default(),
        }
    }
}

#[cfg(test)]
mod tests;
