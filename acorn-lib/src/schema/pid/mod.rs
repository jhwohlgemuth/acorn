//! Persistent Identifiers (PID)
//!
//! Contains functions for working with persistent identifiers (PID) such as [`ORCID`], [`DOI`]s, and [RAiD](`raid`)s
//!
//! ### Features
//! - Best in class validation
//! - Convert persistent identifiers into standard formats
//! - Access the sub parts of a persistent identifier
//!
//! [RAiDs]: https://www.raid.org/
use crate::constants::{DEFAULT_ORCID_SCHEMA_URI, RE_DOI, RE_DOI_TEXT, RE_ORCID, RE_ORCID_TEXT};
use crate::util::{regex_capture_lookup, ToStringChunks};
use bon::{builder, Builder};
use std::fmt::Display;

pub mod raid;

/// Add coercion to persistent identifier (PID) functionality to string values
pub trait PersistentIdentifier<T: AsRef<str>> {
    /// Convert `self` into a string standard format PID of a certain type
    /// ```rust
    /// use acorn_lib::schema::pid::{PID, PersistentIdentifier};
    ///
    /// assert_eq!("https://doi.org/10.1234/5678".format_as(PID::DOI), "10.1234/5678");
    /// assert_eq!("0000-0002-2057-9115".format_as(PID::ORCID), "https://orcid.org/0000-0002-2057-9115");
    /// ```
    fn format_as(&self, pid_type: PID) -> String;
    /// Coerce `self` into given PID type.
    /// ```rust
    /// use acorn_lib::schema::pid::{PID, PersistentIdentifier};
    ///
    /// let doi = "https://doi.org/10.1234/5678".to_pid(PID::DOI).to_doi();
    /// assert_eq!(doi.suffix(), "5678");
    /// ```
    fn to_pid(&self, pid_type: PID) -> PersistentIdentifierInternal;
    /// Determines if `self` is of the given PID type.
    /// ```rust
    /// use acorn_lib::schema::pid::{PID, PersistentIdentifier};
    /// assert!("https://doi.org/10.1234/5678".is_pid(PID::DOI));
    /// ```
    fn is_pid(&self, pid_type: PID) -> bool;
    /// Determines if `self` is a DOI
    /// ```rust
    /// use acorn_lib::schema::pid::{PID, PersistentIdentifier};
    ///
    /// assert!("https://doi.org/10.1234/5678".is_doi());
    /// ```
    fn is_doi(&self) -> bool;
    /// Determines if `self` is a ORCID
    /// ```
    /// use acorn_lib::schema::pid::{PID, PersistentIdentifier};
    ///
    /// assert!("https://orcid.org/0000-0000-0000-0000".is_orcid());
    /// ```
    fn is_orcid(&self) -> bool;
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
    /// Widely used persistent identifier, supported by the California Digital Library \[21\], in collaboration with DuraSpace. ARKs work similarly to DOIs, but are more permissive in design.[^ark]
    ///
    /// [^ark]: `M. Stocker et al., "Persistent Identification of Instruments," Data Science Journal, vol. 19, p. 18, May 2020, doi: 10.5334/dsj-2020-018.`
    ARK,
    /// Digital Object Identifier (DOI)
    ///
    /// See [`DOI`]
    DOI,
    /// Open Researcher and Contributor ID (ORCiD)
    ///
    /// See [`ORCID`]
    ORCID,
    /// Persistent Identification of Instruments (PIDINST)
    /// ### Citation
    /// ```text
    /// M. Stocker et al., "Persistent Identification of Instruments," Data Science Journal, vol. 19, p. 18, May 2020, doi: 10.5334/dsj-2020-018.
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
/// DOIs consist of a DOI name which is resolved at <https://doi.org>, with the full URI formulated according to the pattern `https://doi.org/{DOI_name}`. DOI names in turn consist of a prefix and a suffix, separated by a forward slash. The prefix is a code indicating the registrant who issues the DOI, e.g., Harvard University Dataverse - 10.7910; Dryad Digital Repository - 10.5061. The suffix is the identifier, in any form, assigned by the registrant.[^doi]
///
/// See <https://www.doi.org/doi-handbook/HTML/index.html> for more information
///
/// [^doi]: `N. Juty, S. M. Wimalaratne, S. Soiland-Reyes, J. Kunze, C. A. Goble, and T. Clark, "Unique, Persistent, Resolvable: Identifiers as the Foundation of FAIR," Data Intellegence, vol. 2, no. 1-2, pp. 30-39, Jan. 2020, doi: 10.1162/dint_a_00025.`
#[derive(Builder, Clone, Debug)]
#[builder(start_fn = init)]
pub struct DOI {
    /// Schema URI (i.e., <https://doi.org/>)
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
/// Open Researcher and Contributor ID (ORCiD)[^orcid]
///
/// Disambiguates researchers, and connects people with their research activities. This includes employment affiliations, research outputs, funding, peer review activity, research resources, society membership, distinctions and other scholarly infrastructure.
///
/// See <https://orcid.org/> for more information
///
/// [^orcid]: `L. L. Haak, M. Fenner, L. Paglione, E. Pentz, and H. Ratner, "ORCID: a system to uniquely identify researchers," Learned Publishing, vol. 25, no. 4, pp. 259-264, 2012, doi: 10.1087/20120404.`
#[derive(Builder, Clone, Debug)]
#[builder(start_fn = init)]
pub struct ORCID {
    /// Schema URI (i.e., <https://orcid.org/>)
    pub schema_uri: Option<String>,
    /// 16 digit string with hyphens every 4 digits (for readability)
    /// <div class="warning">This value can be stored with or without hyphens. To ensure compliancy, use <code>ORCID::identifier</code> method to access ORCiD identifier.</div>
    pub identifier: Option<String>,
    /// The check digit is the last (16th) digit of the identifier
    /// ### Note
    /// Check digit should be verified IAW by [ISO 7064, MOD 11-2](https://www.iso.org/standard/31531.html) (see [`iso7064_check_digit`])
    pub check_digit: Option<String>,
}
impl DOI {
    /// Convenience method for easily parsing and formatting a DOI from a string value
    /// ### Example
    /// ```rust
    /// use acorn_lib::schema::pid::DOI;
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
    /// use acorn_lib::schema::pid::DOI;
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
    /// ### Conditions
    /// - Must match DOI regular expression (see [`RE_DOI_TEXT`])
    /// - Is valid with or without schema URI[^format]
    /// - `10.5555/` is not a valid DOI prefix
    /// ### Example
    /// ```rust
    /// use acorn_lib::schema::pid::DOI;
    ///
    /// assert!(DOI::is_valid("https://doi.org/10.1000/182"));
    /// assert!(DOI::is_valid("10.1000/182"));
    /// ```
    ///
    /// [^format]: Use `DOI::format(value)` to ensure value is formatted correctly
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
impl Default for DOI {
    fn default() -> Self {
        DOI::init().build()
    }
}
impl Default for ORCID {
    fn default() -> Self {
        ORCID::init().build()
    }
}
impl Display for DOI {
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
impl Display for ORCID {
    /// Format a ORCiD into a standard format of `"{schema_uri}{identifier}"`
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
            .join("");
        write!(f, "{result}")
    }
}
impl ORCID {
    /// Get ORCID identifier
    /// ### Notes
    /// - Will return an empty string if no identifier is present
    /// - Will always return a 19 character string with a hyphen every 4 characters (i.e., "0000-0000-0000-0000")
    pub fn identifier(&self) -> String {
        let stripped = self.identifier.as_ref().cloned().unwrap_or_default().replace("-", "");
        stripped.chunk(4).join("-")
    }
    /// Get ORCID schema URI
    /// ### Notes
    /// - Should always be "<https://orcid.org/>"
    pub fn schema_uri(&self) -> String {
        self.schema_uri.as_ref().cloned().unwrap_or_default()
    }
    /// Convenience method for easily parsing and formatting a ORCID from a string value
    /// ### Example
    /// ```rust
    /// use acorn_lib::schema::pid::ORCID;
    ///
    /// assert_eq!(ORCID::format("https://orcid.org/0000-0002-2057-9115"), "https://orcid.org/0000-0002-2057-9115");
    /// assert_eq!(ORCID::format("0000-0002-2057-9115"), "https://orcid.org/0000-0002-2057-9115");
    /// ```
    pub fn format<S>(value: S) -> String
    where
        S: AsRef<str>,
    {
        ORCID::from_string(value).to_string()
    }
    /// Create new ORCID by parsing raw string value
    /// ### Example
    /// ```rust
    /// use acorn_lib::schema::pid::ORCID;
    ///
    /// let orcid = ORCID::from_string("https://orcid.org/0000-0002-2057-9115");
    /// assert_eq!(orcid.identifier(), "0000-0002-2057-9115");
    /// ```
    pub fn from_string<S>(value: S) -> ORCID
    where
        S: AsRef<str>,
    {
        let names = ["schema_uri", "identifier", "check_digit"];
        let lookup = regex_capture_lookup(RE_ORCID_TEXT, value.as_ref(), names.to_vec());
        ORCID::init()
            .maybe_schema_uri(lookup.get("schema_uri").cloned())
            .maybe_identifier(lookup.get("identifier").cloned())
            .maybe_check_digit(lookup.get("check_digit").cloned())
            .build()
    }
    /// Check if value is a valid ORCiD
    /// ### Conditions
    /// - ORCiD identifier must be 16 characters, 0 thru 9, or "X"
    /// - Last character of identifier must be a valid ISO 7064 check digit (see [`iso7064_check_digit`])
    /// - Value can be valid with or without hyphens in the ORCiD identifier[^format]
    /// - Value can be valid with or without schema URI[^format]
    /// ### Example
    /// ```rust
    /// use acorn_lib::schema::pid::ORCID;
    ///
    /// assert!(ORCID::is_valid("https://orcid.org/0000-0002-2057-9115"));
    /// assert!(ORCID::is_valid("0000-0002-2057-9115"));
    /// assert!(ORCID::is_valid("0000000220579115"));
    /// ```
    ///
    /// [^format]: Use `ORCID::format(value)` to ensure value is formatted correctly
    pub fn is_valid<S>(value: S) -> bool
    where
        S: AsRef<str>,
    {
        let orcid = ORCID::from_string(value.as_ref());
        let identifier = orcid.identifier();
        let last = identifier.chars().last().unwrap_or_default().to_string();
        let checksum = match iso7064_check_digit(identifier.as_str()) {
            | 10 => "X".to_string(),
            | value if (0..10).contains(&value) => format!("{value}"),
            | _ => "".to_string(),
        };
        match RE_ORCID.is_match(value.as_ref()) {
            | Ok(true) if last == checksum && identifier.len() == 19 => true,
            | _ => false,
        }
    }
}
impl StringFormat for DOI {
    fn format(&self) -> String {
        self.to_string()
    }
}
impl StringFormat for ORCID {
    fn format(&self) -> String {
        self.to_string()
    }
}
impl<T: AsRef<str>> PersistentIdentifier<T> for T
where
    T: ToString,
{
    fn format_as(&self, pid_type: PID) -> String {
        match pid_type {
            | PID::DOI => DOI::format(self.as_ref()),
            | PID::ORCID => ORCID::format(self.as_ref()),
            | _ => self.as_ref().to_string(),
        }
    }
    fn to_pid(&self, _pid_type: PID) -> PersistentIdentifierInternal {
        match _pid_type {
            | PID::DOI => PersistentIdentifierInternal {
                value: self.as_ref().to_string(),
                pid_type: PID::DOI,
            },
            | PID::ORCID => PersistentIdentifierInternal {
                value: self.as_ref().to_string(),
                pid_type: PID::ORCID,
            },
            | _ => PersistentIdentifierInternal::default(),
        }
    }
    fn is_pid(&self, pid_type: PID) -> bool {
        match pid_type {
            | PID::DOI => self.is_doi(),
            | PID::ORCID => self.is_orcid(),
            | _ => false,
        }
    }
    fn is_doi(&self) -> bool {
        DOI::is_valid(self.as_ref())
    }
    fn is_orcid(&self) -> bool {
        ORCID::is_valid(self.as_ref())
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
    /// Convert a `PersistentIdentifierInternal` to a `ORCID`
    pub fn to_orcid(&self) -> ORCID {
        let PersistentIdentifierInternal { value, pid_type } = self;
        match pid_type {
            | PID::ORCID => ORCID::from_string(value),
            | _ => ORCID::default(),
        }
    }
}
/// Calculate check digit IAW [ISO 7064, MOD 11-2](https://www.iso.org/standard/31531.html)
///
/// "MOD 11-2" means modulus = 11 and radix = 2
///
/// ### Example
/// ```rust
/// use acorn_lib::schema::pid::iso7064_check_digit;
///
/// assert_eq!(iso7064_check_digit("0000000220579115"), 5);
/// assert_eq!(iso7064_check_digit("0000-0002-2057-9115"), 5);
/// ```
pub fn iso7064_check_digit(value: &str) -> u32 {
    const MODULUS: u32 = 11;
    const RADIX: u32 = 2;
    let working = value.replace("-", "");
    let sum = working.chars().take(15).fold(0, |acc, x| {
        let digit = x.to_digit(10).unwrap_or_default();
        (acc + digit) * RADIX
    });
    let remainder = sum % MODULUS;
    (MODULUS + 1 - remainder) % MODULUS
}

#[cfg(test)]
mod tests;
